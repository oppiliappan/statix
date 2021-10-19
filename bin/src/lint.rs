use crate::err::LintErr;

use lib::{LINTS, Report};
use rnix::WalkEvent;
use vfs::{VfsEntry, FileId};

#[derive(Debug)]
pub struct LintResult {
    pub file_id: FileId,
    pub reports: Vec<Report>,
}

pub fn lint(vfs_entry: VfsEntry) -> Result<LintResult, LintErr> {
    let source = vfs_entry.contents;
    let parsed = rnix::parse(source)
        .as_result()
        .map_err(|e| LintErr::Parse(vfs_entry.file_path.to_path_buf(), e))?;
    let reports = parsed
        .node()
        .preorder_with_tokens()
        .filter_map(|event| match event {
            WalkEvent::Enter(child) => LINTS.get(&child.kind()).map(|rules| {
                rules
                    .iter()
                    .filter_map(|rule| rule.validate(&child))
                    .collect::<Vec<_>>()
            }),
            _ => None,
        })
        .flatten()
        .collect();
    Ok(LintResult {
        file_id: vfs_entry.file_id,
        reports,
    })
}
