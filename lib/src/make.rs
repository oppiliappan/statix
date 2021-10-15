use std::{fmt::Write, iter::IntoIterator};

use rnix::{
    types::{self, TokenWrapper, TypedNode},
    SyntaxNode,
};

fn ast_from_text<N: TypedNode>(text: &str) -> N {
    let parse = rnix::parse(text);
    let node = match parse.node().descendants().find_map(N::cast) {
        Some(it) => it,
        None => {
            panic!(
                "Failed to make ast node `{}` from text `{}`",
                std::any::type_name::<N>(),
                text
            )
        }
    };
    node
}

pub fn parenthesize(node: &SyntaxNode) -> types::Paren {
    ast_from_text(&format!("({})", node))
}

pub fn unary_not(node: &SyntaxNode) -> types::UnaryOp {
    ast_from_text(&format!("!{}", node))
}

pub fn inherit_stmt<'a>(nodes: impl IntoIterator<Item = &'a types::Ident>) -> types::Inherit {
    let inherited_idents = nodes
        .into_iter()
        .map(|i| i.as_str().to_owned())
        .collect::<Vec<_>>()
        .join(" ");
    ast_from_text(&format!("{{ inherit {}; }}", inherited_idents))
}

pub fn inherit_from_stmt<'a>(
    from: SyntaxNode,
    nodes: impl IntoIterator<Item = &'a types::Ident>,
) -> types::Inherit {
    let inherited_idents = nodes
        .into_iter()
        .map(|i| i.as_str().to_owned())
        .collect::<Vec<_>>()
        .join(" ");
    ast_from_text(&format!("{{ inherit ({}) {}; }}", from, inherited_idents))
}

pub fn attrset(
    inherits: impl IntoIterator<Item = types::Inherit>,
    entries: impl IntoIterator<Item = types::KeyValue>,
    recursive: bool,
) -> types::AttrSet {
    let mut buffer = String::new();

    writeln!(buffer, "{}{{", if recursive { "rec " } else { "" }).unwrap();
    for inherit in inherits.into_iter() {
        writeln!(buffer, "  {}", inherit.node().text()).unwrap();
    }
    for entry in entries.into_iter() {
        writeln!(buffer, "  {}", entry.node().text()).unwrap();
    }
    write!(buffer, "}}").unwrap();

    ast_from_text(&buffer)
}

pub fn select(set: &SyntaxNode, index: &SyntaxNode) -> types::Select {
    ast_from_text(&format!("{}.{}", set, index))
}

pub fn ident(text: &str) -> types::Ident {
    ast_from_text(text)
}

pub fn empty() -> types::Root {
    ast_from_text("")
}
