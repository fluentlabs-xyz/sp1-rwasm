use core::borrow::Borrow;
use std::borrow::BorrowMut;

use super::{
    columns::{BinOp32Cols, I32_LEN, NUM_BINOP_MEM_COLS}, trace::{BYTE_MASK, PRODUCT_SIZE}, BinOp32Chip
};
use crate::{
    air::{SP1AirBuilder, SubAirBuilder, WordAirBuilder, WORD_SIZE}, bytes::ByteOpcode, memory::MemoryCols, operations::AddOperation, runtime::{Opcode, SyscallCode}
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
        NUM_BINOP_MEM_COLS
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

        //assert that the pre_stack_ptr_val is indeed post_stack_ptr_val -4 * LEN
        builder.assert_eq(
            local.pre_stack_ptr_val.reduce::<AB>(),
            local.post_stack_ptr_val.reduce::<AB>() + AB::Expr::one(),
        );

        //assert that the x addr and y_addr are different in 4* LEN
        builder.assert_eq(
            local.x_addr.reduce::<AB>(),
            local.y_addr.reduce::<AB>() - AB::Expr::from_canonical_usize(I32_LEN),
        );

        //assert the compuation of arg1+arg2
        AddOperation::<AB::F>::eval(
            builder.when(local.is_add).borrow_mut(),
            local.x_val,
            local.y_val,
            local.addsubres,
            local.shard,
            local.channel,
            local.is_real.into(),
        );

            //assert the compuation of arg1+arg2
            AddOperation::<AB::F>::eval(
            builder.when(local.is_sub).borrow_mut(),
            local.addsubres.value,
            local.y_val,
            local.addsubres,
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

impl BinOp32Chip {
    fn eval_memory<AB: SP1AirBuilder>(&self, builder: &mut AB, local: &BinOp32Cols<AB::Var>) {
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
            local.clk + AB::F::from_canonical_u32(1),
            local.stack_ptr_addr,
            &local.stack_ptr_write_record,
            local.is_real,
        );

        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk,
            local.x_addr.reduce::<AB>(),
            &local.x_memory_record,
            local.is_real,
        );

        builder.eval_memory_access(
            local.shard,
            local.channel,
            local.clk,
            local.y_addr.reduce::<AB>(),
            &local.y_memory_record,
            local.is_real,
        );

        //assert that the x_val has not change after read
        builder.assert_word_eq(local.x_val, *local.x_memory_record.prev_value());
        builder.assert_word_eq(local.x_val, *local.x_memory_record.value());

        //assert that the y_val has not change after read
        builder.assert_word_eq(local.y_val, *local.y_memory_record.prev_value());
        builder.assert_word_eq(local.y_val, *local.y_memory_record.value());

        //assert that the stack_ptr_val has not change after read
        builder.assert_word_eq(
            local.pre_stack_ptr_val,
            *local.stack_ptr_record.prev_value(),
        );
        builder.assert_word_eq(local.pre_stack_ptr_val, *local.stack_ptr_record.value());

        // assert writing result into memoery
        // assert that before writing, this memory address hold arg2 value
        builder.assert_word_eq(local.y_val, *local.y_write_record.prev_value());

        builder.assert_word_eq(local.addsubres.value, *local.y_write_record.value());

        //assert that the correct result of post_stack_ptr_val  has been write into memory
        // assert that before writing, this memory address holds stack_ptr_val
        builder.assert_word_eq(
            local.pre_stack_ptr_val,
            *local.stack_ptr_write_record.prev_value(),
        );
        //assert that the correct result of arg1+arg2 has been write into memory
        builder.assert_word_eq(
            local.post_stack_ptr_val,
            *local.stack_ptr_write_record.value(),
        );
    }

    fn eval_mul<AB: SP1AirBuilder>(&self, builder: &mut AB, local: &BinOp32Cols<AB::Var>){
        let local = local.mulcols.clone();
        let base = AB::F::from_canonical_u32(1 << 8);

        let zero: AB::Expr = AB::F::zero().into();
        let one: AB::Expr = AB::F::one().into();
      
        let byte_mask = AB::F::from_canonical_u8(BYTE_MASK);
         // Calculate the MSBs.
         let (b_msb, c_msb) = {
            let msb_pairs = [
                (local.b_msb, local.b[WORD_SIZE - 1]),
                (local.c_msb, local.c[WORD_SIZE - 1]),
            ];
            let opcode = AB::F::from_canonical_u32(ByteOpcode::MSB as u32);
            for msb_pair in msb_pairs.iter() {
                let msb = msb_pair.0;
                let byte = msb_pair.1;
                builder.send_byte(
                    opcode,
                    msb,
                    byte,
                    zero.clone(),
                    local.shard,
                    local.channel,
                    local.is_real,
                );
            }
            (local.b_msb, local.c_msb)
        };

        // Calculate whether to extend b and c's sign.
        let (b_sign_extend, c_sign_extend) = {
            // MULH or MULHSU
            let is_b_i32 = local.is_mulh + local.is_mulhsu - local.is_mulh * local.is_mulhsu;

            let is_c_i32 = local.is_mulh;

            builder.assert_eq(local.b_sign_extend, is_b_i32 * b_msb);
            builder.assert_eq(local.c_sign_extend, is_c_i32 * c_msb);
            (local.b_sign_extend, local.c_sign_extend)
        };

        // Sign extend local.b and local.c whenever appropriate.
        let (b, c) = {
            let mut b: Vec<AB::Expr> = vec![AB::F::zero().into(); PRODUCT_SIZE];
            let mut c: Vec<AB::Expr> = vec![AB::F::zero().into(); PRODUCT_SIZE];
            for i in 0..PRODUCT_SIZE {
                if i < WORD_SIZE {
                    b[i] = local.b[i].into();
                    c[i] = local.c[i].into();
                } else {
                    b[i] = b_sign_extend * byte_mask;
                    c[i] = c_sign_extend * byte_mask;
                }
            }
            (b, c)
        };

        // Compute the uncarried product b(x) * c(x) = m(x).
        let mut m: Vec<AB::Expr> = vec![AB::F::zero().into(); PRODUCT_SIZE];
        for i in 0..PRODUCT_SIZE {
            for j in 0..PRODUCT_SIZE {
                if i + j < PRODUCT_SIZE {
                    m[i + j] += b[i].clone() * c[j].clone();
                }
            }
        }

        // Propagate carry.
        let product = {
            for i in 0..PRODUCT_SIZE {
                if i == 0 {
                    builder.assert_eq(local.product[i], m[i].clone() - local.carry[i] * base);
                } else {
                    builder.assert_eq(
                        local.product[i],
                        m[i].clone() + local.carry[i - 1] - local.carry[i] * base,
                    );
                }
            }
            local.product
        };

        // Compare the product's appropriate bytes with that of the result.
        {
            let is_lower = local.is_mul;
            let is_upper = local.is_mulh + local.is_mulhu + local.is_mulhsu;
            for i in 0..WORD_SIZE {
                builder.when(is_lower).assert_eq(product[i], local.a[i]);
                builder
                    .when(is_upper.clone())
                    .assert_eq(product[i + WORD_SIZE], local.a[i]);
            }
        }

        // Check that the boolean values are indeed boolean values.
        {
            let booleans = [
                local.b_msb,
                local.c_msb,
                local.b_sign_extend,
                local.c_sign_extend,
                local.is_mul,
                local.is_mulh,
                local.is_mulhu,
                local.is_mulhsu,
                local.is_real,
            ];
            for boolean in booleans.iter() {
                builder.assert_bool(*boolean);
            }
        }

        // If signed extended, the MSB better be 1.
        builder
            .when(local.b_sign_extend)
            .assert_eq(local.b_msb, one.clone());
        builder
            .when(local.c_sign_extend)
            .assert_eq(local.c_msb, one.clone());

        // Calculate the opcode.
        let opcode = {
            // Exactly one of the op codes must be on.
            builder
                .when(local.is_real)
                .assert_one(local.is_mul + local.is_mulh + local.is_mulhu + local.is_mulhsu);

            let mul: AB::Expr = AB::F::from_canonical_u32(Opcode::MUL as u32).into();
            let mulh: AB::Expr = AB::F::from_canonical_u32(Opcode::MULH as u32).into();
            let mulhu: AB::Expr = AB::F::from_canonical_u32(Opcode::MULHU as u32).into();
            let mulhsu: AB::Expr = AB::F::from_canonical_u32(Opcode::MULHSU as u32).into();
            local.is_mul * mul
                + local.is_mulh * mulh
                + local.is_mulhu * mulhu
                + local.is_mulhsu * mulhsu
        };

        // Range check.
        {
            // Ensure that the carry is at most 2^16. This ensures that
            // product_before_carry_propagation - carry * base + last_carry never overflows or
            // underflows enough to "wrap" around to create a second solution.
            builder.slice_range_check_u16(&local.carry, local.shard, local.channel, local.is_real);

            builder.slice_range_check_u8(&local.product, local.shard, local.channel, local.is_real);
        }

        // Receive the arguments.
        builder.receive_alu(
            opcode,
            local.a,
            local.b,
            local.c,
            local.shard,
            local.channel,
            local.nonce,
            local.is_real,
        );   
    }
}
