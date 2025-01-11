use std::borrow::Cow;

use lib::{session::SessionInfo, Report};
use rnix::{parser::ParseError as RnixParseErr, WalkEvent};

use crate::{
    fix::{FixResult, Fixed},
    utils::LintMap,
};

fn collect_fixes(
    source: &str,
    lints: &LintMap,
    sess: &SessionInfo,
) -> Result<Vec<Report>, Vec<RnixParseErr>> {
    let parsed = lib::parse::ParseResult::parse(source).to_result()?;

    Ok(parsed
        .preorder_with_tokens()
        .filter_map(|event| match event {
            WalkEvent::Enter(child) => lints.get(&child.kind()).map(|rules| {
                rules
                    .iter()
                    .filter_map(|rule| rule.validate(&child, sess))
                    .filter(|report| report.total_suggestion_range().is_some())
                    .collect::<Vec<_>>()
            }),
            _ => None,
        })
        .flatten()
        .collect())
}

fn reorder(mut reports: Vec<Report>) -> Vec<Report> {
    reports.sort_by(|a, b| {
        let a_range = a.range();
        let b_range = b.range();
        // order:
        // - earlier starts come first
        // - in case of same start, shorter Reports come first
        a_range
            .start()
            .cmp(&b_range.start())
            .then_with(|| a_range.end().cmp(&b_range.end()))
    });
    reports
}

impl<'a> Iterator for FixResult<'a> {
    type Item = FixResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let all_reports = collect_fixes(&self.src, self.lints, self.sess).ok()?;
        if all_reports.is_empty() {
            return None;
        }

        let reordered = reorder(all_reports);
        let fixed = reordered
            .iter()
            .map(|r| Fixed {
                at: r.range(),
                code: r.code,
            })
            .collect::<Vec<_>>();
        for report in reordered {
            report.apply(self.src.to_mut());
        }

        Some(FixResult {
            src: self.src.clone(),
            fixed,
            lints: self.lints,
            sess: self.sess,
        })
    }
}

pub fn all_with<'a>(
    src: &'a str,
    lints: &'a LintMap,
    sess: &'a SessionInfo,
) -> Option<FixResult<'a>> {
    let src = Cow::from(src);
    lib::parse::ParseResult::parse(&src).to_result().ok()?;
    let initial = FixResult::empty(src, lints, sess);
    initial.into_iter().last()
}
