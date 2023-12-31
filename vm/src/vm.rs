use std::collections::HashMap;

use common::{chunk::Chunk, Constant, disassembler::disassemble_instruction, opcode::Opcode, Value};

use crate::stack::Stack;

const DEBUG: bool = true;

#[derive(PartialEq, Eq, Debug)]
pub enum InterpretResult {
    OK,
    RuntimeError
}

pub struct VM {
    pub chunk: Chunk,
    pub stack: Stack,
    ip: usize,
    globals: HashMap<String, Value>,
}

impl VM {
    pub fn init(chunk: Chunk) -> VM {
        VM {
            chunk: chunk,
            stack: Stack::init(),
            ip: 0,
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> (InterpretResult, Option<Value>) {
        loop {
            if DEBUG {
                println!("========= ip: {0} =============", self.ip);
                disassemble_instruction(&self.chunk, self.ip);
                println!("");
                println!("{:?}", self.stack);
                println!("===========================================");
            }
            let instruction = self.read_opcode();
            self.advance_ip();
            match instruction {
                Opcode::Constant => {
                    let constant_index = self.read_byte() as usize;
                    let constant = self.read_constant(constant_index);
                    let value = match constant {
                        Constant::Number(number) => Value::Number(*number),
                        Constant::String(s) => Value::String(s.clone()),
                    };
                    self.stack.push(value);
                    self.advance_ip();
                }
                Opcode::Nil => self.stack.push(Value::Nil),
                Opcode::False => self.stack.push(Value::Bool(false)),
                Opcode::True => self.stack.push(Value::Bool(true)),
                Opcode::Not => {
                    let value = self.stack.pop();
                    self.stack.push(Value::Bool(value.is_falsey()));
                },
                Opcode::Equal => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    self.stack.push(Value::Bool(a == b));
                },
                Opcode::Greater => self.binary_op_boolean(|a, b| a > b),
                Opcode::Less => self.binary_op_boolean(|a, b| a < b),
                Opcode::Negate => {
                    if !self.stack.is_number(0) {
                        self.runtime_error("Operand must be a number");
                        return (InterpretResult::RuntimeError, None);
                    }
                    if let Value::Number(constant) = self.stack.pop() {
                        self.stack.push(Value::Number(-constant));
                    }
                }
                Opcode::Add => {
                    if self.stack.is_string(0) && self.stack.is_string(1) {
                        self.concatenate();
                    } else {
                        self.binary_op(|a, b| a + b);
                    }
                },
                Opcode::Subtract => self.binary_op(|a, b| a - b),
                Opcode::Multiply => self.binary_op(|a, b| a * b),
                Opcode::Divide => self.binary_op(|a, b| a / b),
                Opcode::Print => {
                    let value = self.stack.pop();
                    println!("{value}");
                },
                Opcode::Pop => {
                    self.stack.pop();
                    ()
                },
                Opcode::Push => {
                    let value = self.read_byte();
                    self.stack.push(Value::Number(value as f64));
                    self.advance_ip();
                },
                Opcode::Return => {
                    let value = self.stack.pop();
                    return (InterpretResult::OK, Some(value));
                },
                Opcode::DefineGlobal => {
                    let value = self.stack.pop();
                    let name = self.read_next_constant_string();
                    self.globals.insert(name.to_string(), value.clone());
                    self.advance_ip();
                }
                Opcode::GetGlobal => {
                    let name = self.read_next_constant_string();
                    if let Some(value) = self.globals.get(name) {
                        self.stack.push(value.clone());
                    } else {
                        self.runtime_error("Undefined variable");
                    }
                }
                Opcode::SetGlobal => { // TODO add tests
                    let name = self.read_next_constant_string();
                    if self.globals.contains_key(name) {
                        self.globals.insert(name.to_string(), self.stack.peek().clone());
                    } else {
                        self.runtime_error("Undefined variable");
                    }
                }
                Opcode::GetLocal => {
                    // We have to re-push the value at the top of the stack
                    let slot = self.read_byte() as usize;
                    self.stack.push(self.stack.peek_from_bottom(slot).clone());
                    self.advance_ip();
                }
                Opcode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    self.stack.set_at(slot, self.stack.peek().clone());
                    self.advance_ip();
                },
                Opcode::Jump => {
                    let offset = self.read_short() as usize;
                    self.ip += offset;
                },
                Opcode::JumpIfFalse => {
                    let offset = self.read_short() as usize;
                    let condition = self.stack.peek();
                    if condition.is_falsey() {
                        self.ip += offset;
                    } else {
                        self.advance_ip();
                        self.advance_ip();
                    }
                },
            }
        }
    }

