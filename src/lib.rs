#![deny(unsafe_code)]

use handlebars::Handlebars;
use snafu::Snafu;
use std::error::Error;

pub mod assign_helpers;
pub mod env_helpers;
#[cfg(feature = "http")]
pub mod http_helpers;
#[cfg(feature = "json")]
pub mod json_helpers;
pub mod path_helpers;
#[cfg(feature = "string")]
pub mod string_helpers;

#[derive(Debug, Snafu)]
enum HelperError {
    #[snafu(display("missing param {} '{}' of '{}'", position, name, helper_signature))]
    MissingParameter {
        position: usize,
        name: String,
        helper_signature: String,
    },
}

pub fn new_hbs() -> Result<Handlebars, Box<Error>> {
    let mut handlebars = Handlebars::new();
    setup_handlebars(&mut handlebars)?;
    Ok(handlebars)
}

pub fn setup_handlebars(handlebars: &mut Handlebars) -> Result<(), Box<Error>> {
    handlebars.set_strict_mode(true);
    register_all(handlebars)
}

pub fn register_all(handlebars: &mut Handlebars) -> Result<(), Box<Error>> {
    #[cfg(feature = "string")]
    string_helpers::register(handlebars)?;
    #[cfg(feature = "http")]
    http_helpers::register(handlebars)?;
    path_helpers::register(handlebars)?;
    env_helpers::register(handlebars)?;
    #[cfg(feature = "json")]
    json_helpers::register(handlebars)?;
    assign_helpers::register(handlebars)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
    use std::collections::HashMap;

    pub(crate) fn assert_helpers(
        input: &str,
        helper_expected: Vec<(&str, &str)>,
    ) -> Result<(), Box<Error>> {
        let mut vs: HashMap<String, String> = HashMap::new();
        vs.insert("var".into(), input.into());
        let hbs = new_hbs()?;
        for sample in helper_expected {
            let tmpl = format!("{{{{ {} var}}}}", sample.0);
            assert_that!(hbs.render_template(&tmpl, &vs)?)
                .named(sample.0)
                .is_equal_to(sample.1.to_owned());
        }
        Ok(())
    }

    pub(crate) fn assert_renders(samples_expected: Vec<(&str, &str)>) -> Result<(), Box<Error>> {
        let vs: HashMap<String, String> = HashMap::new();
        let hbs = new_hbs()?;
        for sample in samples_expected {
            let tmpl = sample.0;
            assert_that!(hbs.render_template(&tmpl, &vs)?)
                .named(sample.0)
                .is_equal_to(sample.1.to_owned());
        }
        Ok(())
    }

    #[test]
    #[cfg(feature = "string")]
    fn test_chain_of_helpers_with_1_param() -> Result<(), Box<Error>> {
        let vs: HashMap<String, String> = HashMap::new();
        let hbs = new_hbs()?;
        let tmpl = r#"{{ to_upper_case (to_singular "Hello foo-bars")}}"#.to_owned();
        let actual = hbs.render_template(&tmpl, &vs)?;
        assert_that!(&actual).is_equal_to("BAR".to_string());
        Ok(())
    }
}
