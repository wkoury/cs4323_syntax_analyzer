use std::collections::{HashMap, HashSet};

use crate::stack::Stack;
use crate::DEBUG;

// We are using this u8 code to represent epsilon in the rules that have it. This should be a last-resort consideration for transitions.
pub const EPSILON_CODE: u8 = 253;
pub const START_SYMBOL: u8 = 42;

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
            0,
            "None".to_string(),
            "Push start symbol.".to_string(),
        )
    }

    // Run an iteration of the transition function.
    // Return whether the parsing can continue with a path towards acceptance, because we will want to reject as soon as we can.
    // The second entry in the tuple is whether or not a new lookahead token needs to be requested.
    pub fn transition(&mut self, lookahead: u8) -> (bool, bool) {
        let stack_top = self.stack.pop();
        let mut ret: (bool, bool) = (false, false);
        if DEBUG {
            dbg!(stack_top);
            dbg!(lookahead);
            dbg!(ret);
        }
        if !is_terminal_symbol(stack_top) {
            let rule = PARSING_RULES.get(&(stack_top, lookahead));
            if DEBUG {
                dbg!(rule);
            }
            if let Some(..) = rule {
                let tokens = EXPANSION_RULES.get(rule.unwrap()).unwrap().to_owned();

                // Push the required tokens onto the stack in reverse order.
                for code in tokens.iter().rev() {
                    self.stack.push(code.to_owned());
                }
                ret.0 = true;
            } else {
                // TODO: handle the case for epsilon rules
                if DEBUG {
                    println!(
                        "Checking for an epsilon rule for non-terminal {}.",
                        stack_top
                    );
                }
                let epsilon_rule = PARSING_RULES.get(&(stack_top, EPSILON_CODE));
                if let Some(..) = epsilon_rule {
                    self.stack.push(epsilon_rule.unwrap().to_owned());
                    ret.0 = true;
                }
            }
        // If the stack top isn't the lookahead, we cannot accept the string
        } else if stack_top != lookahead {
            ret.0 = false;
        } else {
            // On the other hand, if the two are equal, then we consume, and will need a new lookahead token.
            ret.1 = true;
        }

        if DEBUG {
            dbg!(ret);
        }

        // Print the parse output, with the following format:
        // (Steps, stack top, lookahead, action)
        print_step(
            self.step,
            stack_top,
            lookahead.to_string(),
            "ACTION (TODO)".to_string(),
        );
        self.step += 1;

        ret
    }
}

