use rsc::Client;
use reqwest::blocking::Client as HttpClient;
use reqwest::header::CONTENT_TYPE;
use std::error::Error;

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
    let expected_documents : serde_json::Value = serde_json::from_str(data).unwrap();

    http_client
        .post(format!("{}/solr/{}/update/json/docs?commit=true", host, collection))
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&expected_documents).unwrap())
        .send()?;

    let client = Client::new(host, collection);
    let result = client.query("*:*");
    assert_eq!(result.unwrap().get(0).unwrap().get("egerke").unwrap().get(0).unwrap(), "okapi");

    Ok(())
}

#[test]
fn test_query_responds_rsc_error_with_embedded_network_error() {
    let collection = "default";
    let host = "http://not_existing_host:8983";
    let result = Client::new(host, collection).query("*:*");
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    let original_error_message = error.source().expect("no source error").to_string();
    assert_eq!(error.kind(), "RSCError");
    assert_eq!(original_error_message.contains("dns error"), true)
}