use std::error::Error;
use std::sync::Mutex;
pub mod types;
use types::*;

use reqwest::Client;
use rusqlite::Connection;
use select::document::Document;
use select::predicate::Attr;
use std::time::Duration;

use crate::db;

const LIBGEN_URL: &str = "https://libgen.is/search.php";
const LIBGEN_API_URL: &str = "https://libgen.is/json.php";

type Result<T> = ::std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug)]
pub struct LibgenClient(Client);

impl LibgenClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("could not build libgen client");

        Self(client)
    }

    pub async fn search(&self, query: Search, limit: usize) -> Result<Vec<String>> {
        let res = self
            .0
            .get(LIBGEN_URL)
            .query(&query.search_params())
            .send()
            .await?;

        let html = res.text().await?;
        let doc = Document::from(html.as_str());

        let ids = doc
            .find(Attr("valign", "top"))
            .skip(1)
            .take(limit)
            .filter_map(|n| n.descendants().nth(0).map(|x| x.text()))
            .collect();

        Ok(ids)
    }

    pub async fn get_ids(&self, ids: Vec<String>) -> Result<Vec<Book>> {
        let books = self
            .0
            .get(LIBGEN_API_URL)
            .query(&[
                ("fields", "id,title,author,year,extension,md5"),
                ("ids", &ids.join(",")),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(books)
    }

    pub async fn get_books(&self, query: Search, limit: usize) -> Result<Vec<Book>> {
        let ids = self.search(query, limit).await?;
        if !ids.is_empty() {
            return self.get_ids(ids).await;
        }

        Ok(vec![])
    }
}

#[derive(Debug)]
pub struct Utils {
    pub client: LibgenClient,
    pub db: Mutex<Connection>,
}

impl Utils {
    pub fn new(db_path: String) -> Self {
        let conn = db::get_db(&db_path).expect("cannot open database.");

        Utils {
            client: LibgenClient::new(),
            db: Mutex::new(conn),
        }
    }

    pub fn register(&self, chat_id: i64, message_id: i32, atype: &str) -> Result<()> {
        let lock = self.db.lock().unwrap();
        db::register(&lock, chat_id, message_id, atype).map_err(|e| e.to_string())?;

        Ok(())
    }
}
