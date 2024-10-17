mod air;
pub mod columns;
mod execute;
mod opair;
mod trace;

use p3_keccak_air::KeccakAir;
use serde::{Deserialize, Serialize};

use crate::runtime::{MemoryReadRecord, MemoryWriteRecord};

pub(crate) const I32_LEN: usize = 1;
pub(crate) const I64_LEN: usize = 2;

//This probably not going to work for i64 bin ops. we will need another series of struct to save i64 bin ops.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinOp32Event {
    pub lookup_id: usize,
    pub shard: u32,
    pub channel: u32,
    pub clk: u32,

    pub opcode: u32,
    pub op_addr: u32,
    pub op_read_record: MemoryReadRecord,


   

    pub x_addr: u32,
    pub x_val: u32,
    pub x_read_records: MemoryReadRecord,
    pub y_val: u32,
    pub y_addr: u32,
    pub y_read_records: MemoryReadRecord,
    pub res_val: u32,
    
    
    pub pre_stack_ptr_val: u32,
    pub post_stack_ptr_val: u32,
   
    pub stack_ptr_addr: u32,
    pub stack_ptr_read_record: MemoryReadRecord,

    pub res_write_records: MemoryWriteRecord,
    pub stack_ptr_write_record: MemoryWriteRecord,
    
    pub alu_sub_lookups:[usize;6],
}

pub struct BinOp32Chip {}



impl BinOp32Chip {
    pub const fn new() -> Self {
        Self {}
    }
}
