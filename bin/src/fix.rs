use std::borrow::Cow;

use lib::{Report, LINTS};
use rnix::{parser::ParseError as RnixParseErr, TextRange, WalkEvent};

type Source<'a> = Cow<'a, str>;

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

#[derive(Debug)]
pub struct FixResult<'a> {
    pub src: Source<'a>,
    pub fixed: Vec<Fixed>,
}

#[derive(Debug, Clone)]
pub struct Fixed {
    pub at: TextRange,
    pub code: u32,
}

impl<'a> FixResult<'a> {
    fn empty(src: Source<'a>) -> Self {
        Self { src, fixed: vec![] }
    }
}

fn next(mut src: Source) -> Result<FixResult, RnixParseErr> {
    let all_reports = collect_fixes(&src)?;

    if all_reports.is_empty() {
        return Ok(FixResult::empty(src));
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
        report.apply(src.to_mut());
    }

    Ok(FixResult {
        src,
        fixed
    })
}

pub fn fix(src: &str) -> Result<FixResult, RnixParseErr> {
    let src = Cow::from(src);
    let _ = rnix::parse(&src).as_result()?;
    let mut initial = FixResult::empty(src);

    while let Ok(next_result) = next(initial.src)  {
        if next_result.fixed.is_empty() {
            return Ok(next_result);
        } else {
            initial = FixResult::empty(next_result.src);
        }
    }

    unreachable!("a fix caused a syntax error, please report a bug");
}
