use std::{borrow::Cow, convert::TryFrom};

use lib::{Report, LINTS};
use rnix::{TextRange, TextSize};

use crate::err::SingleFixErr;
use crate::fix::Source;

pub struct SingleFixResult<'δ> {
    pub src: Source<'δ>,
}

fn pos_to_byte(line: usize, col: usize, src: &str) -> Result<TextSize, SingleFixErr> {
    let mut byte: TextSize = TextSize::of("");
    for (_, l) in src.lines().enumerate().take_while(|(i, _)| i <= &line) {
        byte += TextSize::of(l);
    }
    byte += TextSize::try_from(col).map_err(|_| SingleFixErr::Conversion(col))?;

    if usize::from(byte) >= src.len() {
        Err(SingleFixErr::OutOfBounds(line, col))
    } else {
        Ok(byte)
    }
}

fn find(offset: TextSize, src: &str) -> Result<Report, SingleFixErr> {
    // we don't really need the source to form a completely parsed tree
    let parsed = rnix::parse(src);

    let elem_at = parsed
        .node()
        .child_or_token_at_range(TextRange::empty(offset))
        .ok_or(SingleFixErr::NoOp)?;

    LINTS
        .get(&elem_at.kind())
        .map(|rules| {
            rules
                .iter()
                .filter_map(|rule| rule.validate(&elem_at))
                .filter(|report| report.total_suggestion_range().is_some())
                .next()
        })
        .flatten()
        .ok_or(SingleFixErr::NoOp)
}

pub fn single(line: usize, col: usize, src: &str) -> Result<SingleFixResult, SingleFixErr> {
    let mut src = Cow::from(src);
    let offset = pos_to_byte(line, col, &*src)?;
    let report = find(offset, &*src)?;

    report.apply(src.to_mut());

    Ok(SingleFixResult {
        src
    })
}
