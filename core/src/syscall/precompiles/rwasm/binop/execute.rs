use p3_keccak_air::{NUM_ROUNDS, RC};
use typenum::False;


use crate::{
    runtime::{self, Syscall},
    syscall::precompiles::{keccak256::KeccakPermuteEvent, SyscallContext},
};

use super::{columns::{I32_LEN, I64_LEN}, BinOpChip, BinOpEvent, RwasmBinOp};

const RHO: [u32; 24] = [
    1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44,
];

const PI: [usize; 24] = [
    10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1,
];

impl Syscall for BinOpChip {
    fn num_extra_cycles(&self) -> u32 {
        1
    }

    fn execute(&self, rt: &mut SyscallContext, arg1: u32, arg2: u32) -> Option<u32> {
        let start_clk = rt.clk;
        let op_addr = arg1;
        let stack_ptr_addr = arg2;
       

        //we read the binary_op from the syscall address
        let (op_read_record,opcode)= rt.mr(arg1);
        let op = RwasmBinOp::from_u32(opcode);
        let (stack_ptr_read_record,stack_ptr_val) = rt.mr(arg2);


        let (x_read_records,y_read_records,x_val_vec,y_val_vec,res)= match  op.is_i32_op(){
            true => {
                let (x_memory_read_record,x_val_vec)= rt.mr_slice(stack_ptr_val, I32_LEN);

                let (y_memory_read_record,y_val_vec)= rt.mr_slice(stack_ptr_val-I32_LEN as u32, I32_LEN);
                match op{
                    RwasmBinOp::I32Add => {
                        let signed_x = x_val_vec[0] as i32;
                        let signed_y = y_val_vec[0] as i32;
                       (x_memory_read_record,y_memory_read_record,x_val_vec,y_val_vec,(signed_x+signed_y) as u64)
                    },
                    RwasmBinOp::I32Sub => {
                        let signed_x = x_val_vec[0] as i32;
                        let signed_y = y_val_vec[0] as i32;
                        (x_memory_read_record,y_memory_read_record,x_val_vec,y_val_vec,(signed_x - signed_y) as u64)
                    }

                    
                }
            },
            false => {
                todo!()
            },
        };
       
    
       

        
        rt.clk += 1;
        let mut values_to_write = Vec::new();
        match op.is_i32_op(){
            true=>{
                
                values_to_write.push(res as u32);
            }
            false=>{
                values_to_write.push(res as u32);
                values_to_write.push((res >> 32) as u32);
            }
        }

        let stack_change = match op.is_i32_op() {
            true=>I32_LEN,
            false=>I64_LEN,
        };
        let new_stack_ptr_val = stack_ptr_val-stack_change as u32;
        let stack_ptr_write_record = rt.mw(stack_ptr_addr,new_stack_ptr_val);

        let res_write_records = rt.mw_slice(new_stack_ptr_val, values_to_write.as_slice());
        
       
        // Push the Keccak permute event.
        let shard = rt.current_shard();
        let channel = rt.current_channel();
        let lookup_id = rt.syscall_lookup_id;
        rt.record_mut()
            .rwasm_binop_events
            .push(BinOpEvent {
                lookup_id,
                shard,
                channel,
                clk: start_clk,
                opcode,
                is_i32: op.is_i32_op(),
                stack_ptr_addr,
                pre_stack_ptr_val: stack_ptr_val,
                post_stack_ptr_val: new_stack_ptr_val,
                x_val: x_val_vec,
                y_val: y_val_vec,
                res_val: values_to_write,
                op_read_record,
                x_read_records,
                y_read_records,
                stack_ptr_read_record,
                stack_ptr_write_record,
                res_write_records,  
                x_addr:stack_ptr_val,
                y_addr:stack_ptr_val-I32_LEN as u32,
                op_addr,
            });

        None
    }
}

