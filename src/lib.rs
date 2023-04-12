use reqwest::blocking::Client as HttpClient;

pub struct Client {
    host: String,
    collection: String
}

impl Client {
    pub fn hello() -> &'static str
    {
        "Hello"
    }

    pub fn query_all(&self) -> serde_json::Value {
        let http_client = HttpClient::new();
        let raw_response = http_client
            .get(format!("{}/solr/{}/select?q=*%3A*", self.host, self.collection))
            .send().unwrap();

        raw_response.json::<serde_json::Value>().unwrap()
            .get("response").unwrap().get("docs").unwrap().clone()
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