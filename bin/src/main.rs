use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use ariadne::{Color, Fmt, Label, Report as CliReport, ReportKind as CliReportKind, Source};
use lib::{Report, LINTS};
use rnix::WalkEvent;

fn analyze(file: &str) -> Result<Vec<Report>> {
    let parsed = rnix::parse(file).as_result()?;

    Ok(parsed
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
        .collect())
}

fn print_report(report: Report, file_src: &str, file_path: &Path) -> Result<()> {
    let src_id = file_path.to_str().unwrap_or("<unknown>");
    let offset = report
        .diagnostics
        .iter()
        .map(|d| d.at.start().into())
        .min()
        .unwrap_or(0usize);
    report
        .diagnostics
        .iter()
        .fold(
            CliReport::build(CliReportKind::Warning, src_id, offset),
            |cli_report, diagnostic| {
                let range = {
                    let at = diagnostic.at;
                    at.start().into()..at.end().into()
                };
                cli_report.with_label(
                    Label::new((src_id, range))
                        .with_message(diagnostic.message.as_str().fg(Color::Yellow)),
                )
            },
        )
        .finish()
        .print((src_id, Source::from(file_src)))
        .context("failed to print report to stdout")
}

fn _main() -> Result<()> {
    let args = env::args();
    for (file_src, file_path, reports) in args
        .map(|s| PathBuf::from(&s))
        .filter(|p| p.is_file())
        .filter_map(|path| {
            let s = fs::read_to_string(&path).ok()?;
            analyze(&s)
                .map(|analysis_result| (s, path, analysis_result))
                .ok()
        })
    {
        for r in reports {
            print_report(r, &file_src, &file_path)?
        }
    }
    Ok(())
}
fn main() {
    match _main() {
        Err(e) => eprintln!("{}", e),
        _ => {}
    }
}
