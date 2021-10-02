## statix

`statix` intends to be a static analysis tool for the
Nix programming language.

For the time-being, `statix` works only with ASTs
produced by the `rnix-parser` crate and does not evaluate
any nix code (imports, attr sets etc.). 

## Architecture

`statix` has the following components:

- `bin`: the CLI/entrypoint
- `lib`: library of lints and utilities to define these
  lints
- `macros`: procedural macros to help define a lint

### `bin`

This is the main point of interaction between `statix`
and the end user. It's output is human-readable and should
also support JSON/errorfmt outputs for external tools to
use.

### `lib`

A library of AST-based lints and utilities to help write
those lints. It should be easy for newcomers to write lints
without being familiar with the rest of the codebase.

### `macros`

This crate intends to be a helper layer to declare lints and
their metadata.
