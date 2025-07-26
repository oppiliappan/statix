//! `rnix` changed their way of handling parsing. It won't return a result anymore and instead
//! always returns some parsed `GreenNode` as well as a `Vec<ParseError>`. For our purposes, it's
//! much more useful to get a `Result<GreenNode, Vec<ParseError>>` and this module implements some
//! helper functionality just for that.

/// The results from parsing some kind of nix source code.
///
/// It will always include some kind of parsed code as a [`rnix::SyntaxNode`] and accumulates
/// [`rnix::parser::ParseError`] it encounters.
pub struct ParseResult {
    parsed: rnix::SyntaxNode,
    errors: Vec<rnix::parser::ParseError>,
}

impl ParseResult {
    /// parse a piece of nix source code into a [`ParseResult`]. This is the main way to construct
    /// a [`ParseResult`].
    #[must_use]
    pub fn parse(source: impl AsRef<str>) -> Self {
        let (green, errors) = rnix::parser::parse(rnix::tokenize(source.as_ref()).into_iter());
        let parsed = rnix::SyntaxNode::new_root(green);
        Self { parsed, errors }
    }

    /// retrieves both the raw [`rnix::SyntaxNode`] and all of the collected raw [`rnix::parser::ParseError`]
    #[must_use]
    pub fn to_tuple(self) -> (rnix::SyntaxNode, Vec<rnix::parser::ParseError>) {
        (self.parsed, self.errors)
    }

    /// converts this [`ParseResult`] into a real `Result`
    pub fn to_result(self) -> Result<rnix::SyntaxNode, Vec<rnix::parser::ParseError>> {
        self.errors
            .is_empty()
            .then_some(self.parsed)
            .ok_or(self.errors)
    }
}
