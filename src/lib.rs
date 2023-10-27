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
//!
//! ## Cursor-based pagination
//!
//! Paginated results can be fetched iteratively with the use of [solr cursor](https://solr.apache.org/guide/solr/latest/query-guide/pagination-of-results.html#fetching-a-large-number-of-sorted-results-cursors)
//!
//! ```rust
//! use serde_json::Value;
//! use rsolr::Client;
//! use rsolr::solr_response::SolrResponse;
//! fn cursor_fetch_all_pages() -> Vec<SolrResponse<Value>> {
//!     let mut responses = Vec::new();
//!     let mut client = Client::new("http://solr:8983", "collection");
//!     let result = client
//!         .select("*:*")
//!         .sort("id asc")
//!         .cursor()
//!         .run();
//!     let mut cursor = result.expect("request failed").expect("no cursor");
//!     while cursor.next::<Value>().expect("request failed").is_some() {
//!         responses.push(cursor.get_response::<Value>().expect("parsing failed"));
//!     }
//!     responses
//! }
//! ```

use std::ops::Deref;

use http::StatusCode;
use mockall_double::double;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url;
use url::Url;

#[double]
use http_client::HttpClient;

use crate::cursor::Cursor;
use crate::error::RSolrError;
use crate::solr_response::SolrResponse;

pub mod error;
pub mod solr_response;
pub mod query;
pub mod cursor;
mod facet_fields;
mod http_client;

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

#[derive(Clone, Debug)]
pub struct Client<'a> {
    request_handler: &'a str,
    url: Url,
    payload: Payload,
    collection: &'a str,
    response: Option<Value>
}

impl<'a> Client<'a> {

    pub fn new(base_url: &'a str, collection: &'a str) -> Self {
        let url = Url::parse(base_url).unwrap();
        Client { request_handler: "", url, payload: Payload::None, collection, response: None }
    }

