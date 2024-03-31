use http::header::CONTENT_TYPE;
use reqwest::blocking::{Client as ReqwestClient, Response };
use reqwest::Error;

#[cfg(test)]
use mockall::automock;
use serde_json::Value;

#[allow(dead_code)]
pub struct HttpClient {
    reqwest_client: ReqwestClient,
}

#[cfg_attr(test, automock)]
impl HttpClient {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self{ reqwest_client: ReqwestClient::new() }
    }

    #[allow(dead_code)]
    pub fn get(&self, query : &str ) -> Result<Response, Error> {
        self.reqwest_client
            .get(query)
            .send()
    }

    #[allow(dead_code)]
    pub fn post<'a>(&self, query : &str, body: Option<&'a Value>) -> Result<Response, Error> {
        let request = self.reqwest_client.post(query);
        match body {
            Some(body) => request
                .header(CONTENT_TYPE, "application/json")
                .json::<Value>(body).send(),
            None => request.send()
        }
    }
}


