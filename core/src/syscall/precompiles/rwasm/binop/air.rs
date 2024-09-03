use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_keccak_air::{KeccakAir, NUM_KECCAK_COLS, NUM_ROUNDS, U64_LIMBS};
use p3_matrix::Matrix;
use super::{columns::{BinOpCols, I32_LEN, NUM_BINOP_MEM_COLS}, BinOpChip};
use crate::{
    air::{SP1AirBuilder, SubAirBuilder, WordAirBuilder}, memory::MemoryCols, operations::AddOperation, runtime::SyscallCode
};

impl<F> BaseAir<F> for BinOpChip {
    fn width(&self) -> usize {
        NUM_BINOP_MEM_COLS
    }
}

impl<AB> Air<AB> for BinOpChip
where
    AB: SP1AirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let (local, next) = (main.row_slice(0), main.row_slice(1));
        let local: &BinOpCols<AB::Var> = (*local).borrow();
        let next: &BinOpCols<AB::Var> = (*next).borrow();

        // Constrain the incrementing nonce.
        builder.when_first_row().assert_zero(local.nonce);
        builder
            .when_transition()
            .assert_eq(local.nonce + AB::Expr::one(), next.nonce);


        //assert that the pre_stack_ptr_val is indeed post_stack_ptr_val -4 * LEN
        builder.assert_eq(local.pre_stack_ptr_val.reduce::<AB>(),local.post_stack_ptr_val.reduce::<AB>()+AB::Expr::one());
      
        //assert that the x addr and y_addr are different in 4* LEN
        builder.assert_eq(local.x_addr.reduce::<AB>(), local.y_addr.reduce::<AB>()-AB::Expr::from_canonical_usize(I32_LEN));

     
        //assert the compuation of arg1+arg2
        AddOperation::<AB::F>::eval(
            builder,
            local.x_val,
            local.y_val,
            local.res,
            local.shard,
            local.channel,
            local.is_real.into(),
        );
        //assert memory operations
        self.eval_memory(builder, local);
        
        builder.receive_syscall(
            local.shard,
            local.channel,
            local.clk,
            local.nonce,
            AB::F::from_canonical_u32(SyscallCode::SHA_COMPRESS.syscall_id()),
            local.opcode,
            local.stack_ptr_addr,
            local.is_real,
        );

        
    }
}

impl BinOpChip

{
    fn eval_memory<AB: SP1AirBuilder>(&self, builder: &mut AB, local: &BinOpCols<AB::Var>){
        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk,
            local.stack_ptr_addr,
            &local.stack_ptr_record,
            local.is_real,
        );

        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk+AB::F::from_canonical_u32(1),
            local.stack_ptr_addr,
            &local.stack_ptr_write_record,
            local.is_real,
        );

        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk,
            local.x_addr.reduce::<AB>(),
            &local.x_memory_record[0],
            local.is_real,
        );

        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk,
            local.y_addr.reduce::<AB>(),
            &local.y_memory_record[0],
            local.is_real,
        );

        //assert that the x_val has not change after read
        builder.assert_word_eq(local.x_val,*local.x_memory_record[0].prev_value());
        builder.assert_word_eq(local.x_val,*local.x_memory_record[0].value());

        //assert that the y_val has not change after read
        builder.assert_word_eq(local.y_val,*local.y_memory_record[0].prev_value());
        builder.assert_word_eq(local.y_val,*local.y_memory_record[0].value());

        //assert that the stack_ptr_val has not change after read
        builder.assert_word_eq(local.pre_stack_ptr_val,*local.stack_ptr_record.prev_value());
        builder.assert_word_eq(local.pre_stack_ptr_val,*local.stack_ptr_record.value());

        // assert writing result into memoery 
        // assert that before writing, this memory address hold arg2 value
        builder.assert_word_eq(local.y_val, *local.y_write_record[0].prev_value());
        
        builder.assert_word_eq(local.res.value,*local.y_write_record[0].value());

        //assert that the correct result of post_stack_ptr_val  has been write into memory
        // assert that before writing, this memory address holds stack_ptr_val
        builder.assert_word_eq(local.pre_stack_ptr_val, *local.stack_ptr_write_record.prev_value());
        //assert that the correct result of arg1+arg2 has been write into memory
        builder.assert_word_eq(local.post_stack_ptr_val, *local.stack_ptr_write_record.value());
        
        
    } 
}