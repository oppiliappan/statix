mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: manual_inherit,
    expressions: [
        "{ ... } @ inputs: null"
    ],
}
