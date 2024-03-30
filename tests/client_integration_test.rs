use rsolr::{Client};
use reqwest::blocking::Client as HttpClient;
use reqwest::header::CONTENT_TYPE;
use std::fmt::Debug;
use std::sync::{Mutex, MutexGuard};
use mockall::lazy_static;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use rsolr::error::RSolrError;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct ExcitingDocument {
    desire: String,
    vision: Vec<String>,
}

fn empty_default_collection(host : &str) -> Result<(), reqwest::Error> {
    let http_client = HttpClient::new();
    http_client
        .post(format!("{}{}", host, "/solr/default/update?stream.body=<delete><query>*:*</query></delete>&commit=true"))
        .header(CONTENT_TYPE, "application/json")
        .send()?;
    Ok(())
}

fn empty_techproducts_collection(host : &str) -> Result<(), reqwest::Error> {
    let http_client = HttpClient::new();
    http_client
        .post(format!("{}{}", host, "/solr/techproducts/update?stream.body=<delete><query>*:*</query></delete>&commit=true"))
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
fn query_document_value_returned() -> Result<(), reqwest::Error> {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://127.0.0.1:8983";
    empty_default_collection(host).ok();

    let http_client = HttpClient::new();
    let data = r#"{"egerke": "okapi"}"#;
    let expected_documents : Value = serde_json::from_str(data).unwrap();

    http_client
        .post(format!("{}/solr/{}/update/json/docs?commit=true", host, collection))
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&expected_documents).unwrap())
        .send()?;

    let mut client = Client::new(host, collection);
    let result = client.select("*:*").run();
    assert!(result.is_ok());

    assert_eq!(client.get_response::<Value>().unwrap().response.unwrap().docs.get(0).unwrap().get("egerke").unwrap().get(0).unwrap(), "okapi");
    empty_default_collection(host).ok();
    Ok(())
}

#[test]
fn query_returns_error_if_cannot_serialize() -> Result<(), reqwest::Error> {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://127.0.0.1:8983";
    empty_default_collection(host).ok();

    let http_client = HttpClient::new();
    let data = r#"{"egerke": "okapi"}"#;
    let expected_documents : Value = serde_json::from_str(data).unwrap();

    http_client
        .post(format!("{}/solr/{}/update/json/docs?commit=true", host, collection))
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&expected_documents).unwrap())
        .send()?;

    let mut client = Client::new(host, collection);
    let result= client.select("*:*").run();
    assert!(result.is_ok());
    let response = client.get_response::<ExcitingDocument>();
    let error = response.err().expect("No Error");
    assert!(matches!(error, RSolrError::Serialization(..)));
    assert!(format!("{:?}", error).contains("missing field"));
    empty_default_collection(host).ok();
    Ok(())
}

#[test]
fn query_responds_rsolr_error_with_embedded_network_error() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://not_existing_host:8983";
    let mut client = Client::new(host, collection);
    let result = client.select("*:*").run();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert!(matches!(error, RSolrError::Network {..}));
    assert!(format!("{:?}", error).contains("dns error"));
}

#[test]
fn query_responds_rsolr_error_with_embedded_no_collection_error() {
    let _m = get_lock(&MTX);
    let collection = "not_existing_collection";
    let host = "http://localhost:8983";
    let mut client = Client::new(host, collection);
    let result = client.select("*:*").run();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert!(matches!(error, RSolrError::NotFound))
}

#[test]
fn query_responds_rsolr_error_with_solr_problem_if_query_is_bad() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    let mut client = Client::new(host, collection);
    let result = client.select("bad: query").run();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert!(matches!(error, RSolrError::Syntax(..)));
    assert!(format!("{:?}", error).contains("undefined field bad"))
}

#[test]
fn create_with_auto_commit_inserts_document() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let base_url = "http://localhost:8983";
    empty_default_collection(base_url).ok();

    let document : Value = json!({"okapi": "egerke"});
    let mut client = Client::new(base_url, collection);
    client
        .auto_commit()
        .upload_json(document)
        .run()
        .ok();

    let result = client.select("*:*").run();
    assert!(result.is_ok());
    let solr_response = client.get_response::<Value>();
    assert_eq!(solr_response.unwrap().response.unwrap().docs[0]["okapi"][0], "egerke");
    empty_default_collection(base_url).ok();
}

#[test]
fn create_inserts_any_serializable_document() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let base_url = "http://localhost:8983";
    empty_default_collection(base_url).ok();

    let document = ExcitingDocument { desire: "sausage".to_string(), vision: vec!("firearms".to_string(), "York".to_string(), "Belzebub".to_string()) };

    let mut client = Client::new(base_url, collection);
    client
        .auto_commit()
        .upload_json(document)
        .run()
        .ok();

    client.select("*:*").run().ok();
    let solr_response = client.get_response::<Value>();
    assert_eq!(solr_response.unwrap().response.unwrap().docs[0]["desire"][0], "sausage");
    empty_default_collection(base_url).ok();
}

#[test]
fn create_without_auto_commit_uploads_document_and_index_on_separated_commit_responds_nothing() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    empty_default_collection(host).ok();

    let document : Value = serde_json::from_str(r#"{"okapi": "egerke"}"#).unwrap();
    let mut client = Client::new(host,collection);

    client
        .upload_json(document)
        .run()
        .ok();


    client.select("*:*").run().ok();
    let solr_response = client.get_response::<Value>();
    assert_eq!(solr_response.unwrap().response.unwrap().docs.len(), 0);

    client.commit().run().ok();

    client.select("*:*").run().ok();
    let solr_response = client.get_response::<Value>();
    assert_eq!(solr_response.unwrap().response.unwrap().docs[0]["okapi"][0], "egerke");
    empty_default_collection(host).ok();
}

