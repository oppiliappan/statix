mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: manual_inherit,
    expressions: [
        "builtins.toPath x",
        "toPath x",
        r#"toPath "/abc/def""#,
        r#"builtins.toPath "/some/path""#,
    ],
}
