mod air;

pub mod ops;

pub use air::*;

mod air2;
pub mod columns;
mod execute;
//mod opair;
pub mod trace;

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

    pub pre_stack_ptr_val: u32,
    pub post_stack_ptr_val: u32,
    pub x_val: u32,
    pub y_val: u32,
    pub res_val: u32,

    pub x_addr: u32,
    pub y_addr: u32,
    pub op_addr: u32,
    pub stack_ptr_addr: u32,

    pub op_read_record: MemoryReadRecord,
    pub x_read_records: MemoryReadRecord,
    pub y_read_records: MemoryReadRecord,
    pub stack_ptr_read_record: MemoryReadRecord,
    pub stack_ptr_write_record: MemoryWriteRecord,
    pub res_write_records: MemoryWriteRecord,
}

pub struct BinOp32Chip {}

impl BinOp32Chip {
    pub const fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {

    /*
        use crate::operations::field::params::FieldParameters;
        use crate::{
            io::SP1Stdin,
            runtime::Program,
            utils::{
                self,
                ec::{uint256::U256Field, utils::biguint_from_limbs},
                run_test_io,
                tests::UINT256_MUL_ELF,
            },
        };

        #[test]
        fn test_uint256_mul() {
            utils::setup_logger();
            let program = Program::from(UINT256_MUL_ELF);
            run_test_io(program, SP1Stdin::new()).unwrap();
        }

        #[test]
        fn test_uint256_modulus() {
            assert_eq!(biguint_from_limbs(U256Field::MODULUS), U256Field::modulus());
        }
    */
}
