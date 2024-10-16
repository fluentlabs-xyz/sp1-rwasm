use itertools::multiunzip;
use p3_field::PrimeField32;

use crate::air::{Word, WORD_SIZE};
use crate::alu::{self, create_alu_lookup_id, create_alu_lookups, AluEvent, MulCols, BYTE_SIZE, NUM_MUL_COLS};
use crate::bytes::{ByteLookupEvent, ByteOpcode};
use crate::memory::MemoryReadWriteCols;
use crate::syscall;
use crate::utils::pad_rows;
use crate::{air::MachineAir, runtime::ExecutionRecord};

use super::columns::{ArithSelector, BinOp32Cols, NUM_BINOP32_MEM_COLS};
use super::BinOp32Event;
use super::{super::RwasmOp, BinOp32Chip};

use std::borrow::BorrowMut;
use std::collections::HashMap;


use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_maybe_rayon::prelude::{ParallelIterator, ParallelSlice};

use crate::bytes::event::ByteRecord;
use crate::{runtime::Opcode, runtime::Program, stark::MachineRecord};
pub const PRODUCT_SIZE: usize = 2 * WORD_SIZE;

/// The mask for a byte.
pub const BYTE_MASK: u8 = 0xff;

impl<F: PrimeField32> MachineAir<F> for BinOp32Chip {
    type Record = ExecutionRecord;
    type Program = Program;

    fn name(&self) -> String {
        "RwasmBinOp".to_string()
    }
/*
Important Notice. When we use sp1 alu to delegate our compuation verification
we have to generate an alu trace  and call this before any alu events generation called.
check sp1 divrem alu for similar comment.
fn generate_dependencies
 */
    

    fn generate_trace(&self, input: &Self::Record, output: &mut Self::Record) -> RowMajorMatrix<F> {
        let mut rows = Vec::new();

        let mut new_byte_lookup_events = Vec::new();
        let mut alu_events_vec = Vec::new();
         (rows,new_byte_lookup_events,alu_events_vec)=multiunzip( input.rwasm_binop_events.iter().map(|event:&BinOp32Event|
                        {
                        self.event_to_row::<F>(input,event)        
                        }));
                
        output.add_byte_lookup_events(new_byte_lookup_events.into_iter().flatten().collect());
        let mut alu_hash_map = HashMap::new();
       for item in  alu_events_vec.into_iter(){
            for alu_event in item.into_iter(){
                 alu_hash_map.entry(alu_event.opcode)
                .and_modify(|op_new_events: &mut Vec<AluEvent>| {
                    op_new_events.push(alu_event)
                    
                }).or_insert(vec![alu_event]);
            }
        };    
        output.add_alu_events(alu_hash_map);
        let num_real_rows = rows.len();

        pad_rows(&mut rows, || [F::zero(); NUM_BINOP32_MEM_COLS]);
        // Convert the trace to a row major matrix.
        let mut trace = RowMajorMatrix::new(
            rows.into_iter().flatten().collect::<Vec<_>>(),
            NUM_BINOP32_MEM_COLS,
        );

        // Write the nonces to the trace.
        for i in 0..trace.height() {
            let cols: &mut BinOp32Cols<F> =
                trace.values[i * NUM_BINOP32_MEM_COLS..(i + 1) * NUM_BINOP32_MEM_COLS].borrow_mut();
            cols.nonce = F::from_canonical_usize(i);
        }

        trace
    }

    fn included(&self, shard: &Self::Record) -> bool {
        !shard.rwasm_binop_events.is_empty()
    }
}

