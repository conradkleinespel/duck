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

## What the Duck Toolkit is and how to install it

The Duck Toolkit (aka `dt`) is a command line tool that makes working with Duck easier.

Currently, it allows to:

- run cross platform tests with `cross` for Rust crates that have [local dependencies](https://github.com/rust-embedded/cross/issues/388);
- replay Git commit history from Duck, a monorepo, to Duck's projects own repositories.

To install `dt`, run the following commands:

```shell
cargo install --path projects/dt --debug
```

## How to run tests with `cross`

Now you can run test in Linux and Windows easily using [cross](https://github.com/rust-embedded/cross). For instance:

```shell
# Linux
dt cargo-test projects/rpassword/

# Windows
dt cargo-test projects/rpassword/ -w
```

## How to replay Duck's commit history to a project's own repository

Now you can replay Duck's commit history onto the project's own Git repository. Here's an example for `rpassword`:

```shell
dt repo-history https://github.com/conradkleinespel/duck.git rpassword https://github.com/conradkleinespel/rpassword.git -v
```
