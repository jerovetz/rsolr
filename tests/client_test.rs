use rsc::Client;
use reqwest::blocking::Client as HttpClient;
use reqwest::header::CONTENT_TYPE;
use dotenv::dotenv;

#[test]
fn test_hello() {
    let s = Client::hello();
    assert_eq!(s, "Hello")
}

fn empty_collection(host : &str) -> Result<(), reqwest::Error> {
    let http_client = HttpClient::new();
    http_client
        .post(format!("{}{}", host, "/solr/default/update?stream.body=<delete><query>*:*</query></delete>&commit=true"))
        .header(CONTENT_TYPE, "application/json")
        .send()?;
    Ok(())
}

#[test]
fn test_query_all() -> Result<(), reqwest::Error> {
    dotenv().ok();
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
    let result = client.query_all();
    assert!(result.get(0).unwrap().get("egerke").unwrap().get(0).unwrap() == "okapi");

    Ok(())
}