#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Search {
    Isbn(String),
    Author(String),
    Title(String),
    Default(String),
}

impl Search {
    pub fn search_params(self) -> Vec<(String, String)> {
        let mut q = match self {
            Search::Author(author) => {
                vec![
                    ("req".to_string(), author),
                    ("column".to_string(), "author".to_string()),
                ]
            }
            Search::Isbn(isbn) => {
                vec![
                    ("req".to_string(), isbn),
                    ("column".to_string(), "identifier".to_string()),
                ]
            }
            Search::Title(title) => {
                vec![
                    ("req".to_string(), title),
                    ("column".to_string(), "title".to_string()),
                ]
            }
            Search::Default(text) => {
                vec![
                    ("req".to_string(), text),
                    ("column".to_string(), "def".to_string()),
                ]
            }
        };
        q.push(("view".into(), "simple".into()));
        q.push(("res".into(), "25".into()));
        q.push(("open".into(), "0".into()));

        q
    }
}

