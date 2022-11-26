# Duck, a monorepo for my open source projects

This repository contains most of the code for my open source projects. It makes code-reuse easier.

## What the Duck Toolkit is and how to install it

The Duck Toolkit (aka `dt`) is a command line tool that makes working with Duck easier.

Currently, it allows to:

- run cross platform tests with `cross` for Rust crates that have [local dependencies](https://github.com/rust-embedded/cross/issues/388);
- replay Git commit history from Duck, a monorepo, to Duck's projects own repositories.

To install `dt`, run the following commands:

```shell
cargo install --path projects/dt --debug
```

## How to replay Duck's commit history to a project's own repository

Now you can replay Duck's commit history onto the project's own Git repository. Here's an example for `rpassword`:

```shell
dt repo-history -vvv rpassword
```
