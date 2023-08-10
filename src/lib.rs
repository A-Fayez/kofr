mod error;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ureq::Agent;

use anyhow::{Context, Result};

pub struct Client {
    pub config: Config,
}

impl Client {
    pub fn from_config(config: Config) -> Self {
        Client { config }
    }

    pub fn list_connectors(&self) -> Result<Vec<ConnectorName>> {
        let _endpoint = &format!("{}/connectors", self.config.connect_uri);
        let connectors = self
            .config
            .http_agent
            .get(_endpoint)
            .set("Accept", "application/json")
            .call()
            .with_context(|| format!("Failed sending request to {}", &self.config.connect_uri))?
            .into_json::<Vec<ConnectorName>>()
            .with_context(|| {
                format!(
                    "Could not parse response returned from {}/connectors",
                    &self.config.connect_uri
                )
            })?;

        Ok(connectors)
    }

    pub fn create_connector(&self, c: CreateConnector) -> Result<Connector> {
        let _endpoint = &format!("{}/connectors", self.config.connect_uri);
        let returned_connector = self
            .config
            .http_agent
            .post(_endpoint)
            .send_json(c)
            .with_context(|| format!("could not post connector"))?
            .into_json::<Connector>()
            .with_context(|| format!("could not parse response returned"))?;
        Ok(returned_connector)
    }
}
pub struct Config {
    pub http_agent: Agent,
    pub connect_uri: String,
}

impl Config {
    pub fn from(agent: Agent, uri: String) -> Config {
        Config {
            http_agent: (agent),
            connect_uri: (uri),
        }
    }
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
