use crate::{Lint, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{Lambda, Ident, Apply, TypedNode, TokenWrapper},
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
};

fn mentions_ident(ident: &Ident, node: &SyntaxNode) -> bool {
    if let Some(node_ident) = Ident::cast(node.clone()) {
        node_ident.as_str() == ident.as_str()
    } else {
        node.children().any(|child| mentions_ident(&ident, &child))
    }
}

#[lint(
    name = "eta reduction",
    note = "This function expression is eta reducible",
    code = 7,
    match_with = SyntaxKind::NODE_LAMBDA
)]
struct EtaReduction;

impl Rule for EtaReduction {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(lambda_expr) = Lambda::cast(node.clone());

            if let Some(arg_node) = lambda_expr.arg();
            if let Some(arg) = Ident::cast(arg_node);

            if let Some(body_node) = lambda_expr.body();
            if let Some(body) = Apply::cast(body_node);

            if let Some(value_node) = body.value();
            if let Some(value) = Ident::cast(value_node);

            if arg.as_str() == value.as_str() ;

            if let Some(lambda_node) = body.lambda();
            if !mentions_ident(&arg, &lambda_node);

            then {
                let at = node.text_range();
                let replacement = body.lambda()?;
                let message = 
                    format!(
                        "Found eta-reduction: `{}`",
                        replacement.text().to_string()
                    );
                Some(Self::report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}


