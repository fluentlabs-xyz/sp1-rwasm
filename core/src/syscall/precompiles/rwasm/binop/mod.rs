mod trace;
mod air;
mod execute;
mod opair;
pub mod columns;





use p3_keccak_air::KeccakAir;
use serde::{Deserialize, Serialize};

use crate::runtime::{MemoryReadRecord, MemoryWriteRecord};


pub(crate) const I32_LEN :usize = 1;
pub(crate) const I64_LEN :usize = 2;

//This probably not going to work for i64 bin ops. we will need another series of struct to save i64 bin ops.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinOpEvent {
    pub lookup_id: usize,
    pub shard: u32,
    pub channel: u32,
    pub clk: u32,

    pub opcode:u32,
    pub is_i32:bool,
    
    pub pre_stack_ptr_val:u32,
    pub post_stack_ptr_val:u32,
    pub x_val:Vec<u32>,
    pub y_val:Vec<u32>,
    pub res_val:Vec<u32>,

    pub x_addr:u32,
    pub y_addr:u32,
    pub op_addr:u32,
    

    pub op_read_record:MemoryReadRecord,
    pub x_read_records:Vec<MemoryReadRecord>,
    pub y_read_records:Vec<MemoryReadRecord>,
    pub stack_ptr_read_record:MemoryReadRecord,
    pub stack_ptr_write_record:MemoryWriteRecord,
    pub res_write_records:Vec<MemoryWriteRecord>,
    pub stack_ptr_addr: u32,
    
}

pub struct BinOpChip {
    
}

pub enum RwasmBinOp{
    I32Add = 106,
    I32Sub = 107,
}
impl RwasmBinOp{
    pub fn from_u32(op_code:u32)->RwasmBinOp{
        match op_code {
            106=>RwasmBinOp::I32Add,
            107=>RwasmBinOp::I32Sub,
            _=>unreachable!(),
        }
    }
    pub fn is_i32_op(&self)->bool{
        match self {
            Self::I32Add |Self::I32Sub=>true,
            _ =>false,
        }
    }
}

impl BinOpChip {
    pub const fn new() -> Self {
       Self{}
    }
}
