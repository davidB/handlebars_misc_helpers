use handlebars::HelperDef;
use handlebars::{handlebars_helper, Handlebars};
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

pub fn register(handlebars: &mut Handlebars) -> Vec<Box<dyn HelperDef + 'static>> {
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
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use crate::tests::assert_helpers;
    use crate::tests::assert_renders;
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
        assert_renders(vec![
            (r##"{{ trim "foo" }}"##, r##"foo"##),
            (r##"{{ trim "  foo" }}"##, r##"foo"##),
            (r##"{{ trim "foo  " }}"##, r##"foo"##),
            (r##"{{ trim " foo " }}"##, r##"foo"##),
        ])?;
        Ok(())
    }

    #[test]
    fn test_helper_trim_start() -> Result<(), Box<dyn Error>> {
        assert_renders(vec![
            (r##"{{ trim_start "foo" }}"##, r##"foo"##),
            (r##"{{ trim_start "  foo" }}"##, r##"foo"##),
            (r##"{{ trim_start "foo  " }}"##, r##"foo  "##),
            (r##"{{ trim_start " foo " }}"##, r##"foo "##),
        ])?;
        Ok(())
    }

    #[test]
    fn test_helper_trim_end() -> Result<(), Box<dyn Error>> {
        assert_renders(vec![
            (r##"{{ trim_end "foo" }}"##, r##"foo"##),
            (r##"{{ trim_end "  foo" }}"##, r##"  foo"##),
            (r##"{{ trim_end "foo  " }}"##, r##"foo"##),
            (r##"{{ trim_end " foo " }}"##, r##" foo"##),
        ])?;
        Ok(())
    }

    #[test]
    fn test_helper_replace() -> Result<(), Box<dyn Error>> {
        assert_renders(vec![(r##"{{ replace "foo" "oo" "aa"}}"##, r##"faa"##)])?;
        Ok(())
    }
}
