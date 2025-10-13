mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: manual_inherit,
    expressions: [
        // parens around primitives
        r#"("hello")"#,
        "let b = 0; in (b)",
        "({ f = 2; })",

        // parens around let-value
        "let a = (1 + 2); in null",
        "let h = ({ inherit (builtins) map; }); in null",

        // LATER: binary exprs, function args etc.

        // parens around let body
        "let a = 0; in (null)",

        // select in list (parens not necessary)
        "[(builtins.map)]",
        "[(builtins.pam or map)]",
    ],
}
