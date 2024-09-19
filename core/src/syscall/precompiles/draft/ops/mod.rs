pub use macro_rules_attribute::apply;

use super::*;
use crate::alu::create_alu_lookup_id;
use crate::alu::create_alu_lookups;
use crate::alu::AluEvent;
use crate::runtime::Opcode;
use execute::{OpcodeExecute, OpcodeExecuteBuilder};
use p3_field::PrimeField32;
use trace::{OpcodeTrace, OpcodeTraceBuilder};

macro_rules! skip {
    ($($t:tt)*) => {};
}

pub mod macros;
macros::use_automod!();
