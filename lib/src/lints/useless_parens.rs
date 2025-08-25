use crate::{Diagnostic, Metadata, Report, Rule, Suggestion, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    types::{KeyValue, LetIn, Paren, ParsedType, TypedNode, Wrapper},
};

/// ## What it does
/// Checks for unnecessary parentheses.
///
/// ## Why is this bad?
/// Unnecessarily parenthesized code is hard to read.
///
/// ## Example
///
/// ```nix
/// let
///   double = (x: 2 * x);
///   ls = map (double) [ 1 2 3 ];
/// in
///   (2 + 3)
/// ```
///
/// Remove unnecessary parentheses:
///
/// ```nix
/// let
///   double = x: 2 * x;
///   ls = map double [ 1 2 3 ];
/// in
///   2 + 3
/// ```
#[lint(
    name = "useless_parens",
    note = "These parentheses can be omitted",
    code = 8,
    match_with = [
        SyntaxKind::NODE_KEY_VALUE,
        SyntaxKind::NODE_PAREN,
        SyntaxKind::NODE_LET_IN,
    ]
)]
struct UselessParens;

impl Rule for UselessParens {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let parsed_type_node = ParsedType::cast(node.clone())?;

        let diagnostic = match parsed_type_node {
            ParsedType::KeyValue(kv) => {
                let value_node = kv.value()?;
                let value_range = value_node.text_range();

                Diagnostic::suggest(
                    value_range,
                    "Useless parentheses around value in binding",
                    Suggestion::new(value_range, Paren::cast(value_node)?.inner()?),
                )
            }
            ParsedType::LetIn(let_in) => {
                let body_node = let_in.body()?;
                let body_range = body_node.text_range();
                Diagnostic::suggest(
                    body_range,
                    "Useless parentheses around body of `let` expression",
                    Suggestion::new(body_range, Paren::cast(body_node)?.inner()?),
                )
            }
            ParsedType::Paren(paren_expr) => {
                let paren_expr_range = paren_expr.node().text_range();
                let father_node = paren_expr.node().parent()?;

                // ensure that we don't lint inside let-in statements
                // we already lint such cases in previous match stmt
                if KeyValue::cast(father_node.clone()).is_some() {
                    return None;
                }

                // ensure that we don't lint inside let-bodies
                // if this primitive is a let-body, we have already linted it
                if LetIn::cast(father_node).is_some() {
                    return None;
                }

                let parsed_inner = ParsedType::cast(paren_expr.inner()?)?;

                if !matches!(
                    parsed_inner,
                    ParsedType::List(_)
                        | ParsedType::Paren(_)
                        | ParsedType::Str(_)
                        | ParsedType::AttrSet(_)
                        | ParsedType::Select(_)
                        | ParsedType::Ident(_)
                ) {
                    return None;
                }

                Diagnostic::suggest(
                    paren_expr_range,
                    "Useless parentheses around primitive expression",
                    Suggestion::new(paren_expr_range, parsed_inner.node().clone()),
                )
            }
            _ => return None,
        };

        let mut report = self.report();
        report.diagnostics.push(diagnostic);
        Some(report)
    }
}
