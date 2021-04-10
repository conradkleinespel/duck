# Duck, a monorepo for my open source projects

This repository contains most of the code for my open source projects. It makes code-reuse easier.

## Structure of Rust crates

In order for Rust crates to be able to easily re-use each other, they need to be structured in a specific way.

Here's an example with an imaginary crate named `my-crate`:

```test
my-crate/src
- lib.rs
- my-crate.rs
- my-crate/
  - all other files from my-crate
```

To import a crate in another crate:
- make sure you're using `edition = "2018"` in `Cargo.toml`;
- make a symlink to `my-crate` and `my-crate.rs`.

You can have a look at examples in the `rprompt` or `rpassword` crates.

## How to run tests for a Rust package

Install the Duck Toolkit:

```shell
cargo install --path projects/dt/
```

Now you can run test in Linux and Windows easily using [cross](https://github.com/rust-embedded/cross). For instance:

```shell
# Linux
dt cargo-test projects/rpassword/

# Windows
dt cargo-test projects/rpassword/ -w
```