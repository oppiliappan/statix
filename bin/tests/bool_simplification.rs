mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: bool_simplification,
    expressions: [
        "!(a == b)",

        // non-matches
        "!(a != b)",
        "a != b",
    ],
}
