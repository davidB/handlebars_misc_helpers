use handlebars::{handlebars_helper, Handlebars};

fn env_var_fct<T: AsRef<str>>(key: T) -> String {
    let key = key.as_ref();
    match key {
        "ARCH" => std::env::consts::ARCH.to_owned(),
        "DLL_EXTENSION" => std::env::consts::DLL_EXTENSION.to_owned(),
        "DLL_PREFIX" => std::env::consts::DLL_PREFIX.to_owned(),
        "DLL_SUFFIX" => std::env::consts::DLL_SUFFIX.to_owned(),
        "EXE_EXTENSION" => std::env::consts::EXE_EXTENSION.to_owned(),
        "EXE_SUFFIX" => std::env::consts::EXE_SUFFIX.to_owned(),
        "FAMILY" => std::env::consts::FAMILY.to_owned(),
        "OS" => std::env::consts::OS.to_owned(),
        "USERNAME" => std::env::var("USERNAME")
            .or_else(|_| std::env::var("username"))
            .or_else(|_| std::env::var("USER"))
            .or_else(|_| std::env::var("user"))
            .unwrap_or_else(|_| "noname".to_owned()),
        _ => {
            match std::env::var(key) {
                Ok(s) => s,
                Err(e) => {
                    //TODO better error handler
                    log::info!(
                        "helper: env_var failed for key '{:?}' with error '{:?}'",
                        key,
                        e
                    );
                    "".to_owned()
                }
            }
        }
    }
}

pub fn register<'reg>(handlebars: &mut Handlebars<'reg>) {
    handlebars_helper!(env_var: |v: str| env_var_fct(&v));
    handlebars.register_helper("env_var", Box::new(env_var))
}

#[cfg(test)]
mod tests {
    use crate::tests::assert_helpers;
    use std::error::Error;

    #[test]
    fn test_register_env_helpers() -> Result<(), Box<dyn Error>> {
        let key = "KEY";
        std::env::set_var(key, "VALUE");

        assert_helpers(key, vec![("env_var", "VALUE")])?;
        assert_helpers("A_DO_NOT_EXIST_ENVVAR", vec![("env_var", "")])?;
        Ok(())
    }

    #[test]
    fn test_env_consts() -> Result<(), Box<dyn Error>> {
        let key = "OS";
        let os = std::env::consts::OS;
        assert_ne!(os, "");
        assert_helpers(key, vec![("env_var", os)])?;
        Ok(())
    }
}
