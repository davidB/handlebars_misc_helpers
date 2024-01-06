use crate::outputs::StringOutput;
use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext,
    RenderError, RenderErrorReason, Renderable, ScopedJson,
};
use jmespath;
use serde::Serialize;
use serde_json::Value as Json;
use std::str::FromStr;
use thiserror::Error;
use toml::value::Table;

type TablePartition = Vec<(String, toml::Value)>;

#[derive(Debug, Error)]
enum JsonError {
    #[error("query failure for expression '{expression}'")]
    JsonQueryFailure {
        expression: String,
        source: Box<jmespath::JmespathError>,
    },
    #[error("fail to convert '{input}'")]
    ToJsonValueError {
        input: String,
        source: serde_json::error::Error,
    },
    #[error("data format unknown '{format}'")]
    DataFormatUnknown { format: String },
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
            _ => Err(JsonError::DataFormatUnknown {
                format: s.to_string(),
            }),
        }
    }
}

fn to_opt_res<T, E>(v: Result<Option<T>, E>) -> Option<Result<T, E>> {
    match v {
        Err(e) => Some(Err(e)),
        Ok(v) => v.map(Ok),
    }
}

// HACK for toml because
// see:
// - [ValueAfterTable error · Issue #336 · alexcrichton/toml-rs](https://github.com/alexcrichton/toml-rs/issues/336)
// - [ValueAfterTable fix by PSeitz · Pull Request #339 · alexcrichton/toml-rs](https://github.com/alexcrichton/toml-rs/pull/339)
// previous workaround was to use serde_transcode like [PSeitz/toml-to-json-online-converter: toml to json and json to toml online converter - written in rust with wasm](https://github.com/PSeitz/toml-to-json-online-converter)
// but it failed in some case (added into the test section)
// DataFormat::Toml => {
//     let mut res = String::new();
//     let input = content.into_string()?;
//     let mut deserializer = serde_json::Deserializer::from_str(&input);
//     let mut serializer = toml::ser::Serializer::new(&mut res);
//     serde_transcode::transcode(&mut deserializer, &mut serializer)
//         .map_err(|e| RenderError::from_error("serde_transcode", e))?;
//     res
// }
// DataFormat::TomlPretty => {
//     let mut res = String::new();
//     let input = content.into_string()?;
//     let mut deserializer = serde_json::Deserializer::from_str(&input);
//     let mut serializer = toml::ser::Serializer::pretty(&mut res);
//     serde_transcode::transcode(&mut deserializer, &mut serializer)
//         .map_err(|e| RenderError::from_error("serde_transcode", e))?;
//     res
// }
//
// For toml we recreate a tom struct with map that preserve order (indexedmap)
fn to_ordored_toml_value(data: &Json) -> Result<Option<toml::Value>, RenderError> {
    match data {
        Json::String(v) => Ok(Some(toml::Value::from(v.as_str()))),
        Json::Array(v) => v
            .iter()
            .filter_map(|i| to_opt_res(to_ordored_toml_value(i)))
            .collect::<Result<Vec<_>, _>>()
            .map(|a| Some(toml::Value::Array(a))),
        Json::Object(ref obj) => obj
            .iter()
            .filter_map(|kv| {
                to_opt_res(to_ordored_toml_value(kv.1))
                    .map(|rnv| rnv.map(|nv| (kv.0.to_owned(), nv)))
            })
            .collect::<Result<Table, _>>()
            .map(|m| Some(toml::Value::Table(sort_toml_map(m)))),
        Json::Number(v) => {
            if v.is_i64() {
                Ok(Some(toml::Value::Integer(v.as_i64().unwrap())))
            } else if let Some(x) = v.as_f64() {
                Ok(Some(toml::Value::Float(x)))
            } else {
                Err(RenderErrorReason::Other(format!(
                    "to_toml:can not convert a Json Number: {}",
                    v
                ))
                .into())
            }
        }
        Json::Bool(v) => Ok(Some(toml::Value::Boolean(*v))),
        Json::Null => Ok(None),
    }
}

