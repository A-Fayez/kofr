mod error;

use serde::{Deserialize, Serialize};
use std::fmt;
use ureq::Agent;

use error::Result;

pub fn get_all_connectors(agent: &Agent, uri: &str) -> Result<Vec<Connector>> {
    let connectors: Vec<Connector> = agent
        .get(uri)
        .set("Accept", "application/json")
        .call()?
        .into_json()?;

    Ok(connectors)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Connector {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UnionConnector(String);

impl fmt::Display for Connector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
