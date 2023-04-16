mod error;

use reqwest::blocking::Client as HttpClient;
use crate::error::RSCError;

pub struct Client {
    host: String,
    collection: String
}

impl Client {
    pub fn query(&self, query: &str) -> Result<serde_json::Value, RSCError> {
        let http_client = HttpClient::new();
        let solr_result =  http_client
            .get(format!("{}/solr/{}/select?q={}", self.host, self.collection, query))
            .send();
        let raw_response = match solr_result {
            Ok(response) => response,
            Err(e) => return Err(RSCError { source: Box::new(e) }),
        };

        Ok(raw_response.json::<serde_json::Value>().unwrap()
            .get("response").unwrap().get("docs").unwrap().clone())
    }

    pub fn new(url : &str, collection : &str) -> Client {
        let host = String::from(url);
        let collection = String::from(collection);
        Client {
            host,
            collection
        }
    }
}