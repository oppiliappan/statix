mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: manual_inherit_from,
    expressions: [
        "let a.b = 2; in { b = a.b; }",
        "let a.b = 2; in { c = a.c; }",
        "let a.b = 2; in { b = a.c; }",
        // don't lint if the rhs expr is more than just an access
        "let foo = { }; in { x = foo.x or \"\"; }",
    ],
}
