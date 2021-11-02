use lib::{Report, LINTS};
use rnix::WalkEvent;
use vfs::{FileId, VfsEntry};

#[derive(Debug)]
pub struct LintResult {
    pub file_id: FileId,
    pub reports: Vec<Report>,
}

pub fn lint(vfs_entry: VfsEntry) -> LintResult {
    let file_id = vfs_entry.file_id;
    let source = vfs_entry.contents;
    let parsed = rnix::parse(source);

    let error_reports = parsed
        .errors()
        .into_iter()
        .map(Report::from_parse_err);

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
        .chain(error_reports)
        .collect();

    LintResult { file_id, reports }
}
