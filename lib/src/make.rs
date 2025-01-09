use std::{fmt::Write, iter::IntoIterator};

use rnix::{
    ast::{self, AstNode},
    tokenize, SyntaxNode,
};

fn ast_from_text<N: AstNode>(text: &str) -> N {
    let (green, errors) = rnix::parser::parse(tokenize(text).into_iter());
    if !errors.is_empty() {
        panic!(
            "Failed to make ast node `{}` from text `{}`\n{errors}",
            std::any::type_name::<N>(),
            text,
            errors = errors
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    let untyped_node = SyntaxNode::new_root(green);
    let Some(typed_node) = N::cast(untyped_node) else {
        panic!(
            "Failed to make ast node `{}` from text `{}`",
            std::any::type_name::<N>(),
            text
        );
    };
    typed_node
}

pub fn parenthesize(node: &SyntaxNode) -> ast::Paren {
    ast_from_text(&format!("({})", node))
}

pub fn quote(node: &SyntaxNode) -> ast::Str {
    ast_from_text(&format!("\"{}\"", node))
}

pub fn unary_not(node: &SyntaxNode) -> ast::UnaryOp {
    ast_from_text(&format!("!{}", node))
}

pub fn inherit_stmt<'a>(nodes: impl IntoIterator<Item = &'a ast::Ident>) -> ast::Inherit {
    let inherited_idents = nodes
        .into_iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    ast_from_text(&format!("{{ inherit {}; }}", inherited_idents))
}

pub fn inherit_from_stmt<'a>(
    from: SyntaxNode,
    nodes: impl IntoIterator<Item = &'a ast::Ident>,
) -> ast::Inherit {
    let inherited_idents = nodes
        .into_iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    ast_from_text(&format!("{{ inherit ({}) {}; }}", from, inherited_idents))
}

pub fn attrset(
    inherits: impl IntoIterator<Item = ast::Inherit>,
    entries: impl IntoIterator<Item = ast::Entry>,
    recursive: bool,
) -> ast::AttrSet {
    let mut buffer = String::new();

    writeln!(buffer, "{}{{", if recursive { "rec " } else { "" }).unwrap();
    for inherit in inherits.into_iter() {
        writeln!(buffer, "  {}", inherit.to_string()).unwrap();
    }
    for entry in entries.into_iter() {
        writeln!(buffer, "  {}", entry.to_string()).unwrap();
    }
    write!(buffer, "}}").unwrap();

    ast_from_text(&buffer)
}

pub fn select(set: &SyntaxNode, index: &SyntaxNode) -> ast::Select {
    ast_from_text(&format!("{}.{}", set, index))
}

pub fn ident(text: &str) -> ast::Ident {
    ast_from_text(text)
}

pub fn empty() -> ast::Root {
    ast_from_text("")
}

// TODO: make `op` strongly typed here
pub fn binary(lhs: &SyntaxNode, op: &str, rhs: &SyntaxNode) -> ast::BinOp {
    ast_from_text(&format!("{} {} {}", lhs, op, rhs))
}

pub fn or_default(set: &SyntaxNode, index: &SyntaxNode, default: &SyntaxNode) -> ast::Select {
    ast_from_text(&format!("{}.{} or {}", set, index, default))
}
