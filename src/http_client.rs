use reqwest::blocking::{Client as ReqwestClient, Response };
use reqwest::Error;

#[cfg(test)]
use mockall::automock;

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
}


