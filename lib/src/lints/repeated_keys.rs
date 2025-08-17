use crate::{session::SessionInfo, Metadata, Report, Rule};

use macros::lint;
use rnix::{
    ast::{AttrSet, AttrpathValue, HasEntry, Ident},
    NodeOrToken, SyntaxElement, SyntaxKind,
};
use rowan::ast::AstNode;

/// ## What it does
/// Checks for keys in attribute sets with repetitive keys, and suggests using
/// an attribute set instead.
///
/// ## Why is this bad?
/// Avoiding repetetion helps improve readibility.
///
/// ## Example
/// ```nix
/// {
///   foo.a = 1;
///   foo.b = 2;
///   foo.c = 3;
/// }
/// ```
///
/// Don't repeat.
/// ```nix
/// {
///   foo = {
///     a = 1;
///     b = 2;
///     c = 3;
///   };
/// }
/// ```

#[lint(
    name = "repeated_keys",
    note = "Avoid repeated keys in attribute sets",
    code = 20,
    match_with = SyntaxKind::NODE_ATTRPATH_VALUE
)]
struct RepeatedKeys;

impl Rule for RepeatedKeys {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if let NodeOrToken::Node(node) = node {
            let key_value = AttrpathValue::cast(node.clone())?;
            let key = key_value.attrpath()?;
            let mut components = key.attrs();
            let first_component = components.next()?;
            let first_component_ident = Ident::cast(first_component.syntax().clone())?;
            // ensure that there are >1 components
            components.next()?;

            let parent_node = node.parent()?;
            let parent_attr_set = AttrSet::cast(parent_node)?;

            parent_attr_set.rec_token().is_none().then_some(())?;
            let occurrences = parent_attr_set
                .entries()
                .filter_map(|entry| match entry {
                    rnix::ast::Entry::AttrpathValue(attr) => Some(attr),
                    _ => None,
                })
                .filter_map(|kv_scrutinee| {
                    let scrutinee_key = kv_scrutinee.attrpath()?;
                    let mut kv_scrutinee_components = scrutinee_key.attrs();
                    let kv_scrutinee_first_component = kv_scrutinee_components.next()?;
                    let kv_scrutinee_ident =
                        Ident::cast(kv_scrutinee_first_component.syntax().clone())?;
                    if kv_scrutinee_ident.to_string() == first_component_ident.to_string() {
                        Some((
                            scrutinee_key.syntax().text_range(),
                            std::iter::once(kv_scrutinee_first_component)
                                .chain(kv_scrutinee_components)
                                .map(|n| n.to_string())
                                .collect::<Vec<_>>()
                                .join("."),
                        ))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            (occurrences.first()?.0 == key.syntax().text_range()).then_some(())?;
            (occurrences.len() >= 3).then_some(())?;

            let mut iter = occurrences.into_iter();

            let (first_annotation, first_subkey) = iter.next().unwrap();
            let first_message = format!(
                "The key `{}` is first assigned here ...",
                first_component_ident.to_string()
            );

            let (second_annotation, second_subkey) = iter.next().unwrap();
            let second_message = "... repeated here ...";

            let (third_annotation, third_subkey) = iter.next().unwrap();
            let third_message = {
                let remaining_occurrences = iter.count();
                let mut message = match remaining_occurrences {
                    0 => "... and here.".to_string(),
                    1 => "... and here (`1` occurrence omitted).".to_string(),
                    n => format!("... and here (`{n}` occurrences omitted)."),
                };
                message.push_str(&format!(
                    " Try `{} = {{ {}=...; {}=...; {}=...; }}` instead.",
                    first_component_ident.to_string(),
                    first_subkey.split_once('.')?.1,
                    second_subkey.split_once('.')?.1,
                    third_subkey.split_once('.')?.1
                ));
                message
            };

            Some(
                self.report()
                    .diagnostic(first_annotation, first_message)
                    .diagnostic(second_annotation, second_message)
                    .diagnostic(third_annotation, third_message),
            )
        } else {
            None
        }
    }
}
