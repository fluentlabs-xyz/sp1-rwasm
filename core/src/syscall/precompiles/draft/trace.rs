use p3_field::PrimeField32;

use crate::air::{Word, WORD_SIZE};
use crate::alu::{
    self, create_alu_lookup_id, create_alu_lookups, AluEvent, MulCols, BYTE_SIZE, NUM_MUL_COLS,
};
use crate::bytes::{ByteLookupEvent, ByteOpcode};
use crate::memory::MemoryReadWriteCols;
use crate::syscall;
use crate::utils::pad_rows;
use crate::{air::MachineAir, runtime::ExecutionRecord};

use super::columns::{BinOp32Cols, NUM_BINOP32_MEM_COLS};
use super::BinOp32Chip;
use super::BinOp32Event;

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
        /*
                input.rwasm_binop_events.iter().map(|event:&BinOp32Event|
                                {
                                    self.event_to_row::<F>(event)
                                });
        */

        output.add_byte_lookup_events(new_byte_lookup_events);

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
        // !shard.rwasm_binop_events.is_empty()
        todo!()
    }
}

#[macro_rules_attribute::apply(crate::decl_rwasm_template)]
impl BinOp32Chip {
    fn event_to_row<F: PrimeField32>(
        &self,
        event: &BinOp32Event,
    ) -> ([F; NUM_BINOP32_MEM_COLS], Vec<ByteLookupEvent>) {
        {
            let mut new_byte_lookup_events = Vec::new();
            //let mut new_alu_events= Vec::new();

            let shard = event.shard;
            let channel = event.channel;

            let mut row = [F::zero(); NUM_BINOP32_MEM_COLS];

            let cols: &mut BinOp32Cols<F> = row.as_mut_slice().borrow_mut();

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
            cols.rwasm_opcode = F::from_canonical_u32(event.opcode);

            cols.stack_ptr_addr = F::from_canonical_u32(event.stack_ptr_addr);
            cols.x_addr = Word::from(event.x_addr);
            cols.y_addr = Word::from(event.y_addr);

            //let op = RwasmOp::from_u32(event.opcode);

            cols.is_add = F::from_bool(false);
            cols.is_sub = F::from_bool(false);
            cols.is_mul = F::from_bool(false);
            cols.is_divu = F::from_bool(false);
            cols.is_divs = F::from_bool(false);
            cols.is_remu = F::from_bool(false);
            cols.is_rems = F::from_bool(false);
        }

        {
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

            (row, new_byte_lookup_events)
        }
    }
}

/*
macro_rules! prepend_selectors {
  ([$($selector:ident)*] $macr:ident! { $cols:ident $($code:tt)* }) => { paste::paste! {
      $macr! {
          //let set_sectors_to_false = |cols| {
              $($cols.[<is_ $selector>] = F::from_bool(false);)*
          //};
          $($code)*
      }
  }};
  ($($rest:tt)*) => { $crate::rwasm_selectors! { prepend_selectors [$($rest)*] } };
}
*/

//#[macro_rules_attribute::apply(prepend_selectors)]

pub struct OpcodeTraceBuilder<'a, F: PrimeField32> {
    pub cols: &'a mut BinOp32Cols<F>,
    pub new_alu_events: &'a mut Vec<AluEvent>,
    pub shard: u32,
    pub channel: u32,
    pub event: &'a BinOp32Event,
}

impl<'a, F: PrimeField32> std::ops::Deref for OpcodeTraceBuilder<'a, F> {
    type Target = Self;
    fn deref(&self) -> &Self { self }
}

pub trait OpcodeTrace<'a, const OPCODE: &'static str, F: PrimeField32>
where
    OpcodeTraceBuilder<'a, F>: std::ops::Deref<Target = Self>,
{
    fn opcode_specific(self: &mut OpcodeTraceBuilder<'a, F>);
}

//impl OpcodeTrace<"I32ADD"> for

rwasm_template_impl_bin_op32_chip! { _cols

        //set_selectors_to_false(cols);

        /*
        // we choose one op and disable the rest.
        match op {
            RwasmOp::I32ADD => {

                cols.is_add = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(1); // 1 is the enum value of Opcode::Add
                new_alu_events.push(AluEvent{
                    lookup_id:  create_alu_lookup_id(),
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::ADD,
                     a: event.res_val,
                     b: event.x_val,
                     c: event.y_val,
                     sub_lookups:  create_alu_lookups()})

            }
            RwasmOp::I32SUB => {
                cols.is_sub = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(2); // 2 is the enum value of Opcode::SUB

                new_alu_events.push(AluEvent{
                    lookup_id:  create_alu_lookup_id(),
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::ADD,
                     a: event.x_val,
                     b: event.y_val,
                     c: event.res_val,
                     sub_lookups:  create_alu_lookups()})
            }
            RwasmOp::I32MUL => {
                cols.is_mul = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(30); // 30 is the enum value of Opcode::MUL

                new_alu_events.push(AluEvent{
                    lookup_id:  create_alu_lookup_id(),
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::MUL,
                     a: event.x_val,
                     b: event.y_val,
                     c: event.res_val,
                     sub_lookups:  create_alu_lookups()})
            }
            RwasmOp::I32DIVS => {
                cols.is_divs = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(34); // 34 is the enum value of Opcode::DIVS

                new_alu_events.push(AluEvent{
                    lookup_id:  create_alu_lookup_id(),
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::DIV,
                     a: event.res_val,
                     b: event.x_val,
                     c: event.y_val,
                     sub_lookups:  create_alu_lookups()})

            }

            RwasmOp::I32DIVU=> {
                cols.is_divu = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(35); // 34 is the enum value of Opcode::DIVU

                new_alu_events.push(AluEvent{
                    lookup_id:  create_alu_lookup_id(),
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::DIVU,
                     a: event.res_val,
                     b: event.x_val,
                     c: event.y_val,
                     sub_lookups:  create_alu_lookups()})

            }
            RwasmOp::I32REMS => {
                cols.is_rems = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(36); // 34 is the enum value of Opcode::DIVU

                new_alu_events.push(AluEvent{
                    lookup_id:  create_alu_lookup_id(),
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::REM,
                     a: event.res_val,
                     b: event.x_val,
                     c: event.y_val,
                     sub_lookups:  create_alu_lookups()})
            },
            RwasmOp::I32REMU=> {
                cols.is_rems = F::from_bool(true);
                cols.riscv_opcode = F::from_canonical_u32(37); // 34 is the enum value of Opcode::DIVU

                new_alu_events.push(AluEvent{
                    lookup_id:  create_alu_lookup_id(),
                    shard, channel,
                    clk: event.clk,
                    opcode: Opcode::REMU,
                     a: event.res_val,
                     b: event.x_val,
                     c: event.y_val,
                     sub_lookups:  create_alu_lookups()})
            },
        }
        */


}

pub const fn get_msb(a: [u8; WORD_SIZE]) -> u8 {
    (a[WORD_SIZE - 1] >> (BYTE_SIZE - 1)) & 1
}
