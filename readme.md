# statix

> Lints and suggestions for the Nix programming language.

`statix check` highlights antipatterns in Nix code. `statix
fix` can fix several such occurrences.

For the time-being, `statix` works only with ASTs
produced by the `rnix-parser` crate and does not evaluate
any nix code (imports, attr sets etc.). 

## Examples

```shell
$ statix check tests/c.nix
[W04] Warning: Assignment instead of inherit from
   ╭─[tests/c.nix:2:3]
   │
 2 │   mtl = pkgs.haskellPackages.mtl;
   ·   ───────────────┬───────────────
   ·                  ╰───────────────── This assignment is better written with inherit
───╯

$ statix fix --dry-run tests/c.nix
--- tests/c.nix
+++ tests/c.nix [fixed]
@@ -1,6 +1,6 @@
 let
-  mtl = pkgs.haskellPackages.mtl;
+  inherit (pkgs.haskellPackages) mtl;
 in
 null
```

## Installation

`statix` is available via a nix flake:

```shell
# build from source
nix build git+https://git.peppe.rs/languages/statix
./result/bin/statix --help

# statix also provides a flake app
nix run git+https://git.peppe.rs/languages/statix -- --help

# save time on builds using cachix
cachix use statix
```

## Usage

Basic usage is as simple as:

```shell
# recursively finds nix files and raises lints
statix check /path/to/dir

# ignore generated files, such as Cargo.nix
statix check /path/to/dir -i Cargo.nix

# ignore more than one file
statix check /path/to/dir -i a.nix b.nix c.nix

# ignore an entire directory
statix check /path/to/dir -i .direnv

# statix respects your .gitignore if it exists
# run statix in "unrestricted" mode, to disable that
statix check /path/to/dir -u

# see `statix -h` for a full list of options
```

Certain lints have suggestions. Apply suggestions back to
the source with:

```shell
statix fix /path/to/file

# show diff, do not write to file
statix fix --dry-run /path/to/file
```

`statix` supports a variety of output formats; standard,
json and errfmt:

```shell
statix check /path/to/dir -o json   # only when compiled with --all-features
statix check /path/to/dir -o errfmt # singleline, easy to integrate with vim
```

### Configuration

Ignore lints and fixes by creating a `statix.toml` file at
your project root:

```
# within statix.toml
disabled = [
  "empty_pattern"
]
```

`statix` automatically discovers the configuration file by
traversing parents of the current directory and looking for
a `statix.toml` file. Alternatively, you can pass the path
to the `statix.toml` file on the command line with the
`--config` flag (available on `statix check` and `statix
fix`).

The available lints are:

```
bool_comparison
empty_let_in
manual_inherit
manual_inherit_from
legacy_let_syntax
collapsible_let_in
eta_reduction
useless_parens
empty_pattern
redundant_pattern_bind
unquoted_uri
deprecated_is_null
```

All lints are enabled by default.

## Architecture

`statix` has the following components:

- `bin`: the CLI/entrypoint
- `lib`: library of lints and utilities to define these
  lints
- `vfs`: virtual filesystem
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

### `vfs`

VFS is an in-memory filesystem. It provides cheap-to-copy
handles (`FileId`s) to access paths and file contents.

### `macros`

This crate intends to be a helper layer to declare lints and
their metadata.

## TODO

- Test suite for lints and suggestions
- Resolve imports and scopes for better lints
- Add silent flag that exits with status
