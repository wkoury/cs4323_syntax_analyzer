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
mod rules;
mod scanner;
mod stack;

use crate::bookkeeper::Bookkeeper;
use crate::parser::Parser;

pub const DEBUG: bool = false;

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

    //Initialize the symbol table
    let symtab: Bookkeeper = Bookkeeper::new();

    let mut parser: Parser = Parser::new(s, symtab);

    println!("{}\n", "Parse Output:".blue().bold());
    parser.parse();

    // Print out the contents of the symbol table.
    println!("{}", "Symbol table contents:".blue().bold());
    // Table header
    println!(
        "{0: <30} | {1: <30} | {2: <}",
        "Token", "Symbol Type", "Code"
    );
    for symbol in parser.scanner.symtab.symbols {
        println!("{}", symbol);
    }
}
