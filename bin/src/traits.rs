use std::{
    io::{self, Write},
    str,
};

use crate::{config::OutFormat, lint::LintResult};

use ariadne::{
    CharSet, Color, Config as CliConfig, Fmt, Label, LabelAttach, Report as CliReport,
    ReportKind as CliReportKind, Source,
};
use rnix::{TextRange, TextSize};
use vfs::ReadOnlyVfs;

pub trait WriteDiagnostic {
    fn write(
        &mut self,
        report: &LintResult,
        vfs: &ReadOnlyVfs,
        format: OutFormat,
    ) -> io::Result<()>;
}

impl<T> WriteDiagnostic for T
where
    T: Write,
{
    fn write(
        &mut self,
        lint_result: &LintResult,
        vfs: &ReadOnlyVfs,
        format: OutFormat,
    ) -> io::Result<()> {
        match format {
            #[cfg(feature = "json")]
            OutFormat::Json => json::write_json(self, lint_result, vfs),
            OutFormat::StdErr => write_stderr(self, lint_result, vfs),
            OutFormat::Errfmt => write_errfmt(self, lint_result, vfs),
        }
    }
}

fn write_stderr<T: Write>(
    writer: &mut T,
    lint_result: &LintResult,
    vfs: &ReadOnlyVfs,
) -> io::Result<()> {
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
            .write((src_id, Source::from(src)), &mut *writer)?;
    }
    Ok(())
}

fn write_errfmt<T: Write>(
    writer: &mut T,
    lint_result: &LintResult,
    vfs: &ReadOnlyVfs,
) -> io::Result<()> {
    let file_id = lint_result.file_id;
    let src = str::from_utf8(vfs.get(file_id)).unwrap();
    let path = vfs.file_path(file_id);
    for report in lint_result.reports.iter() {
        for diagnostic in report.diagnostics.iter() {
            let line = line(diagnostic.at.start(), &src);
            let col = column(diagnostic.at.start(), &src);
            writeln!(
                writer,
                "{filename}>{linenumber}:{columnnumber}:{errortype}:{errornumber}:{errormessage}",
                filename = path.to_str().unwrap_or("<unknown>"),
                linenumber = line,
                columnnumber = col,
                errortype = "W",
                errornumber = report.code,
                errormessage = diagnostic.message
            )?;
        }
    }
    Ok(())
}

#[cfg(feature = "json")]
mod json {
    use crate::lint::LintResult;

    use std::io::{self, Write};

    use rnix::TextRange;
    use serde::Serialize;
    use serde_json;
    use vfs::ReadOnlyVfs;

    #[derive(Serialize)]
    struct Out<'μ> {
        #[serde(rename = "file")]
        path: &'μ std::path::Path,
        report: Vec<JsonReport<'μ>>,
    }

    #[derive(Serialize)]
    struct JsonReport<'μ> {
        note: &'static str,
        code: u32,
        diagnostics: Vec<JsonDiagnostic<'μ>>,
    }

    #[derive(Serialize)]
    struct JsonDiagnostic<'μ> {
        at: JsonSpan,
        message: &'μ String,
        suggestion: Option<JsonSuggestion>,
    }

    #[derive(Serialize)]
    struct JsonSuggestion {
        at: JsonSpan,
        fix: String,
    }

    #[derive(Serialize)]
    struct JsonSpan {
        from: Position,
        to: Position,
    }

    #[derive(Serialize)]
    struct Position {
        line: usize,
        column: usize,
    }

    impl JsonSpan {
        fn from_textrange(at: TextRange, src: &str) -> Self {
            let start = at.start();
            let end = at.end();
            let from = Position {
                line: super::line(start, src),
                column: super::column(start, src),
            };
            let to = Position {
                line: super::line(end, src),
                column: super::column(end, src),
            };
            Self { from, to }
        }
    }

    pub fn write_json<T: Write>(
        writer: &mut T,
        lint_result: &LintResult,
        vfs: &ReadOnlyVfs,
    ) -> io::Result<()> {
        let file_id = lint_result.file_id;
        let path = vfs.file_path(file_id);
        let src = vfs.get_str(file_id);
        let report = lint_result
            .reports
            .iter()
            .map(|r| {
                let note = r.note;
                let code = r.code;
                let diagnostics = r
                    .diagnostics
                    .iter()
                    .map(|d| JsonDiagnostic {
                        at: JsonSpan::from_textrange(d.at, src),
                        message: &d.message,
                        suggestion: d.suggestion.as_ref().map(|s| JsonSuggestion {
                            at: JsonSpan::from_textrange(s.at, src),
                            fix: s.fix.to_string(),
                        }),
                    })
                    .collect::<Vec<_>>();
                JsonReport {
                    note,
                    code,
                    diagnostics,
                }
            })
            .collect();
        writeln!(
            writer,
            "{}",
            serde_json::to_string_pretty(&Out { path, report }).unwrap()
        )?;
        Ok(())
    }
}

fn line(at: TextSize, src: &str) -> usize {
    let at = at.into();
    src[..at].chars().filter(|&c| c == '\n').count() + 1
}

fn column(at: TextSize, src: &str) -> usize {
    let at = at.into();
    src[..at].rfind('\n').map(|c| at - c).unwrap_or(at + 1)
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
