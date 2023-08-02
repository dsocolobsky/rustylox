#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ValueType {
    Nil,
    Bool,
    Number,
}

pub(crate) union ValueData {
    nil: (),
    boolean: bool,
    number: f64,
}

pub(crate) struct Value {
    value_type: ValueType,
    value_data: ValueData,
}

pub(crate) fn nil_val() -> Value {
    Value {
        value_type: ValueType::Nil,
        value_data: ValueData { nil: () },
    }
}

pub(crate) fn boolean_val(boolean: bool) -> Value {
    Value {
        value_type: ValueType::Bool,
        value_data: ValueData { boolean },
    }
}

pub(crate) fn number_val(number: f64) -> Value {
    Value {
        value_type: ValueType::Number,
        value_data: ValueData { number },
    }
}

pub(crate) fn as_nil(value: &Value) -> () {
    match value.value_type {
        ValueType::Nil => unsafe { value.value_data.nil },
        ValueType::Bool => (),
        ValueType::Number => (),
    }
}

pub(crate) fn as_boolean(value: &Value) -> bool {
    match value.value_type {
        ValueType::Bool => unsafe { value.value_data.boolean },
        ValueType::Nil => false,
        ValueType::Number => unsafe { value.value_data.number != 0.0 },
    }
}

pub(crate) fn as_number(value: &Value) -> f64 {
    match value.value_type {
        ValueType::Number => unsafe { value.value_data.number },
        ValueType::Nil => 0.0,
        ValueType::Bool => {
            if unsafe { value.value_data.boolean } {
                1.0
            } else {
                0.0
            }
        }
    }
}

pub(crate) fn is_nil(value: &Value) -> bool {
    value.value_type == ValueType::Nil
}

pub(crate) fn is_boolean(value: &Value) -> bool {
    value.value_type == ValueType::Bool
}

pub(crate) fn is_number(value: &Value) -> bool {
    value.value_type == ValueType::Number
}
