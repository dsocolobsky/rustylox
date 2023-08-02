use crate::{chunk, disassembler, compiler, value, stack};
use crate::chunk::Opcode;
use crate::value::Value;

const DEBUG: bool = true;

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum InterpretResult {
    OK,
    CompileError,
    RuntimeError
}

pub(crate) struct VM {
    pub(crate) chunk: chunk::Chunk,
    stack: stack::Stack,
    ip: usize
}

pub(crate) fn init_vm() -> VM {
    VM {
        chunk: chunk::init_chunk(),
        stack: stack::init_stack(),
        ip: 0
    }
}

impl VM {
    pub(crate) fn interpret(&mut self, source: &str) -> (InterpretResult, Option<Value>) {
        if let Some(chunk) = compiler::compile(source) {
            self.chunk = chunk;
            self.ip = 0;
            self.run()
        } else {
            (InterpretResult::CompileError, None)
        }
    }

    pub(crate) fn run(&mut self) -> (InterpretResult, Option<Value>) {
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
                    self.stack.push(value::number_val(constant));
                    self.advance_ip();
                }
                Opcode::Nil => self.stack.push(value::nil_val()),
                Opcode::False => self.stack.push(value::boolean_val(false)),
                Opcode::True => self.stack.push(value::boolean_val(true)),
                Opcode::Not => {
                    let value = self.stack.pop();
                    self.stack.push(value::boolean_val(value::is_falsey(&value)));
                },
                Opcode::Equal => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(value::boolean_val(a == b));
                },
                Opcode::Greater => self.binary_op_boolean(|a, b| a > b),
                Opcode::Less => self.binary_op_boolean(|a, b| a < b),
                Opcode::Negate => {
                    if !self.stack.is_number(0) {
                        self.runtime_error("Operand must be a number");
                        return (InterpretResult::RuntimeError, None);
                    }
                    let constant = self.stack.pop_as_number();
                    self.stack.push(value::number_val(-constant));
                }
                Opcode::Add => self.binary_op(|a, b| a + b),
                Opcode::Subtract => self.binary_op(|a, b| a - b),
                Opcode::Multiply => self.binary_op(|a, b| a * b),
                Opcode::Divide => self.binary_op(|a, b| a / b),
                Opcode::Return => {
                    let value = self.stack.pop();
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

    fn binary_op<F>(&mut self, op: F) where F: Fn(f64, f64) -> f64 {
        if !self.stack.is_number(0) || !self.stack.is_number(1) {
            self.runtime_error("Operands must be numbers");
            return;
        }
        let b = self.stack.pop_as_number();
        let a = self.stack.pop_as_number();
        let result = value::number_val(op(a, b));
        self.stack.push(result);
    }

    fn binary_op_boolean<F>(&mut self, op: F) where F: Fn(f64, f64) -> bool {
        if !self.stack.is_number(0) || !self.stack.is_number(1) {
            self.runtime_error("Operands must be numbers");
            return;
        }
        let b = self.stack.pop_as_number();
        let a = self.stack.pop_as_number();
        let result = value::boolean_val(op(a, b));
        self.stack.push(result);
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
    use crate::value;
    use crate::vm::Opcode;

    #[test]
    fn test_return_float() {
        let mut vm = super::init_vm();
        vm.chunk.write_constant(3.14, 123);
        vm.chunk.write_opcode(Opcode::Return, 124);
        let (status, res) = vm.run();
        assert_eq!(status, super::InterpretResult::OK);
        assert_eq!(value::as_number(&res.unwrap()), 3.14);
    }

    #[test]
    fn test_return_boolean() {
        let mut vm = super::init_vm();
        vm.chunk.write_opcode(Opcode::True, 123);
        vm.chunk.write_opcode(Opcode::Return, 124);
        let (status, res) = vm.run();
        assert_eq!(status, super::InterpretResult::OK);
        assert_eq!(value::as_boolean(&res.unwrap()), true);
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
        assert_eq!(value::as_number(&res.unwrap()), 3.7);
    }
}
