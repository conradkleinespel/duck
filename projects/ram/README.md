# Rustastic Automaton

![CI](https://github.com/conradkleinespel/ram/workflows/CI/badge.svg)
[![Build status](https://ci.appveyor.com/api/projects/status/2b6njxu5s57itbdk?svg=true)](https://ci.appveyor.com/project/conradkleinespel/ram)

`ram` allows you to ease the creation of a language lexer based on finite state machines.

`ram` is made available free of charge. You can support its development through [Liberapay](https://liberapay.com/conradkleinespel/) 💪

## Usage

Add `ram` as a dependency in Cargo.toml:

```toml
[dependencies]
ram = "7.0"
```

Import the `ram` crate and use the `Automaton` struct to create a language lexer. In this example, we are lexing a language that has no tokens other than the "End of source" token:

```rust
extern crate ram;

use ram::Automaton;

enum TokenType {
    End,
}

// Create the FSM (2 states, 0 or 1) that will parse the source code
let mut am = Automaton::new(0, 1);
// When the FSM hits the end of the source, go to state 1, the final state
am.find_end(TokenType::End as i32, 0, 1);

// Run the FSM with an empty string as the source code
let source_code = format!("");
let runner = am.run(source_code);

// Print the parsed tokens to the console
println!("{:?}", runner.tokens);
```

There are multiple `find_*` methods to your disposal, like `find_regex` or `find_whitespace` or
even `find_automaton`, which allows you to combine various finite state machines together
to create more powerful tokenizers.

The full API documentation is available at [https://docs.rs/ram](https://docs.rs/ram).

## License

The source code is released under the Apache 2.0 license.
