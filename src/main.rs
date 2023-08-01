use std::fmt;

extern crate num;
#[macro_use]
extern crate num_derive;

enum InterpretResult {
    OK,
    CompileError,
    RuntimeError
}

#[repr(u8)]
#[derive(FromPrimitive)]
#[derive(Clone, Debug)]
enum Opcode {
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

struct Chunk {
    code: Vec<u8>,
    lines: Vec<usize>,
    constants: Vec<f64>
}

fn init_chunk() -> Chunk {
    Chunk {
        code: Vec::new(),
        lines: Vec::new(),
        constants: Vec::new(),
    }
}

impl Chunk {
    fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    fn write_value(&mut self, value: f64) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}

struct VM {
    chunk: Chunk,
    ip: usize
}

fn init_vm() -> VM {
    VM {
        chunk: init_chunk(),
        ip: 0
    }
}

impl VM {
    fn run(&mut self) -> InterpretResult {
        loop {
            println!("ip: {0}", self.ip);
            disassemble_instruction(&self.chunk, self.ip);
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
        self.chunk.code.len() - 1
    }

    // Similar to write a byte but enforce being an opcode
    fn add_opcode(&mut self, opcode: Opcode, line: usize) -> usize {
        self.chunk.write(opcode as u8, line);
        self.chunk.code.len() - 1
    }

    // Writes a CONSTANT op and the constant index next to it
    fn add_constant_idx(&mut self, constant_index: usize, line: usize) -> usize {
        self.add_opcode(Opcode::Constant, line);
        self.write_byte(constant_index as u8, line);
        self.chunk.code.len() - 2
    }

    fn add_constant_op(&mut self, constant: f64, line: usize) -> usize {
        let constant_index = self.chunk.write_value(constant);
        self.add_constant_idx(constant_index, line)
    }

    fn read_byte(&mut self) -> u8 {
        let new_position = (self.ip as i32 + 1) as usize;
        self.ip = new_position.min(self.chunk.code.len() - 1).max(0);
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

fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {name} ==");
    let mut offset: usize = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
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
        Some(Opcode::Return) => simple_instruction("RETURN", offset),
        Some(Opcode::Constant) => constant_instruction("CONSTANT", chunk, offset),
        None => {
            println!("Unknown opcode {instruction}");
            offset + 1
        }
    };
    to_ret
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code[offset + 1].clone() as usize;
    print!("{:<16} {:>4} '", name, constant);
    let value = chunk.constants[constant];
    println!("{value} '");
    offset + 2
}

fn main() {
    let mut vm = init_vm();

    vm.add_constant_op(1.2, 123);
    vm.add_constant_op(3.14, 124);
    vm.add_opcode(Opcode::Return, 124);
    vm.run();
    //disassemble_chunk(&vm.chunk, "code");
}
