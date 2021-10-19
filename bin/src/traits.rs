use std::{
    io::{self, Write},
    str,
};

use crate::lint::LintResult;

use ariadne::{
    CharSet, Color, Config as CliConfig, Fmt, Label, LabelAttach, Report as CliReport,
    ReportKind as CliReportKind, Source,
};
use rnix::TextRange;
use vfs::ReadOnlyVfs;

pub trait WriteDiagnostic {
    fn write(&mut self, report: &LintResult, vfs: &ReadOnlyVfs) -> io::Result<()>;
}

impl<T> WriteDiagnostic for T
where
    T: Write,
{
    fn write(&mut self, lint_result: &LintResult, vfs: &ReadOnlyVfs) -> io::Result<()> {
        let file_id = lint_result.file_id;
        let src = str::from_utf8(vfs.get(file_id)).unwrap();
        let path = vfs.file_path(file_id);
        let range = |at: TextRange| at.start().into()..at.end().into();
        let src_id = path.to_str().unwrap_or("<unknown>");
        for report in lint_result.reports.iter() {
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
                    CliReport::build(CliReportKind::Warning, src_id, offset)
                        .with_config(
                            CliConfig::default()
                                .with_cross_gap(true)
                                .with_multiline_arrows(false)
                                .with_label_attach(LabelAttach::Middle)
                                .with_char_set(CharSet::Unicode),
                        )
                        .with_message(report.note)
                        .with_code(report.code),
                    |cli_report, diagnostic| {
                        cli_report.with_label(
                            Label::new((src_id, range(diagnostic.at)))
                                .with_message(&colorize(&diagnostic.message))
                                .with_color(Color::Magenta),
                        )
                    },
                )
                .finish()
                .write((src_id, Source::from(src)), &mut *self)?;
        }
        Ok(())
    }
}

// everything within backticks is colorized, backticks are removed
fn colorize(message: &str) -> String {
    message
        .split('`')
        .enumerate()
        .map(|(idx, part)| {
            if idx % 2 == 1 {
                part.fg(Color::Cyan).to_string()
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("")
}
