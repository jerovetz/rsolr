pub mod error;
mod http_client;
use mockall_double::double;

#[double]
use http_client::HttpClient;

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
            .get(&format!("{}/solr/{}/select?q={}", &self.host, &self.collection, &query));
        let raw_response = match solr_result {
            Ok(response) => response,
            Err(e) => return Err(RSCError { source: Some(Box::new(e)), status: None, message: None }),
        };
        let response_status = raw_response.status();
        if response_status == StatusCode::NOT_FOUND {
            return Err(RSCError { source: None, status: Some(response_status), message: None })
        };

        if response_status != StatusCode::OK {
            let message_string = match raw_response.json::<Value>() {
                Ok(r) => r["error"]["msg"].to_string(),
                Err(e) => {
                    return Err(
                        RSCError {
                            source: Some(Box::new(e)),
                            status: Some(response_status),
                            message: Some(String::from("Unparseable body."))
                        })
                }
            };

            return Err(RSCError { source: None, status: Some(response_status), message: Some(message_string.replace("\"", "")) })
        }

        Ok(raw_response.json::<Value>().unwrap()
            .get("response").unwrap().get("docs").unwrap().clone())
    }

    pub fn new(host : &'a str, collection : &'a str) -> Self {
        Self {
            host,
            collection
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_responds_rsc_error_with_other_problem_if_dunno() {
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get().returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"{"error": {"code": 500, "msg": "okapi"}}"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection).query("bad: query");
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);

        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_query_responds_rsc_error_with_raw_text_body_and_status_code_if_no_standard_message() {
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get().returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"some unparseable thing"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection).query("bad: query");
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);

        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

}