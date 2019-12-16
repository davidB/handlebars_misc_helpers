use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperDef, RenderContext, RenderError,
    ScopedJson,
};
use jmespath;
use jmespath::ToJmespath;
use serde::Serialize;
use serde_json;
use serde_json::Value as Json;
use serde_yaml;
use snafu::{ResultExt, Snafu};
use std::str::FromStr;
use toml;

#[derive(Debug, Snafu)]
enum JsonError {
    JsonQueryError {
        expression: String,
        source: jmespath::JmespathError,
    },
    ToJsonValueError {
        input: String,
        source: serde_json::error::Error,
    },
    DataFormatUnknownError {
        format: String,
    },
}

#[derive(Debug, Clone)]
enum DataFormat {
    Json,
    JsonPretty,
    Yaml,
    Toml,
    TomlPretty,
}

impl FromStr for DataFormat {
    type Err = JsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "json_pretty" => Ok(Self::JsonPretty),
            "yaml" => Ok(Self::Yaml),
            "toml" => Ok(Self::Toml),
            "toml_pretty" => Ok(Self::TomlPretty),
            _ => Err(JsonError::DataFormatUnknownError {
                format: s.to_string(),
            }),
        }
    }
}

impl DataFormat {
    fn read_string(&self, data: &str) -> Result<Json, RenderError> {
        match self {
            DataFormat::Json | DataFormat::JsonPretty => {
                serde_json::from_str(&data).map_err(RenderError::with)
            }
            DataFormat::Yaml => serde_yaml::from_str(&data).map_err(RenderError::with),
            DataFormat::Toml | DataFormat::TomlPretty => {
                toml::from_str(&data).map_err(RenderError::with)
            }
        }
    }

    fn write_string(&self, data: &Json) -> Result<String, RenderError> {
        match self {
            DataFormat::Json => serde_json::to_string(data).map_err(RenderError::with),
            DataFormat::JsonPretty => serde_json::to_string_pretty(data).map_err(RenderError::with),
            DataFormat::Yaml => serde_yaml::to_string(data)
                .map_err(RenderError::with)
                .map(|s| s.trim_start_matches("---\n").to_string()),
            DataFormat::Toml => toml::to_string(data).map_err(RenderError::with),
            DataFormat::TomlPretty => toml::to_string_pretty(data).map_err(RenderError::with),
        }
    }
}

fn json_query<T: Serialize, E: AsRef<str>>(expr: E, data: T) -> Result<Json, JsonError> {
    let data = data.to_jmespath();
    let res = jmespath::compile(expr.as_ref())
        .and_then(|e| e.search(data))
        .context(JsonQueryError {
            expression: expr.as_ref().to_string(),
        })?;
    serde_json::to_value(res.as_ref()).context(ToJsonValueError {
        input: format!("{:?}", res),
    })
}

fn find_data_format<'reg: 'rc, 'rc>(h: &Helper<'reg, 'rc>) -> Result<DataFormat, RenderError> {
    let param = h
        .hash_get("format")
        .and_then(|v| v.value().as_str())
        .unwrap_or("json");
    DataFormat::from_str(param).map_err(RenderError::with)
}

fn find_str_param<'reg: 'rc, 'rc>(
    pos: usize,
    h: &Helper<'reg, 'rc>,
) -> Result<String, RenderError> {
    h.param(pos)
        .ok_or_else(|| RenderError::new(format!("param {} (the string) not found", pos)))
        .and_then(|v| {
            serde_json::from_value::<String>(v.value().clone()).map_err(RenderError::with)
        })
}

#[allow(non_camel_case_types)]
pub struct str_to_json_fct;

impl HelperDef for str_to_json_fct {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let format = find_data_format(h)?;
        let data: String = find_str_param(0, h)?;
        let result = format.read_string(&data)?;
        Ok(Some(ScopedJson::Derived(result)))
    }
}

#[allow(non_camel_case_types)]
pub struct json_to_str_fct;

impl HelperDef for json_to_str_fct {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let format = find_data_format(h)?;
        let data = h
            .param(0)
            .ok_or(RenderError::new("param 0 (the json) not found"))
            .map(|v| v.value())?;
        let result = format.write_string(&data)?;
        Ok(Some(ScopedJson::Derived(Json::String(result))))
    }
}

#[allow(non_camel_case_types)]
pub struct json_str_query_fct;

impl HelperDef for json_str_query_fct {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext,
    ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
        let format = find_data_format(h)?;
        let expr = find_str_param(0, h)?;
        let data_str = find_str_param(1, h)?;
        let data = format.read_string(&data_str)?;
        let result = json_query(expr, data)
            .map_err(RenderError::with)
            .and_then(|v| format.write_string(&v))?;
        dbg!(&result);
        Ok(Some(ScopedJson::Derived(Json::String(result))))
    }
}

