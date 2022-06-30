use crate::{session::SessionInfo, Metadata, Report, Rule};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{Lambda, TypedNode, Pattern},
    NodeOrToken, SyntaxElement, SyntaxKind
};

/// ## What it does
/// Prevent long single-line pattern in root lambda.
///
/// ## Why is this bad?
/// It's bad for readability.
///
/// ## Example
/// ```nix
/// { lib, and, many, other, names }:
/// ```
///
/// Split into lines.
/// ```nix
/// { lib
/// , and
/// , many
/// , other
/// , names
/// }:
/// ```

#[lint(
    name = "long_root_pattern",
    note = "Long pattern in the root lambda should be wrapped",
    code = 21,
    match_with = SyntaxKind::NODE_ROOT
)]
struct LongRootPattern;

fn count_pattern_entries(pattern: &Pattern) -> usize {
    pattern.entries().count()
}

impl Rule for LongRootPattern {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(node) = node.children().next();
            if let Some(lambda) = Lambda::cast(node.clone());
            if let Some(pattern) = lambda.arg();
            if let Some(pattern) = Pattern::cast(pattern.clone());
            if count_pattern_entries(&pattern) > 6;
            then {
                let at = pattern.node().text_range();
                let message = "Split the long pattern line into multiple lines";
                Some(
                    self.report()
                        .diagnostic(at, message),
                )
            } else {
                None
            }
        }
    }
}