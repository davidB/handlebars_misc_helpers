use handlebars::HelperDef;
use handlebars::{handlebars_helper, Handlebars};
use std::path::Path;

pub fn register(handlebars: &mut Handlebars) -> Vec<Box<dyn HelperDef + 'static>> {
    vec![{
        handlebars_helper!(read_to_str: |v: str| {
            let p = Path::new(v);
            if p.exists() {
                dbg!(std::fs::read_to_string(p)?)
            } else {
                "".to_owned()
            }
        });
        handlebars.register_helper("read_to_str", Box::new(read_to_str))
    }]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use crate::assert_renders;
    use std::error::Error;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_to_str() -> Result<(), Box<dyn Error>> {
        // Create a file inside of `std::env::temp_dir()`.
        let file_content = "Brian was here. Briefly.";
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", file_content)?;
        assert_renders![
            (r##"{{ read_to_str "" }}"##, ""),
            (r##"{{ read_to_str "/file/not/exists" }}"##, ""),
            (
                &format!("{{{{ read_to_str {:?} }}}}", file.path()),
                file_content
            )
        ]
    }
}
