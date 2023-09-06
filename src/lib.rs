//! A Solr client for Rust.
//!
//! `Rsolr` provides capabilities to manipulate and form
//! requests to the Solr server, and contains some shorthands
//! for them. It uses the blocking version of the reqwest http client.
//!
//! ## Select
//!
//! You can retrieve documents as types with implemented `Clone` and `Deserialize`.
//!
//! ```rust
//! use serde_json::Value;
//! use rsolr::Client;
//! use rsolr::error::RSolrError;
//! use rsolr::solr_response::Response;
//!
//! fn query_all() -> Result<Response<Value>, RSolrError> {
//!     let mut client = Client::new("http://solr:8983", "collection");
//!     let result = client
//!         .select("*:*")
//!         .run();
//!
//!    match result {
//!         Ok(solr_result) => {
//!             let solr_result = client.get_response::<Value>();
//!             Ok(solr_result.expect("Serialization failed").response.expect("Response is OK, but no solr content"))
//!         }
//!         Err(e) => Err(e) // something happened on http
//!     }
//! }
//! ```
//!
//! ## Create
//!
//! You can use types with implemented `Clone` and `Serialize`.
//!
//! ```rust
//!
//! use serde::Serialize;
//! use serde_json::Value;
//! use rsolr::Client;
//!
//! #[derive(Serialize, Clone)]
//! struct SimpleDocument {
//!     field: Vec<String>
//! }
//!
//! fn create() {
//!     let document = SimpleDocument { field: vec!("nice".to_string(), "document".to_string()) };
//!     Client::new("http://solr:8983", "collection")
//!         .create(document)
//!         .run().expect("request failed.");
//! }
//! ```
//! ## Delete
//!
//! ```rust
//! use serde_json::Value;
//! use rsolr::Client;
//! fn delete() {
//!     Client::new("http://solr:8983", "collection")
//!         .delete("delete:query")
//!         .run().expect("request failed.");
//! }
//! ```
//!
//! ## Custom handler with params
//!
//! You can define any handlers as well.
//!
//! ```rust
//!
//! use serde_json::Value;
//! use rsolr::Client;
//! use rsolr::error::RSolrError;
//! use rsolr::solr_response::Response;
//! fn more_like_this()  -> Result<Response<Value>, RSolrError> {
//!     let mut client = Client::new("http://solr:8983", "collection");
//!     let result = client
//!         .request_handler("mlt")
//!         .add_query_param("mlt.fl", "similarity_field")
//!         .add_query_param("mlt.mintf", "4")
//!         .add_query_param("mlt.minwl", "3")
//!         .run();
//!     match result {
//!         Ok(solr_result) => Ok(client.get_response::<Value>().expect("Serialization failed").response.expect("No response")),
//!         Err(e) => Err(e)
//!     }
//! }
//! ```

pub mod error;
pub mod solr_response;
pub mod query;
mod facet_fields;
mod http_client;

use serde::{Deserialize, Serialize};
use http::StatusCode;
use url;
use mockall_double::double;
use serde_json::{json, Value};

#[double]
use http_client::HttpClient;
use crate::error::RSolrError;
use crate::solr_response::SolrResponse;


/// The Payload defines the request method. Body and Empty sets method to POST, None uses GET.
#[derive(Clone, Debug)]
pub enum Payload {
    Body(Value),
    Empty,
    None
}

#[non_exhaustive]
pub struct RequestHandlers;

impl RequestHandlers {
    pub const QUERY: &'static str = "select";
    pub const CREATE: &'static str = "update/json/docs";
    pub const DELETE: &'static str = "update";
}

pub struct Client<'a> {
    request_handler: &'a str,
    url: url::Url,
    payload: Payload,
    collection: &'a str,
    response: Option<Value>
}

impl<'a> Client<'a> {

    pub fn new(base_url: &'a str, collection: &'a str) -> Self {
        let url = url::Url::parse(base_url).unwrap();
        Client { request_handler: "", url, payload: Payload::None, collection, response: None }
    }

    /// Adds custom GET query parameter to the Solr query.
    pub fn add_query_param(&mut self, key: &str, value: &str) -> &mut Self {
        self.url.query_pairs_mut().append_pair(key, value);
        self
    }

