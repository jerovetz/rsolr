use reqwest::blocking::{Client as ReqwestClient, Response };
use reqwest::Error;

pub struct HttpClient {
    reqwest_client: ReqwestClient,
}

impl HttpClient {
    pub fn new() -> Self {
        Self{ reqwest_client: ReqwestClient::new() }
    }

    pub fn get(&self, query : &str, host : &str, collection: &str ) -> Result<Response, Error> {
        self.reqwest_client
            .get(format!("{}/solr/{}/select?q={}", host, collection, query))
            .send()
    }
}