impl BinOp32Chip {
    fn event_to_row <F: PrimeField32>(
        &self,
        input:&ExecutionRecord,
        event: &BinOp32Event,
    ) -> (
        [F; NUM_BINOP32_MEM_COLS],
        Vec<ByteLookupEvent>,
        Vec<AluEvent>,
    ) {
        
        let mut new_byte_lookup_events = Vec::new();
        let mut new_alu_events= Vec::new(); 
       
        let shard = event.shard;
        let channel = event.channel;
        let clk = event.clk;
        let mut row = [F::zero(); NUM_BINOP32_MEM_COLS];
        let cols: &mut BinOp32Cols<F> = row.as_mut_slice().borrow_mut();

        cols.shard=F::from_canonical_u32(shard);
        cols.channel=F::from_canonical_u32(channel);
        cols.clk=F::from_canonical_u32(clk);

        cols.opcode_memory_record.populate(channel,
             event.op_read_record,
             &mut new_byte_lookup_events);
      
        cols.op_addr = F::from_canonical_u32(event.op_addr);
        cols.rwasm_opcode = F::from_canonical_u32(event.opcode);

        cols.stack_ptr_addr = F::from_canonical_u32(event.stack_ptr_addr);
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
        cols.stack_ptr_addr = F::from_canonical_u32(event.stack_ptr_addr);
        cols.x_addr = Word::from(event.x_addr);
        cols.y_addr = Word::from(event.y_addr);

        cols.res = Word::from(event.res_val);
        cols.x_val=Word::from(event.x_val);
        cols.y_val=Word::from(event.y_val);
        cols.alu_event_nonce = F::from_canonical_u32(
            input
                .nonce_lookup
                .get(&event.alu_sub_lookups[0])
                .copied()
                .unwrap_or_default(),
        );
        println!{"input nounce lookup:{:?}",input.nonce_lookup};
        println!{"event sublookup:{} ",&event.alu_sub_lookups[0]};
        let op = RwasmOp::from_u32(event.opcode);
        
        cols.arith_selector.is_add = F::from_bool(false);
        cols.arith_selector.is_sub = F::from_bool(false);
        cols.arith_selector.is_mul = F::from_bool(false);
        cols.arith_selector.is_divu = F::from_bool(false);
        cols.arith_selector.is_divs = F::from_bool(false);
        cols.arith_selector.is_remu = F::from_bool(false);
        cols.arith_selector.is_rems = F::from_bool(false);
        cols.bitop_selector.is_and = F::from_bool(false);
        cols.bitop_selector.is_or = F::from_bool(false);
        cols.bitop_selector.is_xor = F::from_bool(false);
        cols.bitop_selector.is_shl = F::from_bool(false);
        cols.bitop_selector.is_shrs = F::from_bool(false);
        cols.bitop_selector.is_shru= F::from_bool(false);
        cols.bitop_selector.is_rotl= F::from_bool(false);
        cols.bitop_selector.is_rotr=  F::from_bool(false);
        

        // we choose one op and disable the rest.
        match op {
            RwasmOp::I32ADD => {
                cols.arith_selector.is_add = F::from_bool(true);
                cols.is_arith=F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(Opcode::ADD as u32); 
                
                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::ADD,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})
            }
            RwasmOp::I32SUB => {
                cols.arith_selector.is_sub = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(Opcode::SUB as u32); 

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::ADD,
                     a: event.x_val, 
                     b: event.y_val,
                     c: event.res_val, 
                     sub_lookups:  create_alu_lookups()})
            }
            RwasmOp::I32MUL => {
                cols.arith_selector.is_mul= F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(30); // 30 is the enum value of Opcode::MUL

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::MUL,
                     a: event.x_val, 
                     b: event.y_val,
                     c: event.res_val, 
                     sub_lookups:  create_alu_lookups()})
            }
            RwasmOp::I32DIVS => {
                cols.arith_selector.is_divs= F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(34); // 34 is the enum value of Opcode::DIVS

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::DIV,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})

            }

            RwasmOp::I32DIVU=> {
                cols.arith_selector.is_divu = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(Opcode::DIVU as u32); // 34 is the enum value of Opcode::DIVU

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::DIVU,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})

            }
            RwasmOp::I32REMS => {
                cols.arith_selector.is_rems = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(Opcode::REM as u32); // 34 is the enum value of Opcode::DIVU

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::REM,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})
            },
            RwasmOp::I32REMU=> {
                cols.arith_selector.is_rems = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(37); 

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::REMU,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})
            },
            RwasmOp::I32AND => {
                cols.bitop_selector.is_and = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(37); 

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::REMU,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})
            },
            RwasmOp::I32OR => {
                cols.bitop_selector.is_or = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(3); 

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::OR,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})
            },
            RwasmOp::I32XOR => {
                cols.bitop_selector.is_xor = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(3); 

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::XOR,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})
            },
            RwasmOp::I32SHL => {
                cols.bitop_selector.is_shl = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(3); 

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::SLL,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})
            },
            RwasmOp::I32SHRS => {
                cols.bitop_selector.is_shrs = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(3); 

                new_alu_events.push(AluEvent{ 
                    lookup_id: event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::SRL,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})
            },
            RwasmOp::I32SHRU => {
                cols.bitop_selector.is_shrs = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(3); 

                new_alu_events.push(AluEvent{ 
                    lookup_id:  event.alu_sub_lookups[0],
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::SRA,
                     a: event.res_val, 
                     b: event.x_val,
                     c: event.y_val, 
                     sub_lookups:  create_alu_lookups()})
            },
            RwasmOp::I32ROTL => todo!(),
            RwasmOp::I32ROTR => todo!(),
        }

      

        cols.pre_stack_ptr_val = Word::from(event.pre_stack_ptr_val);
        cols.post_stack_ptr_val = Word::from(event.post_stack_ptr_val);

        // cols.y_write_record.populate(
        //     channel,
        //     event.res_write_records,
        //     &mut new_byte_lookup_events,
        // );

        // cols.stack_ptr_write_record.populate(
        //     channel,
        //     event.stack_ptr_write_record,
        //     &mut new_byte_lookup_events,
        // );

        cols.is_real = F::from_bool(true);
        (row, new_byte_lookup_events,new_alu_events)
    }
}
