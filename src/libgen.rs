extern crate reqwest;
extern crate select;
extern crate serde;

use core::fmt;

use serde::{Deserialize};
use reqwest::Client;
use select::document::Document;
use select::predicate::Attr;

const LIBGEN_URL: &str = "https://libgen.is/search.php";
const LIBGEN_API_URL: &str = "https://libgen.is/json.php";

pub struct Utils {
    pub client: Client,
}

impl Utils {
    pub fn new() -> Self {
        Utils { client: Client::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Search {
    ISBN(String),
    Author(String),
    Title(String),
    Default(String)
}

impl Search {
    pub fn from(query: &str) -> Self {
        if query.starts_with("!isbn") {
            return Search::ISBN(query.to_string());
        } else if query.starts_with("!author") {
            return Search::Author(query.to_string());
        } else {
            return Search::Title(query.to_string());
        }
    }

    pub fn search_params(self) -> Vec<(String, String)> {
        let mut q = match self {
            Search::Author(author) => {
                vec![("req".to_string(), author), ("column".to_string(), "author".to_string())]
            }
            Search::ISBN(isbn) => {
                vec![("req".to_string(), isbn), ("column".to_string(), "identifier".to_string())]
            }
            Search::Title(title) => {
                vec![("req".to_string(), title), ("column".to_string(), "title".to_string())]
            }
            Search::Default(text) => {
                vec![("req".to_string(), text), ("column".to_string(), "def".to_string())]
            }
        };
        q.push(("view".into(), "simple".into()));
        q.push(("res".into(), "25".into()));
        q.push(("open".into(), "0".into()));

        q
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub year: String,
    pub md5: String
}

impl Book {
    pub fn pretty(&self) -> String {
        format!(
            "{}\n\
            👤 {}\n",
            self.title,
            self.author
        )
    }

    pub fn pretty_with_index(&self, index: usize) -> String {
        format!(
            "{}. {}\n\
            👤 {}\n",
            index,
            self.title,
            self.author
        )
    }
}

impl std::fmt::Display for Book {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Book({}, {}, {})", self.title, self.author, self.md5)
    }
}

pub async fn search(client: &Client, query: Search, limit: usize) -> Vec<u32> {
    let res = client.get(LIBGEN_URL).query(&query.search_params()).send().await.unwrap();
    let text = res.text().await.unwrap();
    let cursor = std::io::Cursor::new(text.as_str());
    let doc = Document::from_read(cursor).unwrap();
    
    let mut ids = Vec::new();
    for node in doc.find(Attr("valign", "top")).skip(1).take(limit) {
        let id = node.descendants().take(1).next().unwrap().text();
        
        match id.parse::<u32>() {
            Ok(id) => { ids.push(id) }
            Err(_) => {}
        }
    }

    ids
}

pub async fn get_ids(client: &Client, ids: Vec<u32>) -> Vec<Book> {
    assert!(ids.len() > 0);
    let ids = ids.iter()
        .map(|z| z.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let params: Vec<(String, String)> = vec![
        ("fields".into(), "title,author,year,md5".into()),
        ("ids".into(), ids)
    ];

    let res = client.get(LIBGEN_API_URL).query(&params).send().await.unwrap();
    res.json::<Vec<Book>>().await.unwrap()
}

pub async fn get_books(client: &Client, query: Search, limit: usize) -> Vec<Book> {
    let ids = search(client, query, limit).await;
    if ids.len() > 0 {
        return get_ids(client, ids).await
    }

    vec![]
}

#[cfg(test)]
mod test {
    use crate::Search;
    use crate::{search, get_ids, get_books};

    #[tokio::test]
    async fn test_search_invalid_def() {
        let client = reqwest::Client::new();
        let query = Search::Default("Ahaha".into());

        let ids = search(&client, query, 5).await;

        assert_eq!(ids.len(), 0);
    }

    #[tokio::test]
    async fn test_search_title() {
        let client = reqwest::Client::new();
        let query = Search::Title("Rust Programming".into());

        let ids = search(&client, query, 25).await;
        let expected: Vec<u32> = vec![1486260, 1527378, 1729710, 2158512, 2167798];

        assert_eq!(ids.len(), 25);

        let mut trunc_ids = ids.iter().take(5);
        assert_eq!(trunc_ids.next(), Some(&expected[0]));
        assert_eq!(trunc_ids.next(), Some(&expected[1]));
        assert_eq!(trunc_ids.next(), Some(&expected[2]));
        assert_eq!(trunc_ids.next(), Some(&expected[3]));
        assert_eq!(trunc_ids.next(), Some(&expected[4]));
    }

    #[tokio::test]
    async fn test_search_author() {
        let client = reqwest::Client::new();
        let query = Search::Author("Orendorff".into());

        let ids = search(&client, query, 5).await;
        let expected: Vec<u32> = vec![1486260, 1527378, 2158512, 2167798, 2917089];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_search_default() {
        let client = reqwest::Client::new();
        let query = Search::Default("Rust Programming".into());

        let ids = search(&client, query, 5).await;
        let expected: Vec<u32> = vec![349771, 1486260, 1527378, 1729710, 1980775];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_search_isbn() {
        let client = reqwest::Client::new();
        let query = Search::ISBN("978-0132350884".into());

        let ids = search(&client, query, 5).await;
        let expected: Vec<u32> = vec![207243, 1489412, 1525091, 2228027, 2324753];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_search_empty_isbn() {
        let client = reqwest::Client::new();
        let query = Search::ISBN("1823499234".into());

        let ids = search(&client, query, 5).await;
        let expected: Vec<u32> = vec![];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_get_ids() {
        let client = reqwest::Client::new();
        let result = get_ids(&client, vec![349771, 1486260, 1527378, 1729710, 1980775]).await;
        
        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_get_ids_max() {
        let client = reqwest::Client::new();
        let result = get_ids(&client, vec![349771, 1486260, 1527378, 1729710, 1980775, 349771, 349771, 349771]).await;
        
        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_invalid_get_books_def() {
        let client = reqwest::Client::new();
        let query = Search::Default("Ahaha".into());

        let books = get_books(&client, query, 5).await;

        assert_eq!(books.len(), 0);
    }
}