#[cfg(test)]
pub mod tests{
    use std::collections::BTreeMap;

    use super::super::*;
    use crate::{
        runtime::Register,
        utils::{
            tests::{FIBONACCI_ELF, PANIC_ELF, SSZ_WITHDRAWALS_ELF},
            SP1CoreOpts,
        },
    };
    use crate::syscall::precompiles::rwasm::RwasmOp;

    use super::super::{Instruction, Opcode, Program, Runtime};


     pub fn simple_program() -> Program {
        let instructions = vec![
            Instruction::new(Opcode::ADD, 29, 0, 5, false, true),
            Instruction::new(Opcode::ADD, 30, 0, 37, false, true),
            Instruction::new(Opcode::ADD, 31, 30, 29, false, false),
        ];
        Program::new(instructions, 0, 0)
    }   
    #[test]
    fn test_i32add() {
        /*
          let t0 = Register::X5;
                let syscall_id = self.register(t0);
                c = self.rr(Register::X11, MemoryAccessPosition::C);
                b = self.rr(Register::X10, MemoryAccessPosition::B);
                let syscall = SyscallCode::from_u32(syscall_id);
         */

        let sp_addr:u32= 0x00_00_10_00;
        let sp_value:u32 = 0x00_00_20_00;
        let x_value:u32 = 0x11;
        let y_value:u32 = 0x23;
        let op_addr = 0x00_00_30_00;
        let mut mem= BTreeMap::new();
        mem.insert(sp_addr, sp_value);
        mem.insert(sp_value, x_value);
        mem.insert(sp_value-4, y_value);
        mem.insert(op_addr, 106);
        println!("{:?}",mem);
        let instructions = vec![
            Instruction::new(Opcode::ADD, 5, 0, 0x00_00_02_01, false, true),
            Instruction::new(Opcode::ADD, 11, 0, sp_addr, false, true),
            Instruction::new(Opcode::ADD, 10, 0, op_addr, false, true),
            Instruction::new(Opcode::ECALL, 0, 0, 0, false, true),
        ];
        let program = Program{instructions:instructions,
             pc_base:0,
             pc_start:0,
              
            memory_image: mem, };

        let mut runtime = Runtime::new(program, SP1CoreOpts::default());
        runtime.run().unwrap();
        assert_eq!(runtime.register(Register::X5), 0x11+0x23);
    }
}