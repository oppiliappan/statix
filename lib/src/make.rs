use rnix::{SyntaxNode, types::{TypedNode, self}};

fn ast_from_text<N: TypedNode>(text: &str) -> N {
    let parse = rnix::parse(text);
    let node = match parse.node().descendants().find_map(N::cast) {
        Some(it) => it,
        None => {
            panic!("Failed to make ast node `{}` from text {}", std::any::type_name::<N>(), text)
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
