use std::{collections::HashMap, sync::Arc};

use lib::{Lint, LINTS};
use rnix::SyntaxKind;

pub type LintMap = HashMap<SyntaxKind, Vec<Arc<Box<dyn Lint>>>>;

pub fn lint_map_of(lints: &[Arc<Box<dyn Lint>>]) -> LintMap {
    let mut map = HashMap::new();
    for lint in lints.iter() {
        let matches = lint.match_kind();
        for m in matches {
            map.entry(m)
                .and_modify(|v: &mut Vec<_>| v.push(lint.clone()))
                .or_insert_with(|| vec![lint.clone()]);
        }
    }
    map
}

pub fn lint_map() -> LintMap {
    lint_map_of(&LINTS)
}

pub fn get_version_info() -> Option<String> {
    use std::process::Command;
    let program = Command::new("nix").arg("--version").output().ok()?;
    std::str::from_utf8(&program.stdout)
        .ok()?
        .split(' ')
        .nth(2)
        .map(ToOwned::to_owned)
}

pub fn default_nix_version() -> String {
    String::from("2.4")
}