    fn read_next_constant_string(&self) -> &str {
        let constant_index = self.read_byte() as usize;
        if let Constant::String(str) = self.read_constant(constant_index) {
            str
        } else {
            panic!("Expected to read constant string");
        }
    }

    fn advance_ip(&mut self) {
        let new_position = (self.ip as i32 + 1) as usize;
        self.ip = new_position.min(self.chunk.code_len() - 1).max(0);
    }

    /// Reads a raw byte from the chunk's code at current IP
    fn read_byte(&self) -> u8 {
        self.chunk.read_byte(self.ip)
    }

    /// Reads a short (2 bytes) from the chunk's code at current IP
    fn read_short(&self) -> u16 {
        let byte1 = self.chunk.read_byte(self.ip) as u16;
        let byte2 = self.chunk.read_byte(self.ip + 1) as u16;
        (byte1 << 8) | byte2
    }

    /// Reads an opcode from the chunk's code at current IP
    fn read_opcode(&mut self) -> Opcode {
        self.chunk.read_opcode(self.ip)
    }

    /// Read a constant from the chunk's constant pool given it's index
    fn read_constant(&self, index: usize) -> &Constant {
        &self.chunk.read_constant(index)
    }

    fn concatenate(&mut self) {
        if self.stack.is_string(0) && self.stack.is_string(1) {
            if let (Value::String(s2), Value::String(s1)) = (self.stack.pop(), self.stack.pop()) {
                let mut s = s1.clone();
                s.push_str(&s2);
                self.stack.push(Value::String(s));
            } else {
                panic!("Expected to concatenate strings");
            }
        }
    }

    fn binary_op<F>(&mut self, op: F) where F: Fn(f64, f64) -> f64 {
        if let (Value::Number(b), Value::Number(a)) = (self.stack.pop(), self.stack.pop()) {
            self.stack.push(Value::Number(op(a, b)));
        } else {
            panic!("Operands must be numbers");
        }
    }

