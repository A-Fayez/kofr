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
        let _endpoint = &format!("{}connectors", self.config.connect_uri);
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

    pub fn create_connector(&self, c: &CreateConnector) -> Result<Connector> {
        let _endpoint = &format!("{}connectors", self.config.connect_uri);
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

pub type ConnectorConfig = HashMap<String, String>;

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

// impl std::fmt::Display for ConnectorName {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }

impl<T: Into<String>> From<T> for ConnectorName {
    fn from(src: T) -> Self {
        Self(src.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kcmockserver::KcTestServer;
    use std::time::Duration;

    #[tokio::test]
    async fn test_listing_connectors_should_return_empty_vec() {
        let server = KcTestServer::new();
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        let client = Client::from_config(Config {
            http_agent: (agent),
            connect_uri: (server.base_url().to_string()),
        });

        let connectors_vec = tokio::task::spawn_blocking(move || client.list_connectors().unwrap())
            .await
            .unwrap();
        assert_eq!(connectors_vec, Vec::new());
    }

    #[tokio::test]
    async fn test_creating_a_connector_should_return_the_right_connector() {
        let server = KcTestServer::new();
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        let client = Client::from_config(Config {
            http_agent: (agent),
            connect_uri: (server.base_url().to_string()),
        });

        let c = r#"
        {
            "name": "sink-connector",
            "config": {
                "tasks.max": "10",
                "connector.class": "com.example.kafka",
                "name": "sink-connector"
            }
        }"#;

        let c = serde_json::from_str(c).unwrap();
        let expected_connector = Connector::from(&c);

        let returned_connector =
            tokio::task::spawn_blocking(move || client.create_connector(&c).unwrap())
                .await
                .unwrap();

        assert_eq!(returned_connector, expected_connector);
    }

    #[tokio::test]
    async fn test_listing_mutliple_connectors() {
        let server = KcTestServer::new();
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        let client = Client::from_config(Config {
            http_agent: (agent),
            connect_uri: (server.base_url().to_string()),
        });

        let a = r#"
        {
            "name": "sink-connector",
            "config": {
                "tasks.max": "10",
                "connector.class": "com.example.kafka",
                "name": "sink-connector"
            }
        }"#;
        let b = r#"
        {
            "name": "source-connector",
            "config": {
                "tasks.max": "5",
                "connector.class": "com.example.mongo",
                "name": "source-connector"
            }
        }"#;

        let a = serde_json::from_str(&a).unwrap();
        let b = serde_json::from_str(&b).unwrap();

        let response = tokio::task::spawn_blocking(move || {
            client.create_connector(&a).unwrap();
            client.create_connector(&b).unwrap();
            client.list_connectors().unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.len(), 2);
    }
}
