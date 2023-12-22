use std::fmt;

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
}

impl Opcode {
    pub fn from_byte(byte: u8) -> Opcode {
        let maybe_opcode = num_traits::FromPrimitive::from_u8(byte);
        maybe_opcode.expect("Expected {byte} to be an opcode but it is not")
    }
}


#[derive(Debug, PartialEq)]
pub enum Constant {
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

#[derive(Debug,PartialEq, Eq)]
pub enum ValueType {
    Nil,
    Number,
    Bool,
    String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Number(n) => *n == 0.0,
            Value::Bool(b) => *b == false,
            Value::String(s) => s.is_empty(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Number(n)=> write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

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

/// Disassemble the code section of a chunk
pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {name} ==");
    let mut offset: usize = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

/// Disassemble a single instruction
pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    // If this instruction is in the same line as the previous don't show a new line show a |
    // Else, if it has changed, show the line number.
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines[offset]);
    }

    let instruction = chunk.code[offset];
    let maybe_opcode = num::FromPrimitive::from_u8(instruction);
    let to_ret = match maybe_opcode {
        Some(op) => {
            let disasm = |name| {
                disassemble_simple(name, offset)
            };
            match op {
                Opcode::Return => disasm("RETURN"),
                Opcode::Not => disasm("NOT"),
                Opcode::Equal => disasm("EQUAL"),
                Opcode::Greater => disasm("GREATER"),
                Opcode::Less => disasm("LESS"),
                Opcode::Negate => disasm("NEGATE"),
                Opcode::Add => disasm("ADD"),
                Opcode::Subtract => disasm("SUBTRACT"),
                Opcode::Multiply => disasm("MULTIPLY"),
                Opcode::Divide => disasm("DIVIDE"),
                Opcode::Constant => disassemble_constant("CONSTANT", chunk, offset),
                Opcode::Nil => disasm("NIL"),
                Opcode::False => disasm("FALSE"),
                Opcode::True => disasm("TRUE"),
                Opcode::Print => disasm("PRINT"),
                Opcode::Pop => disasm("POP"),
                Opcode::DefineGlobal => disassemble_constant("DEFINE_GLOBAL", chunk, offset),
                Opcode::GetGlobal => disassemble_constant("GET_GLOBAL", chunk, offset),
                Opcode::SetGlobal => disassemble_constant("SET_GLOBAL", chunk, offset),
                Opcode::GetLocal => disassemble_constant("GET_LOCAL", chunk, offset),
                Opcode::SetLocal => disassemble_constant("SET_LOCAL", chunk, offset),
            }
        }
        None => {
            println!("Unknown opcode {instruction}");
            offset + 1
        }
    };
    to_ret
}

// Disassemble a simple (1 byte) opcode
fn disassemble_simple(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

/// Disassemble a CONSTANT opcode
fn disassemble_constant(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code[offset + 1].clone() as usize;
    print!("{:<16} {:>4} '", name, constant);
    let value = &chunk.constants[constant];
    println!("{value}'");
    offset + 2
}
