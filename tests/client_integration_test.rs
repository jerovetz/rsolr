use rsc::{Client};
use reqwest::blocking::Client as HttpClient;
use reqwest::header::CONTENT_TYPE;
use std::error::Error;
use std::fmt::Debug;
use std::sync::{Mutex, MutexGuard};
use mockall::lazy_static;
use reqwest::StatusCode;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ExcitingDocument {
    desire: String,
    vision: Vec<String>,
}

fn empty_collection(host : &str) -> Result<(), reqwest::Error> {
    let http_client = HttpClient::new();
    http_client
        .post(format!("{}{}", host, "/solr/default/update?stream.body=<delete><query>*:*</query></delete>&commit=true"))
        .header(CONTENT_TYPE, "application/json")
        .send()?;
    Ok(())
}

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
fn test_query_document_value_returned() -> Result<(), reqwest::Error> {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://127.0.0.1:8983";
    empty_collection(host).ok();

    let http_client = HttpClient::new();
    let data = r#"{"egerke": "okapi"}"#;
    let expected_documents : Value = serde_json::from_str(data).unwrap();

    http_client
        .post(format!("{}/solr/{}/update/json/docs?commit=true", host, collection))
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&expected_documents).unwrap())
        .send()?;

    let result = Client::new(host, collection).select("*:*").run::<Value>();
    assert_eq!(result.unwrap().unwrap().docs.get(0).unwrap().get("egerke").unwrap().get(0).unwrap(), "okapi");
    empty_collection(host).ok();
    Ok(())
}

#[test]
fn test_query_returns_error_if_cannot_serialize() -> Result<(), reqwest::Error> {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://127.0.0.1:8983";
    empty_collection(host).ok();

    let http_client = HttpClient::new();
    let data = r#"{"egerke": "okapi"}"#;
    let expected_documents : Value = serde_json::from_str(data).unwrap();

    http_client
        .post(format!("{}/solr/{}/update/json/docs?commit=true", host, collection))
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&expected_documents).unwrap())
        .send()?;

    let result = Client::new(host, collection).select("*:*").run::<ExcitingDocument>();
    assert!(result.is_err());
    assert!(result.err().unwrap().source().is_some());
    empty_collection(host).ok();
    Ok(())
}

#[test]
fn test_query_responds_rsc_error_with_embedded_network_error() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://not_existing_host:8983";
    let result = Client::new(host, collection).select("*:*").run::<Value>();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    let original_error_message = error.source().expect("no source error").to_string();
    assert!(matches!(error.kind(), rsc::error::ErrorKind::Network));
    assert_eq!(original_error_message.contains("dns error"), true)
}

#[test]
fn test_query_responds_rsc_error_with_embedded_no_collection_error() {
    let _m = get_lock(&MTX);
    let collection = "not_existing_collection";
    let host = "http://localhost:8983";
    let result = Client::new(host, collection).select("*:*").run::<Value>();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert_eq!(error.status().unwrap(), StatusCode::NOT_FOUND);
    assert!(matches!(error.kind(), rsc::error::ErrorKind::NotFound));
    assert!(error.source().is_none());
}

#[test]
fn test_query_responds_rsc_error_with_solr_problem_if_query_is_bad() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    let result = Client::new(host, collection).select("bad: query").run::<Value>();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert_eq!(error.status().unwrap(), StatusCode::BAD_REQUEST);
    matches!(error.kind(), rsc::error::ErrorKind::SolrSyntax);
    assert!(error.source().is_none());
    assert_eq!(error.message(), Some("undefined field bad"));
}

#[test]
fn test_create_with_auto_commit_inserts_document() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let base_url = "http://localhost:8983";
    empty_collection(base_url).ok();

    let document : Value = json!({"okapi": "egerke"});
    let mut client = Client::new(base_url, collection);
    client
        .auto_commit()
        .create(document)
        .run::<Value>()
        .ok();

    let result = client.select("*:*").run::<Value>();
    assert_eq!(result.unwrap().unwrap().docs[0]["okapi"][0], "egerke");
    empty_collection(base_url).ok();
}

#[test]
fn test_create_inserts_any_serializable_document() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let base_url = "http://localhost:8983";
    empty_collection(base_url).ok();

    let document = ExcitingDocument { desire: "sausage".to_string(), vision: vec!("firearms".to_string(), "York".to_string(), "Belzebub".to_string()) };

    let mut client = Client::new(base_url, collection);
    client
        .auto_commit()
        .create(document)
        .run::<Value>()
        .ok();

    let result = client.select("*:*").run::<Value>();
    assert_eq!(result.unwrap().unwrap().docs[0]["desire"][0], "sausage");
    empty_collection(base_url).ok();
}

