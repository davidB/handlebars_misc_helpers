use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use regex::Regex;

#[allow(non_camel_case_types)]
pub struct regex_captures_fct;

impl HelperDef for regex_captures_fct {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg>, RenderError> {
        let on = h
            .hash_get("on")
            .and_then(|v| v.value().as_str())
            .unwrap_or_default();
        let pattern = h
            .hash_get("pattern")
            .and_then(|v| v.value().as_str())
            .unwrap_or_default();
        let re = Regex::new(pattern).map_err(|err| crate::to_other_error(err.to_string()))?;
        if let Some(caps) = re.captures(on) {
            let collected = re
                .capture_names()
                .filter_map(|v| {
                    v.and_then(|name| {
                        caps.name(name).map(|m| {
                            (
                                name.to_string(),
                                serde_json::Value::String(m.as_str().to_string()),
                            )
                        })
                    })
                })
                .chain(caps.iter().enumerate().filter_map(|(i, mm)| {
                    mm.map(|m| {
                        (
                            format!("_{}", i),
                            serde_json::Value::String(m.as_str().to_string()),
                        )
                    })
                }))
                .collect::<serde_json::Map<_, _>>();
            Ok(ScopedJson::Derived(serde_json::Value::Object(collected)))
        } else {
            Ok(ScopedJson::Derived(serde_json::Value::Null))
        }
    }
}

#[allow(non_camel_case_types)]
pub struct regex_is_match_fct;

impl HelperDef for regex_is_match_fct {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg>, RenderError> {
        let on = h
            .hash_get("on")
            .and_then(|v| v.value().as_str())
            .unwrap_or_default();
        let pattern = h
            .hash_get("pattern")
            .and_then(|v| v.value().as_str())
            .unwrap_or_default();
        let re = Regex::new(pattern).map_err(|err| crate::to_other_error(err.to_string()))?;
        Ok(ScopedJson::Derived(serde_json::Value::Bool(
            re.is_match(on),
        )))
    }
}

pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("regex_captures", Box::new(regex_captures_fct));
    handlebars.register_helper("regex_is_match", Box::new(regex_is_match_fct));
}

#[cfg(test)]
mod tests {
    use crate::assert_renders;
    use std::error::Error;

    #[test]
    fn test_regex_captures() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ regex_captures pattern="foo" on="" }}"##, r##""##),
            (
                r##"{{ regex_captures pattern="(?<first>\\w)(\\w)(?:\\w)\\w(?<last>\\w)" on="today" }}"##,
                r##"[object]"##
            ),
            (
                r##"{{ json_to_str( regex_captures pattern="(?<first>\\w)(\\w)(?:\\w)\\w(?<last>\\w)" on="today" ) }}"##,
                r##"{"_0":"today","_1":"t","_2":"o","_3":"y","first":"t","last":"y"}"##
            ),
            (
                r##"{{ set captures=( regex_captures pattern="(?<first>\\w)(\\w)(?:\\w)\\w(?<last>\\w)" on="today" ) }}{{ captures.last }}"##,
                r##"y"##
            ),
        ]
    }

    #[test]
    fn test_regex_is_match() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (
                r##"{{ regex_is_match pattern="foo" on="" }}"##,
                r##"false"##
            ),
            (
                r##"{{ regex_is_match  pattern="(?<first>\\w)(\\w)(?:\\w)\\w(?<last>\\w)" on="today" }}"##,
                r##"true"##
            ),
            (
                r##"{{#if (regex_is_match pattern="(?<first>\\w)(\\w)(?:\\w)\\w(?<last>\\w)" on="today" ) }}ok{{/if}}"##,
                r##"ok"##
            ),
        ]
    }
}
