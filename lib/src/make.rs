use std::{fmt::Write, iter::IntoIterator};

use rnix::{
    Root, SyntaxNode,
    ast::{self, AstNode},
};
use rowan::ast::AstNode as _;

fn ast_from_text<N: AstNode>(text: &str) -> N {
    let parse = Root::parse(text).ok();

    let Ok(parse) = parse else {
        panic!("Failed to parse `{text:?}`")
    };

    let Some(node) = parse.syntax().descendants().find_map(N::cast) else {
        panic!(
            "Failed to make ast node `{}` from text `{}`",
            std::any::type_name::<N>(),
            text
        );
    };

    node
}

pub fn parenthesize(node: &SyntaxNode) -> ast::Paren {
    ast_from_text(&format!("({node})"))
}

pub fn quote(node: &SyntaxNode) -> ast::Str {
    ast_from_text(&format!("\"{node}\""))
}

pub fn unary_not(node: &SyntaxNode) -> ast::UnaryOp {
    ast_from_text(&format!("!{node}"))
}

pub fn inherit_stmt<'a>(nodes: impl IntoIterator<Item = &'a ast::Ident>) -> ast::Inherit {
    let inherited_idents = nodes
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    ast_from_text(&format!("{{ inherit {inherited_idents}; }}"))
}

pub fn inherit_from_stmt<'a>(
    from: &SyntaxNode,
    nodes: impl IntoIterator<Item = &'a ast::Ident>,
) -> ast::Inherit {
    let inherited_idents = nodes
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    ast_from_text(&format!("{{ inherit ({from}) {inherited_idents}; }}"))
}

pub fn attrset(
    inherits: impl IntoIterator<Item = ast::Inherit>,
    entries: impl IntoIterator<Item = ast::Entry>,
    recursive: bool,
) -> ast::AttrSet {
    let mut buffer = String::new();

    writeln!(buffer, "{}{{", if recursive { "rec " } else { "" }).unwrap();
    for inherit in inherits {
        writeln!(buffer, "  {inherit}").unwrap();
    }
    for entry in entries {
        writeln!(buffer, "  {entry}").unwrap();
    }
    write!(buffer, "}}").unwrap();

    ast_from_text(&buffer)
}

pub fn select(set: &SyntaxNode, index: &SyntaxNode) -> ast::Select {
    ast_from_text(&format!("{set}.{index}"))
}

pub fn ident(text: &str) -> ast::Ident {
    ast_from_text(text)
}

// LATER: make `op` strongly typed here
pub fn binary(lhs: &SyntaxNode, op: &str, rhs: &SyntaxNode) -> ast::BinOp {
    ast_from_text(&format!("{lhs} {op} {rhs}"))
}

pub fn or_default(set: &SyntaxNode, index: &SyntaxNode, default: &SyntaxNode) -> ast::Select {
    ast_from_text(&format!("{set}.{index} or {default}"))
}
