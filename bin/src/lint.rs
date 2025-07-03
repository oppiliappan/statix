use crate::{LintMap, utils};

use lib::{Report, session::SessionInfo};
use rnix::WalkEvent;
use vfs::{FileId, VfsEntry};

#[derive(Debug)]
pub struct LintResult {
    pub file_id: FileId,
    pub reports: Vec<Report>,
}

pub fn lint_with(vfs_entry: VfsEntry, lints: &LintMap, sess: &SessionInfo) -> LintResult {
    let file_id = vfs_entry.file_id;
    let source = vfs_entry.contents;
    let (parsed, errors) = lib::parse::ParseResult::parse(source).to_tuple();

    let error_reports = errors.into_iter().map(Report::from_parse_err);
    let reports = parsed
        .preorder_with_tokens()
        .filter_map(|event| match event {
            WalkEvent::Enter(child) => lints.get(&child.kind()).map(|rules| {
                rules
                    .iter()
                    .filter_map(|rule| rule.validate(&child, sess))
                    .collect::<Vec<_>>()
            }),
            _ => None,
        })
        .flatten()
        .chain(error_reports)
        .collect();

    LintResult { file_id, reports }
}

pub fn lint(vfs_entry: VfsEntry, sess: &SessionInfo) -> LintResult {
    lint_with(vfs_entry, &utils::lint_map(), sess)
}

pub mod main {
    use std::io;

    use super::lint_with;
    use crate::{
        config::{Check as CheckConfig, ConfFile},
        err::StatixErr,
        traits::WriteDiagnostic,
    };

    use lib::session::SessionInfo;
    use rayon::prelude::*;

    pub fn main(check_config: CheckConfig) -> Result<(), StatixErr> {
        let conf_file = ConfFile::discover(&check_config.conf_path)?;
        let lints = conf_file.lints();
        let version = conf_file.version()?;
        let session = SessionInfo::from_version(version);

        let vfs = check_config.vfs(conf_file.ignore.as_slice())?;

        let mut stdout = io::stdout();
        let lint = |vfs_entry| lint_with(vfs_entry, &lints, &session);
        let results = vfs
            .par_iter()
            .map(lint)
            .filter(|lr| !lr.reports.is_empty())
            .collect::<Vec<_>>();

        if !results.is_empty() {
            for r in &results {
                stdout.write(r, &vfs, check_config.format).unwrap();
            }
            std::process::exit(1);
        }

        std::process::exit(0);
    }
}
