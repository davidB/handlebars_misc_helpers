use failure::Error;
use handlebars::{handlebars_helper, Handlebars};
use std::path::{Path, PathBuf};

fn expand(s: &str) -> PathBuf {
    let p = PathBuf::from(s);
    // canonicalize to be able to extract file_name, parent, extension from path like '.'
    // without requested template author to call canonicalize in every place
    if p.exists() {
        p.canonicalize().unwrap_or(p)
    } else {
        p
    }
}

pub fn register(handlebars: &mut Handlebars) -> Result<(), Error> {
    handlebars_helper!(parent: |v: str| {
        expand(&v).parent().and_then(|s| s.to_str()).unwrap_or("").to_owned()
    });
    handlebars.register_helper("parent", Box::new(parent));

    handlebars_helper!(file_name: |v: str| {
        expand(&v).file_name().and_then(|s| s.to_str()).unwrap_or("").to_owned()
    });
    handlebars.register_helper("file_name", Box::new(file_name));

    handlebars_helper!(extension: |v: str| expand(&v).extension().and_then(|s| s.to_str()).unwrap_or("").to_owned());
    handlebars.register_helper("extension", Box::new(extension));

    handlebars_helper!(canonicalize: |v: str| {
        Path::new(v).canonicalize().ok().and_then(|s| s.to_str().map(|v| v.to_owned())).unwrap_or_else(|| "".into())
    });
    handlebars.register_helper("canonicalize", Box::new(canonicalize));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::assert_helpers;

    #[test]
    fn test_register_path_helpers() -> Result<(), Error> {
        assert_helpers(
            "/hello/bar/foo",
            vec![
                ("file_name", "foo"),
                ("parent", "/hello/bar"),
                ("extension", ""),
                ("canonicalize", ""),
            ],
        )?;
        assert_helpers(
            "foo",
            vec![("file_name", "foo"), ("parent", ""), ("extension", "")],
        )?;
        assert_helpers(
            "bar/foo",
            vec![("file_name", "foo"), ("parent", "bar"), ("extension", "")],
        )?;
        assert_helpers(
            "bar/foo.txt",
            vec![
                ("file_name", "foo.txt"),
                ("parent", "bar"),
                ("extension", "txt"),
            ],
        )?;
        assert_helpers(
            "./foo",
            vec![
                ("file_name", "foo"),
                ("parent", "."),
                ("extension", ""),
                ("canonicalize", ""),
            ],
        )?;
        assert_helpers(
            "/hello/bar/../foo",
            vec![
                ("file_name", "foo"),
                ("parent", "/hello/bar/.."),
                ("extension", ""),
                ("canonicalize", ""),
            ],
        )?;
        Ok(())
    }
}
