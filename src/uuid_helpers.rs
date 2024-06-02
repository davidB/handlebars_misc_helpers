use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use uuid::Uuid;

fn uuid_new_v4_fct(
    _: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    out.write(Uuid::new_v4().to_string().as_str())?;
    Ok(())
}

fn uuid_new_v7_fct(
    _: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    out.write(Uuid::new_v4().to_string().as_str())?;
    Ok(())
}

pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("uuid_new_v4", Box::new(uuid_new_v4_fct));
    handlebars.register_helper("uuid_new_v7", Box::new(uuid_new_v7_fct));
}

#[cfg(test)]
mod tests {
    use crate::assert_renders;
    use std::error::Error;

    #[test]
    fn test_regex_captures() -> Result<(), Box<dyn Error>> {
        assert_renders![
            // (r##"{{ uuid_new_v4 }}"##, r##""##),
            (r##"{{ len (uuid_new_v4) }}"##, r##"36"##),
            // (r##"{{ uuid_new_v7 }}"##, r##""##),
            (r##"{{ len( uuid_new_v7 )}}"##, r##"36"##),
        ]
    }
}
