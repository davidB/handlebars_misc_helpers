use handlebars::handlebars_helper;
use handlebars::Handlebars;
use handlebars::HelperDef;
use handlebars::RenderError;
use jmespath;
use jmespath::ToJmespath;
use serde::Serialize;
use serde_json;
use serde_json::Value as Json;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum JsonError {
    JsonQueryError {
        expression: String,
        source: jmespath::JmespathError,
    },
    ToJsonValueError {
        source: serde_json::error::Error,
    },
}

fn json_query<T: Serialize, E: AsRef<str>>(expr: E, data: T) -> Result<Json, JsonError> {
    let data = data.to_jmespath();
    let res = jmespath::compile(expr.as_ref())
        .and_then(|e| e.search(data))
        .context(JsonQueryError {
            expression: expr.as_ref().to_string(),
        })?;
    serde_json::to_value(res.as_ref()).context(ToJsonValueError {})
}

handlebars_helper!(str_to_json_fct: |data: str| { let v: Json = serde_json::from_str(data).map_err(RenderError::with)?; v});
handlebars_helper!(json_to_str_fct: |data: Json| serde_json::to_string(data).map_err(RenderError::with)?);
handlebars_helper!(json_query_fct: |expr: str, data: Json| json_query(expr, data).map_err(RenderError::with)?);
handlebars_helper!(json_str_query_fct: |expr: str, data: str| {
    let v: Json = serde_json::from_str(data).map_err(RenderError::with)?;
    json_query(expr, v).map_err(RenderError::with)
    .and_then(|v| serde_json::to_string(&v).map_err(RenderError::with))?
});

pub fn register(handlebars: &mut Handlebars) -> Vec<Box<dyn HelperDef + 'static>> {
    vec![
        { handlebars.register_helper("json_to_str", Box::new(json_to_str_fct)) },
        { handlebars.register_helper("str_to_json", Box::new(str_to_json_fct)) },
        { handlebars.register_helper("json_query", Box::new(json_query_fct)) },
        { handlebars.register_helper("json_str_query", Box::new(json_str_query_fct)) },
    ]
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
    fn test_search_object_field() -> Result<(), Box<dyn Error>> {
        let json: Json = serde_json::from_str(r##"{"foo":{"bar":{"baz":true}}}"##)?;
        let result = json_query("foo.bar.baz", json)?;
        assert_that!(result).is_equal_to(Json::Bool(true));
        Ok(())
    }

    #[test]
    fn test_search_path_in_empty() -> Result<(), Box<dyn Error>> {
        for v in vec!["{}", "[]", "null", "\"\""] {
            let json: Json = serde_json::from_str(v)?;
            let result = json_query("foo.bar.baz", json)?;
            assert_that!(result).is_equal_to(Json::Null);
        }
        Ok(())
    }

    #[test]
    fn test_helper_json_to_str() -> Result<(), Box<dyn Error>> {
        assert_renders(vec![
            (r##"{{ json_to_str {} }}"##, r##"{}"##),
            (
                r##"{{ json_to_str {"foo":{"bar":{"baz":true}}} }}"##,
                r##"{"foo":{"bar":{"baz":true}}}"##,
            ),
            (
                r##"{{ json_to_str ( str_to_json "{\"foo\":{\"bar\":{\"baz\":true}}}" ) }}"##,
                r##"{"foo":{"bar":{"baz":true}}}"##,
            ),
        ])?;
        Ok(())
    }

    #[test]
    fn test_helper_json_query() -> Result<(), Box<dyn Error>> {
        assert_renders(vec![
            (r##"{{ json_query "foo" {} }}"##, r##""##),
            (
                r##"{{ json_to_str ( json_query "foo" {"foo":{"bar":{"baz":true}}} ) }}"##,
                r##"{"bar":{"baz":true}}"##,
            ),
            (
                r##"{{ json_to_str ( json_query "foo" (str_to_json "{\"foo\":{\"bar\":{\"baz\":true}}}" ) ) }}"##,
                r##"{"bar":{"baz":true}}"##,
            ),
        ])?;
        Ok(())
    }

    #[test]
    fn test_helper_json_str_query() -> Result<(), Box<dyn Error>> {
        assert_renders(vec![
            (
                r##"{{ json_str_query "foo" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}"##,
                r##"{"bar":{"baz":true}}"##,
            ),
            (
                r##"{{ json_str_query "foo.bar.baz" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}"##,
                "true",
            ),
            (
                r##"{{ json_str_query "foo.bar.baz" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}"##,
                "true",
            ),
        ])?;
        Ok(())
    }
}