    fn switch_on_facet(&mut self) {
        for query_pair in self.url.query_pairs() {
            if query_pair.0 == "facet" && query_pair.1 == "on" {
                return
            }
        }
        self.url.query_pairs_mut().append_pair("facet", "on");
    }

    /// Shorthand for facet_field.
    pub fn facet_field(&mut self, field: &str) -> &mut Self {
        self.switch_on_facet();
        self.url
            .query_pairs_mut()
            .append_pair("facet_field", field);

        self
    }

    /// Shorthand for facet_query.
    pub fn facet_query(&mut self, query: &str) -> &mut Self {
        self.switch_on_facet();
        self.url
            .query_pairs_mut()
            .append_pair("facet_query", query);
        self
    }

    /// Sets the Solr request handler in the URL. You can use RequestHandlers const, but it might be any string.
    pub fn request_handler(&mut self, handler: &'a str) -> &mut Self {
        self.request_handler = handler;
        self.payload = Payload::None;
        self.url.path_segments_mut().unwrap()
            .clear()
            .push("solr")
            .push(self.collection)
            .push(self.request_handler);
        self
    }
    /// Shorthand for commit=true, so if set write operations will be immediate.
    pub fn auto_commit(&mut self) -> &mut Self {
        self.add_query_param("commit", "true")
    }

    /// Shorthand for 'start' parameter of Solr basic pagination.
    pub fn start(&mut self, start: u32) -> &mut Self {
        self.add_query_param("start", &start.to_string())
    }

    /// Shorthand for 'rows' parameter of Solr basic pagination.
    pub fn rows(&mut self, rows: u32) -> &mut Self {
        self.add_query_param("rows", &rows.to_string())
    }

    /// Shorthand for 'q' parameter for setting query in the request.
    pub fn query(&mut self, query: &str) -> &mut Self {
        self.add_query_param("q", query)
    }

    /// Shorthand for 'df' parameter.
    pub fn default_field(&mut self, default_field: &str) -> &mut Self {
        self.add_query_param("df", default_field)
    }

    /// Generates the request url as string without sending.
    pub fn url_str(&self) -> &str {
        self.url.as_str()
    }

    /// Sets the payload of the request, only JSON is supported.
    pub fn set_document<P : Clone + Serialize>(&mut self, document: P) -> &mut Self {
        self.payload(Payload::Body(serde_json::to_value::<P>(document).unwrap()))
    }

    /// Empties the payload, it requires for POST requests (i.e. Solr delete or commit).
    pub fn set_empty_payload(&mut self) -> &mut Self {
        self.payload(Payload::Empty)
    }

    /// Clears the payload, now request method will be GET.
    pub fn clear_payload(&mut self) -> &mut Self {
        self.payload(Payload::None)
    }

    /// Runs the prepared request and fetches response to the type specified. Responds Result which contains SolrResult, the response part of Solr response.
    pub fn run(&mut self) -> Result<(), RSolrError> {
        let http_result = match self.payload.clone() {
            Payload::Body(body) => HttpClient::new().post(self.url_str(), Some(body)),
            Payload::Empty => HttpClient::new().post(self.url_str(), None),
            Payload::None => HttpClient::new().get(self.url_str())
        };

        let http_response = match http_result {
            Ok(response) => response,
            Err(e) => return Err(RSolrError { source: Some(Box::new(e)), status: None, message: None }),
        };

        match http_response.status() {
            StatusCode::OK => {
                self.response = http_response.json::<Value>().ok();
                self.url.query_pairs_mut().clear();
                Ok(())
            },
            StatusCode::NOT_FOUND => return Err(RSolrError { source: None, status: Some(StatusCode::NOT_FOUND), message: None }),
            other_status => {
                let body_text = http_response.text().unwrap();
                return match serde_json::from_str::<Value>(&body_text) {
                    Ok(r) => Err(RSolrError { source: None, status: Some(other_status), message: Some(r["error"]["msg"].as_str().unwrap().to_owned()) }),
                    Err(e) => {
                        Err(
                            RSolrError {
                                source: Some(Box::new(e)),
                                status: Some(other_status),
                                message: Some(body_text)
                            })
                    }
                }
            }
        }
    }

