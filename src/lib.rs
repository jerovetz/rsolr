pub mod error;
mod http_client;
mod command;

use mockall_double::double;
use reqwest::blocking::Response;

#[double]
use http_client::HttpClient;

use reqwest::StatusCode;
use serde_json::{json, Value};
use crate::error::RSCError;
use crate::command::Command;

#[derive(PartialEq)]
pub enum AutoCommit {
    YES,
    NO
}

pub struct Client<'a> {
    host: &'a str,
    collection: &'a str,
    http_client: HttpClient,
    auto_commit: AutoCommit
}

impl<'a> Client<'a> {

    fn handle_response(&self, status: StatusCode, raw_response: Response) -> Result<Value, RSCError> {
        match status {
            StatusCode::OK => Ok(raw_response.json::<Value>().unwrap()["response"]["docs"].clone()),
            StatusCode::NOT_FOUND => return Err(RSCError { source: None, status: Some(StatusCode::NOT_FOUND), message: None }),
            other_status => {
                let body_text = raw_response.text().unwrap();
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

    pub fn query(&self, query: &str) -> Result<Value, RSCError> {
        let mut command = Command::new(&self.host, &self.collection);
        command
            .request_handler("select")
            .query(query)
            .run()
    }

    pub fn create(&self, document: Value) -> Result<(), RSCError> {
        let mut command = Command::new(&self.host, &self.collection);
        command
            .request_handler("update/json/docs");

        if AutoCommit::YES == self.auto_commit {
            command.auto_commit();
        }

        let response_or_error = self.http_client.post(command.get_url(), Some(document));
        let response = match response_or_error {
            Ok(r) => r,
            Err(e) => return Err(RSCError { source: Some(Box::new(e)), status: None, message: None }),
        };

        self.handle_response(response.status(), response).map(|_| { () })
    }

    pub fn commit(&self) -> Result<(), RSCError> {
        let mut command = Command::new(&self.host, &self.collection);
        command
            .request_handler("update")
            .auto_commit();

        let _ = self.http_client.post(command.get_url(), None);
        Ok(())
    }

    pub fn delete(&self, query: &str) -> Result<(), RSCError> {
        let delete_payload = Some(json!({
            "delete": { "query": query }
        }));

        let mut command = Command::new(&self.host, &self.collection);
        command
            .request_handler("update");

        if AutoCommit::YES == self.auto_commit {
            command.auto_commit();
        }

        let response_or_error = self.http_client.post(command.get_url(), delete_payload);
        let response = match response_or_error {
            Ok(r) => r,
            Err(e) => return Err(RSCError { source: Some(Box::new(e)), status: None, message: None }),
        };

        self.handle_response(response.status(), response).map(|_| { () })
    }

    pub fn new(host : &'a str, collection : &'a str, auto_commit: AutoCommit) -> Self {
        Self {
            host,
            collection,
            http_client: HttpClient::new(),
            auto_commit
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Mutex, MutexGuard};
    use mockall::lazy_static;

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
    fn test_query_responds_rsc_error_with_other_problem_if_dunno() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();

        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get().returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"{"error": {"code": 500, "msg": "okapi"}}"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection, AutoCommit::NO).query("bad: query");
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "okapi");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_query_responds_rsc_error_with_raw_text_body_and_status_code_if_no_standard_message() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get().returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"some unparseable thing"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection, AutoCommit::NO).query("bad: query");
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "some unparseable thing");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_create_responds_rsc_error_with_other_problem_if_dunno() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"{"error": {"code": 500, "msg": "okapi"}}"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection, AutoCommit::NO)
            .create(serde_json::from_str(r#"{"anything": "anything"}"#).unwrap());
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "okapi");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_create_responds_rsc_error_with_raw_text_body_and_status_code_if_no_standard_message() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"some unparseable thing"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection, AutoCommit::NO)
            .create(serde_json::from_str(r#"{"anything": "anything"}"#).unwrap());
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "some unparseable thing");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_delete_responds_rsc_error_with_other_problem_if_dunno() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"{"error": {"code": 500, "msg": "okapi"}}"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection, AutoCommit::NO)
            .delete("*:*");
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "okapi");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_delete_responds_rsc_error_with_raw_text_body_and_status_code_if_no_standard_message() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"some unparseable thing"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection, AutoCommit::NO)
            .delete("*:*");
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "some unparseable thing");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

}