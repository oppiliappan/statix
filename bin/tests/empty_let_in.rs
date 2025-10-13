mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: empty_let_in,
    expressions: [
        "let in null",
        indoc! {"
            let
              # don't fix this, we have a comment
              # raise the lint though
            in
            null
        "}
    ],
}
