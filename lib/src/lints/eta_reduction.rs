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
        if let NodeOrToken::Node(node) = node
            && let Some(lambda_expr) = Lambda::cast(node.clone())
            && let Some(arg_node) = lambda_expr.arg()
            && let Some(arg) = Ident::cast(arg_node)
            && let Some(body_node) = lambda_expr.body()
            && let Some(body) = Apply::cast(body_node)
            && let Some(value_node) = body.value()
            && let Some(value) = Ident::cast(value_node)
            && arg.as_str() == value.as_str()
            && let Some(lambda_node) = body.lambda()
            && !mentions_ident(&arg, &lambda_node)
            // lambda body should be no more than a single Ident to
            // retain code readability
            && let Some(_) = Ident::cast(lambda_node)
        {
            let at = node.text_range();
            let replacement = body.lambda()?;
            let message = format!("Found eta-reduction: `{}`", replacement.text());
            Some(
                self.report()
                    .suggest(at, message, Suggestion::new(at, replacement)),
            )
        } else {
            None
        }
    }
}

fn mentions_ident(ident: &Ident, node: &SyntaxNode) -> bool {
    if let Some(node_ident) = Ident::cast(node.clone()) {
        node_ident.as_str() == ident.as_str()
    } else {
        node.children().any(|child| mentions_ident(ident, &child))
    }
}