    /// Adds custom GET query parameter to the Solr query.
    pub fn add_query_param(&mut self, key: &str, value: &str) -> &mut Self {
        self.url.query_pairs_mut().append_pair(key, value);
        self
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

    pub fn update_cursor_mark(&mut self, cursor_mark: &str) -> &mut Self {
        let url = self.url.clone();
        let query = url.query().expect("Query part is required.");
        let regex = Regex::new(r"(cursorMark=)(\w|\*)").unwrap();
        let replace = format!("${{1}}{}", cursor_mark);
        let updated = regex.replace(query, replace.as_str());
        self.url.set_query(Some(updated.deref()));
        self
    }

    pub fn url(&mut self, url: &str) -> &mut Self {
        self.url = Url::parse(url).expect("Url parse failed.");
        self
    }

    pub fn sort(&mut self, sort: &str) -> &mut Self {
        self.add_query_param("sort", sort)
    }

    pub fn cursor(&mut self) -> &mut Self {
        self.add_query_param("cursorMark", "*")
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
    pub fn run(&mut self) -> Result<Option<Cursor>, RSolrError> {
        let http_result = match self.payload.clone() {
            Payload::Body(body) => HttpClient::new().post(self.url_str(), Some(body)),
            Payload::Empty => HttpClient::new().post(self.url_str(), None),
            Payload::None => HttpClient::new().get(self.url_str())
        };

        let http_response = match http_result {
            Ok(response) => response,
            Err(e) => return Err(RSolrError::Network { source: e }),
        };

        match http_response.status() {
            StatusCode::OK => {
                self.response = http_response.json::<Value>().ok();
                match self.url.query().unwrap_or("no url").contains("cursorMark") {
                    true => {
                        let cursor_mark = self.get_response::<Value>().unwrap().nextCursorMark.unwrap();
                        let cursor = Cursor::new(self.clone(), cursor_mark);
                        self.url.query_pairs_mut().clear();
                        Ok(Some(cursor))
                    },
                    false => {
                        self.url.query_pairs_mut().clear();
                        Ok(None)
                    }
                }
            },
            StatusCode::NOT_FOUND => Err(RSolrError::NotFound),
            other_status => {
                let body_text = http_response.text().unwrap();
                match serde_json::from_str::<Value>(&body_text) {
                    Ok(r) => Err(RSolrError::Syntax(r["error"]["msg"].as_str().unwrap().to_owned())),
                    Err(e) => {
                        Err( RSolrError::Other { source: Box::new(e), status: other_status, body_text })
                    }
                }
            }
        }
    }

    pub fn get_response<T: for<'de> Deserialize<'de> + Clone + Default>(&self) -> Result<SolrResponse<T>, RSolrError>{
        match self.response.clone() {
            Some(v) => match serde_json::from_value(v) {
                Ok(response) => Ok(response),
                Err(e) => Err(RSolrError::Serialization(e.to_string()) )
            },
            _ => Ok(SolrResponse::default())
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

    /// Shorthand for setting dismax query parser.
    pub fn dismax(&mut self) -> &mut Self {
        self.add_query_param("defType", "dismax")
    }

    /// Shorthand for setting edismax query parser.
    pub fn edismax(&mut self) -> &mut Self {
        self.add_query_param("defType", "edismax")
    }

    fn switch_on_facet(&mut self) {
        for query_pair in self.url.query_pairs() {
            if query_pair.0 == "facet" && query_pair.1 == "on" {
                return
            }
        }
        self.url.query_pairs_mut().append_pair("facet", "on");
    }

    fn payload(&mut self, payload: Payload) -> &mut Self {
        self.payload = payload;
        self
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, MutexGuard};

    use mockall::lazy_static;
    use mockall::predicate::eq;
    use serde_json::json;

    use super::*;

    lazy_static! {
        static ref MTX: Mutex<()> = Mutex::new(());
    }

    fn get_lock(m: &'static Mutex<()>) -> MutexGuard<'static, ()> {
        match m.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn setup_get_mock(url: &'static str, status_code: u16, body: &'static str) -> HttpClient {
        let mut mock = HttpClient::default();
        mock.expect_get()
            .with(eq(url))
            .returning(move |_| Ok(
                reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(status_code)
                    .body(body)
                    .unwrap()))
            );
        mock
    }


    #[test]
    fn build_a_url_from_parameters() {
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .query("*:*");

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?q=*%3A*");
    }

    #[test]
    fn build_a_url_from_parameters_set_autocommit() {
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .auto_commit();

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?commit=true");
    }

    #[test]
    fn build_a_url_with_start_and_rows() {
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .start(135545)
            .rows(12);

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?start=135545&rows=12");
    }

    #[test]
    fn build_a_url_with_default_field() {
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .default_field("defaultfield");

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?df=defaultfield");
    }

    #[test]
    fn url_built_with_facet_if_facet_fields_set() {
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .facet_field("facetfield");

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?facet=on&facet_field=facetfield");
    }

    #[test]
    fn url_built_with_facet_if_facet_query_set() {
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .facet_query("facet");

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?facet=on&facet_query=facet");
    }

    #[test]
    fn url_built_with_facet_correctly_if_both_set() {
        let mut params = Client::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .facet_field("facetfield")
            .facet_query("facet");

        let url_string = params.url_str();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?facet=on&facet_field=facetfield&facet_query=facet");
    }

    #[test]
    fn run_formats_url_and_result() {
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
    fn run_handles_facet_fields() {
        let _m = get_lock(&MTX);
        let body = r#"{
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
                        }"#;
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            setup_get_mock("http://localhost:8983/solr/default/select?q=*%3A*&facet=on&facet_field=exists", 200, body)
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
    fn run_handles_facet_query_and_returns_unimplemented_facets_in_raw() {
        let _m = get_lock(&MTX);
        let body = r#"{
                            "response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success": true }]},
                            "facet_counts": {
                                "facet_queries": {
                                    "anything: *": 324534
                                },
                                "facet_fields": {},
                                "facet_ranges":"interesting ranges",
                                "facet_intervals":"interesting intervals",
                                "facet_heatmaps":"interesting heatmaps"
                            }
                        }"#;

        let ctx = HttpClient::new_context();
        ctx.expect().returning(||
           setup_get_mock("http://localhost:8983/solr/default/select?q=*%3A*&facet=on&facet_query=anything%3A+*", 200, body)
        );

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

        assert_eq!(facets.raw.get("facet_ranges").unwrap(), "interesting ranges");
        assert_eq!(facets.raw.get("facet_intervals").unwrap(), "interesting intervals");
        assert_eq!(facets.raw.get("facet_heatmaps").unwrap(), "interesting heatmaps");
    }

    #[test]
    fn run_deserializes_remaining_fields_into_raw() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        let body = r#"{"response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success": true }]},"anything":"other fields"}"#;

        ctx.expect().returning(||
            setup_get_mock("http://localhost:8983/solr/default/select?q=*%3A*", 200, body)
        );
        let mut client = Client::new("http://localhost:8983", "default");
        let result = client
            .select("*:*")
            .run();
        assert!(result.is_ok());
        assert_eq!(client.get_response::<Value>().unwrap().raw.get("anything").unwrap(),"other fields");
    }

    #[test]
    fn run_calls_post_with_url_and_body() {
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
    fn select_responds_rsolr_error_with_other_problem_if_dunno() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();

        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get()
                .returning(|_| Ok(reqwest::blocking::Response::from(
                    http::response::Builder::new().status(500).body(r#"{"error": {"code": 500, "msg": "okapi"}}"#).unwrap())));
            mock
        });

        let collection = "default";
        let base_url = "http://localhost:8983";
        let mut client = Client::new(base_url, collection);
        let result = client
            .select("bad: query")
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert!(matches!(error, RSolrError::Syntax(..) ));
        assert_eq!(format!("{:?}", error), "Syntax(\"okapi\")");
    }

    #[test]
    fn select_responds_rsolr_error_with_raw_text_body_and_status_code_if_no_standard_message() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get().returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"some unparseable thing"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut client  = Client::new(host, collection);
            let result = client
            .select("bad: query")
            .run();
        let error = result.err().expect("No Error");
        assert!(matches!(error, RSolrError::Other {status: StatusCode::INTERNAL_SERVER_ERROR, ..} ));
        assert!(format!("{:?}", error).contains("some unparseable thing"));
    }

    #[test]
    fn create_responds_rsolr_error_with_other_problem_if_dunno() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"{"error": {"code": 500, "msg": "okapi"}}"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut client = Client::new(host, collection);
        let result = client
            .auto_commit()
            .create(json!({"anything": "anything"}))
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert!(matches!(error, RSolrError::Syntax(..) ));
        assert_eq!(format!("{:?}", error), "Syntax(\"okapi\")");
    }

    #[test]
    fn create_responds_rsolr_error_with_raw_text_body_and_status_code_if_no_standard_message() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"some unparseable thing"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut client  = Client::new(host, collection);
        let result = client
            .auto_commit()
            .create(json!({"anything": "anything"}))
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert!(matches!(error, RSolrError::Other {status: StatusCode::INTERNAL_SERVER_ERROR, ..} ));
        assert!(format!("{:?}", error).contains("some unparseable thing"));
    }

