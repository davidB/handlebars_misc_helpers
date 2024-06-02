#![deny(unsafe_code)]
//#![doc(html_root_url = "https://docs.rs/tower-service/0.3.0")]
// #![warn(
//     missing_debug_implementations,
//     missing_docs,
//     rust_2018_idioms,
//     unreachable_pub
// )]

use handlebars::no_escape;
use handlebars::Handlebars;

#[cfg(feature = "jsontype")]
pub mod assign_helpers;
pub mod env_helpers;
pub mod file_helpers;
#[cfg(any(feature = "http_reqwest", feature = "http_attohttpc"))]
pub mod http_helpers;
#[cfg(feature = "json")]
pub mod json_helpers;
#[cfg(feature = "jsonnet")]
pub mod jsonnet_helpers;
pub mod outputs;
pub mod path_helpers;
#[cfg(feature = "regex")]
pub mod regex_helpers;
#[cfg(feature = "jsontype")]
pub mod region_helpers;
#[cfg(feature = "string")]
pub mod string_helpers;
#[cfg(feature = "uuid")]
pub mod uuid_helpers;

pub fn new_hbs<'reg>() -> Handlebars<'reg> {
    let mut handlebars = Handlebars::new();
    setup_handlebars(&mut handlebars);
    handlebars
}

pub fn setup_handlebars(handlebars: &mut Handlebars) {
    handlebars.set_strict_mode(true);
    handlebars.register_escape_fn(no_escape); //html escaping is the default and cause issue
    register(handlebars);
}

pub fn register(handlebars: &mut Handlebars) {
    #[cfg(feature = "string")]
    string_helpers::register(handlebars);
    #[cfg(any(feature = "http_reqwest", feature = "http_attohttpc"))]
    http_helpers::register(handlebars);
    path_helpers::register(handlebars);
    env_helpers::register(handlebars);
    #[cfg(feature = "json")]
    json_helpers::register(handlebars);
    #[cfg(feature = "jsonnet")]
    jsonnet_helpers::register(handlebars);
    #[cfg(feature = "jsontype")]
    assign_helpers::register(handlebars);
    file_helpers::register(handlebars);
    #[cfg(feature = "jsontype")]
    region_helpers::register(handlebars);
    #[cfg(feature = "regex")]
    regex_helpers::register(handlebars);
    #[cfg(feature = "uuid")]
    uuid_helpers::register(handlebars);
}

#[allow(dead_code)]
pub(crate) fn to_nested_error<E>(cause: E) -> handlebars::RenderError
where
    E: std::error::Error + Send + Sync + 'static,
{
    handlebars::RenderErrorReason::NestedError(Box::new(cause)).into()
}

#[allow(dead_code)]
pub fn to_other_error<T: AsRef<str>>(desc: T) -> handlebars::RenderError {
    handlebars::RenderErrorReason::Other(desc.as_ref().to_string()).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use std::error::Error;
    use unindent::unindent;

    pub(crate) fn assert_helpers(
        input: &str,
        helper_expected: Vec<(&str, &str)>,
    ) -> Result<(), Box<dyn Error>> {
        let mut vs: HashMap<String, String> = HashMap::new();
        vs.insert("var".into(), input.into());
        let hbs = new_hbs();
        for sample in helper_expected {
            let tmpl = format!("{{{{ {} var}}}}", sample.0);
            assert_eq!(
                hbs.render_template(&tmpl, &vs)?,
                sample.1.to_owned(),
                "name: {}",
                sample.0
            );
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn normalize_nl(s: &str) -> String {
        unindent(s).replace("\r\n", "\n").replace('\r', "")
    }

    #[macro_export]
    macro_rules! assert_renders {
        ($($arg:expr),+$(,)?) => {{
            use std::collections::HashMap;
            use pretty_assertions::assert_eq;
            let vs: HashMap<String, String> = HashMap::new();
            let mut hbs = $crate::new_hbs();
            $({
                let sample: (&str, &str) = $arg;
                hbs.register_template_string(&sample.0, &sample.0).expect("register_template_string");
                // assert_that!(hbs.render(&sample.0, &vs).expect("render"))
                //     .named(sample.0)
                //     .is_equal_to(sample.1.to_owned());
                assert_eq!(hbs.render(&sample.0, &vs).expect("render"), sample.1.to_owned());
            })*
            Ok(())
        }}
    }

    // pub(crate) fn assert_renders(
    //     samples_expected: Vec<(&str, &str)>,
    // ) -> Result<(), Box<dyn Error>> {
    //     let vs: HashMap<String, String> = HashMap::new();
    //     let hbs = new_hbs();
    //     for sample in samples_expected {
    //         let tmpl = sample.0;
    //         assert_that!(hbs.render_template(&tmpl, &vs)?)
    //             .named(sample.0)
    //             .is_equal_to(sample.1.to_owned());
    //     }
    //     Ok(())
    // }

    #[test]
    #[cfg(feature = "string")]
    fn test_chain_of_helpers_with_1_param() -> Result<(), Box<dyn Error>> {
        let vs: HashMap<String, String> = HashMap::new();
        let hbs = new_hbs();
        let tmpl = r#"{{ to_upper_case (to_singular "Hello foo-bars")}}"#.to_owned();
        let actual = hbs.render_template(&tmpl, &vs)?;
        assert_eq!(&actual, "BAR");
        Ok(())
    }

    #[test]
    #[cfg(all(feature = "string", feature = "json"))]
    fn test_chain_of_default() -> Result<(), Box<dyn Error>> {
        std::env::set_var("MY_VERSION_1", "1.0.0");
        assert_renders![
            (
                r##"{{ first_non_empty (env_var "MY_VERSION") "0.0.0" }}"##,
                r##"0.0.0"##
            ),
            (
                r##"{{ first_non_empty (env_var "MY_VERSION_1") "0.0.0" }}"##,
                r##"1.0.0"##
            ),
            (
                r##"{{ first_non_empty (unquote (json_str_query "package.edition" (read_to_str "Cargo.toml") format="toml")) (env_var "MY_VERSION") "0.0.0" }}"##,
                r##"2021"##
            ),
        ]
    }
}
