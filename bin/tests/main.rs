mod util {
    #[macro_export]
    macro_rules! test_lint {
        ($($tname:ident),*,) => {
            test_lint!($($tname),*);
        };
        ($($tname:ident),*) => {
            $(
                #[test]
                fn $tname() {
                    use statix::{config::OutFormat, traits::WriteDiagnostic, lint};
                    use vfs::ReadOnlyVfs;

                    let file_path = concat!("data/", stringify!($tname), ".nix");
                    let contents = include_str!(concat!("data/", stringify!($tname), ".nix"));

                    let vfs = ReadOnlyVfs::singleton(file_path, contents.as_bytes());

                    let mut buffer = Vec::new();
                    vfs.iter().map(lint::lint).for_each(|r| {
                        buffer.write(&r, &vfs, OutFormat::StdErr).unwrap();
                    });

                    let stripped = strip_ansi_escapes::strip(&buffer).unwrap();
                    let out =  std::str::from_utf8(&stripped).unwrap();
                    insta::assert_snapshot!(&out);
                }
            )*
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
    unquoted_splices,
    empty_pattern,
    redundant_pattern_bind,
    unquoted_uri,
    deprecated_is_null,
}
