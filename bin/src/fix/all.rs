use std::borrow::Cow;

use lib::{session::SessionInfo, Report};
use rnix::{parser::ParseError as RnixParseErr, tokenize, SyntaxNode, WalkEvent};

use crate::{
    fix::{FixResult, Fixed},
    LintMap,
};

fn collect_fixes(
    source: &str,
    lints: &LintMap,
    sess: &SessionInfo,
) -> Result<Vec<Report>, RnixParseErr> {
    let (green, mut errors) = rnix::parser::parse(tokenize(source).into_iter());

    // reverse here to start with first error when popping
    errors.reverse();
    if let Some(err) = errors.pop() {
        return Err(err);
    }

    Ok(SyntaxNode::new_root(green)
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
    use std::collections::VecDeque;

    reports.sort_by(|a, b| {
        let a_range = a.range();
        let b_range = b.range();
        a_range.end().partial_cmp(&b_range.end()).unwrap()
    });

    reports
        .into_iter()
        .fold(VecDeque::new(), |mut deque: VecDeque<Report>, new_elem| {
            let front = deque.front();
            let new_range = new_elem.range();
            if let Some(front_range) = front.map(|f| f.range()) {
                if new_range.start() > front_range.end() {
                    deque.push_front(new_elem);
                }
            } else {
                deque.push_front(new_elem);
            }
            deque
        })
        .into()
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
    let (_, errors) = rnix::parser::parse(tokenize(&src).into_iter());

    if !errors.is_empty() {
        return None;
    }

    let initial = FixResult::empty(src, lints, sess);
    initial.into_iter().last()
}
