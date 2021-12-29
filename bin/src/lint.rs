use crate::{utils, LintMap};

use lib::{session::SessionInfo, Report};
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
    let parsed = rnix::parse(source);

    let error_reports = parsed.errors().into_iter().map(Report::from_parse_err);
    let reports = parsed
        .node()
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
    lint_with(vfs_entry, &utils::lint_map(), &sess)
}

pub mod main {
    use std::io;

    use super::lint_with;
    use crate::{config::Check as CheckConfig, err::StatixErr, traits::WriteDiagnostic, utils};

    use lib::session::{SessionInfo, Version};

    pub fn main(check_config: CheckConfig) -> Result<(), StatixErr> {
        let vfs = check_config.vfs()?;
        let mut stdout = io::stdout();
        let lints = check_config.lints()?;

        let version = utils::get_version_info()
            .unwrap()
            .parse::<Version>()
            .unwrap();
        let session = SessionInfo::from_version(version);

        let lint = |vfs_entry| lint_with(vfs_entry, &lints, &session);
        vfs.iter().map(lint).for_each(|r| {
            stdout.write(&r, &vfs, check_config.format).unwrap();
        });
        Ok(())
    }
}
