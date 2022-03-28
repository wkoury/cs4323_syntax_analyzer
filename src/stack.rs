// A simple implementation of a stack using Rust's Vec<u8>. I am doing this to ensure that the behaviors are as expected with a stack, since Rust wants us to use a data structure that some might consider inappropriate for this.
pub struct Stack {
    stack: Vec<u8>,
}

impl Stack {
    // Create a new stack
    pub fn new() -> Self {
        // This really doesn't need to be declared as mutable? Interesting.
        let stack = vec![0];
        Stack { stack }
    }

    pub fn push(&mut self, n: u8) {
        self.stack.push(n);
    }

    pub fn pop(&mut self) -> u8 {
        self.stack.pop().unwrap()
    }

    // Determine whether or not we have reached the stack bottom marker.
    pub fn is_empty(&self) -> bool {
        self.stack.len() == 1 && self.stack[0] == 0
    }
}

#[cfg(test)]
mod stack_tests {
    use crate::stack::Stack;

    // Test that the stack is initialized as we expect it.
    #[test]
    fn test_stack_initializes_properly() {
        let s = Stack::new();

        assert_eq!(s.stack.len(), 1);
        assert_eq!(s.stack[0], 0);
    }

    // Test that adding elements to the stack works as expected.
    #[test]
    fn test_stack_adding_elements() {
        let mut s = Stack::new();

        s.push(1);
        s.push(6);
        s.push(69);

        assert!(!s.is_empty());
        assert_eq!(s.stack.len(), 4);
    }

    #[test]
    fn test_stack_removing_elements() {
        let mut s = Stack::new();

        for ii in 1..9 {
            s.push(ii);
        }

        for _ in 1..8 {
            s.pop();
        }

        assert!(!s.is_empty());
        assert_eq!(s.stack.len(), 2);
        assert_eq!(s.pop(), 1);
    }
}
