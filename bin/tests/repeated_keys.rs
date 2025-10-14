mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: useless_has_attr,
    expressions: [
        // fine
        "{ foo.bar = 1; }",

        // do not raise on rec
        indoc! {"
            rec {
              foo.x = foo.y;
              foo.y = 2;
              foo.z = 3;
            }
        "},

        // exactly 3 occurrences
        indoc! {r#"
            {
              foo.bar = 1;
              foo.bar."hello" = 1;
              foo.again = 1;
            }
        "#},

        // more than 3, omit the extra
        indoc! {"
            {
              foo.baz.bar1 = 1;
              foo.baz.bar2 = 2;
              foo.baz.bar3 = 3;
              foo.baz.bar4 = 4;
              foo.baz.bar5 = 5;
            }
        "},
    ],
}
