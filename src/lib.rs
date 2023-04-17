pub mod error;

use reqwest::blocking::Client as HttpClient;
use reqwest::StatusCode;
use serde_json::Value;
use crate::error::RSCError;

pub struct Client<'a> {
    host: &'a str,
    collection: &'a str,
}

impl<'a> Client<'a> {

    pub fn query(&self, query: &str) -> Result<Value, RSCError> {
        let http_client = HttpClient::new();
        let solr_result =  http_client
            .get(format!("{}/solr/{}/select?q={}", self.host, self.collection, query))
            .send();
        let raw_response = match solr_result {
            Ok(response) => response,
            Err(e) => return Err(RSCError { source: Some(Box::new(e)), status: None, message: None }),
        };
        let response_status = raw_response.status();


        if response_status == StatusCode::NOT_FOUND {
            return Err(RSCError { source: None, status: Some(raw_response.status()), message: None })
        };

        let response_body: Value  = raw_response.json::<Value>().unwrap();

        if response_status == StatusCode::BAD_REQUEST {
            let message_string = response_body.get("error").unwrap().get("msg").unwrap().to_string();
            return Err(RSCError { source: None, status: Some(response_status), message: Some(message_string.replace("\"", "")) })
        }
        Ok(response_body
            .get("response").unwrap().get("docs").unwrap().clone())
    }

    pub fn new(host : &'a str, collection : &'a str) -> Client<'a> {
        Client {
            host,
            collection
        }
    }
}