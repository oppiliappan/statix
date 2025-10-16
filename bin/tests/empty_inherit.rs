mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: empty_inherit,
    expressions: [
        "{ inherit; }"
    ],
}
