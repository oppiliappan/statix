use std::borrow::Cow;

use lib::{Report, LINTS};
use rnix::{parser::ParseError as RnixParseErr, WalkEvent};

use crate::fix::{FixResult, Fixed};

fn collect_fixes(source: &str) -> Result<Vec<Report>, RnixParseErr> {
    let parsed = rnix::parse(source).as_result()?;

    Ok(parsed
        .node()
        .preorder_with_tokens()
        .filter_map(|event| match event {
            WalkEvent::Enter(child) => LINTS.get(&child.kind()).map(|rules| {
                rules
                    .iter()
                    .filter_map(|rule| rule.validate(&child))
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
        let all_reports = collect_fixes(&self.src).ok()?;
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
        })
    }
}

pub fn all(src: &str) -> Option<FixResult> {
    let src = Cow::from(src);
    let _ = rnix::parse(&src).as_result().ok()?;
    let initial = FixResult::empty(src);
    initial.into_iter().last()
}
