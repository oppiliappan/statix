use crate::{Metadata, Report, Rule, session::SessionInfo};

use macros::lint;
use rnix::{NodeOrToken, SyntaxElement, SyntaxKind, ast::Apply};
use rowan::ast::AstNode as _;

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
struct DeprecatedToPath;

static ALLOWED_PATHS: &[&str; 2] = &["builtins.toPath", "toPath"];

impl Rule for DeprecatedToPath {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if let NodeOrToken::Node(node) = node
            && let Some(apply) = Apply::cast(node.clone())
            && let lambda_path = apply.lambda()?.to_string()
            && ALLOWED_PATHS.contains(&lambda_path.as_str())
        {
            let at = node.text_range();
            let message = format!(
                "`{lambda_path}` is deprecated, see `:doc builtins.toPath` within the REPL for more"
            );
            Some(self.report().diagnostic(at, message))
        } else {
            None
        }
    }
}