    #[test]
    fn delete_responds_rsolr_error_with_other_problem_if_dunno() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"{"error": {"code": 500, "msg": "okapi"}}"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut client = Client::new(host, collection);
        let result = client
            .auto_commit()
            .delete("*:*")
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert!(matches!(error, RSolrError::Syntax(..) ));
        assert_eq!(format!("{:?}", error), "Syntax(\"okapi\")");
    }

    #[test]
    fn delete_responds_rsolr_error_with_raw_text_body_and_status_code_if_no_standard_message() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_post().returning(|_, _| Ok(reqwest::blocking::Response::from(http::response::Builder::new().status(500).body(r#"some unparseable thing"#).unwrap())));
            mock
        });

        let collection = "default";
        let host = "http://localhost:8983";
        let mut client = Client::new(host, collection);
        let result = client
            .delete("*:*")
            .run();
        assert!(result.is_err());
        let error = result.err().expect("No Error");
        assert!(matches!(error, RSolrError::Other {status: StatusCode::INTERNAL_SERVER_ERROR, ..} ));
        assert!(format!("{:?}", error).contains("some unparseable thing"));
    }

    #[test]
    fn run_responds_cursor_if_cursor_set() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get()
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{"response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success": true }]}, "nextCursorMark": "cursormark"}"#)
                    .unwrap())));
            mock
        });

        let mut client = Client::new("http://localhost:8983", "default");
        let result = client
            .select("*:*")
            .sort("field asc")
            .cursor()
            .run();
        assert!(result.expect("Ok expected").is_some());
    }

    #[test]
    fn next_returns_the_next_response() {
        let _m = get_lock(&MTX);
        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get()
                .with(eq("http://solr.url/solr/dummy/select?q=*%3A*&rows=1&cursorMark=first_cursor_mark&sort=unique+asc"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{"response": {"numFound": 2,"numFoundExact": true,"start": 0,"docs": [{"success": true }]}, "nextCursorMark": "second_cursor_mark"}"#)
                    .unwrap())));

            mock.expect_get()
                .with(eq("http://solr.url/solr/dummy/select?q=*%3A*&rows=1&cursorMark=second_cursor_mark&sort=unique+asc"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{"response": {"numFound": 2,"numFoundExact": true,"start": 0,"docs": [{"success2": true }]}, "nextCursorMark": "third_cursor_mark"}"#)
                    .unwrap())));

            mock.expect_get()
                .with(eq("http://solr.url/solr/dummy/select?q=*%3A*&rows=1&cursorMark=third_cursor_mark&sort=unique+asc"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{"response": {"numFound": 2,"numFoundExact": true,"start": 0,"docs": []}, "nextCursorMark": "third_cursor_mark"}"#)
                    .unwrap())));

            mock
        });

        let mut client = Client::new("http://solr.url", "dummy");
        client
            .select("*:*")
            .rows(1)
            .cursor()
            .sort("unique asc");


        let mut cursor = Cursor::new(client, "first_cursor_mark".to_owned());
        let result = cursor.next::<Value>();
        assert_eq!(result.expect("Ok expected").expect("Response expected").response.expect("solr response expected").docs[0].get("success").unwrap(), true);

        let result2 = cursor.next::<Value>();
        assert_eq!(result2.expect("Ok expected").expect("Response expected").response.expect("solr response expected").docs[0].get("success2").unwrap(), true);

        let result3 = cursor.next::<Value>();
        assert!(result3.expect("Ok expected").is_none());
    }
}