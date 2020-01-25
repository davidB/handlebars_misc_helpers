use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperDef, RenderContext, RenderError,
    ScopedJson,
};
use inflector::Inflector;

#[macro_export]
macro_rules! handlebars_register_inflector {
    ($engine:ident, $fct_name:ident) => {
        {
            handlebars_helper!($fct_name: |v: str| v.$fct_name());
            $engine.register_helper(stringify!($fct_name), Box::new($fct_name))
        }
    }
}

#[allow(non_camel_case_types)]
pub struct first_non_empty_fct;

impl HelperDef for first_non_empty_fct {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let params = h.params();
        Ok(params
            .iter()
            .filter_map(|p| p.value().as_str().filter(|s| !s.is_empty()))
            // .filter_map(|p| {
            //     serde_json::to_string(p.value())
            //         .ok()
            //         .filter(|s| !s.is_empty())
            // })
            .nth(0)
            .map(|v| ScopedJson::Derived(serde_json::Value::String(v.to_owned()))))
    }
}

pub fn register<'reg>(handlebars: &mut Handlebars<'reg>) -> Vec<Box<dyn HelperDef + 'reg>> {
    vec![
        {
            handlebars_helper!(to_lower_case: |v: str| v.to_lowercase());
            handlebars.register_helper("to_lower_case", Box::new(to_lower_case))
        },
        {
            handlebars_helper!(to_upper_case: |v: str| v.to_uppercase());
            handlebars.register_helper("to_upper_case", Box::new(to_upper_case))
        },
        {
            handlebars_helper!(trim: |v: str| v.trim());
            handlebars.register_helper("trim", Box::new(trim))
        },
        {
            handlebars_helper!(trim_start: |v: str| v.trim_start());
            handlebars.register_helper("trim_start", Box::new(trim_start))
        },
        {
            handlebars_helper!(trim_end: |v: str| v.trim_end());
            handlebars.register_helper("trim_end", Box::new(trim_end))
        },
        {
            handlebars_helper!(replace: |v: str, from: str, to: str| v.replace(from, to));
            handlebars.register_helper("replace", Box::new(replace))
        },
        handlebars_register_inflector!(handlebars, to_camel_case),
        handlebars_register_inflector!(handlebars, to_pascal_case),
        handlebars_register_inflector!(handlebars, to_snake_case),
        handlebars_register_inflector!(handlebars, to_screaming_snake_case),
        handlebars_register_inflector!(handlebars, to_kebab_case),
        handlebars_register_inflector!(handlebars, to_train_case),
        handlebars_register_inflector!(handlebars, to_sentence_case),
        handlebars_register_inflector!(handlebars, to_title_case),
        handlebars_register_inflector!(handlebars, to_class_case),
        handlebars_register_inflector!(handlebars, to_table_case),
        handlebars_register_inflector!(handlebars, to_plural),
        handlebars_register_inflector!(handlebars, to_singular),
        {
            handlebars_helper!(quote: |quote_symbol: str, v: str| enquote::enquote(quote_symbol.chars().next().unwrap_or('"'), &v));
            handlebars.register_helper("quote", Box::new(quote))
        },
        {
            handlebars_helper!(unquote: |v: str| match enquote::unquote(&v){
                Err(e) => {
                    log::warn!(
                        "helper: unquote failed for string '{:?}' with error '{:?}'",
                        v, e
                    );
                    v.to_owned()
                }
                Ok(s) => s,
            });
            handlebars.register_helper("unquote", Box::new(unquote))
        },
        handlebars.register_helper("first_non_empty", Box::new(first_non_empty_fct)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use crate::assert_renders;
    use crate::tests::assert_helpers;
    use std::error::Error;

    #[test]
    fn test_register_string_helpers() -> Result<(), Box<dyn Error>> {
        assert_helpers(
            "Hello foo-bars",
            vec![
                ("to_lower_case", "hello foo-bars"),
                ("to_upper_case", "HELLO FOO-BARS"),
                ("to_camel_case", "helloFooBars"),
                ("to_pascal_case", "HelloFooBars"),
                ("to_snake_case", "hello_foo_bars"),
                ("to_screaming_snake_case", "HELLO_FOO_BARS"),
                ("to_kebab_case", "hello-foo-bars"),
                ("to_train_case", "Hello-Foo-Bars"),
                ("to_sentence_case", "Hello foo bars"),
                ("to_title_case", "Hello Foo Bars"),
                ("to_class_case", "HelloFooBar"),
                ("to_table_case", "hello_foo_bars"),
                ("to_plural", "bars"),
                ("to_singular", "bar"),
            ],
        )?;
        Ok(())
    }

    #[test]
    fn test_helper_trim() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ trim "foo" }}"##, r##"foo"##),
            (r##"{{ trim "  foo" }}"##, r##"foo"##),
            (r##"{{ trim "foo  " }}"##, r##"foo"##),
            (r##"{{ trim " foo " }}"##, r##"foo"##)
        ]
    }

    #[test]
    fn test_helper_trim_start() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ trim_start "foo" }}"##, r##"foo"##),
            (r##"{{ trim_start "  foo" }}"##, r##"foo"##),
            (r##"{{ trim_start "foo  " }}"##, r##"foo  "##),
            (r##"{{ trim_start " foo " }}"##, r##"foo "##)
        ]
    }

    #[test]
    fn test_helper_trim_end() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ trim_end "foo" }}"##, r##"foo"##),
            (r##"{{ trim_end "  foo" }}"##, r##"  foo"##),
            (r##"{{ trim_end "foo  " }}"##, r##"foo"##),
            (r##"{{ trim_end " foo " }}"##, r##" foo"##)
        ]
    }

    #[test]
    fn test_helper_quote() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ quote "'" "''" }}"##, r##"'\'\''"##),
            (r##"{{ quote "'" "foo" }}"##, r##"'foo'"##),
            (r##"{{ quote "\"" "foo" }}"##, r##""foo""##),
            (r##"{{ quote "" "foo" }}"##, r##""foo""##),
        ]
    }

    #[test]
    fn test_helper_unquote() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ unquote "''" }}"##, r##""##),
            (r##"{{ unquote "'f'" }}"##, r##"f"##),
            (r##"{{ unquote "foo" }}"##, r##"foo"##),
            (r##"{{ unquote "'foo'" }}"##, r##"foo"##),
            (r##"{{ unquote "\"foo\"" }}"##, r##"foo"##),
            (r##"{{ unquote "foo'" }}"##, r##"foo'"##),
            (r##"{{ unquote "'foo" }}"##, r##"'foo"##),
        ]
    }

    #[test]
    fn test_helper_replace() -> Result<(), Box<dyn Error>> {
        assert_renders![(r##"{{ replace "foo" "oo" "aa"}}"##, r##"faa"##)]
    }

    #[test]
    fn test_helper_first_non_empty() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ first_non_empty ""}}"##, r##""##),
            (r##"{{ first_non_empty "foo"}}"##, r##"foo"##),
            (r##"{{ first_non_empty "foo" "bar"}}"##, r##"foo"##),
            (r##"{{ first_non_empty "" "foo"}}"##, r##"foo"##),
            (r##"{{ first_non_empty "" "foo" "bar"}}"##, r##"foo"##),
            (r##"{{ first_non_empty "" null}}"##, r##""##),
            (r##"{{ first_non_empty "" null 33}}"##, r##""##),
            (r##"{{ first_non_empty "" null "foo" "bar"}}"##, r##"foo"##),
        ]
    }
}
