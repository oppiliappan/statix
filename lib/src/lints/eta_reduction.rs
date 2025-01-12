use crate::{session::SessionInfo, Metadata, Report, Rule, Suggestion};
use rowan::ast::AstNode;

use macros::lint;
use rnix::{
    ast::{Apply, Ident, Lambda, Param},
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
};

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
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };
        let lambda_expr = Lambda::cast(node.clone())?;

        let arg_node = lambda_expr.param()?;
        let arg = {
            let ident = match arg_node {
                Param::IdentParam(ident) => ident.ident(),
                _ => None,
            };
            ident?
        };

        let body_node = lambda_expr.body()?;
        let body = Apply::cast(body_node.syntax().clone())?;

        let value_node = body.argument()?;
        let value = Ident::cast(value_node.syntax().clone())?;

        if arg.to_string() != value.to_string() {
            return None;
        };

        let lambda_node = body.lambda()?;
        if mentions_ident(&arg, lambda_node.syntax()) {
            return None;
        };

        // lambda body should be no more than a single Ident to
        // retain code readability
        Ident::cast(lambda_node.syntax().clone())?;

        let at = node.text_range();
        let replacement = body.lambda()?;
        let message = format!("Found eta-reduction: `{}`", replacement.syntax().text());
        Some(self.report().suggest(
            at,
            message,
            Suggestion::new(at, replacement.syntax().clone()),
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
