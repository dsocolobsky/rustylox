#[macro_export]
pub mod utils {
    macro_rules! write_constant {
    ($vm:expr, $value:expr) => {
        $vm.chunk.write_constant(Constant::Number($value), 123);
    };
    }

    macro_rules! write_string {
    ($vm:expr, $value:expr) => {
        $vm.chunk.write_constant(Constant::String($value.to_string()), 123);
    };
    }

    macro_rules! write_return {
    ($vm:expr) => {
        $vm.chunk.write_opcode(Opcode::Return, 124);
    };
    }

    macro_rules! run_and_expect {
    ($vm:expr, $expected:expr) => {
        let (status, Some(value)) = $vm.run() else { !unreachable!() };;
        assert_eq!(status, super::InterpretResult::OK);
        assert_eq!(value, $expected);
    };
    }

    macro_rules! run_and_expect_str {
    ($vm:expr, $expected:expr) => {
        let (status, Some(value)) = $vm.run() else { !unreachable!() };;
        assert_eq!(status, super::InterpretResult::OK);
        assert_eq!(value, Value::String($expected.to_string()));
    };
    }

    pub(crate) use write_constant;
    pub(crate) use write_string;
    pub(crate) use write_return;
    pub(crate) use run_and_expect;
    pub(crate) use run_and_expect_str;
}
