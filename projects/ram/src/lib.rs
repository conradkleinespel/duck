//! # Examples
//!
//! ## Empty source code matcher
//!
//! ```
//! use ram::Automaton;
//!
//! enum TokenType {
//!     End,
//! }
//!
//! // Create the FSM (2 states, 0 or 1) that will parse the source code
//! let mut am = Automaton::new(0, 1);
//! // When the FSM hits the end of the source, go to state 1, the final state
//! am.find_end(TokenType::End as i32, 0, 1);
//!
//! // Run the FSM with an empty string as the source code
//! let source_code = format!("");
//! let runner = am.run(source_code);
//!
//! // Print the parsed tokens to the console
//! println!("{:?}", runner.tokens);
//! ```

extern crate regex;

use regex::Regex;
use std::ops::Deref;

pub struct Automaton {
    pub state_initial: i32,
    pub state_final: i32,
    finders: Vec<Finder>
}

pub struct Token {
    pub type_id: i32,
    pub text: std::string::String
}

pub struct Runner<'a> {
    pub source: std::string::String,
    automaton: &'a Automaton,
    pub state: i32,
    pub tokens: Vec<Token>
}

pub struct Finder {
    pub state_from: i32,
    pub state_to: i32,
    callback: fn(runner: &mut Runner, finder: & Finder) -> bool,
    regex: Option<Regex>,
    automaton: Option<Automaton>,
    pub token_type: i32,
    pub join_tokens: bool
}

impl<'a> Automaton {
    pub fn new(state_initial: i32, state_final: i32) -> Automaton {
        Automaton {
            state_initial: state_initial,
            state_final: state_final,
            finders: vec![]
        }
    }

    pub fn run(&'a self, source: std::string::String) -> Runner<'a> {
        let mut runner = Runner {
            source: source,
            automaton: self,
            state: self.state_initial,
            tokens: vec![]
        };

        runner.run();

        runner
    }

    pub fn run_loop(&'a self, source: std::string::String) -> Runner<'a> {
        let mut runner = Runner {
            source: source,
            automaton: self,
            state: self.state_initial,
            tokens: vec![]
        };

        runner.run_loop();

        runner
    }

    pub fn find_custom(&mut self, token_type: i32, state_from: i32, state_to: i32, callback: fn(runner: &mut Runner, finder: & Finder) -> bool) {
        self.finders.push(Finder {
            state_from: state_from,
            state_to: state_to,
            callback: callback,
            regex: None,
            automaton: None,
            token_type: token_type,
            join_tokens: false
        })
    }

    fn finder_whitespace(runner: &mut Runner, finder: & Finder) -> bool {
        let ws = &[' ', '\t'];
        if runner.source.len() > 0 && ws.contains(&(runner.source.as_bytes()[0] as char)) {
            let mut num_spaces = 1;
            for i in 1..runner.source.len() {
                if ws.contains(&(runner.source.as_bytes()[i] as char)) {
                    num_spaces += 1;
                } else {
                    break;
                }
            }
            if num_spaces > 0 {
                let text = runner.source.deref()[..num_spaces].to_string();
                runner.add_token(Token::new(finder.token_type, text));
                return true;
            }
        }
        return false;
    }

    pub fn find_whitespace(&mut self, token_type: i32, state_from: i32, state_to: i32) {
        self.find_custom(token_type, state_from, state_to, Automaton::finder_whitespace);
    }

    fn finder_end(runner: &mut Runner, finder: & Finder) -> bool {
        if runner.source.len() == 0 {
            runner.add_token(Token::new(finder.token_type, "".to_string()));
            true
        } else {
            false
        }
    }

    pub fn find_end(&mut self, token_type: i32, state_from: i32, state_to: i32) {
        self.find_custom(token_type, state_from, state_to, Automaton::finder_end);
    }

    fn finder_regex(runner: &mut Runner, finder: & Finder) -> bool {
        match finder.regex.clone().unwrap().find(runner.source.clone().deref()) {
            Some(regex_match) => {
                if regex_match.start() == 0 {
                    let text = runner.source.clone().deref()[..regex_match.end()].to_string();
                    runner.add_token(Token::new(
                        finder.token_type,
                        text
                    ));
                    true
                } else {
                    false
                }
            }
            None => {
                false
            }
        }
    }

    pub fn find_regex(&mut self, token_type: i32, state_from: i32, state_to: i32, re: Regex) {
        self.finders.push(Finder {
            state_from: state_from,
            state_to: state_to,
            callback: Automaton::finder_regex,
            regex: Some(re),
            automaton: None,
            token_type: token_type,
            join_tokens: false
        })
    }

    fn automaton_run(runner: &mut Runner, finder: & Finder, am: & Automaton) -> bool {
        let sub_runner = am.run(runner.source.clone());
        if sub_runner.state == am.state_final {
            if finder.join_tokens {
                let mut full_text = std::string::String::new();
                for part in sub_runner.tokens.iter() {
                    full_text.push_str(part.text.deref());
                }
                runner.tokens.push(Token {
                    type_id: finder.token_type,
                    text: full_text
                });
            } else {
                for t in sub_runner.tokens.deref().iter() {
                    runner.tokens.push(t.clone());
                }
            }
            runner.source = sub_runner.source.clone();
            true
        } else {
            false
        }
    }

    fn finder_automaton(runner: &mut Runner, finder: & Finder) -> bool {
        match finder.automaton {
            Some(ref am) => Automaton::automaton_run(runner, finder, am),
            None => panic!()
        }
    }

    pub fn find_automaton(&'a mut self, state_from: i32, state_to: i32, am: Automaton) -> &'a mut Finder {
        self.finders.push(Finder {
            state_from: state_from,
            state_to: state_to,
            callback: Automaton::finder_automaton,
            regex: None,
            automaton: Some(am),
            token_type: -1,
            join_tokens: false
        });
        self.finders.last_mut().unwrap()
    }

    fn finder_me(runner: &mut Runner, finder: & Finder) -> bool {
        Automaton::automaton_run(runner, finder, runner.automaton)
    }

    pub fn find_me(&'a mut self, state_from: i32, state_to: i32) -> &'a mut Finder {
        self.finders.push(Finder {
            state_from: state_from,
            state_to: state_to,
            callback: Automaton::finder_me,
            regex: None,
            automaton: None,
            token_type: -1,
            join_tokens: false
        });
        self.finders.last_mut().unwrap()
    }
}

impl std::fmt::Debug for Automaton {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "([{} --> {}])", self.state_initial, self.state_final)
    }
}

impl std::clone::Clone for Automaton {
    fn clone(&self) -> Automaton {
        Automaton {
            state_initial: self.state_initial,
            state_final: self.state_final,
            finders: self.finders.clone()
        }
    }
}

impl Token {
    pub fn new(type_id: i32, text: std::string::String) -> Token {
        Token {
            type_id: type_id,
            text: text
        }
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "([{}] \"{}\")", self.type_id, self.text)
    }
}

