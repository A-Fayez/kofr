mod error;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use ureq::Agent;

use error::{KofrError, Result};

pub fn get_all_connectors(agent: &Agent, uri: &str) -> Result<Vec<Connector>> {
    let connectors: Value = agent
        .get(uri)
        .set("Accept", "application/json")
        .call()?
        .into_json()?;

    let connectors = connectors.as_array().ok_or_else(|| KofrError::NotAJsonArrayError(uri.to_owned()))?;
    let mut connectors_vec: Vec<Connector> = Vec::new();
    for c in connectors.into_iter() {
        let connector_name: String = serde_json::from_value(c.to_owned())?;
        connectors_vec.push(Connector {
            name: (connector_name),
        });
    }

    Ok(connectors_vec)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Connector {
    name: String,
}

impl fmt::Display for Connector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
