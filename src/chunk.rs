#[repr(u8)]
#[derive(FromPrimitive)]
#[derive(strum_macros::Display)]
#[derive(Clone, Debug)]
pub(crate) enum Opcode {
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
}

fn byte_to_opcode(byte: u8) -> Opcode {
    let maybe_opcode = num::FromPrimitive::from_u8(byte);
    maybe_opcode.expect("Expected {byte} to be an opcode but it is not")
}

#[derive(Debug)]
pub(crate) enum Constant {
    Number(f64),
    String(String),
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Number(number) => write!(f, "{}", number),
            Constant::String(string) => write!(f, "{}", string),
        }
    }
}

pub(crate) struct Chunk {
    pub(crate) code: Vec<u8>,
    pub(crate) lines: Vec<usize>,
    pub(crate) constants: Vec<Constant>
}

pub(crate) fn init_chunk() -> Chunk {
    Chunk {
        code: Vec::new(),
        lines: Vec::new(),
        constants: Vec::new(),
    }
}

impl Chunk {
    /// Write a raw byte, should be seldom used and rather use the other functions
    pub(crate) fn write_byte(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    /// Write an opcode to the chunk
    pub(crate) fn write_opcode(&mut self, opcode: Opcode, line: usize) {
        self.write_byte(opcode as u8, line);
    }

    /// Write a constant to the constant array and return it's index
    pub(crate) fn add_constant(&mut self, constant: Constant) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub(crate) fn read_constant(&self, index: usize) -> &Constant {
        &self.constants[index]
    }

    /// Add a constant, write a CONSTANT opcode followed by the index
    pub(crate) fn write_constant(&mut self, constant: Constant, line: usize) {
        let constant_index = self.add_constant(constant);
        self.write_opcode(Opcode::Constant, line);
        self.write_byte(constant_index as u8, line);
    }

    /// Reads a byte from the code chunk given an index
    pub(crate) fn read_byte(&self, index: usize) -> u8 {
        self.code[index]
    }

    /// Reads a byte from the code chunk given and index and ensures it's an opcode
    pub(crate) fn read_opcode(&self, index: usize) -> Opcode {
        let opcode = self.read_byte(index);
        byte_to_opcode(opcode)
    }

    pub(crate) fn get_line(&self, index: usize) -> usize {
        self.lines[index]
    }

    pub(crate) fn code_len(&self) -> usize {
        self.code.len()
    }
}
