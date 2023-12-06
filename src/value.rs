use std::fmt;

#[derive(Debug,PartialEq, Eq)]
pub(crate) enum ValueType {
    Nil,
    Number,
    Bool,
    String,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Value {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
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

pub(crate) fn is_falsey(value: &Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Number(n) => *n == 0.0,
        Value::Bool(b) => *b == false,
        Value::String(s) => s.is_empty(),
    }
}
