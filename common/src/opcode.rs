#[repr(u8)]
#[derive(num_derive::FromPrimitive)]
#[derive(strum_macros::Display)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opcode {
    Constant = 0,
    Return = 1,
    Negate = 2,
    Add = 3,
    Subtract = 4,
    Multiply = 5,
    Divide = 6,
    Nil = 7,
    True = 8,
    False = 9,
    Not = 10,
    Equal = 11,
    Greater = 12,
    Less = 13,
    Print = 14,
    Pop = 15,
    DefineGlobal = 16,
    GetGlobal = 17,
    SetGlobal = 18,
    GetLocal = 19,
    SetLocal = 20,
    Jump = 21,
    JumpIfFalse = 22,
    Push = 23,
}

impl Opcode {
    pub fn from_byte(byte: u8) -> Opcode {
        let maybe_opcode = num_traits::FromPrimitive::from_u8(byte);
        maybe_opcode.expect("Expected {byte} to be an opcode but it is not")
    }
}
