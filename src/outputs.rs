use handlebars::Output;
use std::string::FromUtf8Error;

/// Copy of the (private) StringOutput from handlebars
pub struct StringOutput {
    buf: Vec<u8>,
}

impl Output for StringOutput {
    fn write(&mut self, seg: &str) -> Result<(), std::io::Error> {
        self.buf.extend_from_slice(seg.as_bytes());
        Ok(())
    }
}

impl StringOutput {
    pub fn new() -> StringOutput {
        StringOutput {
            buf: Vec::with_capacity(8 * 1024),
        }
    }

    pub fn into_string(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.buf)
    }
}
