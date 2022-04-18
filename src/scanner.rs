#![warn(clippy::all)]

use std::collections::VecDeque;

use crate::bookkeeper::{convert_token_to_symbol_table_token, Bookkeeper, SymbolType, Token};
use crate::error::{Error, ErrorType};

// Override the main global variable.. this is a mess
const DEBUG: bool = false;

// A struct to represent the scanner, keeping track of where the character is consumed, among other things.
#[derive(Clone, Debug)]
pub struct Scanner {
    source: String,
    index: usize,
    line_number: usize,
    scanned_characters: String,
    pub(crate) token: Option<Token>,
    pub(crate) extra_tokens: VecDeque<Option<Token>>,
    pub(crate) error: Option<Error>,
    comment: bool,
    pub(crate) symtab: Bookkeeper,
}

impl Scanner {
    // Create a new source object.
    pub fn new(src: String, symtab: Bookkeeper) -> Self {
        Scanner {
            source: src,
            index: 0,
            line_number: 1,
            scanned_characters: "".to_string(),
            token: None,
            extra_tokens: VecDeque::<Option<Token>>::new(),
            error: None,
            comment: false,
            symtab,
        }
    }

    // Reads a character from the source, and handles some special cases.
    fn read_character(&mut self) -> char {
        if DEBUG {
            eprintln!("self.index = {}", self.index);
        }
        let ret: char = self.source.chars().nth(self.index).unwrap();
        // Increment line number if we encountered a newline on the last read
        if self.index != 0 && self.source.chars().nth(self.index - 1).unwrap() == '\n' {
            self.line_number += 1;
            self.comment = false; // Reset this every time we encounter a newline.
        }

        //If we have a comment, just skip to the next newline.
        if self.comment {
            while !self.is_done() && self.source.chars().nth(self.index).unwrap() != '\n' {
                self.index += 1;
            }
        }

        // Handle special symbols that are attached to a previous token.
        // We want to do this if we encounter a special symbol, and the previous character to that special symbol is not whitespace.
        if is_special_symbol(ret) && !self.scanned_characters.is_empty() {
            if DEBUG {
                eprintln!("Special symbol encountered: {}", ret);
                eprintln!(
                    "The previous character is: {}",
                    self.source.chars().nth(self.index - 1).unwrap()
                );
            }

            // Make an exception for leq (<=)
            if ret == '=' && self.source.chars().nth(self.index - 1).unwrap() == '<' {
                self.index += 1;
                self.scanned_characters.push(ret);
                return ret;
            }

            self.extra_tokens.push_back(Some(Token {
                token: ret.to_string(),
                symbol_type: SymbolType::SpecialSymbol,
                line_number: self.line_number,
                code: match_special_symbol_to_code(ret),
            }));

            self.index += 1;
            return self.read_character();
        }

        // Handle $, which indicates the end of a program.
        if ret == '$' {
            if DEBUG {
                eprintln!("We have encountered a marker indicating the end of the source program.");
            }
            // Take the easy path out and just jump to the end of the source, and don't accept any further tokens by enabling comments.
            self.index = self.source.len();
            self.comment = true;
            return ' ';
        }

        // Add the scanned character to our potential token, but only if it is not whitespace or a special symbol, excepting =
        if !(ret.is_whitespace() || (is_special_symbol(ret) && ret != '=')) {
            self.scanned_characters.push(ret);
        }

        // Increment the index
        self.index += 1;

        if DEBUG {
            eprintln!("read character {} from the source", ret);
        }

        ret
    }

    // Determine whether we have consumed all characters in the source.
    pub fn is_done(&self) -> bool {
        self.index >= self.source.len()
    }

