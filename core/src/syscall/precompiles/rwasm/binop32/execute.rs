use p3_keccak_air::{NUM_ROUNDS, RC};
use typenum::False;

use crate::{
    runtime::{self, Syscall},
    syscall::precompiles::{keccak256::KeccakPermuteEvent, SyscallContext},
};

use super::{
    super::RwasmOp,
    columns::{I32_LEN, I64_LEN},
    BinOp32Chip, BinOp32Event,
};

impl Syscall for BinOp32Chip {
    fn num_extra_cycles(&self) -> u32 {
        1
    }

    fn execute(&self, rt: &mut SyscallContext, arg1: u32, arg2: u32) -> Option<u32> {
        let start_clk = rt.clk;
        let op_addr = arg1;
        let stack_ptr_addr = arg2;

        //we read the binary_op from the syscall address
        let (op_read_record, opcode) = rt.mr(arg1);
        let op = RwasmOp::from_u32(opcode);
        let (stack_ptr_read_record, stack_ptr_val) = rt.mr(arg2);

        let (x_read_records, y_read_records, x_val, y_val, res) = {
            let (x_memory_read_record, x_val) = rt.mr(stack_ptr_val);

            let (y_memory_read_record, y_val) = rt.mr(stack_ptr_val - 1);
            match op {
                RwasmOp::I32Add => {
                    let signed_x = x_val as i32;
                    let signed_y = y_val as i32;
                    (
                        x_memory_read_record,
                        y_memory_read_record,
                        x_val,
                        y_val,
                        (signed_x.wrapping_add( signed_y)),
                    )
                }
                RwasmOp::I32Sub => {
                    let signed_x = x_val as i32;
                    let signed_y = y_val as i32;
                    (
                        x_memory_read_record,
                        y_memory_read_record,
                        x_val,
                        y_val,
                        (signed_x.wrapping_sub( signed_y)),
                    )
                }
                RwasmOp::I32Mul => {
                    let signed_x = x_val as i32;
                    let signed_y = y_val as i32;
                    (
                        x_memory_read_record,
                        y_memory_read_record,
                        x_val,
                        y_val,
                        (signed_x.wrapping_mul(signed_y)),
                    )
                },
                RwasmOp::I32DiV =>{
                    let signed_x = x_val as i32;
                    let signed_y = y_val as i32;
                    (
                        x_memory_read_record,
                        y_memory_read_record,
                        x_val,
                        y_val,
                        (signed_x.wrapping_div(signed_y)),
                    )
                }

            

                
            }
        };

        rt.clk += 1;

     
        let new_stack_ptr_val = stack_ptr_val - I32_LEN as u32;
        let stack_ptr_write_record = rt.mw(stack_ptr_addr, new_stack_ptr_val);

        let res_write_records = rt.mw(new_stack_ptr_val, res as u32);

        // Push the Keccak permute event.
        let shard = rt.current_shard();
        let channel = rt.current_channel();
        let lookup_id = rt.syscall_lookup_id;
        rt.record_mut().rwasm_binop_events.push(BinOp32Event {
            lookup_id,
            shard,
            channel,
            clk: start_clk,
            opcode,
            is_i32: op.is_i32_op(),
            stack_ptr_addr,
            pre_stack_ptr_val: stack_ptr_val,
            post_stack_ptr_val: new_stack_ptr_val,
            x_val,
            y_val,
            res_val: res as u32,
            op_read_record,
            x_read_records,
            y_read_records,
            stack_ptr_read_record,
            stack_ptr_write_record,
            res_write_records,
            x_addr: stack_ptr_val,
            y_addr: stack_ptr_val - I32_LEN as u32,
            op_addr,
        });

        None
    }
}
