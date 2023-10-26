use serde::Deserialize;
use url::Url;
use crate::Client;
use crate::error::RSolrError;
use crate::solr_response::SolrResponse;

/// Pagination cursor.
#[derive(Debug)]
pub struct Cursor<'a> {
    client: Client<'a>,
    cursor_mark: String,
    url: Option<Url>,
}

impl<'a> Cursor<'a> {

    /// Usually you don't need to instantiate this.
    pub fn new(client: Client<'a>, cursor_mark: String) -> Self {
        Cursor { client, cursor_mark, url: None }
    }

    /// Wrapper of the client response getter, you can get the first page response through the cursor as well.
    pub fn get_response<T: for<'de> Deserialize<'de> + Clone + Default>(&self) -> Result<SolrResponse<T>, RSolrError>{
        self.client.get_response::<T>()
    }

    /// Fetches and parse the pages.
    pub fn next<T: for<'de> Deserialize<'de> + Clone + Default>(&mut self) -> Result<Option<SolrResponse<T>>, RSolrError>{
        if self.url.is_none() {
            self.url = Some(Url::parse(self.client.url_str()).expect("Url parsing failed unexpectedly"));
        } else {
            self.client.url(self.url.as_ref().unwrap().as_str());
        }
        self.client.update_cursor_mark(self.cursor_mark.as_str());
        match self.client.run() {
            Ok(_) => {
                match self.get_response::<T>() {
                    Ok(response) => {
                        let next_cursor_mark = response.clone().nextCursorMark.expect("nextcursormark should be in response");
                        if self.cursor_mark == next_cursor_mark {
                            return Ok(None)
                        }
                        self.cursor_mark = next_cursor_mark;
                        Ok(Some(response))
                    },
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }

}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use super::*;

    #[test]
    fn response_returned_from_client() {
        let client = Client::new("http://solr.url", "dummy");
        let cursor = Cursor { client, cursor_mark: "".to_string(), url: None };

        let response = cursor.get_response::<Value>();
        assert!(response.expect("Unexpected error from get_response").response.is_none());
    }

}