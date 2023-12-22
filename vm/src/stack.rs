use common::{Value, ValueType};

pub struct Stack {
    stack: Vec<Value>,
}

impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.stack)
    }
}

impl Stack {
    pub fn init() -> Stack {
        Stack {
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().expect("Expected stack to not be empty")
    }

    pub fn peek(&self) -> &Value {
        if self.is_empty() {
            panic!("Expected stack to not be empty");
        }
        self.peek_at(0)
    }

    pub fn peek_at(&self, distance: usize) -> &Value {
        if (self.len() - 1) < distance {
            panic!("Expected stack to not be empty at distance {distance}");
        }
        let index = self.len() - distance - 1;
        self.stack.get(index).expect("Expected stack to not be empty")
    }

    pub fn peek_from_bottom(&self, distance: usize) -> &Value {
        if (self.len() - 1) < distance {
            panic!("Expected stack to not be empty at distance {distance}");
        }
        self.stack.get(distance).expect("Expected stack to not be empty")
    }

    pub fn is_number(&self, distance: usize) -> bool {
        self.peek_at_is_type(distance) == ValueType::Number
    }

    pub fn is_string(&self, distance: usize) -> bool {
        self.peek_at_is_type(distance) == ValueType::String
    }

    pub fn peek_at_is_type(&self, distance: usize) -> ValueType {
        if self.is_empty() {
            panic!("Expected stack to not be empty");
        }
        if (self.len() - 1) < distance {
            panic!("Expected stack to not be empty at distance {distance}");
        }
        match self.peek_at(distance) {
            Value::Nil => ValueType::Nil,
            Value::Number(_) => ValueType::Number,
            Value::Bool(_) => ValueType::Bool,
            Value::String(_) => ValueType::String,
        }
    }


    pub fn clear(&mut self) {
        self.stack.clear();
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn set_at(&mut self, index: usize, value: Value) {
        if self.is_empty() {
            panic!("Expected stack to not be empty");
        }
        if (self.len() - 1) < index {
            panic!("Expected stack to not be empty at index {index}");
        }
        self.stack[index] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_pop() {
        let mut stack = Stack::init();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(3.0));
        if let Value::Number(n) = stack.pop() {
            assert_eq!(n, 3.0);
        }
        if let Value::Number(n) = stack.pop() {
            assert_eq!(n, 1.0);
        }
        assert!(stack.is_empty());
    }

    #[test]
    #[should_panic(expected = "Expected stack to not be empty")]
    fn test_stack_pop_empty() {
        let mut stack = Stack::init();
        assert!(stack.is_empty());
        stack.pop();
    }

    #[test]
    fn test_stack_peek() {
        let mut stack = Stack::init();
        stack.push(Value::Number(1.0));
        if let Value::Number(number) = stack.peek() {
            assert_eq!(*number, 1.0);
        }
    }

    #[test]
    #[should_panic(expected = "Expected stack to not be empty")]
    fn test_stack_peek_empty() {
        let stack = Stack::init();
        stack.peek();
    }

    #[test]
    fn test_stack_peek_at() {
        let mut stack = Stack::init();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));
        stack.push(Value::Number(3.0));

        assert_eq!(*stack.peek_at(0), Value::Number(3.0));
        assert_eq!(*stack.peek_at(1), Value::Number(2.0));
        assert_eq!(*stack.peek_at(2), Value::Number(1.0));
    }

    #[test]
    #[should_panic(expected = "Expected stack to not be empty at distance 1")]
    fn test_stack_peek_at_out_of_stack() {
        let mut stack = Stack::init();
        stack.push(Value::Number(1.0));
        stack.peek_at(1);
    }

    #[test]
    fn test_stack_is_number() {
        let mut stack = Stack::init();
        stack.push(Value::Number(1.0));
        assert!(stack.is_number(0));
    }

    #[test]
    #[should_panic(expected = "Expected stack to not be empty")]
    fn test_stack_is_number_empty_stack() {
        let stack = Stack::init();
        assert!(stack.is_number(0));
    }

    #[test]
    #[should_panic(expected = "Expected stack to not be empty at distance 1")]
    fn test_stack_is_number_out_of_stack() {
        let mut stack = Stack::init();
        stack.push(Value::Number(1.0));
        assert!(stack.is_number(1));
    }

    #[test]
    fn test_stack_clear() {
        let mut stack = Stack::init();
        stack.push(Value::Number(1.0));
        stack.clear();
        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_set_at() {
        let mut stack = Stack::init();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));
        stack.push(Value::Number(3.0));

        stack.set_at(0, Value::Number(4.0));
        stack.set_at(1, Value::Number(5.0));
        stack.set_at(2, Value::Number(6.0));
        assert_eq!(*stack.peek_at(0), Value::Number(6.0));
        assert_eq!(*stack.peek_at(1), Value::Number(5.0));
        assert_eq!(*stack.peek_at(2), Value::Number(4.0));
    }

    #[test]
    fn test_peek_from_bottom() {
        let mut stack = Stack::init();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));
        stack.push(Value::Number(3.0));

        assert_eq!(*stack.peek_from_bottom(0), Value::Number(1.0));
        assert_eq!(*stack.peek_from_bottom(1), Value::Number(2.0));
        assert_eq!(*stack.peek_from_bottom(2), Value::Number(3.0));
    }
}
