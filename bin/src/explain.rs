use crate::err::ExplainErr;

use lib::LINTS;

pub fn explain(code: u32) -> Result<&'static str, ExplainErr> {
    match code {
        0 => Ok("syntax error"),
        _ => LINTS
            .values()
            .flatten()
            .find(|l| l.code() == code)
            .map(|l| l.explanation())
            .ok_or(ExplainErr::LintNotFound(code)),
    }
}
