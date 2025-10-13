mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: manual_inherit,
    expressions: [
        "let a.b = 2; in { b = a.b; }",
        "let a.b = 2; in { c = a.c; }",
        "let a.b = 2; in { b = a.c; }",
    ],
}