#[test]
fn create_responds_rsolr_error_with_embedded_network_error() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://not_existing_host:8983";
    let mut client = Client::new(host, collection);
    let result = client
        .upload_json(json!({"anything": "anything"}))
        .run();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert!(matches!(error, RSolrError::Network {..}));
    assert!(format!("{:?}", error).contains("dns error"));
}

#[test]
fn create_responds_rsolr_error_with_embedded_no_collection_error() {
    let _m = get_lock(&MTX);
    let collection = "not_existing_collection";
    let host = "http://localhost:8983";
    let mut client = Client::new(host, collection);
    let result = client
        .upload_json(json!({"anything": "anything"}))
        .run();

    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert!(matches!(error, RSolrError::NotFound));
}

#[test]
fn delete_deletes_docs() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    empty_default_collection(host).ok();
    let mut client = Client::new(host, collection);
    let _ =  client
        .auto_commit()
        .upload_json(json!({"okapi": "another egerke"}))
        .run();

    let result = client
        .auto_commit()
        .delete("*:*")
        .run();

    assert!(result.is_ok());
    client.select("*:*").run().ok();
    let docs = client.get_response::<Value>().unwrap().response.unwrap().docs;
    assert_eq!(docs.len(), 0);

    empty_default_collection(host).ok();
}

#[test]
fn delete_deletes_docs_specified_by_query() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    empty_default_collection(host).ok();
    let mut client = Client::new(host, collection);
    client.upload_json(json!({"okapi": "another egerke"})).run().ok();
    client.upload_json(json!({"okapi2": "egerke"})).run().ok();
    client.commit().run().ok();

    let result = client
        .delete("okapi2: egerke")
        .run();

    assert!(result.is_ok());
    client.select("*:*").run().ok();
    let docs = client.get_response::<Value>().ok().unwrap().response.unwrap().docs;
    assert_eq!(docs[0]["okapi"][0], "another egerke");

    empty_default_collection(host).ok();
}

#[test]
fn without_autocommit_delete_deletes_docs_after_commit_specified_by_query() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    empty_default_collection(host).ok();
    let mut client = Client::new(host, collection);
    client.upload_json(json!({"okapi": "another egerke"})).run().ok();
    client.upload_json(json!({"okapi2": "egerke"})).run().ok();
    client.commit().run().ok();

    client.delete("okapi: another egerke").run().ok();
    client.select("*:*").run().ok();
    let docs = client.get_response::<Value>().ok().unwrap().response.unwrap().docs;
    assert_eq!(docs[0]["okapi"][0], "another egerke");

    client.commit().run().ok();
    client.select("*:*").run().ok();
    let docs = client.get_response::<Value>().unwrap().response.unwrap().docs;
    assert_ne!(docs[0]["okapi"][0], "another egerke");

    empty_default_collection(host).ok();
}

#[test]
fn delete_responds_rsolr_error_with_embedded_network_error() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://not_existing_host:8983";
    let mut client = Client::new(host, collection);
    let result = client.delete("*:*").run();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert!(matches!(error, RSolrError::Network {..}));
    assert!(format!("{:?}", error).contains("dns error"));
}

#[test]
fn delete_responds_rsolr_error_with_embedded_no_collection_error() {
    let _m = get_lock(&MTX);
    let collection = "not_existing_collection";
    let host = "http://localhost:8983";
    let mut client = Client::new(host, collection);
    let result = client.delete("*:*").run();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert!(matches!(error, RSolrError::NotFound))
}

#[test]
fn delete_responds_rsolr_error_with_solr_problem_if_query_is_bad() {
    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";
    let mut client = Client::new(host, collection);
    let result = client.delete("bad: query").run();
    assert!(result.is_err());
    let error = result.err().expect("No Error");
    assert!(matches!(error, RSolrError::Syntax(..)));
    assert!(format!("{:?}", error).contains("undefined field bad"));
    assert!(client.get_response::<Value>().unwrap().facet_counts.is_none());
    assert!(client.get_response::<Value>().unwrap().response.is_none());
}

#[test]
fn cursor_used_to_fetch_data() {
    fn add<P: Serialize + Clone>(client: &mut Client, document: P) {
        client
            .auto_commit()
            .upload_json(document)
            .run()
            .ok();
    }

    let _m = get_lock(&MTX);
    let collection = "default";
    let host = "http://localhost:8983";

    empty_techproducts_collection(host).ok();

    let mut client = Client::new(host, collection);

    add(&mut client,json!({"okapi": "egerke", "id": 1}));
    add(&mut client,json!({"okapi2": "egerke", "id": 2}));
    add(&mut client,json!({"okapi3": "egerke", "id": 3}));

    let mut cursor = client
        .select("*:*")
        .rows(1)
        .sort("id asc")
        .cursor()
        .run()
        .expect("result expected")
        .expect("cursor expected");

    let first_page = cursor.get_response::<Value>().expect("result expected");
    assert_eq!(first_page.response.unwrap().docs.get(0).unwrap().get("okapi").unwrap().get(0).unwrap(), "egerke");

    let second_page = cursor.next::<Value>().expect("result expected");
    assert_eq!(second_page.expect("solr response expected").response.unwrap().docs.get(0).unwrap().get("okapi2").unwrap().get(0).unwrap(), "egerke");

    let third_page = cursor.next::<Value>().expect("result expected");
    assert_eq!(third_page.expect("solr response expected").response.unwrap().docs.get(0).unwrap().get("okapi3").unwrap().get(0).unwrap(), "egerke");

    let no_more = cursor.next::<Value>().expect("result expected");
    assert!(no_more.is_none());

    let and_again = cursor.next::<Value>().expect("result expected");
    assert!(and_again.is_none());

    empty_techproducts_collection(host).ok();
}