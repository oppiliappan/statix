mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: useless_has_attr,
    expressions: [
        // trivial
        "if x ? a then x.a else default",
        "if x.a ? b then x.a.b else default",
        "if x ? a.b then x.a.b else default",

        // complex body
        "if x ? a then x.a else if b then c else d",
        "if x ? a then x.a else b.c",
    ],
}
