pub mod error;
pub mod command;
mod http_client;

use serde_json::{json, Value};
use crate::error::RSCError;
use crate::command::{Command, Payload};

#[derive(PartialEq)]
pub enum AutoCommit {
    YES,
    NO
}

pub struct Client<'a> {
    host: &'a str,
    collection: &'a str,
    auto_commit: AutoCommit
}

impl<'a> Client<'a> {

    pub fn query(&self, query: &str) -> Result<Value, RSCError> {
        let mut command = Command::new(&self.host, &self.collection);
        command
            .request_handler("select")
            .query(query)
            .run()
    }

    pub fn create(&self, document: Value) -> Result<(), RSCError> {
        let mut command_stub = Command::new(&self.host, &self.collection);
        let command = command_stub
            .request_handler("update/json/docs")
            .payload(Payload::Body(document));

        if AutoCommit::YES == self.auto_commit {
            command.auto_commit();
        }

        command.run().map(|_| { () })
    }

    pub fn commit(&self) -> Result<(), RSCError> {
        let mut command = Command::new(&self.host, &self.collection);
        command
            .request_handler("update")
            .auto_commit()
            .payload(Payload::Empty)
            .run().map(|_| { () })
    }

    pub fn delete(&self, query: &str) -> Result<(), RSCError> {
        let delete_payload = json!({
            "delete": { "query": query }
        });

        let mut command_stub = Command::new(&self.host, &self.collection);
        let command = command_stub
            .request_handler("update")
            .payload(Payload::Body(delete_payload));

        if AutoCommit::YES == self.auto_commit {
            command.auto_commit();
        }

        command.run().map(|_| { () })
    }

    pub fn new(host : &'a str, collection : &'a str, auto_commit: AutoCommit) -> Self {
        Self {
            host,
            collection,
            auto_commit
        }
    }
}