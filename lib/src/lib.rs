pub mod lints;

use rnix::{SyntaxElement, SyntaxKind, TextRange};
use std::ops::Deref;

pub trait Rule {
    fn validate(&self, node: &SyntaxElement) -> Option<Diagnostic>;
}

#[derive(Debug)]
pub struct Diagnostic {
    pub at: TextRange,
    pub message: String,
}

impl Diagnostic {
    pub fn new(at: TextRange, message: String) -> Self {
        Self { at, message }
    }
}

pub trait Metadata {
    fn name(&self) -> &str;
    fn note(&self) -> &str;
    fn match_with(&self, with: &SyntaxKind) -> bool;
}

pub trait Lint: Metadata + Rule + Send + Sync {}

// #[macro_export]
// macro_rules! lint_map {
//     ($($s:ident),*,) => {
//         lint_map($($s),*);
//     }
//     ($($s:ident),*) => {
//         use ::std::collections::HashMap;
//         use rnix::SyntaxKind;
//         $(
//             mod $s;
//         )*
//         ::lazy_static::lazy_static! {
//             pub static ref RULES: HashMap<SyntaxKind, &'static Box<dyn $crate::Lint>> = {
//                 vec![$(&*$s::LINT,)*]
//             }
//         }
//     }
// }
