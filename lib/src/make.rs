use std::iter::IntoIterator;

use rnix::{
    SyntaxNode,
    ast::{self, AstNode},
};

fn ast_from_text<N: AstNode>(text: impl AsRef<str>) -> N {
    let parsed = crate::parse::ParseResult::parse(text.as_ref())
        .to_result()
        .unwrap_or_else(|errors| {
            panic!(
                "Failed to make ast node `{}` from text `{}`\n{errors}",
                std::any::type_name::<N>(),
                text.as_ref(),
                errors = errors
                    .into_iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        });

    // for inherit it goes ROOT ( ATTRSET (INHERIT)), hence we have to traverse 3 levels at worst
    // FIXME: just specialize for the structs
    std::iter::once(parsed.clone())
        .chain(parsed.clone().children())
        .chain(parsed.children().flat_map(|child| child.children()))
        .find_map(|child| N::cast(child))
        .unwrap_or_else(|| {
            panic!(
                "Failed to make ast node `{}` from text `{}`",
                std::any::type_name::<N>(),
                text.as_ref()
            )
        })
}

pub fn parenthesize(node: &SyntaxNode) -> ast::Paren {
    ast_from_text(format!("({node})"))
}

pub fn quote(node: &SyntaxNode) -> ast::Str {
    ast_from_text(format!("\"{node}\""))
}

pub fn unary_not(node: &SyntaxNode) -> ast::UnaryOp {
    ast_from_text(format!("!{node}"))
}

pub fn inherit_stmt<'a>(nodes: impl IntoIterator<Item = &'a ast::Ident>) -> ast::Inherit {
    let inherited_idents = nodes
        .into_iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    ast_from_text(format!("{{ inherit {inherited_idents}; }}"))
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
    ast_from_text(format!("{{ inherit ({from}) {inherited_idents}; }}"))
}

pub fn attrset(
    inherits: impl IntoIterator<Item = ast::Inherit>,
    entries: impl IntoIterator<Item = ast::Entry>,
    recursive: bool,
) -> ast::AttrSet {
    let rec = recursive.then_some("rec ").unwrap_or_default();
    let inherits = inherits
        .into_iter()
        .map(|inherit| format!("  {inherit}"))
        .collect::<Vec<_>>()
        .join("\n");
    let entries = entries
        .into_iter()
        .map(|inherit| format!("  {inherit}"))
        .collect::<Vec<_>>()
        .join("\n");

    ast_from_text(format!(
        "{rec}{{
{inherits}
{entries}
}}"
    ))
}

pub fn select(set: &SyntaxNode, index: &SyntaxNode) -> ast::Select {
    ast_from_text(format!("{set}.{index}"))
}

pub fn ident(text: impl AsRef<str>) -> ast::Ident {
    ast_from_text(text)
}

pub fn empty() -> ast::Root {
    ast::Root::parse("{}").ok().unwrap()
}

// TODO: make `op` strongly typed here
pub fn binary(lhs: &SyntaxNode, op: impl AsRef<str>, rhs: &SyntaxNode) -> ast::BinOp {
    ast_from_text(format!("{lhs} {op} {rhs}", op = op.as_ref()))
}

pub fn or_default(set: &SyntaxNode, index: &SyntaxNode, default: &SyntaxNode) -> ast::Select {
    ast_from_text(format!("{set}.{index} or {default}"))
}
