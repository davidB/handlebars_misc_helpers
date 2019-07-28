use crate::HelperError::MissingParameter;
use handlebars::Context;
use handlebars::Handlebars;
use handlebars::Helper;
use handlebars::HelperDef;
use handlebars::HelperResult;
use handlebars::Output;
use handlebars::RenderContext;
use handlebars::RenderError;
use jmespath;
use jmespath::ToJmespath;
use serde::Serialize;
use serde_json;
use serde_json::Value;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum JsonError {
    JsonQueryError {
        expression: String,
        source: jmespath::JmespathError,
    },
    ToStringError {
        source: serde_json::error::Error,
    },
}

fn json_query<T: Serialize, E: AsRef<str>>(expr: E, data_str: T) -> Result<String, JsonError> {
    let data = data_str.to_jmespath();
    let res = jmespath::compile(expr.as_ref())
        .and_then(|e| e.search(data))
        .context(JsonQueryError {
            expression: expr.as_ref().to_string(),
        })?;
    serde_json::to_string(res.as_ref()).context(ToStringError {})
}

fn json_query_fct(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut Output,
) -> HelperResult {
    // get parameter from helper or throw an error
    let expr_param = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .ok_or(RenderError::with(MissingParameter {
            position: 0,
            name: "expr".to_owned(),
            helper_signature: "json_query expr data".to_owned(),
        }))?;
    let data_param = h
        .param(1)
        .and_then(|v| v.value().as_str())
        .ok_or(RenderError::with(MissingParameter {
            position: 1,
            name: "data".to_owned(),
            helper_signature: "json_query expr data".to_owned(),
        }))?;
    let json_str: Value = serde_json::from_str(data_param).map_err(|e| RenderError::with(e))?; //new("failed to parse 'data' into json"))?;
    let result = json_query(expr_param, json_str).map_err(|e| RenderError::with(e))?;
    out.write(&result)?;
    Ok(())
}

pub fn register(handlebars: &mut Handlebars) -> Vec<Box<dyn HelperDef + 'static>> {
    vec![{ handlebars.register_helper("json_query", Box::new(json_query_fct)) }]
        .into_iter()
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::assert_renders;
    use spectral::prelude::*;
    use std::error::Error;

    #[test]
    fn test_search_object_field() -> Result<(), Box<Error>> {
        let json_str: Value = serde_json::from_str(r##"{"foo":{"bar":{"baz":true}}}"##)?;
        let result = json_query("foo.bar.baz", json_str)?;
        assert_that!(result).is_equal_to("true".to_owned());
        Ok(())
    }

    #[test]
    fn test_search_path_in_empty() -> Result<(), Box<Error>> {
        for v in vec!["{}", "[]", "null", "\"\""] {
            let json_str: Value = serde_json::from_str(v)?;
            let result = json_query("foo.bar.baz", json_str)?;
            assert_that!(result).is_equal_to("null".to_owned());
        }
        Ok(())
    }

    #[test]
    fn test_helper_json_query() -> Result<(), Box<Error>> {
        assert_renders(vec![
            (r##"{{ json_query "foo" "{}" }}"##, r##"null"##),
            (
                r##"{{ json_query "foo" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}"##,
                r##"{"bar":{"baz":true}}"##,
            ),
            (
                r##"{{ json_query "foo.bar.baz" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}"##,
                "true",
            ),
        ])?;
        Ok(())
    }
}
