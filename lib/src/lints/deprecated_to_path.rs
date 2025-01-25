use crate::{session::SessionInfo, Metadata, Report, Rule};
use rowan::ast::AstNode;

use macros::lint;
use rnix::{ast::Apply, NodeOrToken, SyntaxElement, SyntaxKind};

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
        let NodeOrToken::Node(node) = node else {
            return None;
        };
        let apply = Apply::cast(node.clone())?;
        let lambda_path = apply.lambda()?.to_string();
        if !ALLOWED_PATHS.iter().any(|&p| p == lambda_path.as_str()) {
            return None;
        };
        let at = node.text_range();
        let message = format!(
            "`{}` is deprecated, see `:doc builtins.toPath` within the REPL for more",
            lambda_path
        );
        Some(self.report().diagnostic(at, message))
    }
}
