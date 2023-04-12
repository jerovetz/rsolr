use rsc::Client;

#[test]
fn test_hello() {
    let s = Client::hello();
    assert_eq!(s, "Hello")
}

#[test]
fn test_query_all() {
    let data = r#"{}"#;
    let expected_documents : serde_json::Value = serde_json::from_str(data).unwrap();
    let host = "http://localhost:8983";
    let collection = "default";
    let client = Client::new(host, collection);
    let result = client.query_all();
    assert_eq!(result.unwrap(), expected_documents);
}