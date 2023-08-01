use crate::{chunk, disassembler, compiler};

const DEBUG: bool = true;

pub(crate) enum InterpretResult {
    OK,
    CompileError,
    RuntimeError
}

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
    stack: Vec<f64>,
    ip: usize
}

pub(crate) fn init_vm() -> VM {
    VM {
        chunk: chunk::init_chunk(),
        stack: Vec::new(),
        ip: 0
    }
}

pub(crate) fn interpret(source: &str) -> InterpretResult {
    let res = compiler::compile(source);
    if !res {
        return InterpretResult::CompileError;
    }
    InterpretResult::OK
}

impl VM {
    pub(crate) fn run(&mut self) -> InterpretResult {
        loop {
            if DEBUG {
                println!("ip: {0}", self.ip);
                disassembler::disassemble_instruction(&self.chunk, self.ip);
            }
            let instruction = self.read_opcode();
            match instruction {
                Opcode::Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                Opcode::Negate => {
                    let constant = self.pop();
                    self.push(-constant);
                }
                Opcode::Add => self.binary_op(|a, b| a + b),
                Opcode::Subtract => self.binary_op(|a, b| a - b),
                Opcode::Multiply => self.binary_op(|a, b| a * b),
                Opcode::Divide => self.binary_op(|a, b| a / b),
                Opcode::Return => {
                    let value = self.pop();
                    println!("{value}");
                    return InterpretResult::OK;
                },
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

    fn advance_ip(&mut self) {
        let new_position = (self.ip as i32 + 1) as usize;
        self.ip = new_position.min(self.chunk.code_len() - 1).max(0);
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip].clone();
        self.advance_ip();
        byte
    }

    fn read_opcode(&mut self) -> Opcode {
        let byte = self.read_byte();
        byte_to_opcode(byte)
    }

    fn read_constant(&mut self) -> f64 {
        let index = self.read_byte() as usize;
        self.chunk.constants[index].clone()
    }

    fn push(&mut self, value: f64) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> f64 {
        let maybe_val = self.stack.pop();
        match maybe_val {
            Some(val) => val,
            None => panic!("Nothing to pop!"),
        }
    }

    fn binary_op<F>(&mut self, op: F) where F: Fn(f64, f64) -> f64 {
        let b = self.pop();
        let a = self.pop();
        self.push(op(a, b));
    }
}
