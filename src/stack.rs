// A simple implementation of a stack using Rust's Vec<u8>. I am doing this to ensure that the behaviors are as expected with a stack, since Rust wants us to use a data structure that some might consider inappropriate for this.

use crate::DEBUG;
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
        if DEBUG {
            println!("Pushing {} to the stack.", n);
        }

        self.stack.push(n);

        if DEBUG {
            dbg!(&self.stack);
        }
    }

    pub fn pop(&mut self) -> u8 {
        if self.is_empty() {
            panic!("Attempting to pop() from an empty stack!");
        }
        let ret = self.stack.pop().unwrap();

        if DEBUG {
            println!("Popping {} from the stack.", ret);
            dbg!(&self.stack);
        }

        ret
    }

    pub fn peek(&mut self) -> u8 {
        if self.is_empty() {
            panic!("Attempting to peek() from an empty stack!");
        }
        let ret = self.stack[self.stack.len() - 1];

        if DEBUG {
            println!("Peeking {} from the stack.", ret);
        }

        ret
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
