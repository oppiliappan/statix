mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: manual_inherit,
    expressions: [
        // trivial
        "let a = 2; in { a = a; }",
        // don't lint
        "let y = 2; in { x.y = y; }",
    ],
}
