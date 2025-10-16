mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: redundant_pattern_bind,
    expressions: [
        "{ ... } @ inputs: null"
    ],
}
