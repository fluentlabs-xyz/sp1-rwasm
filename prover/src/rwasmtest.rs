#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::super::*;
    use sp1_core::{
        runtime::Register,
        utils::{
            SP1CoreOpts,
        },
    };
    use sp1_core::syscall::precompiles::rwasm::RwasmOp;

    use sp1_core::runtime::{Instruction, Opcode, Program, Runtime};
    use sp1_core::runtime::SyscallCode;

  

    use std::fs::File;
    use std::io::{Read, Write};
    use serde::{Deserialize, Serialize};
    use super::*;
    use super::super::*;
    use anyhow::Result;
    use build::try_build_plonk_bn254_artifacts_dev;
    use p3_field::PrimeField32;
    use serial_test::serial;
    use sp1_core::disassembler::Elf;
    use sp1_core::io::SP1Stdin;
    use sp1_core::utils::setup_logger;
    fn build_elf()->Program{
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
             //
            //  Instruction::new(Opcode::ADD, 29, 0,  sp_value, false, true),
            //  Instruction::new(Opcode::SW, 29, 0, sp_addr, false, true),

            //  Instruction::new(Opcode::ADD, 28, 0,  x_value, false, true),
            //  Instruction::new(Opcode::SW, 28, 0, sp_value, false, true),
            //  Instruction::new(Opcode::ADD, 27, 0,  y_value, false, true),
            //  Instruction::new(Opcode::SW, 27, 0, sp_value-4, false, true),
            //  Instruction::new(Opcode::ADD, 25, 0,  106, false, true),
            //  Instruction::new(Opcode::SW, 25, 0, op_addr, false, true),
            
             //


             Instruction::new(Opcode::ADD, 5, 0,  SyscallCode::RWASM_BINOP as u32, false, true),
             Instruction::new(Opcode::ADD, 11, 0, sp_addr, false, true),
             Instruction::new(Opcode::ADD, 10, 0, op_addr, false, true),
             Instruction::new(Opcode::ECALL, 5, 10, 11, false, false),
         ];
         let program = Program{instructions:instructions,
              pc_base:0,
              pc_start:0,
              memory_image: mem };
            //  memory_image: BTreeMap::new() };
            
        program
 
    }
   
   
   
    #[test]
    fn test_rwasm_proof2(){
        setup_logger();
        let prg = build_elf();
        
        tracing::info!("initializing prover");
        let mut prover = SP1Prover::new();
        prover.core_opts.shard_size = 1 << 12;
        prover.core_opts.shard_batch_size=0;
        tracing::info!("setup elf");
        let (pk, vk) = prover.core_machine.setup(&prg);
        let vk = SP1VerifyingKey { vk };
        let pk = SP1ProvingKey {
            pk,
            elf: vec![],
            vk: vk.clone(),
        };
        
      
        tracing::info!("prove core");
        let stdin = SP1Stdin::new();
        let core_proof = prover.prove_core_prg(&stdin,prg).unwrap();
        let public_values = core_proof.public_values.clone();

        tracing::info!("verify core");
        prover.verify(&core_proof.proof, &vk).unwrap();
        println!("done rwasm proof");
    }
   
}