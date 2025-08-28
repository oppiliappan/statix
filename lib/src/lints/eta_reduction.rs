use crate::{Metadata, Report, Rule, Suggestion, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    types::{Apply, Ident, Lambda, TokenWrapper, TypedNode},
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
        let ident = Ident::cast(lambda_expr.arg()?)?;
        let body = Apply::cast(lambda_expr.body()?)?;

        if ident.as_str() != Ident::cast(body.value()?)?.as_str() {
            return None;
        }

        let lambda_node = body.lambda()?;

        if mentions_ident(&ident, &lambda_node) {
            return None;
        }

        // lambda body should be no more than a single Ident to
        // retain code readability
        Ident::cast(lambda_node)?;

        let at = node.text_range();
        let replacement = body.lambda()?;
        let message = format!("Found eta-reduction: `{}`", replacement.text());
        Some(
            self.report()
                .suggest(at, message, Suggestion::with_replacement(at, replacement)),
        )
    }
}

fn mentions_ident(ident: &Ident, node: &SyntaxNode) -> bool {
    if let Some(node_ident) = Ident::cast(node.clone()) {
        node_ident.as_str() == ident.as_str()
    } else {
        node.children().any(|child| mentions_ident(ident, &child))
    }
}
