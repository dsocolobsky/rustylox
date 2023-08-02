use crate::value;
use crate::value::{as_number, Value};

pub(crate) struct Stack {
    stack: Vec<Value>,
}

pub(crate) fn init_stack() -> Stack {
    Stack {
        stack: Vec::new(),
    }
}

impl Stack {
    pub(crate) fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub(crate) fn push_number(&mut self, number: f64) {
        self.push(value::number_val(number));
    }

    pub(crate) fn pop(&mut self) -> Value {
        self.stack.pop().expect("Expected stack to not be empty")
    }

    pub(crate) fn pop_as_number(&mut self) -> f64 {
        as_number(&self.pop())
    }

    pub(crate) fn peek(&self) -> &Value {
        if self.stack.is_empty() {
            panic!("Expected stack to not be empty");
        }
        self.peek_at(0)
    }

    pub(crate) fn peek_at(&self, distance: usize) -> &Value {
        if (self.stack.len() - 1) < distance {
            panic!("Expected stack to not be empty at distance {distance}");
        }
        let index = self.stack.len() - distance - 1;
        self.stack.get(index).expect("Expected stack to not be empty")
    }

    pub(crate) fn is_number(&self, distance: usize) -> bool {
        if self.stack.is_empty() {
            panic!("Expected stack to not be empty");
        }
        value::is_number(self.peek_at(distance))
    }

    pub(crate) fn clear(&mut self) {
        self.stack.clear();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::value::number_val;
    use super::*;

    #[test]
    fn test_stack_pop() {
        let mut stack = init_stack();
        stack.push_number(1.0);
        stack.push_number((3.0));
        let popped = as_number(&stack.pop());
        assert_eq!(popped, 3.0);
        let popped = as_number(&stack.pop());
        assert_eq!(popped, 1.0);
        assert!(stack.is_empty());
    }

    #[test]
    #[should_panic(expected = "Expected stack to not be empty")]
    fn test_stack_pop_empty() {
        let mut stack = init_stack();
        assert!(stack.is_empty());
        stack.pop();
    }

    #[test]
    fn test_stack_pop_as_number() {
        let mut stack = init_stack();
        stack.push_number(1.0);
        stack.push_number((3.0));
        let popped = stack.pop_as_number();
        assert_eq!(popped, 3.0);
    }

    #[test]
    fn test_stack_peek() {
        let mut stack = init_stack();
        stack.push_number(1.0);
        let number = value::as_number(&stack.peek());
        assert_eq!(number, 1.0);
    }

    #[test]
    #[should_panic(expected = "Expected stack to not be empty")]
    fn test_stack_peek_empty() {
        let mut stack = init_stack();
        stack.peek();
    }

    #[test]
    fn test_stack_peek_at() {
        let mut stack = init_stack();
        stack.push_number(1.0);
        stack.push_number(2.0);
        stack.push_number(3.0);
        assert_eq!(value::as_number(stack.peek_at(0)), 3.0);
        assert_eq!(value::as_number(stack.peek_at(1)), 2.0);
        assert_eq!(value::as_number(stack.peek_at(2)), 1.0);
    }

    #[test]
    #[should_panic(expected = "Expected stack to not be empty at distance 1")]
    fn test_stack_peek_at_out_of_stack() {
        let mut stack = init_stack();
        stack.push_number(1.0);
        stack.peek_at(1);
    }

    #[test]
    fn test_stack_is_number() {
        let mut stack = init_stack();
        let value = number_val(1.0);
        stack.push(value);
        assert!(stack.is_number(0));
    }

    #[test]
    #[should_panic(expected = "Expected stack to not be empty")]
    fn test_stack_is_number_empty_stack() {
        let mut stack = init_stack();
        assert!(stack.is_number(0));
    }

    #[test]
    #[should_panic(expected = "Expected stack to not be empty at distance 1")]
    fn test_stack_is_number_out_of_stack() {
        let mut stack = init_stack();
        stack.push_number(1.0);
        assert!(stack.is_number(1));
    }

    #[test]
    fn test_stack_clear() {
        let mut stack = init_stack();
        let value = number_val(1.0);
        stack.push(value);
        stack.clear();
        assert!(stack.is_empty());
    }
}
