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
                vec![("req".into(), author), ("column".into(), "author".into())]
            }
            Search::Isbn(isbn) => {
                vec![("req".into(), isbn), ("column".into(), "identifier".into())]
            }
            Search::Title(title) => {
                vec![("req".into(), title), ("column".into(), "title".into())]
            }
            Search::Default(text) => {
                vec![("req".into(), text), ("column".into(), "def".into())]
            }
        };
        q.push(("view".into(), "simple".into()));
        q.push(("res".into(), "25".into()));
        q.push(("open".into(), "0".into()));

        q
    }
}