    pub fn get_response<T: for<'de> Deserialize<'de> + Clone>(&self) -> Result<SolrResponse<T>, RSolrError>{
        match self.response.clone() {
            Some(v) => match serde_json::from_value(v) {
                Ok(response) => Ok(response),
                Err(e) => Err(RSolrError{ source: Some(Box::new(e)), status: None, message: Some("Cannot deserialize response into object".to_owned()) })
            },
            _ => Ok(SolrResponse { response: None, facet_counts: None })
        }
    }

    /// Shorthand for query.
    pub fn select(&mut self, query: &str) -> &mut Self {
        self
            .request_handler(RequestHandlers::QUERY)
            .query(query)
    }

    /// Shorthand for create.
    pub fn create<P: Serialize + Clone>(&mut self, document: P) -> &mut Self {
        self
            .request_handler(RequestHandlers::CREATE)
            .set_document::<P>(document)
    }

    /// Shorthand for delete.
    pub fn delete(&mut self, query: &str) -> &mut Self {
        let delete_payload = json!({
            "delete": { "query": query }
        });

        self
            .request_handler(RequestHandlers::DELETE)
            .set_document(delete_payload)
    }

    /// Shorthand for direct commit.
    pub fn commit(&mut self) -> &mut Self {
        self
            .request_handler("update")
            .auto_commit()
            .set_empty_payload()
    }

