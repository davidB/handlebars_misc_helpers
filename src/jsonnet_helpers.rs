use crate::outputs::StringOutput;
use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError, Renderable,
};
use jsonnet::JsonnetVm;
use thiserror::Error;

#[derive(Debug, Error)]
enum JsonnetError {
    #[error("jsonnet evaluation failed: {source_str}")]
    EvaluateError { source_str: String },
}

fn jsonnet_block<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    r: &'reg Handlebars,
    ctx: &'rc Context,
    rc: &mut RenderContext<'reg, 'rc>,
    out: &mut dyn Output,
) -> HelperResult {
    let mut content = StringOutput::default();
    h.template()
        .map(|t| t.render(r, ctx, rc, &mut content))
        .unwrap_or(Ok(()))?;
    let input = content.into_string()?;
    let res = if input.is_empty() {
        input
    } else {
        let mut vm = JsonnetVm::new();
        vm.string_output(
            h.hash_get("string_output")
                .and_then(|v| v.value().as_bool())
                .unwrap_or(false),
        );
        // vm.fmt_indent(
        //     h.hash_get("indent")
        //         .and_then(|v| v.value().as_u64())
        //         .unwrap_or(4) as u32,
        // );
        let s = vm
            .evaluate_snippet("snippet", &input)
            .map_err(|e| {
                RenderError::from_error(
                    "jsonnet_block",
                    JsonnetError::EvaluateError {
                        source_str: format!("{:?}", e),
                    },
                )
            })?
            .to_string();
        s
    };

    out.write(&res)
        .map_err(|e| RenderError::from_error("jsonnet_block", e))
}

pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("jsonnet", Box::new(jsonnet_block));
}

#[cfg(test)]
mod tests {
    //use super::*;
    use crate::assert_renders;
    use crate::tests::normalize_nl;
    use std::error::Error;

    #[test]
    fn test_jsonnet_block() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{#jsonnet}}{{/jsonnet}}"##, r##""##),
            (
                r##"{{#jsonnet}}{"foo":{"bar":{"baz":true}}}{{/jsonnet}}"##,
                &normalize_nl(
                    r##"{
                       "foo": {
                          "bar": {
                             "baz": true
                          }
                       }
                    }
                    "##
                )
            ),
            (
                r##"{{#jsonnet}}
                local v = {"foo":{"bar":{"baz":false}}};
                v
                {{/jsonnet}}"##,
                &normalize_nl(
                    r##"{
                       "foo": {
                          "bar": {
                             "baz": false
                          }
                       }
                    }
                    "##
                )
            ),
            (
                r##"{{#jsonnet}}
                local v = {"foo":{"bar":{"baz":false}}};
                v {
                  "v": 3,
                  "vv" +: {
                      "vvv": 333
                  }
                }
                {{/jsonnet}}"##,
                &normalize_nl(
                    r##"{
                       "foo": {
                          "bar": {
                             "baz": false
                          }
                       },
                       "v": 3,
                       "vv": {
                          "vvv": 333
                       }
                    }
                    "##
                )
            ),
            (
                r##"{{#jsonnet}}
                local v = {"foo":{"bar":{"baz":false}}};
                v {
                  "foo" +: {
                      "bar" +: {
                          "baz": true
                      }
                  }
                }
                {{/jsonnet}}"##,
                &normalize_nl(
                    r##"{
                       "foo": {
                          "bar": {
                             "baz": true
                          }
                       }
                    }
                    "##
                )
            ),
            (
                r##"{{#jsonnet}}
                local v = {"foo":{"bar":{"baz":false}}};
                v {
                  "foo" +: {
                      "bar" +: {
                          "baz2": true
                      }
                  }
                }
                {{/jsonnet}}"##,
                &normalize_nl(
                    r##"{
                       "foo": {
                          "bar": {
                             "baz": false,
                             "baz2": true
                          }
                       }
                    }
                    "##
                )
            ),
        ]
    }
}
