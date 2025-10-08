use crate::{Metadata, Report, Rule, Suggestion};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{Expr, Ident, Lambda, Param},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks for eta-reducible functions, i.e.: converts lambda
/// expressions into free standing functions where applicable.
///
/// ## Why is this bad?
/// Oftentimes, eta-reduction results in code that is more natural
/// to read.
///
/// ## Example
///
/// ```nix
/// let
///   double = i: 2 * i;
/// in
/// map (x: double x) [ 1 2 3 ]
/// ```
///
/// The lambda passed to the `map` function is eta-reducible, and the
/// result reads more naturally:
///
/// ```nix
/// let
///   double = i: 2 * i;
/// in
/// map double [ 1 2 3 ]
/// ```
#[lint(
    name = "eta_reduction",
    note = "This function expression is eta reducible",
    code = 7,
    match_with = SyntaxKind::NODE_LAMBDA
)]
struct EtaReduction;

impl Rule for EtaReduction {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let lambda_expr = Lambda::cast(node.clone())?;

        let Some(Param::IdentParam(ident_param)) = lambda_expr.param() else {
            return None;
        };

        let ident = ident_param.ident()?;

        let Some(Expr::Apply(body)) = lambda_expr.body() else {
            return None;
        };

        let Some(Expr::Ident(body_ident)) = body.argument() else {
            return None;
        };

        if ident.to_string() != body_ident.to_string() {
            return None;
        }

        let lambda_node = body.lambda()?;

        if mentions_ident(&ident, lambda_node.syntax()) {
            return None;
        }

        // lambda body should be no more than a single Ident to
        // retain code readability
        let Expr::Ident(_) = lambda_node else {
            return None;
        };

        let at = node.text_range();
        let replacement = body.lambda()?;
        let message = format!("Found eta-reduction: `{}`", replacement.syntax().text());
        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(at, replacement.syntax().clone()),
        ))
    }
}

fn mentions_ident(ident: &Ident, node: &SyntaxNode) -> bool {
    if let Some(node_ident) = Ident::cast(node.clone()) {
        node_ident.to_string() == ident.to_string()
    } else {
        node.children().any(|child| mentions_ident(ident, &child))
    }
}
