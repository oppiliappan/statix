use crate::{Diagnostic, Metadata, Report, Rule, Suggestion, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    types::{Apply, BinOp, KeyValue, LetIn, Paren, ParsedType, TypedNode, Wrapper},
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
        SyntaxKind::NODE_BIN_OP,
    ]
)]
struct UselessParens;

enum OneOrMany<A> {
    One(A),
    Many(Vec<A>),
}

impl Rule for UselessParens {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if let NodeOrToken::Node(node) = node
            && let Some(parsed_type_node) = ParsedType::cast(node.clone())
            && let Some(diagnostic) = do_thing(parsed_type_node)
        {
            let mut report = self.report();
            match diagnostic {
                OneOrMany::One(x) => report.diagnostics.push(x),
                OneOrMany::Many(mut xs) => report.diagnostics.append(&mut xs),
            }
            Some(report)
        } else {
            None
        }
    }
}

fn do_thing(parsed_type_node: ParsedType) -> Option<OneOrMany<Diagnostic>> {
    match parsed_type_node {
        ParsedType::KeyValue(kv) => {
            if let Some(value_node) = kv.value()
                && let value_range = value_node.text_range()
                && let Some(value_in_parens) = Paren::cast(value_node)
                && let Some(inner) = value_in_parens.inner()
            {
                let at = value_range;
                let message = "Useless parentheses around value in binding";
                let replacement = inner;
                Some(OneOrMany::One(Diagnostic::suggest(
                    at,
                    message,
                    Suggestion::new(at, replacement),
                )))
            } else {
                None
            }
        }
        ParsedType::LetIn(let_in) => {
            if let Some(body_node) = let_in.body()
                && let body_range = body_node.text_range()
                && let Some(body_as_parens) = Paren::cast(body_node)
                && let Some(inner) = body_as_parens.inner()
            {
                let at = body_range;
                let message = "Useless parentheses around body of `let` expression";
                let replacement = inner;
                Some(OneOrMany::One(Diagnostic::suggest(
                    at,
                    message,
                    Suggestion::new(at, replacement),
                )))
            } else {
                None
            }
        }
        ParsedType::BinOp(bin_op) => {
            let maybe_diagnostic = |node: SyntaxNode| -> Option<Diagnostic> {
                let range = node.text_range();
                let as_parens = Paren::cast(node)?;
                let inner = as_parens.inner()?;

                // TODO:
                // it would be nice to compare operator precedence
                // currently we only check if inner is function
                if Apply::cast(inner.clone()).is_some() {
                    let at = range;
                    let message = "Useless parentheses in operand of binary operator";
                    let replacement = inner;
                    Some(Diagnostic::suggest(
                        at,
                        message,
                        Suggestion::new(at, replacement),
                    ))
                } else {
                    None
                }
            };

            // Fix rhs then lhs otherwise the position will drift
            let diagnostics = vec![bin_op.rhs(), bin_op.lhs()]
                .into_iter()
                .flatten()
                .filter_map(maybe_diagnostic)
                .collect::<Vec<_>>();

            if diagnostics.len() > 0 {
                Some(OneOrMany::Many(diagnostics))
            } else {
                None
            }
        }
        ParsedType::Paren(paren_expr) => {
            let paren_expr_range = paren_expr.node().text_range();
            if let Some(father_node) = paren_expr.node().parent()
            // ensure that we don't lint inside let-in statements
            // we already lint such cases in previous match stmt
            && KeyValue::cast(father_node.clone()).is_none()

            // ensure that we don't lint inside let-bodies
            // if this primitive is a let-body, we have already linted it
            && LetIn::cast(father_node.clone()).is_none()

            // ensure that we don't lint inside of neither bin-op branches
            && BinOp::cast(father_node).is_none()

            && let Some(inner_node) = paren_expr.inner()
            && let Some(parsed_inner) = ParsedType::cast(inner_node)
            && matches!(
                parsed_inner,
                ParsedType::List(_)
                | ParsedType::Paren(_)
                | ParsedType::Str(_)
                | ParsedType::AttrSet(_)
                | ParsedType::Select(_)
                | ParsedType::Ident(_)
            ) {
                let at = paren_expr_range;
                let message = "Useless parentheses around primitive expression";
                let replacement = parsed_inner.node().clone();
                Some(OneOrMany::One(Diagnostic::suggest(
                    at,
                    message,
                    Suggestion::new(at, replacement),
                )))
            } else {
                None
            }
        }
        _ => None,
    }
}
