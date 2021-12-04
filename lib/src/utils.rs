use rnix::{SyntaxKind, SyntaxNode, TextRange};

pub fn with_preceeding_whitespace(node: &SyntaxNode) -> TextRange {
    let start = node
        .prev_sibling_or_token()
        .map(|t| {
            if t.kind() == SyntaxKind::TOKEN_WHITESPACE {
                t.text_range().start()
            } else {
                t.text_range().end()
            }
        })
        .unwrap_or(node.text_range().start());
    let end = node.text_range().end();
    TextRange::new(start, end)
}
