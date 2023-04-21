use rsc::{AutoCommit, Client};
use reqwest::blocking::Client as HttpClient;
use reqwest::header::CONTENT_TYPE;
use std::error::Error;
use reqwest::StatusCode;
use serde_json::Value;

fn empty_collection(host : &str) -> Result<(), reqwest::Error> {
    let http_client = HttpClient::new();
    http_client
        .post(format!("{}{}", host, "/solr/default/update?stream.body=<delete><query>*:*</query></delete>&commit=true"))
        .header(CONTENT_TYPE, "application/json")
        .send()?;
    Ok(())
}

#[test]
fn test_query_document_value_returned() -> Result<(), reqwest::Error> {
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

    let client = Client::new(host, collection, AutoCommit::NO);
    let result = client.query("*:*");
    assert_eq!(result.unwrap().get(0).unwrap().get("egerke").unwrap().get(0).unwrap(), "okapi");
    empty_collection(host).ok();
    Ok(())
}

#[test]
fn test_query_responds_rsc_error_with_embedded_network_error() {
    let collection = "default";
    let host = "http://not_existing_host:8983";
    let result = Client::new(host, collection, AutoCommit::NO).query("*:*");
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    let original_error_message = error.source().expect("no source error").to_string();
    assert!(matches!(error.kind(), rsc::error::ErrorKind::Network));
    assert_eq!(original_error_message.contains("dns error"), true)
}

#[test]
fn test_query_responds_rsc_error_with_embedded_no_collection_error() {
    let collection = "not_existing_collection";
    let host = "http://localhost:8983";
    let result = Client::new(host, collection, AutoCommit::NO).query("*:*");
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert_eq!(error.status().unwrap(), StatusCode::NOT_FOUND);
    assert!(matches!(error.kind(), rsc::error::ErrorKind::NotFound));
    assert!(error.source().is_none());
}

#[test]
fn test_query_responds_rsc_error_with_solr_problem_if_query_is_bad() {
    let collection = "default";
    let host = "http://localhost:8983";
    let result = Client::new(host, collection, AutoCommit::NO).query("bad: query");
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert_eq!(error.status().unwrap(), StatusCode::BAD_REQUEST);
    matches!(error.kind(), rsc::error::ErrorKind::SolrSyntax);
    assert!(error.source().is_none());
    assert_eq!(error.message(), Some("undefined field bad"));
}

#[test]
fn test_create_with_auto_commit_inserts_document_responds_nothing() {
    let collection = "default";
    let host = "http://localhost:8983";
    empty_collection(host).ok();

    let document : Value = serde_json::from_str(r#"{"okapi": "egerke"}"#).unwrap();
    let client = Client::new(host,collection, AutoCommit::YES);
    let _ = client.create(document);

    let result = client.query("*:*");
    assert_eq!(result.unwrap()[0]["okapi"][0], "egerke");
    empty_collection(host).ok();
}

#[test]
fn test_create_without_auto_commit_uploads_document_and_index_on_separated_commit_responds_nothing() {
    let collection = "default";
    let host = "http://localhost:8983";
    empty_collection(host).ok();

    let document : Value = serde_json::from_str(r#"{"okapi": "egerke"}"#).unwrap();
    let client = Client::new(host,collection, AutoCommit::NO);
    let _ = client.create(document);
    let result = client.query("*:*");
    assert_eq!(result.unwrap(), serde_json::from_str::<Value>("[]").unwrap());

    let _ = client.commit();
    let result = client.query("*:*");
    assert_eq!(result.unwrap()[0]["okapi"][0], "egerke");
    empty_collection(host).ok();
}

#[test]
fn test_create_responds_rsc_error_with_embedded_network_error() {
    let collection = "default";
    let host = "http://not_existing_host:8983";
    let result = Client::new(host, collection, AutoCommit::NO).create(serde_json::from_str(r#"{"anything": "anything"}"#).unwrap());
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    let original_error_message = error.source().expect("no source error").to_string();
    assert!(matches!(error.kind(), rsc::error::ErrorKind::Network));
    assert_eq!(original_error_message.contains("dns error"), true)
}