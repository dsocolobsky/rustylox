use crate::{chunk, disassembler, compiler, value};
use crate::value::{is_number, Value};

const DEBUG: bool = true;

#[derive(PartialEq, Eq, Debug)]
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

pub(crate) struct VM {
    pub(crate) chunk: chunk::Chunk,
    stack: Vec<Value>,
    ip: usize
}

pub(crate) fn init_vm() -> VM {
    VM {
        chunk: chunk::init_chunk(),
        stack: Vec::new(),
        ip: 0
    }
}

impl VM {
    pub(crate) fn interpret(&mut self, source: &str) -> (InterpretResult, Option<f64>) {
        if let Some(chunk) = compiler::compile(source) {
            self.chunk = chunk;
            self.ip = 0;
            self.run()
        } else {
            (InterpretResult::CompileError, None)
        }
    }

    pub(crate) fn run(&mut self) -> (InterpretResult, Option<f64>) {
        loop {
            if DEBUG {
                println!("ip: {0}", self.ip);
                disassembler::disassemble_instruction(&self.chunk, self.ip);
            }
            let instruction = self.read_opcode();
            self.advance_ip();
            match instruction {
                Opcode::Constant => {
                    let constant_index = self.read_byte() as usize;
                    let constant = self.read_constant(constant_index);
                    self.push(value::number_val(constant));
                    self.advance_ip();
                }
                Opcode::Negate => {
                    if !value::is_number(self.peek()) {
                        self.runtime_error("Operand must be a number");
                        return (InterpretResult::RuntimeError, None);
                    }
                    let constant = value::as_number(self.pop());
                    self.push(value::number_val(-constant));
                }
                Opcode::Add => self.binary_op(|a, b| a + b),
                Opcode::Subtract => self.binary_op(|a, b| a - b),
                Opcode::Multiply => self.binary_op(|a, b| a * b),
                Opcode::Divide => self.binary_op(|a, b| a / b),
                Opcode::Return => {
                    let value = value::as_number(self.pop());
                    println!("{value}");
                    return (InterpretResult::OK, Some(value));
                },
            }
        }
    }

    fn advance_ip(&mut self) {
        let new_position = (self.ip as i32 + 1) as usize;
        self.ip = new_position.min(self.chunk.code_len() - 1).max(0);
    }

    /// Reads a raw byte from the chunk's code at current IP
    fn read_byte(&mut self) -> u8 {
        self.chunk.read_byte(self.ip)
    }

    /// Reads an opcode from the chunk's code at current IP
    fn read_opcode(&mut self) -> Opcode {
        self.chunk.read_opcode(self.ip)
    }

    /// Read a constant from the chunk's constant pool given it's index
    fn read_constant(&mut self, index: usize) -> f64 {
        self.chunk.read_constant(index)
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn peek(&self) -> &Value {
        self.peek_at(0)
    }

    fn peek_at(&self, distance: usize) -> &Value {
        let maybe_val = self.stack.get(self.stack.len() - distance - 1);
        maybe_val.expect("Nothing to peek!")
    }

    fn pop(&mut self) -> Value {
        let maybe_val = self.stack.pop();
        maybe_val.expect("Nothing to pop!")
    }

    fn binary_op<F>(&mut self, op: F) where F: Fn(f64, f64) -> f64 {
        if !is_number(self.peek_at(0)) || !is_number(self.peek_at(1)) {
            self.runtime_error("Operands must be numbers");
            return;
        }
        let b = value::as_number(self.pop());
        let a = value::as_number(self.pop());
        self.push(value::number_val(op(a, b)));
    }

    fn runtime_error(&mut self, message: &str) {
        let instruction = self.ip - 1;
        let line = self.chunk.get_line(instruction);
        eprintln!("[line {line}] error: {message}");
        self.stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::Opcode;

    #[test]
    fn test_constant() {
        let mut vm = super::init_vm();
        vm.chunk.write_constant(3.14, 123);
        vm.chunk.write_opcode(Opcode::Return, 124);
        let (status, res) = vm.run();
        assert_eq!(status, super::InterpretResult::OK);
        assert_eq!(res.unwrap(), 3.14);
    }


    #[test]
    fn test_add() {
        let mut vm = super::init_vm();
        vm.chunk.write_constant(1.2, 123);
        vm.chunk.write_constant(2.5, 123);
        vm.chunk.write_opcode(super::Opcode::Add, 123);
        vm.chunk.write_opcode(super::Opcode::Return, 123);
        let (status, res) = vm.run();
        assert_eq!(status, super::InterpretResult::OK);
        assert_eq!(res.unwrap(), 3.7);
    }
}
