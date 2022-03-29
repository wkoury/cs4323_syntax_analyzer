# About

Syntactical analyzer for Dr. Changwook Kim's Compiler Construction course. Spring 2022.

# Notes
## Integer Codes
There are some special integer codes that need to be kept track of by the programmer and outside observers:
- `255` -> errors
- `254` -> `#`
- `253` -> `epsilon`

# To Do
- [ ] Replace the `u8`s in the PDA with `Token`s
- [x] Debug the LL(1) parse table and deterministic PDA
- [ ] Add new parser errors as you discover them
- [ ] Print out each task as you perform it
- [x] Refactor `main` to do everything in the `parse()` function, which should only be called one time
- [x] Add integer codes to the symbol table
- [x] Refactor the scanner to only send tokens to the parser upon a token request
- [x] Implement the LL(1) parse table and deterministic PDA
