pub struct Client {
    host: String,
    collection: String
}

impl Client {
    pub fn hello() -> &'static str
    {
        "Hello"
    }

    pub fn query_all(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::from_str(r#"{}"#)
    }

    pub fn new(url : &str, collection : &str) -> Client {
        let host = String::from(url);
        let collection = String::from(collection);
        Client {
            host,
            collection
        }
    }
}