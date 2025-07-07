use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{NodeOrToken, SyntaxElement, SyntaxKind, TextRange, ast::LetIn};
use rowan::Direction;

/// ## What it does
/// Checks for `let-in` expressions whose body is another `let-in`
/// expression.
///
/// ## Why is this bad?
/// Unnecessary code, the `let-in` expressions can be merged.
///
/// ## Example
///
/// ```nix
/// let
///   a = 2;
/// in
/// let
///   b = 3;
/// in
///   a + b
/// ```
///
/// Merge both `let-in` expressions:
///
/// ```nix
/// let
///   a = 2;
///   b = 3;
/// in
///   a + b
/// ```
#[lint(
    name = "collapsible_let_in",
    note = "These let-in expressions are collapsible",
    code = 6,
    match_with = SyntaxKind::NODE_LET_IN
)]
struct CollapsibleLetIn;

impl Rule for CollapsibleLetIn {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(let_in_expr) = LetIn::cast(node.clone());
            if let Some(body) = let_in_expr.body();

            if LetIn::cast(body.syntax().clone()).is_some();
            then {
                let first_annotation = node.text_range();
                let first_message = "This `let in` expression contains a nested `let in` expression";

                let second_annotation = body.syntax().text_range();
                let second_message = "This `let in` expression is nested";

                let replacement_at = {
                    let start = body.syntax()
                        .siblings_with_tokens(Direction::Prev)
                        .find(|elem| elem.kind() == SyntaxKind::TOKEN_IN)?
                        .text_range()
                        .start();
                    let end = body.syntax()
                        .descendants_with_tokens()
                        .find(|elem| elem.kind() == SyntaxKind::TOKEN_LET)?
                        .text_range()
                        .end();
                    TextRange::new(start, end)
                };
                let replacement = make::empty();

                Some(
                    self.report()
                        .diagnostic(first_annotation, first_message)
                        .suggest(second_annotation, second_message, Suggestion::new(replacement_at, replacement.syntax().clone()))
                )
            } else {
                None
            }
        }
    }
}
