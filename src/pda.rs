use crate::bookkeeper::Token;
use crate::rules::{code_to_string, EPSILON_CODE, EXPANSION_RULES, PARSING_RULES, START_SYMBOL};
use crate::stack::Stack;
use crate::DEBUG;

#[allow(clippy::upper_case_acronyms)]
pub struct PDA {
    pub(crate) q: bool, // the one state that we have. we can only accept if this is set to true.
    step: u32,          // What step in the computation we are at.
    pub(crate) stack: Stack,
}

impl PDA {
    pub fn new() -> Self {
        let stack: Stack = Stack::new();

        PDA {
            q: false,
            step: 1,
            stack,
        }
    }

    // Initialize the PDA by pushing the start symbol onto the stack.
    pub fn initialize(&mut self) {
        self.stack.push(START_SYMBOL); // 42, aka <scala>, is the start symbol in our grammar.
        self.q = true;

        print_step(
            self.step,
            "z0 (0)".to_string(),
            "None".to_string(),
            "Push start symbol.".to_string(),
        );
        self.step += 1;
    }

    // Run an iteration of the transition function.
    // Return whether the parsing can continue with a path towards acceptance, because we will want to reject as soon as we can.
    // The second entry in the tuple is whether or not a new lookahead token needs to be requested.
    pub fn transition(&mut self, lookahead: Token) -> (bool, bool) {
        // Pop the stack, create a default action message (error)
        let stack_top = self.stack.pop();
        let mut action: String = "ERROR".to_string();
        let mut ret: (bool, bool) = (false, false);
        if DEBUG {
            dbg!(stack_top);
            dbg!(&lookahead);
            dbg!(ret);
        }
        if !is_terminal_symbol(stack_top) {
            // Get our parsing rules, which we need to do first before we get our expansion rules.
            let rule = PARSING_RULES.get(&(stack_top, lookahead.code));
            if DEBUG {
                dbg!(rule);
            }
            if let Some(..) = rule {
                let tokens = EXPANSION_RULES.get(rule.unwrap()).unwrap().to_owned();
                action = format!("Use rule {}.", rule.unwrap());

                // Push the required tokens onto the stack in reverse order.
                for code in tokens.iter().rev() {
                    self.stack.push(code.to_owned());
                }
                ret.0 = true;
            } else {
                if DEBUG {
                    println!(
                        "Checking for an epsilon rule for non-terminal {}.",
                        stack_top
                    );
                }
                let epsilon_rule = PARSING_RULES.get(&(stack_top, EPSILON_CODE));

                if DEBUG {
                    dbg!(epsilon_rule);
                }

                if let Some(..) = epsilon_rule {
                    action = format!("Use rule {}.", epsilon_rule.unwrap());
                    let tokens = EXPANSION_RULES
                        .get(epsilon_rule.unwrap())
                        .unwrap()
                        .to_owned();

                    // Push the required tokens onto the stack in reverse order.
                    for code in tokens.iter().rev() {
                        self.stack.push(code.to_owned());
                    }
                    ret.0 = true;
                }
            }
        // If the stack top isn't the lookahead, we cannot accept the string
        } else if stack_top != lookahead.code {
            ret.0 = false;
        } else {
            // On the other hand, if the two are equal, then we consume, and will need a new lookahead token.
            if DEBUG {
                println!("MATCH.");
            }
            action = "Match.".to_string();
            ret.0 = true;
            ret.1 = true;
        }

        if DEBUG {
            dbg!(ret);
        }

        // Print the parse output, with the following format:
        // (Steps, stack top, lookahead, action)
        print_step(
            self.step,
            code_to_string(stack_top),
            format!("{} ({})", lookahead.token, lookahead.code),
            action,
        );
        self.step += 1;

        ret
    }
}

// Print a step in the parse outupt.
pub fn print_step(step: u32, stack_top: String, lookahead: String, action: String) {
    println!(
        "{0: <30} | {1: <30} | {2: <30} | {3: <}",
        step.to_string(),
        stack_top,
        lookahead,
        action
    );
}

// Determine whether a symbol is terminal or nonterminal.
pub fn is_terminal_symbol(code: u8) -> bool {
    code <= 41 && code > 0
}

#[cfg(test)]
mod is_terminal_symbol_tests {
    use crate::pda::{is_terminal_symbol, EPSILON_CODE};

    #[test]
    fn test_zero() {
        assert!(!is_terminal_symbol(0));
    }

    #[test]
    fn test_one() {
        assert!(is_terminal_symbol(1));
    }

    #[test]
    fn test_forty_one() {
        assert!(is_terminal_symbol(41));
    }

    #[test]
    fn test_forty_two() {
        assert!(!is_terminal_symbol(42));
    }

    #[test]
    fn test_forty_three() {
        assert!(!is_terminal_symbol(43));
    }

    #[test]
    fn test_sixty_nine() {
        assert!(!is_terminal_symbol(69));
    }

    // Testing a couple of edge cases.
    #[test]
    fn test_seventy() {
        assert!(!is_terminal_symbol(70));
    }

    #[test]
    fn test_epsilon() {
        assert!(!is_terminal_symbol(EPSILON_CODE));
    }
}
