use crate::{Metadata, Report, Rule, session::SessionInfo};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{NodeOrToken, SyntaxElement, SyntaxKind, ast::Apply};

/// ## What it does
/// Checks for usage of the `toPath` function.
///
/// ## Why is this bad?
/// `toPath` is deprecated.
///
/// ## Example
///
/// ```nix
/// builtins.toPath "/path"
/// ```
///
/// Try these instead:
///
/// ```nix
/// # to convert the string to an absolute path:
/// /. + "/path"
/// # => /abc
///
/// # to convert the string to a path relative to the current directory:
/// ./. + "/bin"
/// # => /home/np/statix/bin
/// ```
#[lint(
    name = "deprecated_to_path",
    note = "Found usage of deprecated builtin toPath",
    code = 17,
    match_with = SyntaxKind::NODE_APPLY
)]
struct DeprecatedIsNull;

static ALLOWED_PATHS: &[&str; 2] = &["builtins.toPath", "toPath"];

impl Rule for DeprecatedIsNull {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(apply) = Apply::cast(node.clone());
            let lambda_path = apply.lambda()?.to_string();
            if ALLOWED_PATHS.iter().any(|&p| p == lambda_path.as_str());
            then {
                let at = node.text_range();
                let message = format!("`{}` is deprecated, see `:doc builtins.toPath` within the REPL for more", lambda_path);
                Some(self.report().diagnostic(at, message))
            } else {
                None
            }
        }
    }
}