    fn binary_op_boolean<F>(&mut self, op: F) where F: Fn(f64, f64) -> bool {
        if let (Value::Number(b), Value::Number(a)) = (self.stack.pop(), self.stack.pop()) {
            self.stack.push(Value::Bool(op(a, b)));
        } else {
            panic!("Operands must be numbers");
        }
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
    use common::*;
    use common::{run_and_expect, run_and_expect_str, write_constant, write_return, write_string};
    use common::chunk::Chunk;
    use common::Constant;
    use common::opcode::Opcode;

    use crate::vm::VM;

    #[test]
    fn test_return_float() {
        let mut vm = VM::init(Chunk::init());
        write_constant!(vm, 3.14);
        write_return!(vm);
        run_and_expect!(vm, Value::Number(3.14));
    }

    #[test]
    fn test_float_equality() {
        let mut vm = VM::init(Chunk::init());
        write_constant!(vm, 3.14);
        write_constant!(vm, 3.14);
        vm.chunk.write_opcode(Opcode::Equal, 124);
        write_return!(vm);
        run_and_expect!(vm, Value::Bool(true));
    }

    #[test]
    fn test_return_boolean() {
        let mut vm = VM::init(Chunk::init());
        vm.chunk.write_opcode(Opcode::True, 123);
        write_return!(vm);
        run_and_expect!(vm, Value::Bool(true));
    }

    #[test]
    fn test_return_string() {
        let mut vm = VM::init(Chunk::init());
        write_string!(vm, "Hello, world!");
        write_return!(vm);
        run_and_expect_str!(vm, "Hello, world!");
    }


    #[test]
    fn test_add() {
        let mut vm = VM::init(Chunk::init());
        write_constant!(vm, 1.2);
        write_constant!(vm, 2.5);
        vm.chunk.write_opcode(super::Opcode::Add, 123);
        write_return!(vm);
        run_and_expect!(vm, Value::Number(3.7));
    }

    #[test]
    fn test_string_concat() {
        let mut vm = VM::init(Chunk::init());
        write_string!(vm, "Hello, ");
        write_string!(vm, "world!");
        vm.chunk.write_opcode(super::Opcode::Add, 123);
        write_return!(vm);
        run_and_expect_str!(vm, "Hello, world!");
    }

    #[test]
    fn test_string_equality() {
        let mut vm = VM::init(Chunk::init());
        write_string!(vm, "Banana");
        write_string!(vm, "Banana");
        vm.chunk.write_opcode(Opcode::Equal, 124);
        write_return!(vm);
        run_and_expect!(vm, Value::Bool(true));
    }

    #[test]
    fn test_print_string() {
        let mut vm = VM::init(Chunk::init());
        write_string!(vm, "Banana");
        vm.chunk.write_opcode(Opcode::Print, 124);
        write_constant!(vm, 0.0);
        write_return!(vm);
        run_and_expect!(vm, Value::Number(0.0));
    }

    #[test]
    fn test_global_variables() {
        let mut vm = VM::init(Chunk::init());
        vm.chunk.add_constant(Constant::String("myvar".to_string()));
        vm.chunk.add_constant(Constant::Number(4.0));
        vm.chunk.write_opcode(Opcode::Constant, 123);
        vm.chunk.write_byte(1, 123);
        vm.chunk.write_opcode(Opcode::DefineGlobal, 124);
        vm.chunk.write_byte(0, 124);
        vm.chunk.write_opcode(Opcode::GetGlobal, 124);
        vm.chunk.write_byte(0, 124);
        write_return!(vm);
        run_and_expect!(vm, Value::Number(4.0));
    }

    #[test]
    fn test_get_local_variable() {
        let mut vm = VM::init(Chunk::init());

        // I push some garbage
        vm.stack.push(Value::Number(5.0));
        vm.stack.push(Value::Number(6.0));
        vm.stack.push(Value::Number(7.0));
        vm.stack.push(Value::Number(8.0));

        // Local variable index and then GET_LOCAL
        vm.chunk.write_opcode(Opcode::GetLocal, 124);
        vm.chunk.write_byte(2, 124);
        write_return!(vm);

        run_and_expect!(vm, Value::Number(7.0));
    }

    #[test]
    fn test_set_local_variable() {
        let mut vm = VM::init(Chunk::init());

        // I push some garbage
        vm.stack.push(Value::Number(5.0));
        vm.stack.push(Value::Number(6.0));
        vm.stack.push(Value::Number(7.0));
        // New value to set
        vm.stack.push(Value::Number(16.0));

        // Local variable index and then GET_LOCAL
        vm.chunk.write_opcode(Opcode::SetLocal, 124);
        vm.chunk.write_byte(1, 124); // Set stack[1] = 16.0
        vm.chunk.write_opcode(Opcode::GetLocal, 124);
        vm.chunk.write_byte(1, 124);
        write_return!(vm);

        run_and_expect!(vm, Value::Number(16.0));
    }

    #[test]
    fn test_jump() {
        let mut vm = VM::init(Chunk::init());
        vm.chunk.write_opcode(Opcode::Jump, 123);
        vm.chunk.write_short(4, 123); // Skip 4 instructions towards the Multiply
        vm.stack.push(Value::Number(2.0));
        vm.stack.push(Value::Number(3.0));
        vm.chunk.write_opcode(Opcode::Add, 124);
        write_return!(vm);
        vm.chunk.write_opcode(Opcode::Multiply, 124);
        write_return!(vm);
        vm.chunk.write_opcode(Opcode::Subtract, 124);
        write_return!(vm);
        run_and_expect!(vm, Value::Number(6.0));
    }

    #[test]
    fn test_jump_if_false() {
        let mut vm = VM::init(Chunk::init());
        vm.stack.push(Value::Bool(false));
        vm.chunk.write_opcode(Opcode::JumpIfFalse, 123);
        vm.chunk.write_short(6, 123); // Skip 4 instructions towards the Push 6
        vm.chunk.write_opcode(Opcode::Pop, 124);

        vm.chunk.write_opcode(Opcode::Push, 124);
        vm.chunk.write_byte(5, 124);
        write_return!(vm);

        vm.chunk.write_opcode(Opcode::Push, 124); // Jump here
        vm.chunk.write_byte(6, 124);
        write_return!(vm);

        vm.chunk.write_opcode(Opcode::Push, 124);
        vm.chunk.write_byte(7, 124);
        write_return!(vm);

        run_and_expect!(vm, Value::Number(6.0));
    }

    #[test]
    fn test_jump_if_false_do_not_take() {
        let mut vm = VM::init(Chunk::init());
        vm.stack.push(Value::Bool(true));
        vm.chunk.write_opcode(Opcode::JumpIfFalse, 123);
        vm.chunk.write_short(6, 123); // Skip 4 instructions towards the Push 6
        vm.chunk.write_opcode(Opcode::Pop, 124);

        vm.chunk.write_opcode(Opcode::Push, 124);
        vm.chunk.write_byte(5, 124);
        write_return!(vm);

        vm.chunk.write_opcode(Opcode::Push, 124); // Jump here
        vm.chunk.write_byte(6, 124);
        write_return!(vm);

        run_and_expect!(vm, Value::Number(5.0));
    }
}
