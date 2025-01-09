mod bool_comparison;
mod bool_simplification;
mod collapsible_let_in;
mod deprecated_to_path;
mod empty_inherit;
mod empty_let_in;
mod empty_list_concat;
mod empty_pattern;
mod eta_reduction;
mod faster_groupby;
mod faster_zipattrswith;
mod legacy_let_syntax;
mod manual_inherit;
mod manual_inherit_from;
mod redundant_pattern_bind;
mod repeated_keys;
mod unquoted_uri;
mod useless_has_attr;
mod useless_parens;

pub static LINTS: std::sync::LazyLock<Vec<std::sync::Arc<Box<dyn crate::Lint>>>> =
    std::sync::LazyLock::new(|| {
        [
            bool_comparison::BoolComparison::new() as Box<dyn crate::Lint>,
            bool_simplification::BoolSimplification::new() as Box<dyn crate::Lint>,
            collapsible_let_in::CollapsibleLetIn::new() as Box<dyn crate::Lint>,
            deprecated_to_path::DeprecatedIsNull::new() as Box<dyn crate::Lint>,
            empty_inherit::EmptyInherit::new() as Box<dyn crate::Lint>,
            empty_let_in::EmptyLetIn::new() as Box<dyn crate::Lint>,
            empty_list_concat::EmptyListConcat::new() as Box<dyn crate::Lint>,
            empty_pattern::EmptyPattern::new() as Box<dyn crate::Lint>,
            eta_reduction::EtaReduction::new() as Box<dyn crate::Lint>,
            faster_groupby::FasterGroupBy::new() as Box<dyn crate::Lint>,
            faster_zipattrswith::FasterZipAttrsWith::new() as Box<dyn crate::Lint>,
            legacy_let_syntax::LegacyLetSyntax::new() as Box<dyn crate::Lint>,
            manual_inherit::ManualInherit::new() as Box<dyn crate::Lint>,
            manual_inherit_from::ManualInheritFrom::new() as Box<dyn crate::Lint>,
            redundant_pattern_bind::RedundantPatternBind::new() as Box<dyn crate::Lint>,
            repeated_keys::RepeatedKeys::new() as Box<dyn crate::Lint>,
            unquoted_uri::UnquotedUri::new() as Box<dyn crate::Lint>,
            useless_has_attr::UselessHasAttr::new() as Box<dyn crate::Lint>,
            useless_parens::UselessParens::new() as Box<dyn crate::Lint>,
        ]
        .map(std::sync::Arc::new)
        .to_vec()
    });
