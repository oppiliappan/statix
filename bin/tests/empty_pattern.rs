mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: manual_inherit,
    expressions: [
        // match
        "({ ... }: 42)",
        "({ ... } @ inputs: inputs)",

        // don't match
        "({ a, ... }: a)",

        // nixos module, don't match
        "({ ... }: { imports = []; })",
    ],
}
