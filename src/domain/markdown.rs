use markdown::to_html;
use std::fs;
use std::io;
use std::path::Path;

pub struct Markdown {
    content: String,
}

impl Markdown {
    #[must_use]
    pub fn as_html(&self) -> String {
        to_html(&self.content)
    }
    /// # Errors
    /// Will return Err if the file content cannot be read.
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(Self { content })
    }
}
