use crate::HelperError::MissingParameter;
use handlebars::Context;
use handlebars::Handlebars;
use handlebars::Helper;
use handlebars::HelperResult;
use handlebars::Output;
use handlebars::RenderContext;
use handlebars::RenderError;
use std::error::Error;

fn assign_fct(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    rc: &mut RenderContext,
    _: &mut Output,
) -> HelperResult {
    // get parameter from helper or throw an error
    let name = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .ok_or(RenderError::with(MissingParameter {
            position: 0,
            name: "var_name".to_owned(),
            helper_signature: "assign var_name value".to_owned(),
        }))?;
    let value = h
        .param(1)
        .map(|v| v.value())
        .cloned()
        .ok_or(RenderError::with(MissingParameter {
            position: 1,
            name: "value".to_owned(),
            helper_signature: "assign var_name value".to_owned(),
        }))?;
    rc.set_local_var(name.to_owned(), value);
    Ok(())
}

pub fn register(handlebars: &mut Handlebars) -> Result<(), Box<Error>> {
    handlebars.register_helper("assign", Box::new(assign_fct));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::assert_renders;

    #[test]
    fn test_helper_assign() -> Result<(), Box<Error>> {
        assert_renders(vec![
            (r##"{{ assign "foo" "{}" }}"##, r##""##),
            (r##"{{ assign "foo" "{}" }}{{ foo }}"##, r##"{}"##),
            (r##"{{ assign "foo" {} }}{{ foo }}"##, r##"[object]"##),
            (
                r##"{{ assign "foo" {"bar": 33} }}{{ foo }}"##,
                r##"[object]"##,
            ),
            (
                r##"{{ assign "foo" "hello world" }}{{ foo }}"##,
                r##"hello world"##,
            ),
        ])?;
        Ok(())
    }
}
