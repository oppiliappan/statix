mod lints;
mod make;

pub use lints::LINTS;

use rnix::{SyntaxElement, SyntaxKind, TextRange};
use std::{default::Default, convert::Into};

pub trait Rule {
    fn validate(&self, node: &SyntaxElement) -> Option<Report>;
}

#[derive(Debug)]
pub struct Diagnostic {
    pub at: TextRange,
    pub message: String,
    pub suggestion: Option<Suggestion>,
}

impl Diagnostic {
    pub fn new(at: TextRange, message: String) -> Self {
        Self { at, message, suggestion: None }
    }
    pub fn suggest(at: TextRange, message: String, suggestion: Suggestion) -> Self {
        Self { at, message, suggestion: Some(suggestion) }
    }
}

#[derive(Debug)]
pub struct Suggestion {
    pub at: TextRange,
    pub fix: SyntaxElement,
}

impl Suggestion {
    pub fn new<E: Into<SyntaxElement>>(at: TextRange, fix: E) -> Self {
        Self {
            at,
            fix: fix.into()
        }
    }
}

#[derive(Debug, Default)]
pub struct Report {
    pub diagnostics: Vec<Diagnostic>,
    pub note: &'static str
}

impl Report {
    pub fn new(note: &'static str) -> Self {
        Self {
            note,
            ..Default::default()
        }
    }
    pub fn diagnostic(mut self, at: TextRange, message: String) -> Self {
        self.diagnostics.push(Diagnostic::new(at, message));
        self
    }
    pub fn suggest(mut self, at: TextRange, message: String, suggestion: Suggestion) -> Self {
        self.diagnostics.push(Diagnostic::suggest(at, message, suggestion));
        self
    }

}

pub trait Metadata {
    fn name() -> &'static str where Self: Sized;
    fn note() -> &'static str where Self: Sized;
    fn match_with(&self, with: &SyntaxKind) -> bool;
    fn match_kind(&self) -> SyntaxKind;
}

pub trait Lint: Metadata + Rule + Send + Sync {}

#[macro_export]
macro_rules! lint_map {
    ($($s:ident),*,) => {
        lint_map!($($s),*);
    };
    ($($s:ident),*) => {
        use ::std::collections::HashMap;
        use ::rnix::SyntaxKind;
        $(
            mod $s;
        )*
        ::lazy_static::lazy_static! {
            pub static ref LINTS: HashMap<SyntaxKind, Vec<&'static Box<dyn $crate::Lint>>> = {
                let mut map = HashMap::new();
                $(
                    {
                        let temp_lint = &*$s::LINT;
                        let temp_match = temp_lint.match_kind();
                        map.entry(temp_match)
                           .and_modify(|v: &mut Vec<_>| v.push(temp_lint))
                           .or_insert_with(|| vec![temp_lint]);
                    }
                )*
                map
            };
        }
    }
}
