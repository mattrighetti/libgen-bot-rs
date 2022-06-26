use std::fmt::{Display, self};
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub year: String,
    pub extension: String,
    pub md5: String,
}

impl Book {
    pub fn pretty(&self) -> String {
        format!(
            "<b>{}</b>\n\n\
            👤 {}\n\
            Format: {}\n",
            self.title,
            self.author,
            self.extension
        )
    }

    pub fn pretty_with_index(&self, index: usize) -> String {
        format!(
            "{}. <b>{}</b>\n\
            👤 {}\n\
            Year: {}, Type: {}\n",
            index,
            self.title,
            self.author,
            self.year,
            self.extension
        )
    }

    pub fn md5_url(&self) -> String {
        format!("http://libgen.is/get.php?md5={}", self.md5)
    }
}

impl Display for Book {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Book({}, {}, {})", self.title, self.author, self.md5)
    }
}