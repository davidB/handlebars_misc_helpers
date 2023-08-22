use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, Renderable,
};
use log::warn;

pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("replace_section", Box::new(ReplaceSectionHelper))
}

#[derive(Clone, Copy)]
pub struct ReplaceSectionHelper;

impl HelperDef for ReplaceSectionHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        hbs: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let Some(tmpl) = h.template() else {
            warn!("`replace_section` helper require a template");
            return Ok(());
        };

        let Some(input) = h.param(0).and_then(|it| it.value().as_str())  else {
            warn!("`replace_section` helper require an string parameter");
            return Ok(());
        };

        let remove_boundaries = h
            .hash_get("remove_boundaries")
            .and_then(|it| it.value().as_bool())
            .unwrap_or_default();

        let Some(begin) = h.hash_get("begin").and_then(|it| it.value().as_str()) else {
            warn!("`replace_section` helper require a 'begin' string value");
            return Ok(());
        };
        let Some((before, inner)) = input.split_once(begin) else {
            warn!("Begin region '{begin}' not found in '{input}'");
            return Ok(())
        };

        let Some(end) = h.hash_get("end").and_then(|it| it.value().as_str()) else {
            warn!("`replace_section` helper require a 'end' string value ");
            return Ok(());
        };
        let Some((_, after)) = inner.split_once(end) else {
            warn!("End region '{end}' not found in '{inner}'");
            return Ok(())
        };

        out.write(before)?;
        if !remove_boundaries {
            out.write(begin)?;
        }
        tmpl.render(hbs, ctx, rc, out)?;
        if !remove_boundaries {
            out.write(end)?;
        }
        out.write(after)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use serde_json::json;
    use similar_asserts::assert_eq;

    use crate::new_hbs;

    const INPUT: &str = r#"
<!DOCTYPE html>
<html lang="en">
    <head>
    <meta charset="UTF-8" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Document</title>
    </head>
    <body>
    <!-- #region head-->
    Something by default
    <!-- #endregion head -->
    </body>
</html>
"#;

    #[test]
    fn test_helper_replace_section() -> Result<(), Box<dyn Error>> {
        let data = json!({
            "content": INPUT,
        });
        let mut hbs = new_hbs();
        hbs.register_template_string(
            "test",
            r#"{{#replace_section  begin="<!-- #region head-->" end="<!-- #endregion head -->" content }}

    This is the new content of the block
    {{/replace_section}}"#,
        )?;

        let result = hbs.render("test", &data)?;

        assert_eq!(
            result,
            r#"
<!DOCTYPE html>
<html lang="en">
    <head>
    <meta charset="UTF-8" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Document</title>
    </head>
    <body>
    <!-- #region head-->
    This is the new content of the block
    <!-- #endregion head -->
    </body>
</html>
"#,
        );

        Ok(())
    }

    #[test]
    fn test_helper_replace_section_remove_remove_boundaries() -> Result<(), Box<dyn Error>> {
        let data = json!({
            "content": INPUT,
        });
        let mut hbs = new_hbs();
        hbs.register_template_string(
            "test",
            r#"{{~#replace_section  begin="<!-- #region head-->" end="<!-- #endregion head -->" remove_boundaries=true content }}
This is the new content of the block
{{~/replace_section}}"#,
        )?;

        let result = hbs.render("test", &data)?;

        assert_eq!(
            result,
            r#"
<!DOCTYPE html>
<html lang="en">
    <head>
    <meta charset="UTF-8" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Document</title>
    </head>
    <body>
    This is the new content of the block
    </body>
</html>
"#,
        );

        Ok(())
    }
}
