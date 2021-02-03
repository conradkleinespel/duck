use ram::Automaton;
use regex::Regex;

#[derive(Debug)]
enum TokenType {
    Let,
    Varname,
    Equals,
    Number,
    Whitespace,
}

enum State {
    Let,
    Varname,
    Equals,
    Number,
    End,
}

fn main() {
    let mut am = Automaton::new(State::Let as i32, State::End as i32);
    am.find_regex(
        TokenType::Let as i32,
        State::Let as i32,
        State::Varname as i32,
        Regex::new(r"let").unwrap(),
    );
    am.find_whitespace(
        TokenType::Whitespace as i32,
        State::Varname as i32,
        State::Varname as i32,
    );
    am.find_regex(
        TokenType::Varname as i32,
        State::Varname as i32,
        State::Equals as i32,
        Regex::new(r"[a-z]+").unwrap(),
    );
    am.find_whitespace(
        TokenType::Whitespace as i32,
        State::Equals as i32,
        State::Equals as i32,
    );
    am.find_regex(
        TokenType::Equals as i32,
        State::Equals as i32,
        State::Number as i32,
        Regex::new(r"=").unwrap(),
    );
    am.find_whitespace(
        TokenType::Whitespace as i32,
        State::Number as i32,
        State::Number as i32,
    );
    am.find_regex(
        TokenType::Number as i32,
        State::Number as i32,
        State::End as i32,
        Regex::new(r"[0-9]+").unwrap(),
    );

    // Run the FSM with an empty string as the source code
    let source_code = format!("let answer = 42");
    let runner = am.run(source_code);

    // Print the parsed tokens to the console
    for token in runner.tokens {
        println!(
            "{:?} - '{}'",
            match token.type_id {
                x if x == TokenType::Let as i32 => "Let",
                x if x == TokenType::Varname as i32 => "Varname",
                x if x == TokenType::Equals as i32 => "Equals",
                x if x == TokenType::Number as i32 => "Number",
                x if x == TokenType::Whitespace as i32 => "Whitespace",
                _ => unimplemented!(),
            },
            token.text
        );
    }
}
