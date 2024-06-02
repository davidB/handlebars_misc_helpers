use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderErrorReason,
};

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
    let mut ctx = rc.context().as_deref().unwrap_or(ctx).clone();
    if let Some(ref mut m) = ctx.data_mut().as_object_mut() {
        m.insert(name.to_owned(), value);
    }
    rc.set_context(ctx);
    Ok(())
}

fn set_fct(
    h: &Helper,
    _: &Handlebars,
    ctx: &Context,
    rc: &mut RenderContext,
    _: &mut dyn Output,
) -> HelperResult {
    let mut ctx = rc.context().as_deref().unwrap_or(ctx).clone();
    if let Some(ref mut m) = ctx.data_mut().as_object_mut() {
        for (k, v) in h.hash() {
            m.insert(k.to_string(), v.value().clone());
        }
    }
    rc.set_context(ctx);
    Ok(())
}

pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("assign", Box::new(assign_fct));
    handlebars.register_helper("set", Box::new(set_fct));
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
            ),
            (
                r##"{{ assign "foo" "world" }}{{ assign "bar" "hello" }}>{{ bar }} {{ foo }}<"##,
                r##">hello world<"##,
            )
        ]
    }

    #[test]
    fn test_helper_set() -> Result<(), Box<dyn Error>> {
        assert_renders![
            (r##"{{ set foo="{}" }}"##, r##""##),
            (r##"{{ set foo="{}" }}{{ foo }}"##, r##"{}"##),
            (r##"{{ set foo={} }}{{ foo }}"##, r##"[object]"##),
            (r##"{{ set foo={"bar": 33} }}{{ foo }}"##, r##"[object]"##,),
            (
                r##"{{ set foo={"bar": 33} }}{{ json_to_str foo }}"##,
                r##"{"bar":33}"##,
            ),
            (r##"{{ set foo={"bar": 33} }}{{ foo.bar }}"##, r##"33"##,),
            (
                r##"{{ set foo="hello world" }}{{ foo }}"##,
                r##"hello world"##,
            ),
            (
                r##"{{ set foo="world" bar="hello" }}>{{ bar }} {{ foo }}<"##,
                r##">hello world<"##,
            ),
            (
                r##"{{ set foo="world" }}{{ set bar="hello" }}>{{ bar }} {{ foo }}<"##,
                r##">hello world<"##,
            ),
            (r##"{{ set foo=(eq 12 12) }}{{ foo }}"##, r##"true"##,)
        ]
    }
}
