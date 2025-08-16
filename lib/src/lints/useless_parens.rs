use crate::{Diagnostic, Metadata, Report, Rule, Suggestion, session::SessionInfo};

use if_chain::if_chain;
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
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(parsed_type_node) = ParsedType::cast(node.clone());

            if let Some(diagnostic) = do_thing(parsed_type_node);
            then {
                let mut report = self.report();
                report.diagnostics.push(diagnostic);
                Some(report)
            } else {
                None
            }
        }
    }
}

fn do_thing(parsed_type_node: ParsedType) -> Option<Diagnostic> {
    match parsed_type_node {
        ParsedType::KeyValue(kv) => if_chain! {
            if let Some(value_node) = kv.value();
            let value_range = value_node.text_range();
            if let Some(value_in_parens) = Paren::cast(value_node);
            if let Some(inner) = value_in_parens.inner();
            then {
                let at = value_range;
                let message = "Useless parentheses around value in binding";
                let replacement = inner;
                Some(Diagnostic::suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        },
        ParsedType::LetIn(let_in) => if_chain! {
            if let Some(body_node) = let_in.body();
            let body_range = body_node.text_range();
            if let Some(body_as_parens) = Paren::cast(body_node);
            if let Some(inner) = body_as_parens.inner();
            then {
                let at = body_range;
                let message = "Useless parentheses around body of `let` expression";
                let replacement = inner;
                Some(Diagnostic::suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        },
        ParsedType::Paren(paren_expr) => if_chain! {
            let paren_expr_range = paren_expr.node().text_range();
            if let Some(father_node) = paren_expr.node().parent();

            // ensure that we don't lint inside let-in statements
            // we already lint such cases in previous match stmt
            if KeyValue::cast(father_node.clone()).is_none();

            // ensure that we don't lint inside let-bodies
            // if this primitive is a let-body, we have already linted it
            if LetIn::cast(father_node).is_none();

            if let Some(inner_node) = paren_expr.inner();
            if let Some(parsed_inner) = ParsedType::cast(inner_node);
            if matches!(
                parsed_inner,
                ParsedType::List(_)
                | ParsedType::Paren(_)
                | ParsedType::Str(_)
                | ParsedType::AttrSet(_)
                | ParsedType::Select(_)
                | ParsedType::Ident(_)
            );
            then {
                let at = paren_expr_range;
                let message = "Useless parentheses around primitive expression";
                let replacement = parsed_inner.node().clone();
                Some(Diagnostic::suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        },
        _ => None,
    }
}
