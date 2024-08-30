use crate::runtime::{Syscall, SyscallContext,Register,MemoryAccessPosition};

pub struct OpDrop;

impl OpDrop {
    pub const fn new() -> Self {
        Self
    }
}
//Implement the RWASM OP DROP
//Register X2 saves the stack pointer and the it will be reduce by 4.
impl Syscall for OpDrop {
    fn execute(&self, ctx: &mut SyscallContext, exit_code: u32, _: u32) -> Option<u32> {
        let mut stack_len = ctx.rt.rr(Register::X2,MemoryAccessPosition::C);
        stack_len = stack_len-4;
        ctx.rt.rw(Register::X2,stack_len);
        None
    }
}
