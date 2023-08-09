mod error;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ureq::Agent;

use anyhow::{Context, Result};

pub fn list_connectors(agent: &Agent, uri: &str) -> Result<Vec<ConnectorName>> {
    let connectors = agent
        .get(&format!("{}/connectors", &uri))
        .set("Accept", "application/json")
        .call()
        .with_context(|| format!("Failed sending request to {}", &uri))?
        .into_json::<Vec<ConnectorName>>()
        .with_context(|| format!("Could not parse response returned from {}/connectors", &uri))?;

    Ok(connectors)
}

type ConnectorConfig = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ConnectorName(pub String);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateConnector {
    pub name: ConnectorName,
    pub config: ConnectorConfig,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Connector {
    pub name: ConnectorName,
    pub config: ConnectorConfig,
    pub tasks: Vec<Task>,
    #[serde(rename = "type")]
    pub connector_type: ConnectorType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub connector: ConnectorName,
    #[serde(rename = "task")]
    pub id: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectorType {
    Sink,
    Source,
}

impl From<&CreateConnector> for Connector {
    fn from(connector: &CreateConnector) -> Self {
        let mut tasks = Vec::<Task>::new();
        let c_type = if connector.name.0.to_lowercase().contains("sink") {
            ConnectorType::Sink
        } else {
            ConnectorType::Source
        };

        tasks.push(Task {
            connector: (connector.name.clone()),
            id: (0),
        });
        Connector {
            name: (connector.name.clone()),
            config: (connector.config.clone()),
            tasks: (tasks),
            connector_type: (c_type),
        }
    }
}
