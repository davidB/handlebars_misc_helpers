use handlebars::Context;
use handlebars::Handlebars;
use handlebars::Helper;
use handlebars::HelperResult;
use handlebars::Output;
use handlebars::RenderContext;
use handlebars::RenderErrorReason;

fn assign_fct(
    h: &Helper,
    _: &Handlebars,
    ctx: &Context,
    rc: &mut RenderContext,
    _: &mut dyn Output,
) -> HelperResult {
    // get parameter from helper or throw an error
    let name = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .ok_or(RenderErrorReason::ParamNotFoundForIndex("assign", 0))?;
    let value = h
        .param(1)
        .map(|v| v.value())
        .cloned()
        .ok_or(RenderErrorReason::ParamNotFoundForIndex("assign", 1))?;
    let mut ctx = ctx.clone();
    match ctx.data_mut() {
        serde_json::value::Value::Object(m) => m.insert(name.to_owned(), value),
        _ => None,
    };
    rc.set_context(ctx);
    Ok(())
}

pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("assign", Box::new(assign_fct))
}

#[cfg(test)]
mod tests {
    use crate::assert_renders;
    use std::error::Error;

    #[test]
    fn test_helper_assign() -> Result<(), Box<dyn Error>> {
        assert_renders![
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
            )
        ]
    }
}
