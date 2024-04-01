### RSolr

A Solr client for Rust.

`Rsolr` provides capabilities to manipulate and form
requests to the Solr server, and contains some shorthands
for them, with support for some facet features and query builder. 
It uses the blocking version of the reqwest http client.

 ## Select

 You can retrieve documents as types with implemented `Clone` and `Deserialize`.

 ```rust
 use serde_json::Value;
 use rsolr::Client;
 use rsolr::error::RSolrError;
 use rsolr::solr_result::SolrResponse;

 fn query_all() -> Result<SolrResponse<Value>, RSolrError> {
     let result = Client::new("http://solr:8983", "collection")
         .select("*:*")
         .run::<Value>();
     match result {
         Ok(solr_result) => Ok(solr_result.expect("Request is OK, but no response; in select it's a failure on Solr side.")),
         Err(e) => Err(e)
     }
 }
 ```

 ## Upload JSON doc(s)

 You should use types with implemented `Clone` and `Serialize`.

 ```rust

 use serde::Serialize;
 use serde_json::Value;
 use rsolr::Client;

 #[derive(Serialize, Clone)]
 struct SimpleDocument {
     field: Vec<String>
 }

 fn upload() {
     let document = SimpleDocument { field: vec!("nice".to_string(), "document".to_string()) };
     Client::new("http://solr:8983", "collection")
         .upload_json(document)
         .run().expect("request failed.");
 }
 ```
 ## Delete

 ```rust
 use serde_json::Value;
 use rsolr::Client;
 fn delete() {
     Client::new("http://solr:8983", "collection")
         .delete("delete:query")
         .run::<Value>().expect("panic, request failed.");
 }
 ```

 ## Custom handler with params

 You can define any handlers as well.

 ```rust

 use serde_json::Value;
 use rsolr::Client;
 use rsolr::error::RSolrError;
 use rsolr::solr_result::SolrResponse;
 fn more_like_this()  -> Result<SolrResponse<Value>, RSolrError> {
     let result = Client::new("http://solr:8983", "collection")
         .request_handler("mlt")
         .add_query_param("mlt.fl", "similarity_field")
         .add_query_param("mlt.mintf", "4")
         .add_query_param("mlt.minwl", "3")
         .run::<Value>();
     match result {
         Ok(solr_result) => Ok(solr_result.expect("Request is OK, but no response; in select it's a failure on Solr side.")),
         Err(e) => Err(e)
     }
 }
 ```

## Cursor-based pagination

Paginated results can be fetched iteratively with the use of [solr cursor](https://solr.apache.org/guide/solr/latest/query-guide/pagination-of-results.html#fetching-a-large-number-of-sorted-results-cursors)

```rust
use serde_json::Value;
use rsolr::Client;
use rsolr::solr_response::SolrResponse;
fn cursor_fetch_all_pages() -> Vec<SolrResponse<Value>> {
    let mut responses = Vec::new();
    let mut client = Client::new("http://solr:8983", "collection");
    let result = client
        .select("*:*")
        .sort("id asc")
        .cursor()
        .run();
    let mut cursor = result.expect("request failed").expect("no cursor");
    while cursor.next::<Value>().expect("request failed").is_some() {
        responses.push(cursor.get_response::<Value>().expect("parsing failed"));
    }
    responses
}
```

## Development
I use [Cargo Run Script](https://crates.io/crates/cargo-run-script) to setup and manage a Solr locally, specified the latest Solr to stay up-to-date. Solr 8+ is supported. You'll also need a [Docker](https://docs.docker.com/get-docker/). After checkout you should run

    cargo run-script solr-start
    cargo run-script solr-provision

Now you can reach your local Solr on `http://localhost:8983`. For testing I created a default collection without any schema def. Practically it means every value will be [multivalue](https://solr.apache.org/guide/7_4/field-type-definitions-and-properties.html#field-default-properties) by default.

