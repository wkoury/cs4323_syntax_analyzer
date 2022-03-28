#![warn(clippy::all)]

use crate::bookkeeper::{SymbolType, Token};
use crate::error::{Error, ErrorType};
use crate::pda::PDA;
use crate::scanner::Scanner;

pub struct Parser {
    scanner: Scanner,
    lookahead: Option<Token>,
    // pda: PDA,
    error: Option<Error>,
}

impl Parser {
    pub fn new(src: String) -> Self {
        let mut scanner: Scanner = Scanner::new(src);

        Parser {
            scanner,
            lookahead: None,
            error: None,
        }
    }

    pub fn parse(&mut self) {
        println!("This is the parsing function, it should only get called once.");

        // The first step in the parsing will always be the same–insert the stack bottom symbol and the start symbol into the stack.
        // self.stack.push(0);

        while !self.scanner.is_done() {
            // First, we need to fetch a new lookahead token.
        }
    }
}