// Print a step in the parse outupt.
pub fn print_step(step: u32, stack_top: u8, lookahead: String, action: String) {
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

// This is the static HashMap that we will use to store the LL(1) parsing rules.
// The key is a tuple of u8s, like so: (top of stack, lookahead)
// The value is the id of the rule that we will need to use for the expansion. There are going to be a lot of combinations here, so try to use iterators wherever possible since we will only have to perform this operation once per runtime.
lazy_static! {
    pub static ref PARSING_RULES: HashMap<(u8, u8), u8> = {
        let mut rules = HashMap::<(u8, u8), u8>::new();

        // <scala>
        for tkn in FIRST_SCALA.iter() {
            rules.insert((42, tkn.to_owned()), 1);
        }

        // <packages>
        rules.insert((43, 3), 2);
        for tkn in FOLLOW_PACKAGES.iter() {
            rules.insert((43, tkn.to_owned()), 3);
        }

        // <imports>
        rules.insert((44, 4), 4);
        for tkn in FOLLOW_IMPORTS.iter() {
            rules.insert((44, tkn.to_owned()), 5);
        }

        // <scala-body>
        for tkn in FIRST_MODIFIER.iter() {
            rules.insert((45, tkn.to_owned()), 6);
        }
        rules.insert((45, EPSILON_CODE), 7);

        // <subbody>
        for tkn in FIRST_MODIFIER.iter() {
            rules.insert((46, tkn.to_owned()), 8);
        }

        // <modifier>
        rules.insert((47, 5), 9);
        rules.insert((47, 6), 10);
        rules.insert((47, 7), 11);
        rules.insert((47, 8), 12);
        rules.insert((47, 9), 13);

        // <subbody-tail>
        rules.insert((48, 10), 14);
        rules.insert((48, 11), 14);

        // <tail-type>
        rules.insert((49, 10), 15);
        rules.insert((49, 11), 15);

        // <block>
        rules.insert((50, 32), 16);

        // <stmts>
        for tkn in FIRST_STATEMENT.iter() {
            rules.insert((51, tkn.to_owned()), 18);
        }
        rules.insert((51, 33), 19);

        // <stmt>
        rules.insert((52, 12), 20);
        rules.insert((52, 13), 20);
        rules.insert((52, 1), 21);
        rules.insert((52, 15), 22);
        rules.insert((52, 17), 23);
        rules.insert((52, 18), 24);
        rules.insert((52, 20), 25);
        rules.insert((52, 21), 26);
        rules.insert((52, 22), 27);
        rules.insert((52, 32), 28);

        // <dcl>
        rules.insert((53, 12), 29);
        rules.insert((53, 13), 30);

        // <dcl-tail>
        rules.insert((54, 1), 31);

        // <ids>
        rules.insert((55, 1), 32);

        // <more-ids>
        rules.insert((56, 37), 33);
        rules.insert((56, 36), 34);
        rules.insert((56, 35), 34);

        // <type>
        rules.insert((57, 28), 35);
        rules.insert((57, 29), 36);
        rules.insert((57, 30), 37);

        // <asmt>
        rules.insert((58, 1), 38);

        // <if>
        rules.insert((59, 15), 39);

        // <while>
        rules.insert((60, 17), 40);

        // <case>
        rules.insert((61, 18), 41);

        // <in>
        rules.insert((62, 20), 42);

        // <out>
        rules.insert((63, 21), 43);

        // <return>
        rules.insert((64, 22), 44);

        // <expr>
        rules.insert((65, 1), 45);
        rules.insert((65, 2), 45);
        rules.insert((65, 34), 45);
        rules.insert((65, 23), 46);
        rules.insert((65, 24), 46);
        rules.insert((65, 25), 46);
        rules.insert((65, 41), 46);

        // <arith-expr>
        rules.insert((66, 1), 47);
        rules.insert((66, 2), 48);
        rules.insert((66, 34), 49);

        // <arith>
        rules.insert((67, 39), 50);
        rules.insert((67, 40), 51);
        for tkn in FOLLOW_ARITH.iter() {
            rules.insert((67, tkn.to_owned()), 52);
        }

        // <bool-expr>
        rules.insert((68, 23), 53);
        rules.insert((68, 24), 54);
        rules.insert((68, 25), 55);
        rules.insert((68, 41), 56);

        // <bool>
        rules.insert((69, 26), 57);
        rules.insert((69, 27), 58);
        rules.insert((69, EPSILON_CODE), 59);

        rules
    };
}

#[cfg(test)]
mod test_parsing_rules {
    use crate::pda::PARSING_RULES;

    #[test]
    fn test_rule_one_contains_first_scala() {
        assert!(PARSING_RULES.get(&(42, 5)).is_some());
    }

    #[test]
    fn test_rule_three_contains_follow_packages() {
        assert!(PARSING_RULES.get(&(43, 4)).is_some());
    }

    #[test]
    fn test_invalid_key_returns_none() {
        assert!(PARSING_RULES.get(&(69, 69)).is_none());
    }
}

// This is the static HashMap that we use to store the expansion rules.
// The key is the id (u8) of the rule, and the value is the list of tokens to add back to the stack.
lazy_static! {
    pub static ref EXPANSION_RULES: HashMap<u8, Vec<u8>> = {
        let mut rules = HashMap::new();

        /*
        We are going to put them in the order of the rules for the sake of readability. When we put these symbols into the stack, however, it needs to be done in reverse order. See below on how to do that:

        let a = vec![1,2,3];
        for i in a.iter().rev() {
            println!("{}", i);
        }

        */

        // Insert our rules, based on page 3 of the spec sheet and my markups in Notability.
        rules.insert(1, vec![43, 44, 45]);
        rules.insert(2, vec![3, 1, 31, 43]);
        rules.insert(3, vec![]); // this is an epsilon rule
        rules.insert(4, vec![4, 1, 31, 44]);
        rules.insert(5, vec![]);
        rules.insert(6, vec![46, 45]);
        rules.insert(7, vec![]);
        rules.insert(8, vec![47, 48]);
        rules.insert(9, vec![5]);
        rules.insert(10, vec![6]);
        rules.insert(11, vec![7]);
        rules.insert(12, vec![8]);
        rules.insert(13, vec![9]);
        rules.insert(14, vec![49, 50]);
        rules.insert(15, vec![10]);
        rules.insert(16, vec![11]);
        rules.insert(17, vec![32, 51, 33]);
        rules.insert(18, vec![52, 31, 51]);
        rules.insert(19, vec![]);
        rules.insert(20, vec![53]);
        rules.insert(21, vec![58]);
        rules.insert(22, vec![59]);
        rules.insert(23, vec![60]);
        rules.insert(24, vec![61]);
        rules.insert(25, vec![62]);
        rules.insert(26, vec![63]);
        rules.insert(27, vec![64]);
        rules.insert(28, vec![50]);
        rules.insert(29, vec![12, 54]);
        rules.insert(30, vec![13, 1, 34, 55, 35, 50]);
        rules.insert(31, vec![55, 36, 57]);
        rules.insert(32, vec![1, 56]);
        rules.insert(33, vec![37, 1, 56]);
        rules.insert(34, vec![]);
        rules.insert(35, vec![28]);
        rules.insert(36, vec![29]);
        rules.insert(37, vec![30]);
        rules.insert(38, vec![1, 14, 65]);
        rules.insert(39, vec![15, 34, 65, 35, 52, 31, 16, 52]);
        rules.insert(40, vec![17, 34, 65, 35, 52]);
        rules.insert(41, vec![18, 1, 38, 65, 19, 52]);
        rules.insert(42, vec![20, 34, 55, 35]);
        rules.insert(43, vec![21, 34, 55, 35]);
        rules.insert(44, vec![22, 34, 65, 35]);
        rules.insert(45, vec![66]);
        rules.insert(46, vec![67]);
        rules.insert(47, vec![1, 67]);
        rules.insert(48, vec![2, 67]);
        rules.insert(49, vec![34, 66, 35, 67]);
        rules.insert(50, vec![39, 66]);
        rules.insert(51, vec![40, 66]);
        rules.insert(52, vec![]);
        rules.insert(53, vec![23, 34, 68, 35, 69]);
        rules.insert(54, vec![24, 69]);
        rules.insert(55, vec![25, 69]);
        rules.insert(56, vec![41, 66, 66]);
        rules.insert(57, vec![26, 68]);
        rules.insert(58, vec![27, 68]);
        rules.insert(59, vec![]); // Rule 59 is indeed an epsilon rule

        rules
    };
}

#[cfg(test)]
mod test_rules_static_hashmap {
    use crate::pda::EXPANSION_RULES;

    #[test]
    fn test_rule_1() {
        let expected: u8 = 43;
        let actual = EXPANSION_RULES.get(&1).unwrap()[0];

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_rule_17() {
        let expected: u8 = vec![32, 51, 33][0];
        let actual: u8 = EXPANSION_RULES.get(&17).unwrap()[0];

        assert_eq!(expected, actual);
    }
}

// These are the FIRST(1) and FOLLOW(1) sets that we will want to use when we populate our transition functions.
lazy_static! {
    pub static ref FIRST_SCALA: HashSet<u8> = {
        let mut first_scala: HashSet<u8> = HashSet::<u8>::new();

        first_scala.insert(3);
        first_scala.insert(4);
        first_scala.insert(EPSILON_CODE); // Inserting the epsilon rule.
        // Also inserting the FIRST(1) of <modifier>
        for tkn in FIRST_MODIFIER.iter() {
            first_scala.insert(tkn.to_owned());
        }

        first_scala
    };

    pub static ref FIRST_MODIFIER: HashSet<u8> = {
        let mut first_modifier: HashSet<u8> = HashSet::<u8>::new();

        first_modifier.insert(5);
        first_modifier.insert(6);
        first_modifier.insert(7);
        first_modifier.insert(8);
        first_modifier.insert(9);

        first_modifier
    };

    pub static ref FOLLOW_PACKAGES: HashSet<u8> = {
        let mut follow_packages = HashSet::<u8>::new();

        follow_packages.insert(4);
        follow_packages.insert(EPSILON_CODE);
        for tkn in FIRST_MODIFIER.iter() {
            follow_packages.insert(tkn.to_owned());
        }

        follow_packages
    };

    pub static ref FOLLOW_IMPORTS: HashSet<u8> = {
        let mut follow_imports = HashSet::<u8>::new();

        follow_imports.insert(EPSILON_CODE);
        for tkn in FIRST_MODIFIER.iter() {
            follow_imports.insert(tkn.to_owned());
        }

        follow_imports
    };

    pub static ref FIRST_STATEMENT: HashSet<u8> = {
        let mut first_statement = HashSet::<u8>::new();

        first_statement.insert(12);
        first_statement.insert(13);
        first_statement.insert(1);
        first_statement.insert(15);
        first_statement.insert(17);
        first_statement.insert(18);
        first_statement.insert(20);
        first_statement.insert(21);
        first_statement.insert(22);
        first_statement.insert(32);

        first_statement
    };

    pub static ref FOLLOW_ARITH: HashSet<u8> = {
        let mut follow_arith = HashSet::<u8>::new();

        follow_arith.insert(31);
        follow_arith.insert(35);
        follow_arith.insert(19);
        follow_arith.insert(1);
        follow_arith.insert(2);
        follow_arith.insert(34);

        follow_arith
    };
}

// These are unit tests for our transition function rules.
#[cfg(test)]
mod test_transition_rules {
    use crate::pda::{EPSILON_CODE, FIRST_MODIFIER, FIRST_SCALA, FOLLOW_PACKAGES};

    #[test]
    fn test_first_scala_contains_package() {
        let expected: bool = true;
        let package_code: u8 = 3;
        let actual: bool = FIRST_SCALA.contains(&package_code);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_first_modifier_contains_sealed() {
        let expected: bool = true;
        let sealed_code: u8 = 7;
        let actual: bool = FIRST_MODIFIER.contains(&sealed_code);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_first_scala_contains_first_modifier() {
        let expected: bool = true;
        let sealed_code = 7;
        let actual: bool = FIRST_SCALA.contains(&sealed_code);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_follow_packages_contains_epsilon() {
        let expected: bool = true;
        let actual: bool = FOLLOW_PACKAGES.contains(&EPSILON_CODE);

        assert_eq!(expected, actual);
    }
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
