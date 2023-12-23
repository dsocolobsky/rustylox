use common::Value;
#[cfg(test)]
use compiler;
use vm::vm::{InterpretResult, VM};

macro_rules! run_code {
    ($code:expr, $expected:expr) => {
        let chunk = compiler::compile($code).expect("Failed to compile");
        let (status, Some(value)) = VM::init(chunk).run() else { panic!("failed to execute vm") };
        assert_eq!(status, InterpretResult::OK);
        assert_eq!(value, $expected);
    };
}

#[test]
fn basic_return() {
    let code = r#"
        return 4;
    "#;
    run_code!(code, Value::Number(4.0));
}

#[test]
fn test_arithmetic() {
    let code = r#"
        var a = 2;
        var b = 3;
        var c = a * b + 1;
        return c;
    "#;
    run_code!(code, Value::Number(7.0));
}

#[test]
fn test_with_if() {
    let code = r#"
        var a = 1;
        var b = 1;
        if (a == b) {
            a = 3;
        } else {
            a = 4;
        }
        return a;
    "#;
    run_code!(code, Value::Number(3.0));
}