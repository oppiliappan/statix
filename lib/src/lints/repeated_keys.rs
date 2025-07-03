use crate::{Metadata, Report, Rule, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    types::{AttrSet, EntryHolder, Ident, KeyValue, TokenWrapper, TypedNode},
};

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
    match_with = SyntaxKind::NODE_KEY_VALUE
)]
struct RepeatedKeys;

impl Rule for RepeatedKeys {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if let NodeOrToken::Node(node) = node
            && let Some(key_value) = KeyValue::cast(node.clone())
            && let Some(key) = key_value.key()
            && let mut components = key.path()
            && let Some(first_component) = components.next()
            && let Some(first_component_ident) = Ident::cast(first_component)
            // ensure that there are >1 components
            && components.next().is_some()
            && let Some(parent_node) = node.parent()
            && let Some(parent_attr_set) = AttrSet::cast(parent_node)
            && !parent_attr_set.recursive()
            && let occurrences = parent_attr_set.entries().filter_map(|kv_scrutinee| {
                let scrutinee_key = kv_scrutinee.key()?;
                let mut kv_scrutinee_components = scrutinee_key.path();
                let kv_scrutinee_first_component = kv_scrutinee_components.next()?;
                let kv_scrutinee_ident = Ident::cast(kv_scrutinee_first_component)?;
                if kv_scrutinee_ident.as_str() == first_component_ident.as_str() {
                    Some((
                            kv_scrutinee.key()?.node().text_range(),
                            kv_scrutinee_components
                                .map(|n| n.to_string())
                                .collect::<Vec<_>>()
                                .join("."),
                    ))
                } else {
                    None
                }
            }).collect::<Vec<_>>()
            && occurrences.first()?.0 == key.node().text_range()
            && occurrences.len() >= 3
        {
            let mut iter = occurrences.into_iter();

            let (first_annotation, first_subkey) = iter.next().unwrap();
            let first_message = format!(
                "The key `{}` is first assigned here ...",
                first_component_ident.as_str()
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
                    first_component_ident.as_str(),
                    first_subkey,
                    second_subkey,
                    third_subkey
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