fn sort_toml_map(data: Table) -> Table {
    let (tables, non_tables): (TablePartition, TablePartition) =
        data.into_iter().partition(|v| v.1.is_table());
    let (arrays, others): (TablePartition, TablePartition) =
        non_tables.into_iter().partition(|v| v.1.is_array());
    let mut m = Table::new();
    m.extend(others);
    m.extend(arrays);
    m.extend(tables);
    m
}

impl DataFormat {
    fn read_string(&self, data: &str) -> Result<Json, RenderError> {
        if data.is_empty() {
            //return Ok(Json::Null);
            return Ok(Json::String("".to_owned()));
        }
        match self {
            DataFormat::Json | DataFormat::JsonPretty => {
                serde_json::from_str(data).map_err(RenderErrorReason::from)
            }
            DataFormat::Yaml => {
                serde_yaml::from_str(data).map_err(|e| RenderErrorReason::NestedError(Box::new(e)))
            }
            DataFormat::Toml | DataFormat::TomlPretty => {
                toml::from_str(data).map_err(|e| RenderErrorReason::NestedError(Box::new(e)))
            }
        }
        .map_err(Into::into)
    }

    fn write_string(&self, data: &Json) -> Result<String, RenderError> {
        match data {
            Json::Null => Ok("".to_owned()),
            Json::String(c) if c.is_empty() => Ok("".to_owned()),
            _ => match self {
                DataFormat::Json => serde_json::to_string(data).map_err(RenderErrorReason::from),
                DataFormat::JsonPretty => {
                    serde_json::to_string_pretty(data).map_err(RenderErrorReason::from)
                }
                DataFormat::Yaml => serde_yaml::to_string(data)
                    .map_err(|e| RenderErrorReason::NestedError(Box::new(e)))
                    .map(|s| s.trim_start_matches("---\n").to_string()),
                DataFormat::Toml => {
                    let data_toml = to_ordored_toml_value(data)?;
                    toml::to_string(&data_toml)
                        .map_err(|e| RenderErrorReason::NestedError(Box::new(e)))
                }
                DataFormat::TomlPretty => {
                    let data_toml = to_ordored_toml_value(data)?;
                    toml::to_string_pretty(&data_toml)
                        .map_err(|e| RenderErrorReason::NestedError(Box::new(e)))
                }
            },
        }
        .map_err(Into::into)
    }
}

fn json_query<T: Serialize, E: AsRef<str>>(expr: E, data: T) -> Result<Json, JsonError> {
    // let data = data.to_jmespath();
    let res = jmespath::compile(expr.as_ref())
        .and_then(|e| e.search(data))
        .map_err(|source| JsonError::JsonQueryFailure {
            expression: expr.as_ref().to_string(),
            source: Box::new(source),
        })?;
    serde_json::to_value(res.as_ref()).map_err(|source| JsonError::ToJsonValueError {
        input: format!("{:?}", res),
        source,
    })
}

fn find_data_format<'reg: 'rc, 'rc>(h: &Helper<'rc>) -> Result<DataFormat, RenderError> {
    let param = h
        .hash_get("format")
        .and_then(|v| v.value().as_str())
        .unwrap_or("json");
    DataFormat::from_str(param).map_err(|e| RenderErrorReason::NestedError(Box::new(e)).into())
}

fn find_str_param<'reg: 'rc, 'rc>(pos: usize, h: &Helper<'rc>) -> Result<String, RenderError> {
    h.param(pos)
        .ok_or_else(|| {
            RenderErrorReason::Other(format!("param {} (the string) not found", pos)).into()
        })
        // .and_then(|v| {
        //     serde_json::from_value::<String>(v.value().clone()).map_err(RenderError::with)
        // })
        .map(|v| v.value().as_str().unwrap_or("").to_owned())
}

#[allow(non_camel_case_types)]
pub struct str_to_json_fct;

impl HelperDef for str_to_json_fct {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        let data: String = find_str_param(0, h)?;
        let format = find_data_format(h)?;
        let result = format.read_string(&data)?;
        Ok(ScopedJson::Derived(result))
    }
}