handlebars_helper!(json_query_fct: |expr: str, data: Json| json_query(expr, data).map_err(RenderError::with)?);

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
    use crate::assert_renders;
    use indoc::indoc;
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

    fn normalize_nl(s: &str) -> String {
        s.replace("\r\n", "\n").replace("\r", "")
    }

    fn assert_data_format_write_eq_read(f: DataFormat, data: &str) -> Result<(), Box<dyn Error>> {
        let data = normalize_nl(data);
        let actual = normalize_nl(&f.write_string(&f.read_string(&data)?)?);
        assert_that!(&actual).is_equal_to(data);
        Ok(())
    }

    #[test]
    fn test_data_format_symmetry() -> Result<(), Box<dyn Error>> {
        assert_data_format_write_eq_read(DataFormat::Json, r##"{"foo":{"bar":{"baz":true}}}"##)?;
        assert_data_format_write_eq_read(
            DataFormat::JsonPretty,
            indoc!(
                r##"{
                  "foo": {
                    "bar": {
                      "baz": true
                    }
                  }
                }"##
            ),
        )?;
        assert_data_format_write_eq_read(
            DataFormat::Yaml,
            indoc!(
                r##"
                foo:
                  bar:
                    baz: true"##
            ),
        )?;
        assert_data_format_write_eq_read(
            DataFormat::Toml,
            indoc!(
                r##"
                [foo.bar]
                baz = true
                "##
            ),
        )?;
        assert_data_format_write_eq_read(
            DataFormat::TomlPretty,
            indoc!(
                r##"
                [foo.bar]
                baz = true
                "##
            ),
        )?;
        Ok(())
    }

    #[test]
    fn test_helper_json_to_str() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ json_to_str {} }}"##, r##"{}"##),
            (
                r##"{{ json_to_str {"foo":{"bar":{"baz":true}}} }}"##,
                r##"{"foo":{"bar":{"baz":true}}}"##,
            ),
            (
                r##"{{ json_to_str ( str_to_json "{\"foo\":{\"bar\":{\"baz\":true}}}" ) }}"##,
                r##"{"foo":{"bar":{"baz":true}}}"##,
            ),
            (
                r##"{{ json_to_str ( str_to_json "{\"foo\":{\"bar\":{\"baz\":true}}}" ) format="json_pretty"}}"##,
                &normalize_nl(indoc!(r##"{
                  "foo": {
                    "bar": {
                      "baz": true
                    }
                  }
                }"##)),
            )
        ]
    }

    #[test]
    fn test_helper_json_query() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ json_query "foo" {} }}"##, r##""##),
            (
                r##"{{ json_to_str ( json_query "foo" {"foo":{"bar":{"baz":true}}} ) }}"##,
                r##"{"bar":{"baz":true}}"##,
            ),
            (
                r##"{{ json_to_str ( json_query "foo" (str_to_json "{\"foo\":{\"bar\":{\"baz\":true}}}" ) ) }}"##,
                r##"{"bar":{"baz":true}}"##,
            )
        ]
    }

    #[test]
    fn test_helper_json_str_query() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (
                r##"{{ json_str_query "foo" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}"##,
                r##"{"bar":{"baz":true}}"##,
            ),
            (
                r##"{{ json_str_query "foo" "{\"foo\":{\"bar\":{\"baz\":true}}}" format="json"}}"##,
                r##"{"bar":{"baz":true}}"##,
            ),
            (
                r##"{{ json_str_query "foo.bar.baz" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}"##,
                "true",
            )
        ]
    }

    #[test]
    fn test_helper_json_str_query_on_yaml() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (
                r##"{{ json_str_query "foo" "{\"foo\":{\"bar\":{\"baz\":true}}}" format="yaml"}}"##,
                &normalize_nl(indoc!(
                    "
                bar:
                  baz: true"
                ))
            ),
            (
                r##"{{ json_str_query "foo" "foo:\n bar:\n  baz: true\n" format="yaml"}}"##,
                &normalize_nl(indoc!(
                    "
                bar:
                  baz: true"
                ))
            ),
            (
                r##"{{ json_str_query "foo.bar.baz" "foo:\n bar:\n  baz: true\n" format="yaml"}}"##,
                "true",
            )
        ]
    }

    #[test]
    fn test_helper_json_str_query_on_toml() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (
                r##"{{ json_str_query "foo" "[foo.bar]\nbaz=true\n" format="toml"}}"##,
                indoc!(
                    "[bar]
                    baz = true
                    "
                ),
            ),
            (
                r##"{{ json_str_query "foo.bar.baz" "[foo.bar]\nbaz=true\n" format="toml"}}"##,
                "true",
            )
        ]
    }
}
