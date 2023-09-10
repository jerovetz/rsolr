use serde::Deserialize;
use crate::Client;
use crate::error::RSolrError;
use crate::solr_response::SolrResponse;

pub struct Cursor<'a> {
    client: &'a Client<'a>
}

impl<'a> Cursor<'a> {

    pub fn get_response<T: for<'de> Deserialize<'de> + Clone>(&self) -> Result<SolrResponse<T>, RSolrError>{
        self.client.get_response::<T>()
    }

}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use super::*;

    #[test]
    fn test_response_returned_from_client() {
        let client = Client::new("http://solr.url", "dummy");
        let cursor = Cursor { client: &client };

        let response = cursor.get_response::<Value>();
        assert!(response.expect("Unexpected error from get_response").response.is_none());
    }

}