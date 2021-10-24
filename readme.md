# statix

> Lints and suggestions for the Nix programming language.

`statix` highlights antipatterns in Nix code. `statix fix`
can fix several such occurrences.

For the time-being, `statix` works only with ASTs
produced by the `rnix-parser` crate and does not evaluate
any nix code (imports, attr sets etc.). 

## Installation

`statix` is available via a nix flake:

```
nix run git+https://git.peppe.rs/languages/statix

# or

nix build git+https://git.peppe.rs/languages/statix
./result/bin/statix --help
```

## Usage

```
statix 0.1.0

Akshay <nerdy@peppe.rs>

Lints and suggestions for the Nix programming language

USAGE:
    statix [FLAGS] [OPTIONS] [--] [TARGET]

ARGS:
    <TARGET>    File or directory to run statix on [default: .]

FLAGS:
    -d, --dry-run    Do not fix files in place, display a diff instead
    -f, --fix        Find and fix issues raised by statix
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
    -i, --ignore <IGNORE>...    Globs of file patterns to skip
    -o, --format <FORMAT>       Output format. Supported values: errfmt, json (on feature flag only)
```

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

## TODO

- Offline documentation for each lint
- Test suite for lints and suggestions
- Output singleline/errfmt + vim plugin