impl std::clone::Clone for Token {
    fn clone(&self) -> Token {
        Token::new(self.type_id, self.text.clone())
    }
}

impl<'a> Runner<'a> {
    fn run(&mut self) {
        for finder in self.automaton.finders.iter() {
            let func = finder.callback;
            if self.state == finder.state_from && func(self, finder) == true {
                self.state = finder.state_to;
            }
        }
    }

    fn run_loop(&mut self) {
        let mut has_reached_end = false;
        loop {
            self.run();
            if self.completed() == false || has_reached_end {
                break;
            }
            self.state = self.automaton.state_initial;
            // we let the automaton run one last time before going out,
            // which allows it to catch an "EOF" token type if needed
            has_reached_end = self.source.len() == 0;
        }
    }

    pub fn add_token(&mut self, token: Token) {
        let len = token.text.len();
        self.tokens.push(token);
        self.source = self.source.deref()[len..].to_string();
    }

    pub fn completed(& self) -> bool {
        self.state == self.automaton.state_final
    }
}

impl<'a> std::fmt::Debug for Runner<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(runner [automaton: {:?}, current_state: {}])", self.automaton, self.state)
    }
}

impl Finder {
    pub fn join_tokens(&mut self,  token_type: i32) {
        self.join_tokens = true;
        self.token_type = token_type;
    }
}

impl std::clone::Clone for Finder {
    fn clone(&self) -> Finder {
        Finder {
            state_from: self.state_from,
            state_to: self.state_to,
            callback: self.callback,
            regex: self.regex.clone(),
            automaton: self.automaton.clone(),
            token_type: self.token_type,
            join_tokens: self.join_tokens
        }
    }
}
