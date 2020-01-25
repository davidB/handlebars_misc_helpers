use attohttpc;
use handlebars::HelperDef;
use handlebars::RenderError;
use handlebars::{handlebars_helper, Handlebars};

fn http_get_fct<T: AsRef<str>>(url: T) -> Result<String, attohttpc::Error> {
    attohttpc::get(url.as_ref()).send()?.text()
}

pub fn register<'reg>(handlebars: &mut Handlebars<'reg>) -> Vec<Box<dyn HelperDef + 'reg>> {
    vec![{
            handlebars_helper!(http_get: |v: str| http_get_fct(&v).map_err(RenderError::with)?);
            handlebars.register_helper("http_get", Box::new(http_get))
        },{
            handlebars_helper!(gitignore_io: |v: str| http_get_fct(format!("https://www.gitignore.io/api/{}", v)).map_err(RenderError::with)?);
            handlebars.register_helper("gitignore_io", Box::new(gitignore_io))
        }]
        .into_iter()
        .flatten()
        .collect()
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     // use crate::tests::assert_renders;
//     use spectral::prelude::*;
//     use std::error::Error;

//     #[test]
//     fn try_http_get_fct() -> Result<(), Box<dyn Error>> {
//         assert_that!(http_get_fct("https://www.gitignore.io/api/text")?).is_equal_to(r#"
// # Created by https://www.gitignore.io/api/text
// # Edit at https://www.gitignore.io/?templates=text

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

// # End of https://www.gitignore.io/api/text
// "#.to_string());
//         Ok(())
//     }
// }
