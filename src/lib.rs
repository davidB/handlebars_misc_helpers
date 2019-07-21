#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn test() {
        assert_that!(&true).is_equal_to(&true);
    }
}