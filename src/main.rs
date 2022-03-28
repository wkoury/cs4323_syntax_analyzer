#![warn(clippy::all)]

// Importing standard library modules that we need.
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;

// This is a third-party library that enables printing of styled text to the terminal. It is not noticeable in the .txt output, but it was helpful in debugging this program.
use colored::*;
// lazy_static is a third-party library that enables the creation of "static" (create once, use forever) data structures in Rust.
#[macro_use]
extern crate lazy_static;

// Importing our third-party files.
mod bookkeeper;
mod error;
mod parser;
mod pda;
mod scanner;

use crate::bookkeeper::{convert_token_to_symbol_table_token, Bookkeeper, SymbolType, Token};
use crate::error::Error;
use crate::parser::Parser;
use crate::scanner::Scanner;

// Main. What gets called when we invoke the program.
fn main() {
    // Collect the command-line arguments
    let args: Vec<String> = env::args().collect();
    // Check for invalid use and terminate if required
    if args.len() != 2 {
        print!("{}", "Usage: ".bold().red());
        println!("{}", "./scanner <filename>".red());
        process::exit(1);
    }

    // Attempt to open the file
    let path = Path::new(&args[1]);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("Couldn't read {}: {}", display, why),
        Ok(_) => println!("{}\n{}", "Source program:".blue().bold(), s),
    };

    let s_clone = s.clone();

    // Initialize the source
    let mut src: Scanner = Scanner::new(s_clone);
    // The above one is the old one, soon it will be replaced with this one:
    let mut _parser: Parser = Parser::new(s);

    //Initialize the symbol table
    let mut symtab: Bookkeeper = Bookkeeper::new();

    // Print the header of our table to STDOUT.
    println!(
        "{0: <30} | {1: <30} | {2: <30}",
        "Token", "Symbol Type", "Line Number"
    );

    // While the source is not done, keep scanning for tokens.
    while !src.is_done() {
        let scan_result = src.token_request();
        // These are options, which be of type Some() or None.
        let tkn: Option<&Token> = scan_result.0;
        let err: Option<&Error> = scan_result.1;

        if let Some(..) = err {
            print!("{}", "An error occurred: ".red().bold());
            println!("{:?}", err);
        }

        if let Some(..) = tkn {
            // If we have a token of some kind, print it.
            let tkn_ref = tkn.unwrap();
            println!("{}", tkn_ref);

            // If the token belongs in the symbol table, add it.
            if tkn.unwrap().symbol_type == SymbolType::Constant
                || tkn.unwrap().symbol_type == SymbolType::Identifier
            {
                symtab.insert(convert_token_to_symbol_table_token(tkn.unwrap().clone()));
            }
        }
    }

    // Print out the contents of the symbol table.
    println!("{}", "Symbol table contents:".blue().bold());
    // Table header
    println!(
        "{0: <30} | {1: <30} | {2: <}",
        "Token", "Symbol Type", "Code"
    );
    for symbol in symtab.symbols {
        println!("{}", symbol);
    }
}
