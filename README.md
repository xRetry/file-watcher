# Description

A simple file watcher, which reacts to file changes by executing terminal commands.

The file regex and the corresponding terminal command are defined in a YAML config file:

```yaml
# A list of paths. 
# The whole subtree will watched.
# Defaults to the current execution directory.
paths:
    - src
    - tests

# A list of regular expressions with the corresponding terminal command.
# It is also possible to run multiple commands in sequence using `chain`.
# The shell needs to be specified manually, e.g. `bash -c` on linux and `cmd /c` on windows.
commands:
    # Examples of single commands
    - regex: ".*src/.*.rs$"
      cmd: bash -c "cargo build"

    - regex: ".*test/.*.rs$"
      cmd: bash -c "cargo test"

    # Example of sequential commands
    - regex: ".*src/.*.rs$"
      chain: 
        - bash -c "cargo build"
        - bash -c "target/debug/file-watcher config.yml"
```

# Motivation

At work I ran into problems using the SASS file watcher.
So I decided to implement my own.
The functionality will be extended as needed.
