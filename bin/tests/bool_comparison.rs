mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: bool_comparison,
    expressions: [
        // trivial
        "a == true",
        "b == true",
        "true == c",
        "true == d",
        // not equals
        "e != true",
        "f != false",
        "true != g",
        "false != h",
        // non-matches
        "i == j",
        "k != l",
        // both sides bool
        "false == false",
        "false == true",
        "true == false",
        "true == true",
        // has attr
        "false == m ? n",
        "true == o ? p",
        // don't lint if we're not 100% sure it's legit
        "let foo = {}; in (foo.bar or false) == true",
    ],
}
