use std::fmt::Write as _;

use crate::{Metadata, Report, Rule};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{Attr, AttrSet, AttrpathValue, Entry, HasEntry as _},
};
use rowan::ast::AstNode as _;

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
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let attrpath_value = AttrpathValue::cast(node.clone())?;
        let attrpath = attrpath_value.attrpath()?;
        let mut components = attrpath.attrs();
        let first_component = components.next()?;

        let Attr::Ident(first_component_ident) = first_component else {
            return None;
        };

        // ensure that there are >1 components
        components.next()?;

        let parent_node = node.parent()?;
        let parent_attr_set = AttrSet::cast(parent_node)?;

        if parent_attr_set.rec_token().is_some() {
            return None;
        }

        let occurrences = parent_attr_set
            .entries()
            .filter_map(|kv_scrutinee| {
                let Entry::AttrpathValue(kv_scrutinee) = kv_scrutinee else {
                    return None;
                };

                let scrutinee_key = kv_scrutinee.attrpath()?;
                let mut kv_scrutinee_components = scrutinee_key.attrs();
                let kv_scrutinee_first_component = kv_scrutinee_components.next()?;

                let Attr::Ident(kv_scrutinee_ident) = kv_scrutinee_first_component else {
                    return None;
                };

                if kv_scrutinee_ident.to_string() != first_component_ident.to_string() {
                    return None;
                }

                Some((
                    scrutinee_key.syntax().text_range(),
                    kv_scrutinee_components
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join("."),
                ))
            })
            .collect::<Vec<_>>();

        if occurrences.first()?.0 != attrpath.syntax().text_range() {
            return None;
        }

        if occurrences.len() < 3 {
            return None;
        }

        let mut iter = occurrences.into_iter();

        let (first_annotation, first_subkey) = iter.next().unwrap();
        let first_message = format!("The key `{first_component_ident}` is first assigned here ...");

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
            write!(
                message,
                " Try `{first_component_ident} = {{ {first_subkey}=...; {second_subkey}=...; {third_subkey}=...; }}` instead."
            )
            .unwrap();
            message
        };

        Some(
            self.report()
                .diagnostic(first_annotation, first_message)
                .diagnostic(second_annotation, second_message)
                .diagnostic(third_annotation, third_message),
        )
    }
}
