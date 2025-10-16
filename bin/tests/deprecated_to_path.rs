mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: deprecated_to_path,
    expressions: [
        "builtins.toPath x",
        "toPath x",
        r#"toPath "/abc/def""#,
        r#"builtins.toPath "/some/path""#,
    ],
}
