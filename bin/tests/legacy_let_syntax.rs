mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: manual_inherit,
    expressions: [
        indoc! {r#"
            let {
              body = x + y;
              x = "hello,";
              y = " world!";
            }
        "#}
    ],
}
