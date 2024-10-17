use core::borrow::Borrow;
use std::borrow::BorrowMut;

use super::{
    columns::{BinOp32Cols, NUM_BINOP32_MEM_COLS}, trace::{BYTE_MASK, PRODUCT_SIZE}, BinOp32Chip
};
use crate::{
    air::{SP1AirBuilder, SubAirBuilder, WordAirBuilder, WORD_SIZE}, bytes::ByteOpcode, memory::MemoryCols, operations::AddOperation, runtime::{Opcode, SyscallCode}, syscall::precompiles::rwasm::RwasmOp
};
use num::zero;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_keccak_air::{KeccakAir, NUM_KECCAK_COLS, NUM_ROUNDS, U64_LIMBS};
use p3_matrix::Matrix;



use core::mem::size_of;


use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::ParallelIterator;
use p3_maybe_rayon::prelude::ParallelSlice;
use sp1_derive::AlignedBorrow;

impl<F> BaseAir<F> for BinOp32Chip {
    fn width(&self) -> usize {
        NUM_BINOP32_MEM_COLS
    }
}

impl<AB> Air<AB> for BinOp32Chip
where
    AB: SP1AirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let (local, next) = (main.row_slice(0), main.row_slice(1));
        let local: &BinOp32Cols<AB::Var> = (*local).borrow();
        let next: &BinOp32Cols<AB::Var> = (*next).borrow();

        // Constrain the incrementing nonce.
        builder.when_first_row().assert_zero(local.nonce);
        builder
            .when_transition()
            .assert_eq(local.nonce + AB::Expr::one(), next.nonce);

        //assert that the pre_stack_ptr_val is indeed post_stack_ptr_val + 4 
        builder.when(local.is_real).
        assert_eq(
            local.pre_stack_ptr_val.reduce::<AB>(),
            local.post_stack_ptr_val.reduce::<AB>() + AB::Expr::from_canonical_usize(8),
        );

        //assert that the x addr and y_addr are different in 4
        builder.when(local.is_real).
            assert_eq(
            local.x_addr.reduce::<AB>(),
            local.y_addr.reduce::<AB>() + AB::Expr::from_canonical_usize(4),
        );

        // Instead of wrting constraint for rwasm op we simply use the sp1 alu to  do the job.
        // note that we have to generate sp1 alu event in generate dependencies.
        builder.
        send_alu(local.riscv_opcode,
             local.res, 
             local.x_val, 
            local.y_val,
             local.shard,
             local.channel,
              local.alu_event_nonce, 
            local.is_arith);
        
        
        
        
        //assert memory operations
        self.eval_memory(builder, local);

        builder.receive_syscall(
            local.shard,
            local.channel,
            local.clk,
            local.nonce,
            AB::F::from_canonical_u32(SyscallCode::RWASM_BINOP.syscall_id()),
            local.op_addr,
            local.stack_ptr_addr,
            local.is_real,
        );
    }
}

impl BinOp32Chip {
    fn eval_memory<AB: SP1AirBuilder>(&self, builder: &mut AB, local: &BinOp32Cols<AB::Var>) {
        let mut clk_counter = 0usize;
        //read op_code
        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk,
            local.op_addr,
            &local.opcode_memory_record,
            local.is_real,
        );
        // read stack_ptr
        clk_counter+=1;
        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk+AB::F::from_canonical_usize(clk_counter),
            local.stack_ptr_addr,
            &local.stack_ptr_record,
            local.is_real,
        );
        //read x
        clk_counter+=1;
        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk +AB::F::from_canonical_usize(clk_counter),
            local.x_addr.reduce::<AB>(),
            &local.x_memory_record,
            local.is_real,
        );

        //read y
        clk_counter+=1;
        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk + AB::F::from_canonical_usize(clk_counter),
            local.y_addr.reduce::<AB>(),
            &local.y_memory_record,
            local.is_real,
        );

        // write res
        clk_counter+=1;
       
        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk+ AB::F::from_canonical_usize(clk_counter),
            local.post_stack_ptr_val.reduce::<AB>(),
            &local.y_write_record,
            local.is_real,
        );

        // //write stack pointer
        // clk_counter+=1;
        // builder.eval_memory_access(
        //     local.shard,
        //     local.channel,
        //     local.clk + AB::F::from_canonical_usize(clk_counter),
        //     local.stack_ptr_addr,
        //     &local.stack_ptr_write_record,
        //     local.is_real,
        // );

        // // assert that the x_val has not change after read
        // builder.when(local.is_real).assert_word_eq(local.x_val, *local.x_memory_record.prev_value());
        // builder.when(local.is_real).assert_word_eq(local.x_val, *local.x_memory_record.value());

        // //assert that the y_val has not change after read
        // builder.when(local.is_real).assert_word_eq(local.y_val, *local.y_memory_record.prev_value());
        // builder.when(local.is_real).assert_word_eq(local.y_val, *local.y_memory_record.value());

        // //assert that the stack_ptr_val has not change after read
        // builder.when(local.is_real).assert_word_eq(
        //     local.pre_stack_ptr_val,
        //     *local.stack_ptr_record.prev_value(),
        // );
        // builder.when(local.is_real).assert_word_eq(local.pre_stack_ptr_val, *local.stack_ptr_record.value());

        // // // assert writing result into memoery
        // // assert that before writing, this memory address hold arg2 value
        // builder.assert_word_eq(local.y_val, *local.y_write_record.prev_value());

        // builder.assert_word_eq(local.res, *local.y_write_record.value());

        // //assert that the correct result of post_stack_ptr_val  has been write into memory
        // // assert that before writing, this memory address holds stack_ptr_val
        // builder.assert_word_eq(
        //     local.pre_stack_ptr_val,
        //     *local.stack_ptr_write_record.prev_value(),
        // );
        // //assert that the correct result of arg1+arg2 has been write into memory
        // builder.assert_word_eq(
        //     local.post_stack_ptr_val,
        //     *local.stack_ptr_write_record.value(),
        // );
    }

}
