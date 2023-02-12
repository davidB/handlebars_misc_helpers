#[cfg(feature = "http_attohttpc")]
use attohttpc;
use handlebars::RenderError;
use handlebars::{handlebars_helper, Handlebars};
#[cfg(all(feature = "http_reqwest", not(feature = "http_attohttpc")))]
use reqwest;

#[cfg(feature = "http_attohttpc")]
fn http_get_fct<T: AsRef<str>>(url: T) -> Result<String, attohttpc::Error> {
    attohttpc::get(url.as_ref()).send()?.text()
}

#[cfg(all(feature = "http_reqwest", not(feature = "http_attohttpc")))]
fn http_get_fct<T: AsRef<str>>(url: T) -> Result<String, reqwest::Error> {
    reqwest::blocking::get(url.as_ref())?.text()
}

pub fn register(handlebars: &mut Handlebars) {
    {
        handlebars_helper!(http_get: |v: str| http_get_fct(v).map_err(|e| RenderError::from_error("http_get", e))?);
        handlebars.register_helper("http_get", Box::new(http_get))
    }
    {
        handlebars_helper!(gitignore_io: |v: str| http_get_fct(format!("https://www.gitignore.io/api/{}", v)).map_err(|e|RenderError::from_error("http_get", e))?);
        handlebars.register_helper("gitignore_io", Box::new(gitignore_io))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     // use crate::tests::assert_renders;
//     use spectral::prelude::*;
//     use std::error::Error;

//     #[test]
//     fn try_http_get_fct() -> Result<(), Box<dyn Error>> {
//         assert_that!(http_get_fct("https://www.gitignore.io/api/text")?).is_equal_to(
//             r#"
// # Created by https://www.toptal.com/developers/gitignore/api/text
// # Edit at https://www.toptal.com/developers/gitignore?templates=text

// ### Text ###
// *.doc
// *.docx
// *.log
// *.msg
// *.pages
// *.rtf
// *.txt
// *.wpd
// *.wps

// # End of https://www.toptal.com/developers/gitignore/api/text
// "#
//             .to_string(),
//         );
//         Ok(())
//     }
// }
