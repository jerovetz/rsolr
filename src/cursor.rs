use serde::Deserialize;
use url::Url;
use crate::Client;
use crate::error::RSolrError;
use crate::solr_response::SolrResponse;

pub struct Cursor<'a> {
    client: Client<'a>,
    cursor_mark: String,
    url: Option<Url>,
    the_end: bool
}

impl<'a> Cursor<'a> {

    pub fn new(client: Client<'a>) -> Self {
        Cursor { client, the_end: false, cursor_mark: "".to_string(), url: None }
    }

    pub fn get_response<T: for<'de> Deserialize<'de> + Clone>(&self) -> Result<SolrResponse<T>, RSolrError>{
        self.client.get_response::<T>()
    }

    pub fn next<T: for<'de> Deserialize<'de> + Clone>(&mut self) -> Result<Option<SolrResponse<T>>, RSolrError>{
        if self.the_end {
            return Ok(None)
        }

        if self.url.is_none() {
            self.url = Some(Url::parse(self.client.url_str()).expect("Url parsing failed unexpectedly"));
        } else {
            self.client.url(self.url.as_ref().unwrap().as_str());
        }

        self.client.cursor_mark(self.cursor_mark.as_str());
        match self.client.run() {
            Ok(_) => {
                match self.get_response::<T>() {
                    Ok(response) => {
                        let next_cursor_mark = response.clone().nextCursorMark.expect("nextcursormark should be there");
                        if self.cursor_mark == next_cursor_mark {
                            self.the_end = true;
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
    use mockall::predicate::eq;
    use mockall_double::double;
    use serde_json::Value;

    #[double]
    use crate::http_client::HttpClient;

    use super::*;

    #[test]
    fn test_response_returned_from_client() {
        let client = Client::new("http://solr.url", "dummy");
        let cursor = Cursor { client, cursor_mark: "".to_string(), url: None, the_end: false };

        let response = cursor.get_response::<Value>();
        assert!(response.expect("Unexpected error from get_response").response.is_none());
    }

    #[test]
    fn test_next_returns_the_next_response() {

        let ctx = HttpClient::new_context();
        ctx.expect().returning(|| {
            let mut mock = HttpClient::default();
            mock.expect_get()
                .with(eq("http://solr.url/solr/dummy/select?q=*%3A*&rows=1&sort=unique+asc&cursorMark=first_cursor_mark"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{"response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success": true }]}, "nextCursorMark": "second_cursor_mark"}"#)
                    .unwrap())));

            mock.expect_get()
                .with(eq("http://solr.url/solr/dummy/select?q=*%3A*&rows=1&sort=unique+asc&cursorMark=second_cursor_mark"))
                .returning(|_| Ok(reqwest::blocking::Response::from(http::response::Builder::new()
                    .status(200)
                    .body(r#"{"response": {"numFound": 1,"numFoundExact": true,"start": 0,"docs": [{"success2": true }]}, "nextCursorMark": "second_cursor_mark"}"#)
                    .unwrap())));

            mock
        });

        let mut client = Client::new("http://solr.url", "dummy");
        client
            .select("*:*")
            .rows(1)
            .sort("unique asc");

        let mut cursor = Cursor { client, cursor_mark: "first_cursor_mark".to_string(), url: None, the_end: false };
        let result = cursor.next::<Value>();
        assert_eq!(result.expect("Ok expected").expect("Response expected").response.expect("solr response expected").docs[0].get("success").unwrap(), true);

        let result2 = cursor.next::<Value>();
        assert_eq!(result2.expect("Ok expected").expect("Response expected").response.expect("solr response expected").docs[0].get("success2").unwrap(), true);

        let result3 = cursor.next::<Value>();
        assert!(result3.expect("Ok expected").is_none());
    }

}