use std::fmt;
use crate::{chunk, disassembler};

pub(crate) enum InterpretResult {
    OK,
    CompileError,
    RuntimeError
}

#[repr(u8)]
#[derive(FromPrimitive)]
#[derive(Clone, Debug)]
pub(crate) enum Opcode {
    Constant = 0,
    Return = 1,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Opcode::Constant => write!(f, "CONSTANT"),
            Opcode::Return => write!(f, "RETURN"),
        }
    }
}

fn byte_to_opcode(byte: u8) -> Opcode {
    let maybe_opcode = num::FromPrimitive::from_u8(byte);
    match maybe_opcode {
        Some(opcode) => opcode,
        None => panic!("Expected {byte} to be an opcode but it is not"),
    }
}

pub(crate) struct VM {
    pub(crate) chunk: chunk::Chunk,
    ip: usize
}

pub(crate) fn init_vm() -> VM {
    VM {
        chunk: chunk::init_chunk(),
        ip: 0
    }
}

impl VM {
    pub(crate) fn run(&mut self) -> InterpretResult {
        loop {
            println!("ip: {0}", self.ip);
            disassembler::disassemble_instruction(&self.chunk, self.ip);
            let instruction = self.read_opcode();
            match instruction {
                Opcode::Constant => {
                    let constant = self.read_constant();
                    println!("{constant}");
                    self.ip = self.ip + 1;
                }
                Opcode::Return => return InterpretResult::OK,
            }
        }
    }

    // Write a byte, should be seldom used and rather use the other functions
    fn write_byte(&mut self, byte: u8, line: usize) -> usize {
        self.chunk.write(byte, line);
        self.chunk.code_len() - 1
    }

    // Similar to write a byte but enforce being an opcode
    pub(crate) fn add_opcode(&mut self, opcode: Opcode, line: usize) -> usize {
        self.chunk.write(opcode as u8, line);
        self.chunk.code_len() - 1
    }

    // Writes a CONSTANT op and the constant index next to it
    fn add_constant_idx(&mut self, constant_index: usize, line: usize) -> usize {
        self.add_opcode(Opcode::Constant, line);
        self.write_byte(constant_index as u8, line);
        self.chunk.code_len() - 2
    }

    pub(crate) fn add_constant_op(&mut self, constant: f64, line: usize) -> usize {
        let constant_index = self.chunk.write_value(constant);
        self.add_constant_idx(constant_index, line)
    }

    fn read_byte(&mut self) -> u8 {
        let new_position = (self.ip as i32 + 1) as usize;
        self.ip = new_position.min(self.chunk.code_len() - 1).max(0);
        self.chunk.code[self.ip].clone()
    }

    fn read_opcode(&mut self) -> Opcode {
        let byte = self.read_byte();
        byte_to_opcode(byte)
    }

    fn read_constant(&mut self) -> f64 {
        let index = self.read_byte() as usize;
        self.chunk.constants[index].clone()
    }
}
