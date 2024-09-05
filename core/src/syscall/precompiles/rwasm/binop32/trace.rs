use p3_field::PrimeField32;

use crate::air::{Word, WORD_SIZE};
use crate::alu::{MulCols, BYTE_SIZE, NUM_MUL_COLS};
use crate::bytes::{ByteLookupEvent, ByteOpcode};
use crate::memory::MemoryReadWriteCols;
use crate::utils::pad_rows;
use crate::{air::MachineAir, runtime::ExecutionRecord};

use super::columns::{BinOp32Cols, NUM_BINOP_MEM_COLS};
use super::{super::RwasmOp, BinOp32Chip};

use std::borrow::BorrowMut;

use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_maybe_rayon::prelude::{ParallelIterator, ParallelSlice};

use crate::bytes::event::ByteRecord;
use crate::{runtime::Program, stark::MachineRecord};
pub const PRODUCT_SIZE: usize = 2 *WORD_SIZE;



/// The mask for a byte.
pub const BYTE_MASK: u8 = 0xff;

impl<F: PrimeField32> MachineAir<F> for BinOp32Chip {
    type Record = ExecutionRecord;
    type Program = Program;

    fn name(&self) -> String {
        "RwasmBinOp".to_string()
    }

    fn generate_trace(&self, input: &Self::Record, output: &mut Self::Record) -> RowMajorMatrix<F> {
        let mut rows = Vec::new();

        let mut new_byte_lookup_events = Vec::new();
        for i in 0..input.rwasm_binop_events.len() {
            let mut event = input.rwasm_binop_events[i].clone();
            let shard = event.shard;
            let channel = event.channel;

            let mut first_row = [F::zero(); NUM_BINOP_MEM_COLS];

            let cols: &mut BinOp32Cols<F> = first_row.as_mut_slice().borrow_mut();

            cols.x_memory_record.populate(
                channel,
                event.x_read_records,
                &mut new_byte_lookup_events,
            );

            cols.y_memory_record.populate(
                channel,
                event.y_read_records,
                &mut new_byte_lookup_events,
            );

            cols.stack_ptr_record.populate(
                channel,
                event.stack_ptr_read_record,
                &mut new_byte_lookup_events,
            );
            cols.opcode = F::from_canonical_u32(event.opcode);
            cols.stack_ptr_addr = F::from_canonical_u32(event.stack_ptr_addr);
            cols.x_addr = Word::from(event.x_addr);
            cols.y_addr = Word::from(event.y_addr);

            let op = RwasmOp::from_u32(event.opcode);
            cols.is_add= F::from_bool(false);
            cols.is_sub= F::from_bool(false);
            cols.is_mul= F::from_bool(false);
            cols.is_div =F::from_bool(false);
            // we choose one op and disable the rest.
            match op {
                RwasmOp::I32Add => {
                    cols.is_add = F::from_bool(true);
                    cols.addsubres
                        .populate(output, shard, channel, event.x_val, event.y_val);
                    
                }
                RwasmOp::I32Sub => {
                    cols.is_sub = F::from_bool(true);
                    cols.addsubres
                        .populate(output, shard, channel, event.res_val, event.y_val);
                }
                RwasmOp::I32Mul => {

                // copied from alu::mul
                //in future we should have our own implementation
                // removed code for MULH MULHU MULSHU
            
                    cols.is_mul =  F::from_bool(true);
                    {
                        let mut mul_row = [F::zero(); NUM_MUL_COLS];
                        let mul_cols: &mut MulCols<F> = mul_row.as_mut_slice().borrow_mut();

                        let a_word = event.res_val.to_le_bytes();
                        let b_word = event.x_val.to_le_bytes();
                        let c_word = event.y_val.to_le_bytes();

                        let mut b = b_word.to_vec();
                        let mut c = c_word.to_vec();

                        
                            let b_msb = get_msb(b_word);
                            mul_cols.b_msb = F::from_canonical_u8(b_msb);
                            let c_msb = get_msb(c_word);
                            mul_cols.c_msb = F::from_canonical_u8(c_msb);

                            // Insert the MSB lookup events.
                            {
                                let words = [b_word, c_word];
                                let mut blu_events: Vec<ByteLookupEvent> = vec![];
                                for word in words.iter() {
                                    let most_significant_byte = word[WORD_SIZE - 1];
                                    blu_events.push(ByteLookupEvent {
                                        shard: event.shard,
                                        channel: event.channel,
                                        opcode: ByteOpcode::MSB,
                                        a1: get_msb(*word) as u32,
                                        a2: 0,
                                        b: most_significant_byte as u32,
                                        c: 0,
                                    });
                                }
                                output.add_byte_lookup_events(blu_events);
                            }
                        
                            let mut product = [0u32; PRODUCT_SIZE];
                            for i in 0..b.len() {
                                for j in 0..c.len() {
                                    if i + j < PRODUCT_SIZE {
                                        product[i + j] += (b[i] as u32) * (c[j] as u32);
                                    }
                                }
                            }

                        // Calculate the correct product using the `product` array. We store the correct carry
                        // value for verification.
                        let base = 1 << BYTE_SIZE;
                        let mut carry = [0u32; PRODUCT_SIZE];
                        for i in 0..PRODUCT_SIZE {
                            carry[i] = product[i] / base;
                            product[i] %= base;
                            if i + 1 < PRODUCT_SIZE {
                                product[i + 1] += carry[i];
                            }
                            mul_cols.carry[i] = F::from_canonical_u32(carry[i]);
                        }

                        mul_cols.product = product.map(F::from_canonical_u32);
                        mul_cols.a = Word(a_word.map(F::from_canonical_u8));
                        mul_cols.b = Word(b_word.map(F::from_canonical_u8));
                        mul_cols.c = Word(c_word.map(F::from_canonical_u8));
                        mul_cols.is_real = F::one();
                        mul_cols.is_mul = F::from_bool(true);
                        mul_cols.is_mulh = F::from_bool(false);
                        mul_cols.is_mulhu = F::from_bool(false);
                        mul_cols.is_mulhsu = F::from_bool(false);
                        mul_cols.shard = F::from_canonical_u32(event.shard);
                        mul_cols.channel = F::from_canonical_u32(event.channel);

                        // Range check.
                        {
                            output.add_u16_range_checks(event.shard, event.channel, &carry);
                            output.add_u8_range_checks(
                                event.shard,
                                event.channel,
                                &product.map(|x| x as u8),
                            );
                        }
                        
                    }
                    
                },
            }
            cols.is_real = F::from_bool(true);

            cols.pre_stack_ptr_val = Word::from(event.pre_stack_ptr_val);
            cols.post_stack_ptr_val = Word::from(event.post_stack_ptr_val);

            cols.y_write_record.populate(
                channel,
                event.res_write_records,
                &mut new_byte_lookup_events,
            );

            cols.stack_ptr_write_record.populate(
                channel,
                event.stack_ptr_write_record,
                &mut new_byte_lookup_events,
            );
            rows.push(first_row);
        }

        output.add_byte_lookup_events(new_byte_lookup_events);

        let num_real_rows = rows.len();

        pad_rows(&mut rows, || [F::zero(); NUM_BINOP_MEM_COLS]);
        // Convert the trace to a row major matrix.
        let mut trace = RowMajorMatrix::new(
            rows.into_iter().flatten().collect::<Vec<_>>(),
            NUM_BINOP_MEM_COLS,
        );

        // Write the nonces to the trace.
        for i in 0..trace.height() {
            let cols: &mut BinOp32Cols<F> =
                trace.values[i * NUM_BINOP_MEM_COLS..(i + 1) * NUM_BINOP_MEM_COLS].borrow_mut();
            cols.nonce = F::from_canonical_usize(i);
        }

        trace
    }

    fn included(&self, shard: &Self::Record) -> bool {
        !shard.rwasm_binop_events.is_empty()
    }
}

pub const fn get_msb(a: [u8; WORD_SIZE]) -> u8 {
    (a[WORD_SIZE - 1] >> (BYTE_SIZE - 1)) & 1
}

