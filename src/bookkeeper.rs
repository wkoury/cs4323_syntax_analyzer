#![warn(clippy::all)]

use std::collections::HashSet;

// Types of symbols in the Simple Scala programming language.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SymbolType {
    Keyword,
    Constant,
    Identifier,
    SpecialSymbol,
}

// This tells the program how to println a SymbolType in a nice way.
impl std::fmt::Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable = match *self {
            SymbolType::Keyword => "Keyword".to_string(),
            SymbolType::Constant => "Constant".to_string(),
            SymbolType::Identifier => "Identifier".to_string(),
            SymbolType::SpecialSymbol => "Special Symbol".to_string(),
        };

        write!(f, "{}", printable)
    }
}

// A struct to store each token that we create.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Token {
    pub(crate) token: String,
    pub(crate) symbol_type: SymbolType,
    pub(crate) line_number: usize,
    pub(crate) code: u8,
}

// This tells the program how to println a token in a nice way.
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{0: <30} | {1: <30} | {2: <30}",
            self.token,
            self.symbol_type.to_string(),
            self.line_number.to_string()
        )
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SymbolTableToken {
    pub(crate) token: String,
    pub(crate) symbol_type: SymbolType,
    pub(crate) code: u8,
}

// This tells the program how to println a symbol table token in a nice way.
impl std::fmt::Display for SymbolTableToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{0: <30} | {1: <30} | {2: <}",
            self.token,
            self.symbol_type.to_string(),
            self.code
        )
    }
}

// Given a Token, return a SymbolTableToken.
pub fn convert_token_to_symbol_table_token(tkn: Token) -> SymbolTableToken {
    SymbolTableToken {
        token: tkn.token.clone(),
        symbol_type: tkn.symbol_type,
        code: tkn.code,
    }
}

// An implementation of our Bookkeeper.
pub struct Bookkeeper {
    pub(crate) symbols: HashSet<SymbolTableToken>,
}

impl Bookkeeper {
    // Create a new bookkeeper.
    pub fn new() -> Self {
        Bookkeeper {
            symbols: HashSet::new(),
        }
    }

    // Insert a token into the bookkeeper, prevent the creation of duplicates using a HashSet.
    pub fn insert(&mut self, t: SymbolTableToken) {
        self.symbols.insert(t);
    }
}

// A NOTE: everything below this comment is a unit test and can be disregarded by the grader.

#[cfg(test)]
mod symbol_table_tests {
    use crate::bookkeeper::*;

    #[test]
    fn test_one_entry() {
        let mut symtab = Bookkeeper::new();
        let tkn = SymbolTableToken {
            token: "test".to_string(),
            symbol_type: SymbolType::Identifier,
            code: 1,
        };

        symtab.insert(tkn);

        let expected = 1;
        let actual = symtab.symbols.len();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_duplicate_entries() {
        let mut symtab = Bookkeeper::new();
        let tkn = SymbolTableToken {
            token: "test".to_string(),
            symbol_type: SymbolType::Identifier,
            code: 1,
        };

        let dup_tkn = tkn.clone();

        symtab.insert(tkn);
        symtab.insert(dup_tkn);

        let expected = 1;
        let actual = symtab.symbols.len();

        assert_eq!(expected, actual);
    }
}