    fn payload(&mut self, payload: Payload) -> &mut Self {
        self.payload = payload;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Mutex, MutexGuard};
    use mockall::lazy_static;
    use mockall::predicate::eq;
    use serde_json::json;
    use crate::error;

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
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .query("*:*");

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?q=*%3A*");
    }

    #[test]
    fn test_build_a_url_from_parameters_set_autocommit() {
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .auto_commit();

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?commit=true");
    }

    #[test]
    fn test_build_a_url_with_start_and_rows() {
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .start(135545)
            .rows(12);

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?start=135545&rows=12");
    }

    #[test]
    fn test_build_a_url_with_default_field() {
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .default_field("defaultfield");

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?df=defaultfield");
    }

    #[test]
    fn test_run_calls_get() {
        let _m = get_lock(&MTX);

        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get()
                .with(eq("http://localhost:8983/solr/default/select?q=*%3A*"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{"response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success": true }]}}"#)
                    .unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut command = Client::new(host, collection);
        let result = command
            .request_handler("select")
            .query("*:*")
            .run();
        assert!(result.is_ok());
        assert_eq!(command.get_response::<Value>().unwrap().response.unwrap().docs[0]["success"], true);
    }

    #[test]
    fn test_run_calls_get_with_single_facet() {
        let _m = get_lock(&MTX);

        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get()
                .with(eq("http://localhost:8983/solr/default/select?q=*%3A*&facet=on&facet_field=exists"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{
                            "response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success": true }]},
                            "facet_counts": {
                                "facet_queries": {},
                                "facet_fields": {
                                    "exists": [
                                        "term1", 23423, "term2", 993939
                                    ]
                                },
                                "facet_ranges":{},
                                "facet_intervals":{},
                                "facet_heatmaps":{}
                            }
                        }"#)
                    .unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut client = Client::new(host, collection);
        let result = client
            .request_handler("select")
            .query("*:*")
            .facet_field("exists")
            .run();
        assert!(result.is_ok());
        let facets = client.get_response::<Value>().unwrap().facet_counts.unwrap();
        assert_eq!(facets.facet_fields.fields, serde_json::from_str::<Value>(r#"{"exists":["term1", 23423,"term2",993939]}"#).unwrap());
    }

    #[test]
    fn test_run_calls_get_with_facet_query() {
        let _m = get_lock(&MTX);

        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get()
                .with(eq("http://localhost:8983/solr/default/select?q=*%3A*&facet=on&facet_query=anything%3A+*"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{
                            "response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success": true }]},
                            "facet_counts": {
                                "facet_queries": {
                                    "anything: *": 324534
                                },
                                "facet_fields": {},
                                "facet_ranges":{},
                                "facet_intervals":{},
                                "facet_heatmaps":{}
                            }
                        }"#)
                    .unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut command = Client::new(host, collection);
        let result = command
            .request_handler("select")
            .query("*:*")
            .facet_query("anything: *")
            .run();
        assert!(result.is_ok());
        let facets = command.get_response::<Value>().unwrap().facet_counts.unwrap();
        assert_eq!(facets.facet_queries, serde_json::from_str::<Value>(r#"{"anything: *": 324534 }"#).unwrap());
    }

    #[test]
    fn test_run_calls_get_with_facet_query_and_fields_with_a_single_facet_switch() {
        let _m = get_lock(&MTX);

        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get()
                .with(eq("http://localhost:8983/solr/default/select?q=*%3A*&facet=on&facet_query=anything%3A+*&facet_field=exists"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{
                            "response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success": true }]},
                            "facet_counts": {
                                "facet_queries": {
                                    "anything: *": 324534
                                },
                                "facet_fields": {
                                   "exists": [
                                        "term1", 23423, "term2", 993939
                                    ]
                                },
                                "facet_ranges":{},
                                "facet_intervals":{},
                                "facet_heatmaps":{}
                            }
                        }"#)
                    .unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut command = Client::new(host, collection);
        let result = command
            .request_handler("select")
            .query("*:*")
            .facet_query("anything: *")
            .facet_field("exists")
            .run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_calls_get_with_facet_query_and_fields_with_a_single_facet_switch_facet_comes_first() {
        let _m = get_lock(&MTX);

        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get()
                .with(eq("http://localhost:8983/solr/default/select?facet=on&facet_query=anything%3A+*&facet_field=exists&q=*%3A*"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{
                            "response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success": true }]},
                            "facet_counts": {
                                "facet_queries": {
                                    "anything: *": 324534
                                },
                                "facet_fields": {
                                   "exists": [
                                        "term1", 23423, "term2", 993939
                                    ]
                                },
                                "facet_ranges":{},
                                "facet_intervals":{},
                                "facet_heatmaps":{}
                            }
                        }"#)
                    .unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut command = Client::new(host, collection);
        let result = command
            .request_handler("select")
            .facet_query("anything: *")
            .facet_field("exists")
            .query("*:*")
            .run();
        assert!(result.is_ok());
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
                    .body(r#"{"response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success": true }]}}"#)
                    .unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut command = Client::new(host, collection);
        let result = command
            .request_handler("update/json/docs")
            .auto_commit()
            .set_document(json!({ "this is": "a document"}))
            .run();
        assert!(result.is_ok());
        assert_eq!(command.get_response::<Value>().unwrap().response.unwrap().docs[0]["success"], true);
    }

    #[test]
    fn test_select_responds_rsolr_error_with_other_problem_if_dunno() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();

        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get().returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"{"error": {"code": 500, "msg": "okapi"}}"#).unwrap())));
            mock
        });

        let collection = "default";
        let base_url = "http://localhost:8983";
        let result = Client::new(base_url, collection)
            .select("bad: query")
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "okapi");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_select_responds_rsolr_error_with_raw_text_body_and_status_code_if_no_standard_message() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get().returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"some unparseable thing"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection)
            .select("bad: query")
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "some unparseable thing");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_create_responds_rsolr_error_with_other_problem_if_dunno() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"{"error": {"code": 500, "msg": "okapi"}}"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection)
            .auto_commit()
            .create(json!({"anything": "anything"}))
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "okapi");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_create_responds_rsolr_error_with_raw_text_body_and_status_code_if_no_standard_message() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"some unparseable thing"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection)
            .auto_commit()
            .create(json!({"anything": "anything"}))
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "some unparseable thing");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_delete_responds_rsolr_error_with_other_problem_if_dunno() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"{"error": {"code": 500, "msg": "okapi"}}"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection)
            .auto_commit()
            .delete("*:*")
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "okapi");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }

    #[test]
    fn test_delete_responds_rsolr_error_with_raw_text_body_and_status_code_if_no_standard_message() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"some unparseable thing"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let result = Client::new(host, collection)
            .delete("*:*")
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert_eq!(error.status().unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message().unwrap(), "some unparseable thing");
        assert!(matches!(error.kind(), error::ErrorKind::Other));
    }
}