#[test]
fn test_create_without_auto_commit_uploads_document_and_index_on_separated_commit_responds_nothing() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    empty_collection(host).ok();

    let document : Value = serde_json::from_str(r#"{"okapi": "egerke"}"#).unwrap();
    let mut client = Client::new(host,collection);

    client
        .create(document)
        .run::<Value>()
        .ok();

    let result = client.select("*:*").run::<Value>();
    assert_eq!(result.unwrap().unwrap().docs.len(), 0);

    client.commit().run::<Value>().ok();

    let result = client.select("*:*").run::<Value>();
    assert_eq!(result.unwrap().unwrap().docs[0]["okapi"][0], "egerke");
    empty_collection(host).ok();
}

#[test]
fn test_create_responds_rsc_error_with_embedded_network_error() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://not_existing_host:8983";
    let result = Client::new(host, collection)
        .create(json!({"anything": "anything"}))
        .run::<Value>();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    let original_error_message = error.source().expect("no source error").to_string();
    assert!(matches!(error.kind(), rsc::error::ErrorKind::Network));
    assert_eq!(original_error_message.contains("dns error"), true)
}

#[test]
fn test_create_responds_rsc_error_with_embedded_no_collection_error() {
    let _m = get_lock(&MTX);
    let collection = "not_existing_collection";
    let host = "http://localhost:8983";

    let result = Client::new(host, collection)
        .create(json!({"anything": "anything"}))
        .run::<Value>();

    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert_eq!(error.status().unwrap(), StatusCode::NOT_FOUND);
    assert!(matches!(error.kind(), rsc::error::ErrorKind::NotFound));
    assert!(error.source().is_none());
}

#[test]
fn test_delete_deletes_docs() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    empty_collection(host).ok();
    let mut client = Client::new(host, collection);
    let _ =  client
        .auto_commit()
        .create(json!({"okapi": "another egerke"}))
        .run::<Value>();

    let result = client
        .auto_commit()
        .delete("*:*")
        .run::<Value>();

    assert!(result.is_ok());
    let docs = client.select("*:*").run::<Value>().unwrap().unwrap().docs;
    assert_eq!(docs.len(), 0);

    empty_collection(host).ok();
}

#[test]
fn test_delete_deletes_docs_specified_by_query() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    empty_collection(host).ok();
    let mut client = Client::new(host, collection);
    client.create(json!({"okapi": "another egerke"})).run::<Value>().ok();
    client.create(json!({"okapi2": "egerke"})).run::<Value>().ok();
    client.commit().run::<Value>().ok();

    let result = client
        .delete("okapi2: egerke")
        .run::<Value>();

    assert!(result.is_ok());
    let docs = client.select("*:*").run::<Value>().unwrap().unwrap().docs;
    assert_eq!(docs[0]["okapi"][0], "another egerke");

    empty_collection(host).ok();
}

#[test]
fn test_without_autocommit_delete_deletes_docs_after_commit_specified_by_query() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    empty_collection(host).ok();
    let mut client = Client::new(host, collection);
    client.create(json!({"okapi": "another egerke"})).run::<Value>().ok();
    client.create(json!({"okapi2": "egerke"})).run::<Value>().ok();
    client.commit().run::<Value>().ok();

    client.delete("okapi: another egerke").run::<Value>().ok();

    let docs = client.select("*:*").run::<Value>().unwrap().unwrap().docs;
    assert_eq!(docs[0]["okapi"][0], "another egerke");

    client.commit().run::<Value>().ok();

    let docs = client.select("*:*").run::<Value>().unwrap().unwrap().docs;
    assert_ne!(docs[0]["okapi"][0], "another egerke");

    empty_collection(host).ok();
}

#[test]
fn test_delete_responds_rsc_error_with_embedded_network_error() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://not_existing_host:8983";
    let result = Client::new(host, collection).delete("*:*").run::<Value>();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    let original_error_message = error.source().expect("no source error").to_string();
    assert!(matches!(error.kind(), rsc::error::ErrorKind::Network));
    assert_eq!(original_error_message.contains("dns error"), true)
}

#[test]
fn test_delete_responds_rsc_error_with_embedded_no_collection_error() {
    let _m = get_lock(&MTX);
    let collection = "not_existing_collection";
    let host = "http://localhost:8983";
    let result = Client::new(host, collection).delete("*:*").run::<Value>();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert_eq!(error.status().unwrap(), StatusCode::NOT_FOUND);
    assert!(matches!(error.kind(), rsc::error::ErrorKind::NotFound));
    assert!(error.source().is_none());
}

#[test]
fn test_delete_responds_rsc_error_with_solr_problem_if_query_is_bad() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    let result = Client::new(host, collection).delete("bad: query").run::<Value>();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert_eq!(error.status().unwrap(), StatusCode::BAD_REQUEST);
    matches!(error.kind(), rsc::error::ErrorKind::SolrSyntax);
    assert!(error.source().is_none());
    assert_eq!(error.message(), Some("undefined field bad"));
}