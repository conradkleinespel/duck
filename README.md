# Duck, a monorepo for my open source projects

This repository contains most of the code for my open source projects.

## Replay commit history to mirrors

To replay Duck's commit history onto the project's own Git repository, I have built a utility that I
use in the following way:

```shell
cargo install --path projects/dt --debug
dt repo-history -vvv rpassword
```
