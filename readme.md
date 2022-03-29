# About

Syntactical analyzer for Dr. Changwook Kim's Compiler Construction course. Spring 2022.

# Notes
## Integer Codes
There are some special integer codes that need to be kept track of by the programmer and outside observers:
- `255` -> errors
- `254` -> `#`
- `253` -> `epsilon`

# To Do
- [x] Be able to print a token given a `u8` code.
- [x] Print out each task as you perform it
- [x] Debug the LL(1) parse table and deterministic PDA
- [x] Add new parser errors as you discover them
- [x] Refactor `main` to do everything in the `parse()` function, which should only be called one time
- [x] Add integer codes to the symbol table
- [x] Refactor the scanner to only send tokens to the parser upon a token request
- [x] Implement the LL(1) parse table and deterministic PDA
