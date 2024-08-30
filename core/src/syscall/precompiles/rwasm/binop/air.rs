use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_keccak_air::{KeccakAir, NUM_KECCAK_COLS, NUM_ROUNDS, U64_LIMBS};
use p3_matrix::Matrix;
use super::{columns::{BinOpCols, NUM_BINOP_MEM_COLS}, BinOpChip};
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
      
        self.

        AddOperation::<AB::F>::eval(
            builder,
            local.x_val,
            local.y_val,
            local.res,
            local.shard,
            local.channel,
            local.is_real.into(),
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
            local.x_addr,
            &local.x_memory_record[0],
            local.is_real,
        );

        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk,
            local.y_addr,
            &local.y_memory_record[0],
            local.is_real,
        );

        
    } 
}