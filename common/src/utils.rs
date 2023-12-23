#[allow(unused_macros)]
#[macro_use]

pub mod utils {

    #[macro_export]
    macro_rules! write_constant {
    ($vm:expr, $value:expr) => {
        $vm.chunk.write_constant(Constant::Number($value), 123)
    };
    }

    #[macro_export]
    macro_rules! write_string {
    ($vm:expr, $value:expr) => {
        $vm.chunk.write_constant(Constant::String($value.to_string()), 123);
    };
    }

    #[macro_export]
    macro_rules! write_return {
    ($vm:expr) => {
        $vm.chunk.write_opcode(Opcode::Return, 124);
    };
    }

    #[macro_export]
    macro_rules! run_and_expect {
    ($vm:expr, $expected:expr) => {
        let (status, Some(value)) = $vm.run() else { panic!("failed to execute vm") };
        assert_eq!(status, super::InterpretResult::OK);
        assert_eq!(value, $expected);
    };
    }

    #[macro_export]
    macro_rules! run_and_expect_str {
    ($vm:expr, $expected:expr) => {
        let (status, Some(value)) = $vm.run() else { panic!("failed to execute vm") };
        assert_eq!(status, super::InterpretResult::OK);
        assert_eq!(value, Value::String($expected.to_string()));
    };
    }
}
