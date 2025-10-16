mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: unquoted_uri,
    expressions: [
        "github:nerdypepper/statix"
    ],
}
