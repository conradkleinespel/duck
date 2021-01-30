extern crate ram;

use ram::Automaton;

enum TokenType {
    End,
}

fn main() {
    // Create the FSM (2 states, 0 or 1) that will parse the source code
    let mut am = Automaton::new(0, 1);
    // When the FSM hits the end of the source, go to state 1, the final state
    am.find_end(TokenType::End as i32, 0, 1);

    // Run the FSM with an empty string as the source code
    let source_code = format!("");
    let runner = am.run(source_code);

    // Print the parsed tokens to the console
    println!("{:?}", runner.tokens);
}
