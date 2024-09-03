use core::mem::size_of;

use p3_keccak_air::KeccakCols;
use sp1_derive::AlignedBorrow;

use crate::{air::Word, memory::{MemoryReadCols, MemoryReadWriteCols, MemoryWriteCols}, operations::AddOperation, runtime::MemoryWriteRecord};
//the length of a word of rwasm in RISCV word.
pub const I32_LEN: usize =1;
pub const I64_LEN: usize =2;

///  BinOpCols
/// The columns defined in the `p3_keccak_air` crate are embedded here as `keccak`. Other columns
/// are used to track the VM context.
#[derive(AlignedBorrow)]
#[repr(C)]
pub(crate) struct BinOpCols<T> {
    /// Keccak columns from p3_keccak_air. Note it is assumed in trace gen to be the first field.


    pub shard: T,
    pub channel: T,
    pub clk: T,
    pub nonce: T,


    pub opcode: T,
    pub stack_ptr_addr: T,

    pub x_addr: Word<T>,
    pub y_addr: Word<T>,
    
    pub x_val :Word<T>,
    pub y_val :Word<T>, 
    pub pre_stack_ptr_val: Word<T>,
    pub post_stack_ptr_val: Word<T>,
    pub res :AddOperation<T>,
    pub x_memory_record :[MemoryReadCols<T>;I64_LEN],
    pub y_memory_record :[MemoryReadCols<T>;I64_LEN],
    pub stack_ptr_record:MemoryReadCols<T>,
    pub y_write_record:[MemoryWriteCols<T>;I64_LEN],
    pub stack_ptr_write_record:MemoryWriteCols<T>,
    pub is_real :T,

   
}

pub const NUM_BINOP_MEM_COLS: usize = size_of::<BinOpCols<u8>>();
