pub mod drop;
pub mod binop32;


pub enum RwasmOp{
    I32Add = 106,
    I32Sub = 107,
    I32Mul = 108,
    I32DiV = 109,
}

impl RwasmOp{
    pub fn from_u32(op_code:u32)->RwasmOp{
        match op_code {
            106=>RwasmOp::I32Add,
            107=>RwasmOp::I32Sub,
            108=>RwasmOp::I32Mul,
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