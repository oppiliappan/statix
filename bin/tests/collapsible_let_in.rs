mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: collapsible_let_in,
    expressions: [
        indoc! {r"
            let
              a = 2;
              b = 3;
            in
              let
                c = 5;
                d = 6;
              in
              a + b + c + d
        "}
    ],
}
