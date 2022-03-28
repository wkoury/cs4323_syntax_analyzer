# About

Syntactical analyzer for Dr. Changwook Kim's Compiler Construction course. Spring 2022.

# Notes
## Integer Codes
There are some special integer codes that need to be kept track of by the programmer and outside observers:
- `255` -> errors
- `254` -> `#`
- `253` -> `epsilon`

# To Do
- [ ] Refactor `main` to do everything in the `parse()` function, which should only be called one time
- [x] Add integer codes to the symbol table
- [ ] Refactor the scanner to only send tokens to the parser upon a token request
- [ ] Add new parser errors as you discover them
- [ ] Implement the LL(1) parse table and deterministic PDA