    // Start moving along the DFA.
    pub fn token_request(&mut self) -> (Option<&Token>, Option<&Error>, bool) {
        // Reset the potential token, previously accepted token, potential extra token, etc.
        self.scanned_characters = "".to_string();
        self.error = None;
        self.token = None;

        if !self.extra_tokens.is_empty() {
            if DEBUG {
                eprintln!("The extra token flag is marked.");
            }
            // Pop the queue to return the token.
            self.token = self.extra_tokens.pop_front().unwrap();
            return (self.token.as_ref(), self.error.as_ref(), self.is_done());
        }

        if self.is_done() {
            return (None, None, self.is_done());
        }

        self.initial_state();

        // If the token belongs in the symbol table, add it.
        if self.token.is_some()
            && (self.token.as_ref().unwrap().symbol_type == SymbolType::Constant
                || self.token.as_ref().unwrap().symbol_type == SymbolType::Identifier)
        {
            self.symtab.insert(convert_token_to_symbol_table_token(
                self.token.as_ref().unwrap().clone(),
            ));
        }

        (self.token.as_ref(), self.error.as_ref(), self.is_done())
    }

    // Start another iteration of the DFA. Scan for another token, though it may return an error instead.
    fn initial_state(&mut self) {
        if DEBUG {
            eprintln!("entered initial state");
        }

        let mut c = self.read_character();

        // If the first character we encounter is whitespace, skip it until we find the beginning of another potential token.
        while c.is_whitespace() {
            if DEBUG {
                eprintln!("found whitespace");
            }

            if !self.is_done() {
                c = self.read_character();
            } else {
                return;
            }
        }

        // A NOTE: this is where the DFA begins, if it is of any help to the grader.
        match c {
            'p' => self.state_1(),
            'i' => self.state_23(),
            'a' => self.state_32(),
            'f' => self.state_42(),
            's' => self.state_51(),
            'b' => self.state_57(),
            'c' => self.state_61(),
            'd' => self.state_69(),
            'e' => self.state_72(),
            '=' => self.state_76(),
            '<' => self.state_78(),
            'n' => self.state_80(),
            'o' => self.state_83(),
            'r' => self.state_90(),
            't' => self.state_98(),
            'v' => self.state_102(),
            'w' => self.state_105(),
            // By putting these below the above, we should be able to handle all letters so that we can go to the <id> partition of the DFA.
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_110(),
            '0'..='9' => self.state_112(),
            // Special symbols to accept outright
            '#' => self.state_115(),
            ';' => self.state_116(),
            '{' => self.state_117(),
            '}' => self.state_118(),
            '(' => self.state_119(),
            ')' => self.state_120(),
            ':' => self.state_121(),
            ',' => self.state_122(),
            '+' => self.state_123(),
            '*' => self.state_124(),
            '@' => self.state_125(),
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    // This begins the keywords partition of the DFA, though in certain cases it may exit to the identifiers partition.
    fn state_1(&mut self) {
        let c = self.read_character();

        match c {
            'a' => self.state_2(),
            'r' => self.state_8(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_2(&mut self) {
        let c = self.read_character();

        match c {
            'c' => self.state_3(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_3(&mut self) {
        let c = self.read_character();

        match c {
            'k' => self.state_4(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_4(&mut self) {
        let c = self.read_character();

        match c {
            'a' => self.state_5(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_5(&mut self) {
        let c = self.read_character();

        match c {
            'g' => self.state_6(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_6(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_7(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_7(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "package".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 3,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                c if is_separator(c) => {
                    self.token = Some(Token {
                        token: self.scanned_characters.clone(),
                        symbol_type: SymbolType::Identifier,
                        line_number: self.line_number,
                        code: 1,
                    })
                }
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_8(&mut self) {
        let c = self.read_character();

        match c {
            'i' => self.state_9(),
            'o' => self.state_16(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_9(&mut self) {
        let c = self.read_character();

        match c {
            'v' => self.state_10(),
            'n' => self.state_14(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_10(&mut self) {
        let c = self.read_character();

        match c {
            'a' => self.state_11(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_11(&mut self) {
        let c = self.read_character();

        match c {
            't' => self.state_12(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_12(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_13(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_13(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "private".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 8,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                c if is_separator(c) => {
                    self.token = Some(Token {
                        token: self.scanned_characters.clone(),
                        symbol_type: SymbolType::Identifier,
                        line_number: self.line_number,
                        code: 1,
                    })
                }
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_14(&mut self) {
        let c = self.read_character();

        match c {
            't' => self.state_15(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_15(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "print".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 21,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_16(&mut self) {
        let c = self.read_character();

        match c {
            't' => self.state_17(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_17(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_18(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_18(&mut self) {
        let c = self.read_character();

        match c {
            'c' => self.state_19(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_19(&mut self) {
        let c = self.read_character();

        match c {
            't' => self.state_20(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_20(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_21(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_21(&mut self) {
        let c = self.read_character();

        match c {
            'd' => self.state_22(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_22(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "protected".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 9,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_23(&mut self) {
        let c = self.read_character();

        match c {
            'm' => self.state_24(),
            'f' => self.state_29(),
            'n' => self.state_30(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_24(&mut self) {
        let c = self.read_character();

        match c {
            'p' => self.state_25(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_25(&mut self) {
        let c = self.read_character();

        match c {
            'o' => self.state_26(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_26(&mut self) {
        let c = self.read_character();

        match c {
            'r' => self.state_27(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_27(&mut self) {
        let c = self.read_character();

        match c {
            't' => self.state_28(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_28(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "import".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 4,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_29(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "if".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 15,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                c if is_separator(c) => {
                    self.token = Some(Token {
                        token: self.scanned_characters.clone(),
                        symbol_type: SymbolType::Identifier,
                        line_number: self.line_number,
                        code: 1,
                    })
                }
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_30(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "in".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 20,
            });
        } else {
            match c {
                't' => self.state_31(),
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_31(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "int".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 28,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_32(&mut self) {
        let c = self.read_character();

        match c {
            'b' => self.state_33(),
            'n' => self.state_40(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_33(&mut self) {
        let c = self.read_character();

        match c {
            's' => self.state_34(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_34(&mut self) {
        let c = self.read_character();

        match c {
            't' => self.state_35(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_35(&mut self) {
        let c = self.read_character();

        match c {
            'r' => self.state_36(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_36(&mut self) {
        let c = self.read_character();

        match c {
            'a' => self.state_37(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_37(&mut self) {
        let c = self.read_character();

        match c {
            'c' => self.state_38(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_38(&mut self) {
        let c = self.read_character();

        match c {
            't' => self.state_39(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_39(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "abstract".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 5,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_40(&mut self) {
        let c = self.read_character();

        match c {
            'd' => self.state_41(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_41(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "and".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 26,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_42(&mut self) {
        let c = self.read_character();

        match c {
            'i' => self.state_43(),
            'a' => self.state_47(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_43(&mut self) {
        let c = self.read_character();

        match c {
            'n' => self.state_44(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_44(&mut self) {
        let c = self.read_character();

        match c {
            'a' => self.state_45(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_45(&mut self) {
        let c = self.read_character();

        match c {
            'l' => self.state_46(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_46(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "final".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 6,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_47(&mut self) {
        let c = self.read_character();

        match c {
            'l' => self.state_48(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_48(&mut self) {
        let c = self.read_character();

        match c {
            's' => self.state_49(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_49(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_50(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_50(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "false".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 25,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_51(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_52(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_52(&mut self) {
        let c = self.read_character();

        match c {
            'a' => self.state_53(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_53(&mut self) {
        let c = self.read_character();

        match c {
            'l' => self.state_54(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_54(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_55(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_55(&mut self) {
        let c = self.read_character();

        match c {
            'd' => self.state_56(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_56(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "sealed".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 7,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_57(&mut self) {
        let c = self.read_character();

        match c {
            'o' => self.state_58(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_58(&mut self) {
        let c = self.read_character();

        match c {
            'o' => self.state_59(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_59(&mut self) {
        let c = self.read_character();

        match c {
            'l' => self.state_60(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_60(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "bool".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 30,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_61(&mut self) {
        let c = self.read_character();

        match c {
            'l' => self.state_62(),
            'a' => self.state_66(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_62(&mut self) {
        let c = self.read_character();

        match c {
            'a' => self.state_63(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_63(&mut self) {
        let c = self.read_character();

        match c {
            's' => self.state_64(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_64(&mut self) {
        let c = self.read_character();

        match c {
            's' => self.state_65(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_65(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "class".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 10,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_66(&mut self) {
        let c = self.read_character();

        match c {
            's' => self.state_67(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_67(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_68(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_68(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "case".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 18,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                c if is_separator(c) => {
                    self.token = Some(Token {
                        token: self.scanned_characters.clone(),
                        symbol_type: SymbolType::Identifier,
                        line_number: self.line_number,
                        code: 1,
                    })
                }
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_69(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_70(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_70(&mut self) {
        let c = self.read_character();

        match c {
            'f' => self.state_71(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_71(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "def".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 13,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                c if is_separator(c) => {
                    self.token = Some(Token {
                        token: self.scanned_characters.clone(),
                        symbol_type: SymbolType::Identifier,
                        line_number: self.line_number,
                        code: 1,
                    })
                }
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_72(&mut self) {
        let c = self.read_character();

        match c {
            'l' => self.state_73(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_73(&mut self) {
        let c = self.read_character();

        match c {
            's' => self.state_74(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_74(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_75(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_75(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "else".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 16,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_76(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "=".to_string(),
                symbol_type: SymbolType::SpecialSymbol,
                line_number: self.line_number,
                code: 38,
            });
        } else {
            match c {
                '>' => self.state_77(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_77(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "=>".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 19,
            });
        } else {
            self.error = Some(Error {
                error_type: ErrorType::InvalidSymbol,
                token: self.scanned_characters.clone(),
            });
        }
    }

    fn state_78(&mut self) {
        let c = self.read_character();

        if DEBUG {
            eprintln!("enter state_78");
        }

        match c {
            '=' => self.state_79(),
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_79(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "<=".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 14,
            });
        } else {
            self.error = Some(Error {
                error_type: ErrorType::InvalidSymbol,
                token: self.scanned_characters.clone(),
            });
        }
    }

    fn state_80(&mut self) {
        let c = self.read_character();

        match c {
            'o' => self.state_81(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_81(&mut self) {
        let c = self.read_character();

        match c {
            't' => self.state_82(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_82(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "not".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 23,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_83(&mut self) {
        let c = self.read_character();

        match c {
            'r' => self.state_84(),
            'b' => self.state_85(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_84(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "or".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 27,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_85(&mut self) {
        let c = self.read_character();

        match c {
            'j' => self.state_86(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_86(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_87(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_87(&mut self) {
        let c = self.read_character();

        match c {
            'c' => self.state_88(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_88(&mut self) {
        let c = self.read_character();

        match c {
            't' => self.state_89(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_89(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "object".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 11,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_90(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_91(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_91(&mut self) {
        let c = self.read_character();

        match c {
            't' => self.state_92(),
            'a' => self.state_96(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_92(&mut self) {
        let c = self.read_character();

        match c {
            'u' => self.state_93(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_93(&mut self) {
        let c = self.read_character();

        match c {
            'r' => self.state_94(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_94(&mut self) {
        let c = self.read_character();

        match c {
            'n' => self.state_95(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_95(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "return".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 22,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_96(&mut self) {
        let c = self.read_character();

        match c {
            'l' => self.state_97(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_97(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "real".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 29,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_98(&mut self) {
        let c = self.read_character();

        match c {
            'r' => self.state_99(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_99(&mut self) {
        let c = self.read_character();

        match c {
            'u' => self.state_100(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_100(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_101(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_101(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "true".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 24,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_102(&mut self) {
        let c = self.read_character();

        match c {
            'a' => self.state_103(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_103(&mut self) {
        let c = self.read_character();

        match c {
            'l' => self.state_104(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_104(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "val".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 12,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_105(&mut self) {
        let c = self.read_character();

        match c {
            'h' => self.state_106(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_106(&mut self) {
        let c = self.read_character();

        match c {
            'i' => self.state_107(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_107(&mut self) {
        let c = self.read_character();

        match c {
            'l' => self.state_108(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_108(&mut self) {
        let c = self.read_character();

        match c {
            'e' => self.state_109(),
            c if c.is_ascii_alphabetic() => self.state_114(),
            '.' => self.state_114(),
            '0'..='9' => self.state_114(),
            c if is_separator(c) => {
                self.token = Some(Token {
                    token: self.scanned_characters.clone(),
                    symbol_type: SymbolType::Identifier,
                    line_number: self.line_number,
                    code: 1,
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_109(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: "while".to_string(),
                symbol_type: SymbolType::Keyword,
                line_number: self.line_number,
                code: 17,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '.' => self.state_114(),
                '0'..='9' => self.state_114(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_110(&mut self) {
        let c = self.read_character();

        match c {
            '0'..='9' => self.state_111(),
            '.' => {
                self.error = Some(Error {
                    error_type: ErrorType::ConstantHasTooManyPeriods,
                    token: self.scanned_characters.clone(),
                })
            }
            _ => {
                self.error = Some(Error {
                    error_type: ErrorType::InvalidSymbol,
                    token: self.scanned_characters.clone(),
                })
            }
        }
    }

    fn state_111(&mut self) {
        let c = self.read_character();

        if is_separator(c) {
            self.token = Some(Token {
                token: self.scanned_characters.clone(),
                symbol_type: SymbolType::Constant,
                line_number: self.line_number,
                code: 2,
            });
        } else {
            match c {
                '0'..='9' => self.state_111(), // Recurse
                '.' => {
                    self.error = Some(Error {
                        error_type: ErrorType::ConstantHasTooManyPeriods,
                        token: self.scanned_characters.clone(),
                    })
                }
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_112(&mut self) {
        let c = self.read_character();

        if DEBUG {
            eprintln!("currently in state 112");
        }

        if is_separator(c) {
            self.token = Some(Token {
                token: self.scanned_characters.clone(),
                symbol_type: SymbolType::Constant,
                line_number: self.line_number,
                code: 2,
            });
        } else {
            match c {
                '0'..='9' => self.state_112(), // Recurse
                '.' => self.state_113(),
                c if c.is_alphabetic() => self.state_127(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    fn state_113(&mut self) {
        let c = self.read_character();

        if DEBUG {
            eprintln!("entered state 113");
        }

        if is_separator(c) {
            self.token = Some(Token {
                token: self.scanned_characters.clone(),
                symbol_type: SymbolType::Constant,
                line_number: self.line_number,
                code: 2,
            });
        } else {
            match c {
                '0'..='9' => self.state_113(), // Recurse
                '.' => self.state_126(),
                _ => {
                    self.error = Some(Error {
                        error_type: ErrorType::InvalidSymbol,
                        token: self.scanned_characters.clone(),
                    })
                }
            }
        }
    }

    // This state is reserved for the identifiers partition of the DFA.
    fn state_114(&mut self) {
        let c = self.read_character();
        if DEBUG {
            eprintln!("state 114 entered");
        }

        if is_separator(c) {
            self.token = Some(Token {
                token: self.scanned_characters.clone(),
                symbol_type: SymbolType::Identifier,
                line_number: self.line_number,
                code: 1,
            });
        } else {
            match c {
                c if c.is_ascii_alphabetic() => self.state_114(),
                '0'..='9' => self.state_114(),
                '.' => self.state_114(),
                _ => self.state_128(),
            }
        }
    }

    // These states are reserved for the special symbol portions of the DFA.
    // Since special symbols can also be considered token separators, we do not need to worry about checking for whitespace after the symbol.
    fn state_115(&mut self) {
        // Any time, no matter what, that we encounter the pound sign, we should set the comment flag to true.
        self.comment = true;

        self.token = Some(Token {
            token: "#".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 254,
        });
    }

    fn state_116(&mut self) {
        self.token = Some(Token {
            token: ";".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 31,
        });
    }

    fn state_117(&mut self) {
        self.token = Some(Token {
            token: "{".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 32,
        });
    }

    fn state_118(&mut self) {
        self.token = Some(Token {
            token: "}".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 33,
        });
    }

    fn state_119(&mut self) {
        self.token = Some(Token {
            token: "(".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 34,
        });
    }

    fn state_120(&mut self) {
        self.token = Some(Token {
            token: ")".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 35,
        });
    }

    fn state_121(&mut self) {
        self.token = Some(Token {
            token: ":".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 36,
        });
    }

    fn state_122(&mut self) {
        self.token = Some(Token {
            token: ",".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 37,
        });
    }

    fn state_123(&mut self) {
        self.token = Some(Token {
            token: "+".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 39,
        });
    }

    fn state_124(&mut self) {
        self.token = Some(Token {
            token: "*".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 40,
        });
    }

    fn state_125(&mut self) {
        self.token = Some(Token {
            token: "@".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: self.line_number,
            code: 41,
        });
    }

    // This portion of the DFA is reserved for known errors.
    // This is an additional state intended to handle the case where a constant has too many periods.
    fn state_126(&mut self) {
        let c = self.read_character();

        if DEBUG {
            eprintln!("entered state 126");
        }

        if is_separator(c) {
            self.error = Some(Error {
                error_type: ErrorType::ConstantHasTooManyPeriods,
                token: self.scanned_characters.clone(),
            });
        } else {
            self.state_126();
        }
    }

    // Handle the case where what appears to be an identifier begins with a number
    fn state_127(&mut self) {
        let c = self.read_character();

        if DEBUG {
            eprintln!("entered state 127");
        }

        if is_separator(c) {
            self.error = Some(Error {
                error_type: ErrorType::IdentifierBeginsWithNumber,
                token: self.scanned_characters.clone(),
            });
        } else {
            self.state_127();
        }
    }

    // Handle the case where we encounter a clearly invalid symbol based on characters that are not allowed.
    fn state_128(&mut self) {
        let c = self.read_character();

        if DEBUG {
            eprintln!("entered state 128");
        }

        if is_separator(c) {
            self.error = Some(Error {
                error_type: ErrorType::InvalidSymbol,
                token: self.scanned_characters.clone(),
            });
        } else {
            self.state_128();
        }
    }
}

// Keeping track of all of the special symbols in our language.
const SPECIAL_SYMBOLS: [char; 12] = ['#', ';', '{', '}', '(', ')', ':', ',', '=', '+', '*', '@'];

// Given a character, determine if the symbol is a special symbol.
fn is_special_symbol(c: char) -> bool {
    SPECIAL_SYMBOLS.contains(&c)
}

fn is_separator(c: char) -> bool {
    c.is_whitespace() || is_special_symbol(c)
}

// Given a special symbol, match it to its corresponding integer code for use in the scanner.
fn match_special_symbol_to_code(special_symbol: char) -> u8 {
    match special_symbol {
        ';' => 31,
        '{' => 32,
        '}' => 33,
        '(' => 34,
        ')' => 35,
        ':' => 36,
        ',' => 37,
        '=' => 38,
        '+' => 39,
        '*' => 40,
        '@' => 41,
        _ => 255, // this is bad and we do not want to encounter it
    }
}

// A NOTE: EVERYTHING BELOW THIS IS NOT PART OF THE DFA. THESE ARE ALL UNIT TESTS AND ARE NOT PERTINENT TO THE GRADING OF THIS PROGRAM.

#[cfg(test)]
mod scanner_keyword_tests {
    use crate::bookkeeper::Bookkeeper;
    use crate::scanner::*;

    #[test]
    fn test_whitespace() {
        let src_str = " \t\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn: Option<&Token> = src.token_request().0;
        let expected: Option<&Token> = None;

        // Basically what this test is doing is checking if tkn == None.
        debug_assert_eq!(tkn, expected);
    }

    // Verify that the scanner can recognize the keyword package.
    #[test]
    fn test_package() {
        let src_str = "package a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "package".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 3,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    // Verify that the scanner can recognize the protected keyword.
    #[test]
    fn test_protected() {
        let src_str = "protected package a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "protected".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 9,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    // Verify that the scanner can recognize the keyword "int."
    #[test]
    fn test_int() {
        let src_str = "int a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "int".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 28,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    // Verify that the scanner can recognize the keyword "if."
    #[test]
    fn test_if() {
        let src_str = "if a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "if".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 15,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    // Verify that the scanner can recognize the keyword "in."
    #[test]
    fn test_in() {
        let src_str = "in a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "in".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 20,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    // Verify that the scanner can recognize the keyword "import."
    #[test]
    fn test_import() {
        let src_str = "import package a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "import".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 4,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_abstract() {
        let src_str = "abstract package a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "abstract".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 5,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_and() {
        let src_str = "and is true".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "and".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 26,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_final() {
        let src_str = "final int a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "final".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 6,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_false() {
        let src_str = "false and true".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "false".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 25,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_sealed() {
        let src_str = "sealed int a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "sealed".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 7,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_class() {
        let src_str = "class a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "class".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 10,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_object() {
        let src_str = "object a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "object".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 11,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_val() {
        let src_str = "val a = false;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "val".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 12,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_def() {
        let src_str = "def this.is.function".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "def".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 13,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_leq() {
        let src_str = "x <= 5".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        // Skip first token just to see what happens
        src.token_request();
        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "<=".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 14,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_else() {
        let src_str = "else if".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "else".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 16,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_while() {
        let src_str = "while is true".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "while".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 17,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_case() {
        let src_str = "case a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "case".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 18,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_geq() {
        let src_str = "x => 5".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        src.token_request();
        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "=>".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 19,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_return() {
        let src_str = "return a;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "return".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 22,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_not() {
        let src_str = "not true".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "not".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 23,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_true() {
        let src_str = "true and false = false".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "true".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 24,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_or() {
        let src_str = "or is true".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "or".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 27,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_real() {
        let src_str = "real a = 5.0".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "real".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 29,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    #[test]
    fn test_bool() {
        let src_str = "bool b = false;".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let tkn = src.token_request().0.unwrap();

        let expected = &Some(Token {
            token: "bool".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 1,
            code: 30,
        })
        .unwrap();

        assert_eq!(tkn, expected);
    }

    // Test that the scanner recognizes an invalid keyword and throws the proper error.
    #[test]
    fn test_invalid_keyword() {
        let src_str = "this_is_not_a_valid_keyword\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        src.token_request();

        let expected = Some(Error {
            error_type: ErrorType::InvalidSymbol,
            token: "this_is_not_a_valid_keyword".to_string(),
        });

        let actual = src.error;

        assert_eq!(expected, actual);
    }
}

#[cfg(test)]
mod scanner_constant_tests {
    use crate::scanner::*;

    #[test]
    fn test_zero_point_zero() {
        let src_str = "0.0\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "0.0".to_string(),
            symbol_type: SymbolType::Constant,
            line_number: 1,
            code: 2,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_two_hundred_point_six() {
        let src_str = "200.6\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "200.6".to_string(),
            symbol_type: SymbolType::Constant,
            line_number: 1,
            code: 2,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_point_four_seven() {
        let src_str = ".47\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: ".47".to_string(),
            symbol_type: SymbolType::Constant,
            line_number: 1,
            code: 2,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_zero() {
        let src_str = "00\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "00".to_string(),
            symbol_type: SymbolType::Constant,
            line_number: 1,
            code: 2,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_too_many_periods() {
        let src_str = "25.2.5\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        src.token_request();

        let expected_error: Error = Some(Error {
            error_type: ErrorType::ConstantHasTooManyPeriods,
            token: "25.2.5".to_string(),
        })
        .unwrap();

        let actual_error: Error = src.error.unwrap();

        assert_eq!(expected_error, actual_error);
    }
}

#[cfg(test)]
mod scanner_id_tests {
    use crate::scanner::*;

    #[test]
    fn test_x() {
        let src_str = "x\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "x".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_xx() {
        let src_str = "xx\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "xx".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_x_with_semicolon() {
        let src_str = "x;\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "x".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_a() {
        let src_str = "a\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "a".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_aa() {
        let src_str = "aa\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "aa".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_aa_with_semicolon() {
        let src_str = "aa;\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "aa".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_print_but_without_t() {
        let src_str = "prin\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "prin".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_print_but_without_t_and_with_semicolon() {
        let src_str = "prin;\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "prin".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_one_of_dr_kim_crazy_identifiers() {
        let src_str = "b.c...67\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "b.c...67".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_one_of_dr_kim_crazy_identifiers_with_semicolon() {
        let src_str = "b.c...67;\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "b.c...67".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_one_of_dr_kim_crazy_identifiers_with_parentheses() {
        let src_str = "b.c...67()\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "b.c...67".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);

        let expected: &Token = &Some(Token {
            token: "(".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 34,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);

        let expected: &Token = &Some(Token {
            token: ")".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 35,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_i() {
        let src_str = "i\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "i".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_i_with_semicolon() {
        let src_str = "i;\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "i".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_ii() {
        let src_str = "ii\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "ii".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_ii_with_semicolon() {
        let src_str = "ii;\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "ii".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 1,
            code: 1,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }
}

#[cfg(test)]
mod scanner_special_symbol_tests {
    use crate::scanner::*;

    #[test]
    fn test_equal_sign() {
        let src_str = "=\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "=".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 38,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_semicolon() {
        let src_str = ";\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: ";".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 31,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_left_bracket() {
        let src_str = "{\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "{".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 32,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_right_bracket() {
        let src_str = "}\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "}".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 33,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_left_parenthesis() {
        let src_str = "(\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "(".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 34,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_right_parenthesis() {
        let src_str = ")\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: ")".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 35,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_colon() {
        let src_str = ":\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: ":".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 36,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_comma() {
        let src_str = ",\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: ",".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 37,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_plus_sign() {
        let src_str = "+\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "+".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 39,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_star() {
        let src_str = "*\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "*".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 40,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_at_sign() {
        let src_str = "@\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        let expected: &Token = &Some(Token {
            token: "@".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 1,
            code: 41,
        })
        .unwrap();

        let actual: &Token = src.token_request().0.unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_comments() {
        let src_str = "int a # this is a comment\nint b".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        src.token_request();
        src.token_request();
        src.token_request();

        let mut tkn = src.token_request().0;
        while tkn.is_none() {
            tkn = src.token_request().0;
        }

        let expected_tkn: &Token = &Some(Token {
            token: "int".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 2,
            code: 28,
        })
        .unwrap();

        assert_eq!(expected_tkn, tkn.unwrap());
    }

    #[test]
    fn test_dollar_sign() {
        let src_str = "\n$\n".to_string();
        let symtab: Bookkeeper = Bookkeeper::new();
        let mut src = Scanner::new(src_str, symtab);

        println!("{:?}", src.token_request());

        let expected: bool = true;
        let actual: bool = src.is_done();

        assert_eq!(expected, actual);
    }
}

#[cfg(test)]
mod bigger_scanner_tests {
    use crate::bookkeeper::Bookkeeper;
    use crate::scanner::*;

    #[test]
    fn test_some_text() {
        let src_str = "
        int a;
        package b;
        integers
        this is a test of identifiers
        # nothing in this line should be taken seriously
        int c;
        $
        "
        .to_string();
        let symtab = Bookkeeper::new();
        let mut src: Scanner = Scanner::new(src_str, symtab);

        let mut tkn = src.token_request().0.unwrap();

        let expected: &Token = &Some(Token {
            token: "int".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 2,
            code: 28,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected = &Some(Token {
            token: "a".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 2,
            code: 1,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: ";".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 2,
            code: 31,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "package".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 3,
            code: 3,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "b".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 3,
            code: 1,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: ";".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 3,
            code: 31,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "integers".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 4,
            code: 1,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "this".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 5,
            code: 1,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "is".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 5,
            code: 1,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "a".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 5,
            code: 1,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "test".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 5,
            code: 1,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "of".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 5,
            code: 1,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "identifiers".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 5,
            code: 1,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "#".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 6,
            code: 254,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "int".to_string(),
            symbol_type: SymbolType::Keyword,
            line_number: 7,
            code: 28,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: "c".to_string(),
            symbol_type: SymbolType::Identifier,
            line_number: 7,
            code: 1,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        tkn = src.token_request().0.unwrap();
        let expected: &Token = &Some(Token {
            token: ";".to_string(),
            symbol_type: SymbolType::SpecialSymbol,
            line_number: 7,
            code: 31,
        })
        .unwrap();

        assert_eq!(expected, tkn);

        src.token_request();
        let expected: bool = true;
        let actual: bool = src.is_done();

        assert_eq!(expected, actual);
    }
}
