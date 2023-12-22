use std::fmt;

pub mod opcode;
pub mod disassembler;
pub mod chunk;

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
