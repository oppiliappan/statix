use std::fmt::Write as _;

use crate::{Metadata, Report, Rule, Suggestion, make};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, TextRange,
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

struct Occurrence {
    attrpath_range: TextRange,
    entry_range: TextRange,
    subkey: String,
    nested_entry_text: String,
}

fn relative_range(parent_start: usize, range: TextRange) -> (usize, usize) {
    (
        usize::from(range.start()) - parent_start,
        usize::from(range.end()) - parent_start,
    )
}

fn find_entry_indent(parent_text: &str, first_entry_start: usize) -> &str {
    let line_start = parent_text[..first_entry_start]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    &parent_text[line_start..first_entry_start]
}

fn indent_nested_entry(text: &str, entry_indent: &str) -> String {
    let mut lines = text.lines();
    let first_line = lines.next().unwrap_or_default();
    let nested_indent = format!("{entry_indent}  ");

    let mut indented = format!("{nested_indent}{first_line}");
    for line in lines {
        indented.push('\n');
        indented.push_str(entry_indent);
        indented.push_str(line);
    }

    indented
}

fn build_grouped_entry(
    first_component: &str,
    occurrences: &[Occurrence],
    entry_indent: &str,
) -> String {
    let nested_entries = occurrences
        .iter()
        .map(|occurrence| indent_nested_entry(&occurrence.nested_entry_text, entry_indent))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{first_component} = {{\n{nested_entries}\n{entry_indent}}};")
}

fn collect_occurrences(parent_attr_set: &AttrSet, first_component: &str) -> Vec<Occurrence> {
    parent_attr_set
        .entries()
        .filter_map(|entry| {
            let Entry::AttrpathValue(attrpath_value) = entry else {
                return None;
            };

            let attrpath = attrpath_value.attrpath()?;
            let mut components = attrpath.attrs();
            let first_attr = components.next()?;

            let Attr::Ident(ident) = first_attr else {
                return None;
            };

            if ident.to_string() != first_component {
                return None;
            }

            let entry_text = attrpath_value.syntax().to_string();
            let nested_entry_text = entry_text
                .strip_prefix(&format!("{first_component}."))
                .map(str::to_owned)?;

            Some(Occurrence {
                attrpath_range: attrpath.syntax().text_range(),
                entry_range: attrpath_value.syntax().text_range(),
                subkey: components
                    .map(|component| component.to_string())
                    .collect::<Vec<_>>()
                    .join("."),
                nested_entry_text,
            })
        })
        .collect()
}

fn build_third_message(
    first_component: &str,
    first_subkey: &str,
    second_subkey: &str,
    third_subkey: &str,
    remaining_occurrences: usize,
) -> String {
    let mut message = match remaining_occurrences {
        0 => "... and here.".to_string(),
        1 => "... and here (`1` occurrence omitted).".to_string(),
        n => format!("... and here (`{n}` occurrences omitted)."),
    };
    write!(
        message,
        " Try `{first_component} = {{ {first_subkey}=...; {second_subkey}=...; {third_subkey}=...; }}` instead."
    )
    .unwrap();
    message
}

fn build_fix(
    parent_attr_set: &AttrSet,
    first_component: &str,
    occurrences: &[Occurrence],
) -> Option<Suggestion> {
    let parent_range = parent_attr_set.syntax().text_range();
    let first_entry_range = occurrences.first()?.entry_range;
    let last_entry_range = occurrences.last()?.entry_range;
    let parent_text = parent_attr_set.syntax().to_string();
    let parent_start = usize::from(parent_range.start());
    let first_entry_start = relative_range(parent_start, first_entry_range).0;
    let entry_indent = find_entry_indent(&parent_text, first_entry_start);
    let grouped_entry = build_grouped_entry(first_component, occurrences, entry_indent);

    let rewritten_middle = {
        let mut rewritten = String::new();
        let mut cursor = first_entry_start;

        for entry in parent_attr_set.entries() {
            let entry_range = entry.syntax().text_range();
            if entry_range.end() <= first_entry_range.start()
                || entry_range.start() >= last_entry_range.end()
            {
                continue;
            }

            let (entry_start, entry_end) = relative_range(parent_start, entry_range);
            if entry_range == first_entry_range {
                rewritten.push_str(&parent_text[cursor..entry_start]);
                rewritten.push_str(&grouped_entry);
            } else if occurrences
                .iter()
                .any(|occurrence| occurrence.entry_range == entry_range)
            {
                rewritten.push_str(
                    parent_text[cursor..entry_start].trim_end_matches(char::is_whitespace),
                );
            } else {
                rewritten.push_str(&parent_text[cursor..entry_start]);
                rewritten.push_str(&parent_text[entry_start..entry_end]);
            }

            cursor = entry_end;
        }

        let last_entry_end = relative_range(parent_start, last_entry_range).1;
        rewritten.push_str(&parent_text[cursor..last_entry_end]);
        rewritten
    };

    let last_entry_end = relative_range(parent_start, last_entry_range).1;
    let rewritten_parent = format!(
        "{}{}{}",
        &parent_text[..first_entry_start],
        rewritten_middle,
        &parent_text[last_entry_end..]
    );
    let replacement = make::attrset_from_text(&rewritten_parent);

    Some(Suggestion::with_replacement(
        parent_range,
        replacement.syntax().clone(),
    ))
}

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

        let first_component_ident_text = first_component_ident.to_string();
        let occurrences = collect_occurrences(&parent_attr_set, &first_component_ident_text);

        if occurrences.first()?.attrpath_range != attrpath.syntax().text_range() {
            return None;
        }

        if occurrences.len() < 3 {
            return None;
        }

        let fix = build_fix(&parent_attr_set, &first_component_ident_text, &occurrences);
        let mut iter = occurrences.into_iter();

        let Occurrence {
            attrpath_range: first_annotation,
            subkey: first_subkey,
            ..
        } = iter.next().unwrap();
        let first_message = format!("The key `{first_component_ident}` is first assigned here ...");

        let Occurrence {
            attrpath_range: second_annotation,
            subkey: second_subkey,
            ..
        } = iter.next().unwrap();
        let second_message = "... repeated here ...";

        let Occurrence {
            attrpath_range: third_annotation,
            subkey: third_subkey,
            ..
        } = iter.next().unwrap();
        let third_message = build_third_message(
            &first_component_ident_text,
            &first_subkey,
            &second_subkey,
            &third_subkey,
            iter.count(),
        );

        let report = if let Some(fix) = fix {
            self.report().suggest(first_annotation, first_message, fix)
        } else {
            self.report().diagnostic(first_annotation, first_message)
        };

        Some(
            report
                .diagnostic(second_annotation, second_message)
                .diagnostic(third_annotation, third_message),
        )
    }
}
