use url;

pub struct Command<'b> {
    collection: &'b str,
    request_handler: &'b str,
    url: url::Url
}

impl<'b> Command<'b> {

    pub fn new(base_url: &'b str, collection: &'b str) -> Self {
        Command { collection, request_handler: "", url: url::Url::parse(base_url).unwrap() }
    }

    pub fn add_query_param(&mut self, key: &str, value: &str) -> &mut Self {
        self.url.query_pairs_mut().append_pair(key, value);
        self
    }

    pub fn request_handler(&mut self, handler: &'b str) -> &mut Self {
        self.request_handler = handler;
        self
    }

    pub fn auto_commit(&mut self) -> &mut Self {
        self.add_query_param("commit", "true");
        self
    }

    pub fn query(&mut self, query: &str) -> &mut Self {
        self.add_query_param("q", query);
        self
    }

    pub fn get_url(&'b mut self) -> &'b str {
        self.url.path_segments_mut().unwrap().push("solr");
        self.url.path_segments_mut().unwrap().push(self.collection);
        self.url.path_segments_mut().unwrap().push(self.request_handler);
        self.url.as_str()
    }

}

#[cfg(test)]
mod tests {
    use crate::command::Command;

    #[test]
    fn test_build_a_url_from_parameters() {
        let mut params = Command::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .query("*:*");

        let url_string = params.get_url();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?q=*%3A*");
    }

    #[test]
    fn test_build_a_url_from_parameters_set_autocommit() {
        let mut params = Command::new("http://host:8983", "collection");
        params
            .request_handler("request_handler")
            .auto_commit();

        let url_string = params.get_url();
        assert_eq!(url_string, "http://host:8983/solr/collection/request_handler?commit=true");
    }


}