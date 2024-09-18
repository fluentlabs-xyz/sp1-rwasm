use core::mem::size_of;

use p3_keccak_air::KeccakCols;
use sp1_derive::AlignedBorrow;

use crate::{
    air::Word,
    alu::MulCols,
    memory::{MemoryReadCols, MemoryReadWriteCols, MemoryWriteCols},
    operations::AddOperation,
    runtime::MemoryWriteRecord,
};
//the length of a word of rwasm in RISCV word.
pub const I32_LEN: usize = 1;
pub const I64_LEN: usize = 2;

#[macro_rules_attribute::apply(crate::append_rwasm_selectors)]
///  BinOpCols
/// The columns defined in the `p3_keccak_air` crate are embedded here as `keccak`. Other columns
/// are used to track the VM context.
#[derive(AlignedBorrow)]
#[repr(C)]
pub(crate) struct BinOp32Cols<T> {
    /// Keccak columns from p3_keccak_air. Note it is assumed in trace gen to be the first field.
    pub shard: T,
    pub channel: T,
    pub clk: T,
    pub nonce: T,

    pub rwasm_opcode: T,
    pub riscv_opcode: T,
    pub stack_ptr_addr: T,

    pub x_addr: Word<T>,
    pub y_addr: Word<T>,

    pub x_val: Word<T>,
    pub y_val: Word<T>,
    pub pre_stack_ptr_val: Word<T>,
    pub post_stack_ptr_val: Word<T>,
    pub res: Word<T>,
    pub x_memory_record: MemoryReadCols<T>,
    pub y_memory_record: MemoryReadCols<T>,
    pub stack_ptr_record: MemoryReadCols<T>,
    pub y_write_record: MemoryWriteCols<T>,
    pub stack_ptr_write_record: MemoryWriteCols<T>,
    pub is_real: T,
}

pub const NUM_BINOP32_MEM_COLS: usize = size_of::<BinOp32Cols<u8>>();
