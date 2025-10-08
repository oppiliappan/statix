use std::path::Path;

mod util {
    #[macro_export]
    macro_rules! test_lint {
        ($tname:ident, $($tail:tt)*) => {
            test_lint!($tname);
            test_lint!($($tail)*);
        };
        ($tname:ident, $($tail:tt)*) => {
                test_lint!($tname);
                test_lint!($($tail)*);
        };
        ($tname:ident) => {
            paste::paste! {
                #[test]
                fn [<$tname _lint>](){
                    let file_path = concat!("data/", stringify!($tname), ".nix");
                    let contents = include_str!(concat!("data/", stringify!($tname), ".nix"));
                    test_lint(file_path, contents);
                }

                #[test]
                fn [<$tname _fix>](){
                    let file_path = concat!("tests/data/", stringify!($tname), ".nix");
                    test_fix(file_path);
                }
            }

        };
    }
}

fn test_lint(file_path: impl AsRef<Path>, contents: &str) {
    use statix::{config::OutFormat, lint, traits::WriteDiagnostic};
    use vfs::ReadOnlyVfs;

    let vfs = ReadOnlyVfs::singleton(file_path, contents.as_bytes());

    let mut buffer = Vec::new();
    vfs.iter().map(|entry| lint::lint(&entry)).for_each(|r| {
        buffer.write(&r, &vfs, OutFormat::StdErr).unwrap();
    });

    let stripped = strip_ansi_escapes::strip(&buffer).unwrap();
    let out = std::str::from_utf8(&stripped).unwrap();
    insta::assert_snapshot!(&out);
}
fn test_fix(file_path: &str) {
    let output = std::process::Command::new("cargo")
        .arg("run")
        .arg("fix")
        .arg("-d")
        .arg(file_path)
        .output()
        .expect("command runs successfully");

    let stdout = String::from_utf8(output.stdout).expect("output is valid utf8");

    insta::assert_snapshot!(&stdout);
}

test_lint! {
    bool_comparison,
    empty_let_in,
    manual_inherit,
    manual_inherit_from,
    legacy_let_syntax,
    collapsible_let_in,
    eta_reduction,
    useless_parens,
    empty_pattern,
    redundant_pattern_bind,
    unquoted_uri,
    empty_inherit,
    deprecated_to_path,
    bool_simplification,
    useless_has_attr,
    repeated_keys,
    empty_list_concat
}