#[allow(non_camel_case_types)]
pub struct json_to_str_fct;

impl HelperDef for json_to_str_fct {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        let format = find_data_format(h)?;
        let data = h
            .param(0)
            .ok_or_else(|| RenderErrorReason::Other("param 0 (the json) not found".to_string()))
            .map(|v| v.value())?;
        let result = format.write_string(data)?;
        Ok(ScopedJson::Derived(Json::String(result)))
    }
}

#[allow(non_camel_case_types)]
pub struct json_str_query_fct;

impl HelperDef for json_str_query_fct {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        let format = find_data_format(h)?;
        let expr = find_str_param(0, h)?;
        let data_str = find_str_param(1, h)?;
        let data = format.read_string(&data_str)?;
        let result = json_query(expr, data)
            .map_err(|e| RenderErrorReason::NestedError(Box::new(e)).into())
            .and_then(|v| {
                let output_format = if v.is_array() || v.is_object() {
                    format
                } else {
                    DataFormat::Json
                };
                output_format.write_string(&v).map(|s| {
                    if v.is_array() || v.is_object() {
                        s
                    } else {
                        s.trim().to_owned()
                    }
                })
            })?;
        Ok(ScopedJson::Derived(Json::String(result)))
    }
}

fn from_json_block<'reg, 'rc>(
    h: &Helper<'rc>,
    r: &'reg Handlebars,
    ctx: &'rc Context,
    rc: &mut RenderContext<'reg, 'rc>,
    out: &mut dyn Output,
) -> HelperResult {
    let format = find_data_format(h)?;
    let mut content = StringOutput::default();
    h.template()
        .map(|t| t.render(r, ctx, rc, &mut content))
        .unwrap_or(Ok(()))?;
    let data = DataFormat::Json.read_string(&content.into_string()?)?;
    let res = format.write_string(&data)?;

    out.write(&res)
        .map_err(|e| RenderErrorReason::NestedError(Box::new(e)).into())
}

fn to_json_block<'reg, 'rc>(
    h: &Helper<'rc>,
    r: &'reg Handlebars,
    ctx: &'rc Context,
    rc: &mut RenderContext<'reg, 'rc>,
    out: &mut dyn Output,
) -> HelperResult {
    let format = find_data_format(h)?;
    let mut content = StringOutput::default();
    h.template()
        .map(|t| t.render(r, ctx, rc, &mut content))
        .unwrap_or(Ok(()))?;
    let data = format.read_string(&content.into_string()?)?;
    let res = DataFormat::JsonPretty.write_string(&data)?;
    out.write(&res).map_err(RenderError::from)
}

handlebars_helper!(json_query_fct: |expr: str, data: Json| json_query(expr, data).map_err(|e| <RenderErrorReason as Into<RenderError>>::into(RenderErrorReason::NestedError(Box::new(e))))?);

pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("json_to_str", Box::new(json_to_str_fct));
    handlebars.register_helper("str_to_json", Box::new(str_to_json_fct));
    handlebars.register_helper("from_json", Box::new(from_json_block));
    handlebars.register_helper("to_json", Box::new(to_json_block));
    handlebars.register_helper("json_query", Box::new(json_query_fct));
    handlebars.register_helper("json_str_query", Box::new(json_str_query_fct));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_renders;
    use crate::tests::normalize_nl;
    use spectral::prelude::*;
    use std::error::Error;

    #[test]
    fn test_empty_input_return_empty() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ json_to_str "" }}"##, ""),
            (r##"{{ json_to_str "" format="json"}}"##, ""),
            (r##"{{ json_to_str "" format="yaml"}}"##, ""),
            (r##"{{ json_to_str "" format="toml"}}"##, ""),
            (r##"{{ str_to_json "" }}"##, ""),
            (r##"{{ str_to_json "" format="json"}}"##, ""),
            (r##"{{ str_to_json "" format="yaml"}}"##, ""),
            (r##"{{ str_to_json "" format="toml"}}"##, ""),
            (r##"{{ json_to_str (str_to_json "") }}"##, ""),
            (r##"{{ str_to_json (json_to_str "") }}"##, ""),
            (r##"{{ json_query "foo" "" }}"##, ""),
            (r##"{{ json_str_query "foo" "" }}"##, ""),
            (r##"{{ json_str_query "foo" "" format="json"}}"##, ""),
            (r##"{{ json_str_query "foo" "" format="yaml"}}"##, ""),
            (r##"{{ json_str_query "foo" "" format="toml"}}"##, "")
        ]
    }

    #[test]
    fn test_null_input_return_empty() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ json_to_str null }}"##, ""),
            (r##"{{ str_to_json null }}"##, ""),
            (r##"{{ json_to_str (str_to_json null) }}"##, ""),
            (r##"{{ str_to_json (json_to_str null) }}"##, ""),
            (r##"{{ json_query "foo" null }}"##, ""),
            (r##"{{ json_str_query "foo" null }}"##, "")
        ]
    }

    #[test]
    fn test_search_object_field() -> Result<(), Box<dyn Error>> {
        let json: Json = serde_json::from_str(r##"{"foo":{"bar":{"baz":true}}}"##)?;
        let result = json_query("foo.bar.baz", json)?;
        assert_that!(result).is_equal_to(Json::Bool(true));
        Ok(())
    }

    #[test]
    fn test_search_path_in_empty() -> Result<(), Box<dyn Error>> {
        for v in ["{}", "[]", "null", "\"\""] {
            let json: Json = serde_json::from_str(v)?;
            let result = json_query("foo.bar.baz", json)?;
            assert_that!(result).is_equal_to(Json::Null);
        }
        Ok(())
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
            &normalize_nl(
                r##"{
                  "foo": {
                    "bar": {
                      "baz": true
                    }
                  }
                }"##,
            ),
        )?;
        assert_data_format_write_eq_read(
            DataFormat::Yaml,
            &normalize_nl(
                r##"
                    a: a
                    foo:
                      bar:
                        baz: true
                "##,
            ),
        )?;
        assert_data_format_write_eq_read(
            DataFormat::Toml,
            &normalize_nl(
                r##"
                [foo.bar]
                baz = true
                "##,
            ),
        )?;
        assert_data_format_write_eq_read(
            DataFormat::TomlPretty,
            &normalize_nl(
                r##"
                [foo.bar]
                baz = true
                "##,
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
                &normalize_nl(
                    r##"{
                  "foo": {
                    "bar": {
                      "baz": true
                    }
                  }
                }"##
                ),
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
                &normalize_nl(
                    "
                    bar:
                      baz: true
                    "
                )
            ),
            (
                r##"{{ json_str_query "foo" "foo:\n bar:\n  baz: true\n" format="yaml"}}"##,
                &normalize_nl(
                    "
                    bar:
                      baz: true
                    "
                )
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
                &normalize_nl(
                    "[bar]
                    baz = true
                    "
                ),
            ),
            // returning a single value is not a valid toml
            (
                r##"{{ json_str_query "foo.bar.baz" "[foo.bar]\nbaz=true\n" format="toml"}}"##,
                "true",
            )
        ]
    }

    #[test]
    fn test_block_to_json() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{#to_json}}{{/to_json}}"##, r##""##),
            (
                r##"{{#to_json}}{"foo":{"bar":{"baz":true}}}{{/to_json}}"##,
                &normalize_nl(
                    r##"{
                      "foo": {
                        "bar": {
                          "baz": true
                        }
                      }
                    }"##
                ),
            ),
            (
                &normalize_nl(
                    r##"{{#to_json format="yaml"}}
                    foo:
                        bar:
                            baz: true
                    {{/to_json}}"##
                ),
                &normalize_nl(
                    r##"{
                      "foo": {
                        "bar": {
                          "baz": true
                        }
                      }
                    }"##
                ),
            ),
            (
                &normalize_nl(
                    r##"{{#to_json format="toml"}}
                    [foo]
                    bar = { baz = true }
                    hello = "1.2.3"
                    {{/to_json}}"##
                ),
                &normalize_nl(
                    r##"{
                      "foo": {
                        "bar": {
                          "baz": true
                        },
                        "hello": "1.2.3"
                      }
                    }"##
                ),
            ),
        ]
    }

    #[test]
    fn test_block_from_json() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{#from_json}}{{/from_json}}"##, r##""##),
            (
                r##"{{#from_json}}{"foo":{"bar":{"baz":true}}}{{/from_json}}"##,
                r##"{"foo":{"bar":{"baz":true}}}"##
            ),
            (
                r##"{{#from_json format="json_pretty"}}{"foo":{"bar":{"baz":true}}}{{/from_json}}"##,
                &normalize_nl(
                    r##"{
                      "foo": {
                        "bar": {
                          "baz": true
                        }
                      }
                    }"##
                ),
            ),
            (
                r##"{{#from_json format="yaml"}}{"foo":{"bar":{"baz":true}}}{{/from_json}}"##,
                &normalize_nl(
                    r##"
                    foo:
                      bar:
                        baz: true
                    "##
                )
            ),
            (
                r##"{{#from_json format="toml"}}{"foo":{"bar":{"baz":true}}}{{/from_json}}"##,
                &normalize_nl(
                    r##"
                    [foo.bar]
                    baz = true
                    "##
                ),
            ),
            (
                r##"{{#from_json format="toml"}}{"foo":{"hello":"1.2.3", "bar":{"baz":true} }}{{/from_json}}"##,
                &normalize_nl(
                    r##"
                    [foo]
                    hello = "1.2.3"

                    [foo.bar]
                    baz = true
                    "##
                ),
            ),
            (
                r##"{{#from_json format="toml_pretty"}}{"foo":{"hello":"1.2.4", "bar":{"baz":true} }}{{/from_json}}"##,
                &normalize_nl(
                    r##"
                    [foo]
                    hello = "1.2.4"

                    [foo.bar]
                    baz = true
                    "##
                ),
            ),
        ]
    }

    #[test]
    fn test_block_from_json_to_toml_with_tables_at_anyplace() -> Result<(), Box<dyn Error>> {
        assert_renders![(
            r##"{{#from_json format="toml"}}
                {"foo":{
                    "f0": null,
                    "f1":"1.2.3",
                    "f2":{
                        "f20": true,
                        "f21": { "f210": false}
                    },
                    "f3": [1,2,3],
                    "f4":"1.2.3",
                    "f5":{
                        "f50": true,
                        "f51": { "f510": false}
                    }
                }}
                {{/from_json}}"##,
            &normalize_nl(
                r##"
                    [foo]
                    f1 = "1.2.3"
                    f4 = "1.2.3"
                    f3 = [1, 2, 3]
                    
                    [foo.f2]
                    f20 = true
                    
                    [foo.f2.f21]
                    f210 = false
                    
                    [foo.f5]
                    f50 = true
                    
                    [foo.f5.f51]
                    f510 = false
                    "##
            ),
        ),]
    }

    #[test]
    fn test_sort_toml_map() {
        let mut actual = toml::map::Map::new();
        actual.insert("f1".to_string(), toml::Value::String("s1".to_owned()));
        actual.insert("f2".to_string(), toml::Value::Table(toml::map::Map::new()));
        actual.insert("f3".to_string(), toml::Value::Boolean(true));
        actual.insert("f4".to_string(), toml::Value::Table(toml::map::Map::new()));
        let mut expected = toml::map::Map::new();
        expected.insert("f1".to_string(), toml::Value::String("s1".to_owned()));
        expected.insert("f3".to_string(), toml::Value::Boolean(true));
        expected.insert("f2".to_string(), toml::Value::Table(toml::map::Map::new()));
        expected.insert("f4".to_string(), toml::Value::Table(toml::map::Map::new()));
        assert_eq!(sort_toml_map(actual), expected)
    }
}
