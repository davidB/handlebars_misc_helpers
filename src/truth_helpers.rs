use handlebars::handlebars_helper;
use handlebars::Handlebars;
use handlebars::HelperDef;

pub fn register(handlebars: &mut Handlebars) -> Vec<Box<dyn HelperDef + 'static>> {
    vec![
        {
            handlebars_helper!(eq: |x: Json, y: Json| x == y);
            handlebars.register_helper("eq", Box::new(eq))
        },
        {
            handlebars_helper!(ne: |x: Json, y: Json| x != y);
            handlebars.register_helper("ne", Box::new(ne))
        },
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    // use super::*;
    use crate::tests::assert_renders;
    // use spectral::prelude::*;
    use std::error::Error;

    #[test]
    fn test_helper_eq() -> Result<(), Box<Error>> {
        assert_renders(vec![
            (r##"{{ eq 2 3 }}"##, r##"false"##),
            (r##"{{ eq 42 42 }}"##, r##"true"##),
            (r##"{{ eq true false }}"##, r##"false"##),
            (r##"{{ eq false false }}"##, r##"true"##),
            (r##"{{ eq "foo" "bar" }}"##, r##"false"##),
            (r##"{{ eq "foo" "foo" }}"##, r##"true"##),
            (r##"{{ eq "foo" "FOO" }}"##, r##"false"##),
            (r##"{{ eq [ 2, 3] [ 2 ] }}"##, r##"false"##),
            (r##"{{ eq [ 2, 3] [ 2, 3 ] }}"##, r##"true"##),
        ])?;
        Ok(())
    }

    #[test]
    fn test_helper_not() -> Result<(), Box<Error>> {
        assert_renders(vec![
            (r##"{{ not false }}"##, r##"true"##),
            (r##"{{ not true }}"##, r##"false"##),
            (r##"{{ not (eq 2 3) }}"##, r##"true"##),
            (r##"{{ not (eq 42 42) }}"##, r##"false"##),
        ])?;
        Ok(())
    }

    #[test]
    fn test_helper_ne() -> Result<(), Box<Error>> {
        assert_renders(vec![
            (r##"{{ ne 2 3 }}"##, r##"true"##),
            (r##"{{ ne 42 42 }}"##, r##"false"##),
            (r##"{{ ne true false }}"##, r##"true"##),
            (r##"{{ ne false false }}"##, r##"false"##),
            (r##"{{ ne "foo" "bar" }}"##, r##"true"##),
            (r##"{{ ne "foo" "foo" }}"##, r##"false"##),
            (r##"{{ ne "foo" "FOO" }}"##, r##"true"##),
        ])?;
        Ok(())
    }

    #[test]
    fn test_if_eq_string() -> Result<(), Box<Error>> {
        assert_renders(vec![(
            r##"{{#if (eq "OK" "OK")}}OK{{ else }}KO{{/if}}"##,
            r##"OK"##,
        )])?;
        Ok(())
    }

    // #[test]
    // fn test_helper_gt() -> Result<(), Box<Error>> {
    //     assert_renders(vec![
    //         (r##"{{ gt 2 3 }}"##, r##"false"##),
    //         (r##"{{ gt 3 2 }}"##, r##"true"##),
    //         (r##"{{ gt 42 42 }}"##, r##"false"##),
    //         (r##"{{ gt "foo" "boo" }}"##, r##"false"##),
    //         (r##"{{ gt "foo" "bar" }}"##, r##"false"##),
    //         (r##"{{ gt "bar" "foo}}"##, r##"true"##),
    //     ])?;
    //     Ok(())
    // }
}
