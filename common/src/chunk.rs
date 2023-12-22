use crate::Constant;
use crate::opcode::Opcode;

pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<Constant>
}

impl Chunk {
    pub fn init() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Write a raw byte, should be seldom used and rather use the other functions
    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    /// Write an opcode to the chunk
    pub fn write_opcode(&mut self, opcode: Opcode, line: usize) {
        self.write_byte(opcode as u8, line);
    }

    /// Write a constant to the constant array and return it's index
    pub fn add_constant(&mut self, constant: Constant) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn read_constant(&self, index: usize) -> &Constant {
        &self.constants[index]
    }

    /// Add a constant, write a CONSTANT opcode followed by the index
    pub fn write_constant(&mut self, constant: Constant, line: usize) -> usize {
        let constant_index = self.add_constant(constant);
        self.write_opcode(Opcode::Constant, line);
        self.write_byte(constant_index as u8, line);
        constant_index
    }

    /// Write a variable's name as constant to the chunk's constant table.
    // Globals are looked up by name during runtime and the name is too big
    // to fit in the stack so we ought to save it here.
    pub fn write_identifier_constant(&mut self, ident: String) -> usize {
        self.add_constant(Constant::String(ident))
    }

    /// Reads a byte from the code chunk given an index
    pub fn read_byte(&self, index: usize) -> u8 {
        self.code[index]
    }

    /// Reads a byte from the code chunk given and index and ensures it's an opcode
    pub fn read_opcode(&self, index: usize) -> Opcode {
        let opcode = self.read_byte(index);
        Opcode::from_byte(opcode)
    }

    pub fn get_line(&self, index: usize) -> usize {
        self.lines[index]
    }

    pub fn code_len(&self) -> usize {
        self.code.len()
    }
}
