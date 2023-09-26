use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use ureq::{Agent, Error};

pub struct HTTPClient {
    pub config: HTTPClientConfig,
}

impl HTTPClient {
    pub fn from_config(config: HTTPClientConfig) -> Self {
        Self { config }
    }

    pub fn list_connectors(&self) -> Result<Vec<ConnectorName>> {
        let uri = &self.config.connect_uri;
        let _endpoint = self.valid_uri(uri);

        let connectors = self
            .config
            .http_agent
            .get(&_endpoint)
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

    pub fn list_connectors_status(&self) -> Result<Vec<VerboseConnector>> {
        let uri = &self.config.connect_uri;
        let _endpoint = self.valid_uri(uri);

        let response = self
            .config
            .http_agent
            .get(&_endpoint)
            .set("Accept", "application/json")
            .query("expand", "status")
            .call()
            .with_context(|| format!("Failed sending request to {}", &_endpoint))?;

        let response_body = response.into_string()?;

        let mut _vec: Vec<VerboseConnector> = Vec::new();
        if response_body == "[]" {
            return Ok(_vec);
        }

        let response: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&response_body)?;
        let connectors_status: serde_json::Map<String, serde_json::Value> = response;

        for entry in &connectors_status {
            _vec.push(VerboseConnector {
                name: ConnectorName(String::from(entry.0)),
                tasks: entry
                    .1
                    .get("status")
                    .unwrap()
                    .get("tasks")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .len(),
                state: State::from_str(
                    entry
                        .1
                        .get("status")
                        .unwrap()
                        .get("connector")
                        .unwrap()
                        .get("state")
                        .unwrap()
                        .as_str()
                        .unwrap(),
                )
                .unwrap(),
                worker_id: entry
                    .1
                    .get("status")
                    .unwrap()
                    .get("connector")
                    .unwrap()
                    .get("worker_id")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            })
        }
        Ok(_vec)
    }

    pub fn create_connector(&self, c: &CreateConnector) -> Result<Connector> {
        let uri = &self.config.connect_uri;
        let _endpoint = self.valid_uri(uri);

        match self
            .config
            .http_agent
            .post(&_endpoint)
            .set("Content-Type", "application/json")
            .send_json(c)
        {
            Ok(response) => response
                .into_json::<Connector>()
                .with_context(|| format!("could not parse response returned")),
            Err(Error::Status(_, response)) => {
                return Err(anyhow!(" {:?}", response.into_string()?));
            }
            Err(err) => return Err(anyhow!("{}", err)),
        }
    }

    fn valid_uri(&self, uri: &str) -> String {
        if uri.ends_with("/") {
            return format!("{}connectors", uri);
        }
        format!("{}/connectors", uri)
    }
}
pub struct HTTPClientConfig {
    pub http_agent: Agent,
    pub connect_uri: String,
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

#[derive(Debug)]
pub struct VerboseConnector {
    pub name: ConnectorName,
    pub tasks: usize,
    pub state: State,
    pub worker_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum State {
    Running,
    Failed,
    Unassigned,
    Paused,
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

impl std::str::FromStr for State {
    type Err = ();

    fn from_str(input: &str) -> Result<State, Self::Err> {
        match input {
            "RUNNING" => Ok(State::Running),
            "PAUSED" => Ok(State::Paused),
            "UNASSIGNED" => Ok(State::Unassigned),
            "FAILED" => Ok(State::Failed),
            _ => Err(()),
        }
    }
}

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

        let client = HTTPClient::from_config(HTTPClientConfig {
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

        let client = HTTPClient::from_config(HTTPClientConfig {
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

        let client = HTTPClient::from_config(HTTPClientConfig {
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

    #[tokio::test]
    async fn test_listing_connector_status() {
        let server = KcTestServer::new();
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        let client = HTTPClient::from_config(HTTPClientConfig {
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

        let c: CreateConnector = serde_json::from_str(c).unwrap();
        let connectors_vec = tokio::task::spawn_blocking(move || {
            client.create_connector(&c).unwrap();
            client.list_connectors_status().unwrap()
        })
        .await
        .unwrap();

        assert_eq!(
            connectors_vec[0].name,
            ConnectorName(String::from("sink-connector"))
        );
        assert_eq!(connectors_vec[0].tasks, 1);
        assert_eq!(connectors_vec[0].state, State::Running);
    }

    #[tokio::test]
    async fn test_listing_empty_connector_status() {
        let server = KcTestServer::new();
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        let client = HTTPClient::from_config(HTTPClientConfig {
            http_agent: (agent),
            connect_uri: (server.base_url().to_string()),
        });

        let connectors: Vec<VerboseConnector> =
            tokio::task::spawn_blocking(move || client.list_connectors_status().unwrap())
                .await
                .unwrap();

        dbg!(&connectors);
        assert_eq!(connectors.len(), 0);
    }
}
