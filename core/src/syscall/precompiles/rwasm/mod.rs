pub mod drop;
pub mod binop32;


pub enum RwasmOp{
    I32ADD = 106,
    I32SUB = 107,
    I32MUL = 108,
    I32DIVS = 109,
    I32DIVU = 110,
    I32REMS = 111,
    I32REMU = 112,
    I32AND = 113,
    I32OR = 114,
    I32XOR = 115,
    I32SHL = 116,
    I32SHRS = 117,
    I32SHRU = 118,
    I32ROTL = 119,
    I32ROTR = 120,
}

impl RwasmOp{
    pub fn from_u32(op_code:u32)->RwasmOp{
        match op_code {
            106=>RwasmOp::I32ADD,
            107=>RwasmOp::I32SUB,
            108=>RwasmOp::I32MUL,
            109=>RwasmOp::I32DIVS,
            110=>RwasmOp::I32DIVU,
            111=>RwasmOp::I32REMS,
            112=>RwasmOp::I32REMU,

            _=>unreachable!(),
        }
    }
}