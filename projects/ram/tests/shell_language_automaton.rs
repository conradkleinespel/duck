use ram::*;
use regex::Regex;

#[cfg(test)]
enum TokenType {
    StringSeparator,
    StringSingleQuoteContent,
    StringDoubleQuoteContent,
    StringJoined,
    Whitespace,
    End,
}

#[cfg(test)]
fn automaton_string_quoted_single() -> Automaton {
    let mut am = Automaton::new(0, 3);

    am.find_regex(
        TokenType::StringSeparator as i32,
        0,
        1,
        Regex::new("'").unwrap(),
    );
    am.find_regex(
        TokenType::StringSingleQuoteContent as i32,
        1,
        2,
        Regex::new("[^']*").unwrap(),
    );
    am.find_regex(
        TokenType::StringSeparator as i32,
        2,
        3,
        Regex::new("'").unwrap(),
    );

    am
}

#[cfg(test)]
fn automaton_string_quoted_double() -> Automaton {
    let mut am = Automaton::new(0, 3);

    am.find_regex(
        TokenType::StringSeparator as i32,
        0,
        1,
        Regex::new("\"").unwrap(),
    );
    am.find_regex(
        TokenType::StringDoubleQuoteContent as i32,
        1,
        2,
        Regex::new("(\\\\\"|[^\"])*").unwrap(),
    );
    am.find_regex(
        TokenType::StringSeparator as i32,
        2,
        3,
        Regex::new("\"").unwrap(),
    );
    am
}

#[cfg(test)]
fn automaton_string() -> Automaton {
    let mut am = Automaton::new(0, 1);

    am.find_automaton(0, 1, automaton_string_quoted_single());
    am.find_automaton(0, 1, automaton_string_quoted_double());

    am
}

#[cfg(test)]
fn automaton_shell() -> Automaton {
    let mut am = Automaton::new(0, 1);

    am.find_end(TokenType::End as i32, 0, 1);
    am.find_whitespace(TokenType::Whitespace as i32, 0, 1);
    am.find_automaton(0, 1, automaton_string());

    am
}

#[cfg(test)]
fn automaton_shell_rec() -> Automaton {
    let mut am = Automaton::new(0, 2);

    am.find_end(TokenType::End as i32, 0, 2);
    am.find_whitespace(TokenType::Whitespace as i32, 0, 1);
    am.find_automaton(0, 1, automaton_string())
        .join_tokens(TokenType::StringJoined as i32);
    am.find_me(1, 2);

    am
}

#[cfg(test)]
fn get_test_source_code() -> String {
    "'That thing'  \t  \"Hello\\\"'stuff\"".to_string()
}

#[cfg(test)]
fn test_finder(_runner: &mut Runner, _finder: &Finder) -> bool {
    false
}

#[test]
fn test_find_custom() {
    let source_code = get_test_source_code();

    let mut am = Automaton::new(0, 1);
    am.find_custom(0, 0, 1, test_finder);

    let runner = am.run(source_code);

    assert!(!runner.completed());
}

#[test]
fn test_rec() {
    let source_code = get_test_source_code();

    // recursive version
    let am = automaton_shell_rec();
    let runner = am.run(source_code);

    assert!(runner.tokens[0].text == "'That thing'".to_string());
    assert!(runner.tokens[1].text == "  \t  ".to_string());
    assert!(runner.tokens[2].text == "\"Hello\\\"'stuff\"".to_string());
    assert!(runner.tokens[3].text == "".to_string());

    assert!(runner.tokens[0].type_id == TokenType::StringJoined as i32);
    assert!(runner.tokens[1].type_id == TokenType::Whitespace as i32);
    assert!(runner.tokens[2].type_id == TokenType::StringJoined as i32);
    assert!(runner.tokens[3].type_id == TokenType::End as i32);

    assert!(runner.completed());
}

#[test]
fn test_ite() {
    let source_code = get_test_source_code();

    // iterative version
    let am = automaton_shell();
    let runner = am.run_loop(source_code);

    assert!(runner.tokens[0].text == "'".to_string());
    assert!(runner.tokens[1].text == "That thing".to_string());
    assert!(runner.tokens[2].text == "'".to_string());
    assert!(runner.tokens[3].text == "  \t  ".to_string());
    assert!(runner.tokens[4].text == "\"".to_string());
    assert!(runner.tokens[5].text == "Hello\\\"'stuff".to_string());
    assert!(runner.tokens[6].text == "\"".to_string());
    assert!(runner.tokens[7].text == "".to_string());

    assert!(runner.tokens[0].type_id == TokenType::StringSeparator as i32);
    assert!(runner.tokens[1].type_id == TokenType::StringSingleQuoteContent as i32);
    assert!(runner.tokens[2].type_id == TokenType::StringSeparator as i32);
    assert!(runner.tokens[3].type_id == TokenType::Whitespace as i32);
    assert!(runner.tokens[4].type_id == TokenType::StringSeparator as i32);
    assert!(runner.tokens[5].type_id == TokenType::StringDoubleQuoteContent as i32);
    assert!(runner.tokens[6].type_id == TokenType::StringSeparator as i32);
    assert!(runner.tokens[7].type_id == TokenType::End as i32);

    assert!(runner.completed());
}
