#![warn(clippy::all)]

use crate::bookkeeper::{Bookkeeper, Token};
use crate::error::{Error, ErrorType};
use crate::pda::{is_terminal_symbol, PDA};
use crate::scanner::Scanner;
use crate::DEBUG;

pub struct Parser {
    pub(crate) scanner: Scanner,
    lookahead: Option<Token>,
    pda: PDA,
    // error: Option<Error>,
}

impl Parser {
    pub fn new(src: String, symtab: Bookkeeper) -> Self {
        let scanner: Scanner = Scanner::new(src, symtab);
        let pda = PDA::new();

        Parser {
            scanner,
            lookahead: None,
            pda,
            // error: None,
        }
    }

    // Parse the source. If the parsing is successful, return true. If not, return false.
    pub fn parse(&mut self) -> bool {
        // Print out the table header for the parse output
        println!(
            "{0: <30} | {1: <30} | {2: <30} | {3: <}",
            "Steps", "Stack Top", "Lookahead", "Action"
        );

        self.pda.initialize();
        let mut needs_new_lookahead = true;
        let mut token_request_result: (Option<&Token>, Option<&Error>, bool) = (None, None, false);
        let mut scanner_is_done: bool = false;
        while !scanner_is_done {
            // First, we need to fetch a new lookahead token.
            if needs_new_lookahead {
                if DEBUG {
                    println!("New lookahead needed, making token request.");
                }
                token_request_result /*: (Option<&Token>, Option<&Error>) */ = self.scanner.token_request().to_owned();
                needs_new_lookahead = false;
            }

            if token_request_result.to_owned().1.is_none()
                && token_request_result.to_owned().0.is_some()
            {
                self.lookahead = Some(token_request_result.0.unwrap().to_owned());

                if DEBUG {
                    dbg!(&self.lookahead);
                }

                let transition_result =
                    self.pda.transition(self.lookahead.to_owned().unwrap().code);
                if !transition_result.0 {
                    println!("REJECT");
                    return false;
                }

                let symbol_code = self.lookahead.as_ref().unwrap().code;
                if is_terminal_symbol(symbol_code) && transition_result.1 {
                    // consume the symbol, reset the lookahead
                    needs_new_lookahead = true;
                }
            }

            scanner_is_done = token_request_result.to_owned().2;
        }

        let ret: bool = self.pda.q && self.pda.stack.is_empty();
        if ret {
            // FIXME this is not really the right way to do this
            println!("ACCEPT");
        } else {
            println!("REJECT");
        }

        ret
    }
}

#[cfg(test)]
mod parser_tests {
    use crate::bookkeeper::Bookkeeper;
    use crate::parser::Parser;

    // Initialize the parser
    fn init(src: String) -> Parser {
        let symtab = Bookkeeper::new();

        Parser::new(src, symtab)
    }

    #[test]
    fn test_package_a() {
        let mut p = init("package a;\n$\n".to_string());

        assert!(p.parse());
    }

    #[test]
    fn test_package_b() {
        let mut p = init("package b;\n$\n".to_string());

        assert!(p.parse());
    }

    #[test]
    fn test_packages_and_imports() {
        let mut p = init("package b;\n\nimport a;\nimport b;\n$\n".to_string());

        assert!(p.parse());
    }

    #[test]
    fn test_empty_string() {
        let mut p = init("".to_string());

        assert!(p.parse());
    }

    #[test]
    fn test_empty_string_with_terminator() {
        let mut p = init("\n$\n".to_string());

        assert!(p.parse());
    }

    #[test]
    fn test_something_clearly_incorrect() {
        let mut p = init("this is clearly not within our grammar at all.\n".to_string());

        assert!(!p.parse());
    }

    #[test]
    fn test_body_only() {
        let mut p = init("abstract class a {}\n$\n".to_string());

        assert!(p.parse());
    }
}
