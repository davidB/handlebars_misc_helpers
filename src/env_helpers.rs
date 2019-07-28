use handlebars::HelperDef;
use handlebars::{handlebars_helper, Handlebars};

fn env_var_fct<T: AsRef<str>>(key: T) -> String {
    match std::env::var(key.as_ref()) {
        Ok(s) => s,
        Err(e) => {
            //TODO better error handler
            //use slog::warn;
            //warn!(ctx.logger, "helper: http_get"; "url" => format!("{:?}", url), "err" => format!("{:?}", e))
            eprintln!(
                "helper: env_var failed for key '{:?}' with error '{:?}'",
                key.as_ref(),
                e
            );
            "".to_owned()
        }
    }
}

pub fn register(handlebars: &mut Handlebars) -> Vec<Box<dyn HelperDef + 'static>> {
    vec![{
        handlebars_helper!(env_var: |v: str| env_var_fct(&v));
        handlebars.register_helper("env_var", Box::new(env_var))
    }]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use crate::tests::assert_helpers;
    use std::error::Error;

    #[test]
    fn test_register_env_helpers() -> Result<(), Box<Error>> {
        let key = "KEY";
        std::env::set_var(key, "VALUE");

        assert_helpers(key, vec![("env_var", "VALUE")])?;
        assert_helpers("A_DO_NOT_EXIST_ENVVAR", vec![("env_var", "")])?;
        Ok(())
    }
}
