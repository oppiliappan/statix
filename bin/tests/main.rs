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
                    let file_path = concat!("tests/data/", stringify!($tname), ".nix");
                    test_cli(concat!(stringify!($tname), "_lint"), file_path, &["check"]);
                }

                #[test]
                fn [<$tname _fix>](){
                    let file_path = concat!("tests/data/", stringify!($tname), ".nix");
                    test_cli(concat!(stringify!($tname), "_fix"), file_path, &["fix", "--dry-run"]);
                }
            }

        };
    }
}

fn test_cli(test_name: &str, file_path: &str, args: &[&str]) {
    let output = std::process::Command::new("cargo")
        .arg("run")
        .arg("--")
        .args(args)
        .arg(file_path)
        .output()
        .expect("command runs successfully");

    let stdout = strip_ansi_escapes::strip(output.stdout).unwrap();
    let stdout = String::from_utf8(stdout).unwrap();

    insta::assert_snapshot!(test_name, &stdout);
}

test_lint! {
    collapsible_let_in,
    eta_reduction,
    useless_parens,
    empty_pattern,
    redundant_pattern_bind,
    unquoted_uri,
    empty_inherit,
    deprecated_to_path,
    useless_has_attr,
    repeated_keys,
    empty_list_concat
}
