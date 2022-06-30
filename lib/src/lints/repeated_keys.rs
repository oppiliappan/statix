use crate::{session::SessionInfo, Metadata, Report, Rule};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{AttrSet, EntryHolder, Ident, KeyValue, TokenWrapper, TypedNode},
    NodeOrToken, SyntaxElement, SyntaxKind,
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
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(key_value) = KeyValue::cast(node.clone());
            if let Some(key) = key_value.key();
            if let mut components = key.path();
            if let Some(first_component) = components.next();
            if let Some(first_component_ident) = Ident::cast(first_component);
            // ensure that there are >1 components
            if components.next().is_some();

            if let Some(parent_node) = node.parent();
            if let Some(parent_attr_set) = AttrSet::cast(parent_node.clone());

            if !parent_attr_set.recursive();
            let occurrences = parent_attr_set.entries().filter_map(|kv_scrutinee| {
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
            }).collect::<Vec<_>>();

            if occurrences.first()?.0 == key.node().text_range();
            if occurrences.len() >= 3;

            then {
                let mut iter = occurrences.into_iter();

                let (first_annotation, first_subkey) = iter.next().unwrap();
                let first_message = format!("The key `{}` is first assigned here ...", first_component_ident.as_str());

                let (second_annotation, second_subkey) = iter.next().unwrap();
                let second_message = "... repeated here ...";

                let (third_annotation, third_subkey) = iter.next().unwrap();
                let third_message = {
                    let remaining_occurrences = iter.count();
                    let mut message = match remaining_occurrences {
                        0 => format!("... and here."),
                        1 => format!("... and here (`1` occurrence omitted)."),
                        n => format!("... and here (`{}` occurrences omitted).", n),
                    };
                    message.push_str(&format!(" Try `{} = {{ {}=...; {}=...; {}=...; }}` instead.", first_component_ident.as_str(), first_subkey, second_subkey, third_subkey));
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
}
