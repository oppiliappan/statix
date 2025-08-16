use lib::session::{SessionInfo, Version};

macro_rules! session_info {
    ($version:expr) => {{
        let v: Version = $version.parse().unwrap();
        SessionInfo::from_version(v)
    }};
}

mod util {
    #[macro_export]
    macro_rules! test_lint {
        ($tname:ident => $sess:expr, $($tail:tt)*) => {
            test_lint!($tname => $sess);
            test_lint!($($tail)*);
        };
        ($tname:ident, $($tail:tt)*) => {
                test_lint!($tname);
                test_lint!($($tail)*);
        };
        ($tname:ident) => {
            test_lint!($tname => session_info!("2.6"));
        };
        ($tname:ident => $sess:expr) => {
            paste::paste! {
                #[test]
                fn [<$tname _lint>](){
                    use statix::{config::OutFormat, traits::WriteDiagnostic, lint};
                    use vfs::ReadOnlyVfs;

                    let file_path = concat!("data/", stringify!($tname), ".nix");
                    let contents = include_str!(concat!("data/", stringify!($tname), ".nix"));

                    let vfs = ReadOnlyVfs::singleton(file_path, contents.as_bytes());

                    let session = $sess;

                    let mut buffer = Vec::new();
                    vfs.iter().map(|entry| lint::lint(entry, &session)).for_each(|r| {
                        buffer.write(&r, &vfs, OutFormat::StdErr).unwrap();
                    });

                    let stripped = strip_ansi_escapes::strip(&buffer).unwrap();
                    let out =  std::str::from_utf8(&stripped).unwrap();
                    insta::assert_snapshot!(&out);
                }

                #[test]
                fn [<$tname _fix>](){
                    let file_path = concat!("tests/data/", stringify!($tname), ".nix");

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
            }

        };
    }
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
    deprecated_to_path => session_info!("2.4"),
    bool_simplification,
    useless_has_attr,
    repeated_keys,
    empty_list_concat
}
