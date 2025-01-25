use std::{collections::HashMap, sync::Arc};

use lib::{Lint, LINTS};
use rnix::SyntaxKind;

pub type LintMap = HashMap<SyntaxKind, Vec<Arc<Box<dyn Lint>>>>;

pub fn lint_map_of(lints: &[Arc<Box<dyn Lint>>]) -> LintMap {
    lints
        .iter()
        .flat_map(|lint| lint.match_kind().into_iter().map(move |m| (m, lint)))
        .fold(HashMap::new(), |mut hm, (m, lint)| {
            hm.entry(m)
                .and_modify(|v| v.push(lint.clone()))
                .or_insert_with(|| vec![lint.clone()]);
            hm
        })
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
