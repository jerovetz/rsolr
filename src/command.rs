use http::StatusCode;
use url;
use mockall_double::double;
use reqwest::blocking::Response;
use serde_json::Value;
use crate::error::RSCError;

#[double]
use crate::http_client::HttpClient;

pub struct Command<'b> {
    request_handler: &'b str,
    url: url::Url,
}

impl<'b> Command<'b> {

    pub fn new(base_url: &'b str, collection: &'b str) -> Self {
        let mut url = url::Url::parse(base_url).unwrap();
        url.path_segments_mut().unwrap().push("solr");
        url.path_segments_mut().unwrap().push(collection);
        Command { request_handler: "", url }
    }

    pub fn add_query_param(&mut self, key: &str, value: &str) -> &mut Self {
        self.url.query_pairs_mut().append_pair(key, value);
        self
    }

    pub fn request_handler(&mut self, handler: &'b str) -> &mut Self {
        self.request_handler = handler;
        self.url.path_segments_mut().unwrap().push(self.request_handler);
        self
    }

    pub fn auto_commit(&mut self) -> &mut Self {
        self.add_query_param("commit", "true");
        self
    }

    pub fn query(&mut self, query: &str) -> &mut Self {
        self.add_query_param("q", query);
        self
    }

    pub fn generate_url_str(&'b self) -> &'b str {
        self.url.as_str()
    }

    pub fn run(&'b self) -> Result<Value, RSCError> {
        let solr_result = HttpClient::new().get(self.generate_url_str());

        let response = match solr_result {
            Ok(response) => response,
            Err(e) => return Err(RSCError { source: Some(Box::new(e)), status: None, message: None }),
        };

        self.handle_response(response)
    }

    pub fn run_with_body(&'b self, body: Option<Value>) -> Result<Value, RSCError> {
        println!("{:?}", self.generate_url_str());
        let solr_result = HttpClient::new().post(self.generate_url_str(), body);

        let response = match solr_result {
            Ok(response) => response,
            Err(e) => return Err(RSCError { source: Some(Box::new(e)), status: None, message: None }),
        };

        self.handle_response(response)
    }

    fn handle_response(&self, response: Response) -> Result<Value, RSCError> {
        match response.status() {
            StatusCode::OK => Ok(response.json::<Value>().unwrap()["response"]["docs"].clone()),
            StatusCode::NOT_FOUND => return Err(RSCError { source: None, status: Some(StatusCode::NOT_FOUND), message: None }),
            other_status => {
                let body_text = response.text().unwrap();
                let message_string = match serde_json::from_str::<Value>(&body_text) {
                    Ok(r) => r["error"]["msg"].to_string(),
                    Err(e) => {
                        return Err(
                            RSCError {
                                source: Some(Box::new(e)),
                                status: Some(other_status),
                                message: Some(body_text)
                            })
                    }
                };
                return Err(RSCError { source: None, status: Some(other_status), message: Some(message_string.replace("\"", "")) })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Mutex, MutexGuard};
    use mockall::lazy_static;
    use mockall::predicate::eq;
    use serde_json::json;

    lazy_static! {
        static ref MTX: Mutex<()> = Mutex::new(());
    }

    fn get_lock(m: &'static Mutex<()>) -> MutexGuard<'static, ()> {
        match m.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    #[test]
    fn test_build_a_url_from_parameters() {
        let mut params = Command::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .query("*:*");

        let url_string = params.generate_url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?q=*%3A*");
    }

    #[test]
    fn test_build_a_url_from_parameters_set_autocommit() {
        let mut params = Command::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .auto_commit();

        let url_string = params.generate_url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?commit=true");
    }

    #[test]
    fn test_run_calls_get_with_url() {
        let _m = get_lock(&MTX);

        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get()
                .with(eq("http://localhost:8983/solr/default/select?q=*%3A*"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{"response": {"docs": [{"success": true}]}}"#)
                    .unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut command = Command::new(host, collection);
        let result = command
            .request_handler("select")
            .query("*:*")
            .run();
        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0]["success"], true);
    }

    #[test]
    fn test_run_calls_post_with_url_and_body() {
        let _m = get_lock(&MTX);

        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post()
                .withf(| url, body | url == "http://localhost:8983/solr/default/update%2Fjson%2Fdocs?commit=true" && *body == Some(json!({ "this is": "a document"})) )
                .returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{"response": {"docs": [{"success": true}]}}"#)
                    .unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut command = Command::new(host, collection);
        let result = command
            .request_handler("update/json/docs")
            .auto_commit()
            .run_with_body(Some(json!({ "this is": "a document"})));
        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0]["success"], true);
    }
}