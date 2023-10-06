use std::fmt::Display;
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
            let status = entry.1.get("status").with_context(|| {
                format!(
                    "no field named status in response json struct: {}",
                    &response_body
                )
            })?;
            let tasks = status
                .get("tasks")
                .with_context(|| {
                    format!(
                        "no field named tasks in response json struct: {}",
                        &response_body
                    )
                })?
                .as_array()
                .with_context(|| {
                    format!(
                        r#"expected array in "tasks" key, found something: {}"#,
                        &response_body
                    )
                })?
                .len();
            let state = State::from_str(
                status
                    .get("connector")
                    .with_context(|| {
                        format!(
                            r#"no field named "connector" in response json struct: {}"#,
                            &response_body
                        )
                    })?
                    .get("state")
                    .with_context(|| {
                        format!(
                            r#"no field named "state" in response json struct: {}"#,
                            &response_body
                        )
                    })?
                    .as_str()
                    .with_context(|| {
                        format!(
                            r#"expected string value of key "state" found something else: {}"#,
                            &response_body
                        )
                    })?,
            )?;
            let worker_id = status
                .get("connector")
                .with_context(|| {
                    format!(
                        r#"no field named "connector" in response json struct: {}"#,
                        &response_body
                    )
                })?
                .get("worker_id")
                .with_context(|| {
                    format!(
                        r#"no field named "worker_id" in response json struct: {}"#,
                        &response_body
                    )
                })?
                .as_str()
                .with_context(|| {
                    format!(
                        r#"expected string value of key "worker_id" found something else: {}"#,
                        &response_body
                    )
                })?;

            _vec.push(VerboseConnector {
                name: ConnectorName(String::from(entry.0)),
                tasks,
                state,
                worker_id: worker_id.to_string(),
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

    pub fn desribe_connector(&self, name: &str) -> Result<DescribeConnector> {
        let uri = &self.config.connect_uri;
        let status_endpoint = format!("{}/{}/status", self.valid_uri(&uri), name);
        let config_endpoint = format!("{}/{}/config", self.valid_uri(&uri), name);

        let connector_status: ConnectorStatus = match self
            .config
            .http_agent
            .get(&status_endpoint)
            .set("Accept", "application/json")
            .call()
        {
            Ok(response) => response.into_json()?,
            Err(ureq::Error::Status(404, _)) => {
                print!("endpoint: {}", &status_endpoint);
                return Err(anyhow!("connector: {} was not found", name));
            }
            Err(err) => return Err(anyhow!("{}", err)),
        };

        let connector_config: ConnectorConfig = match self
            .config
            .http_agent
            .get(&config_endpoint)
            .set("Accept", "application/json")
            .call()
        {
            Ok(response) => response.into_json()?,
            Err(ureq::Error::Status(404, _)) => {
                print!("{}", &config_endpoint);
                return Err(anyhow!("connector: {} was not found", name));
            }
            Err(err) => return Err(anyhow!("{}", err)),
        };

        Ok(DescribeConnector {
            name: ConnectorName(String::from(name)),
            connector_type: connector_status.connector_type,
            config: connector_config,
            state: connector_status.connector_state,
            tasks: connector_status.tasks,
        })
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatus {
    pub id: usize,
    pub state: State,
    pub worker_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectorState {
    pub state: State,
    pub worker_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectorStatus {
    pub name: ConnectorName,
    #[serde(rename = "connector")]
    pub connector_state: ConnectorState,
    pub tasks: Vec<TaskStatus>,
    #[serde(rename = "type")]
    pub connector_type: ConnectorType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DescribeConnector {
    pub name: ConnectorName,
    pub config: ConnectorConfig,
    #[serde(rename = "connector")]
    pub state: ConnectorState,
    pub tasks: Vec<TaskStatus>,
    #[serde(rename = "type")]
    pub connector_type: ConnectorType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectorType {
    Sink,
    Source,
}

#[derive(tabled::Tabled, Debug)]
pub struct VerboseConnector {
    #[tabled(rename = "NAME")]
    pub name: ConnectorName,
    #[tabled(rename = "STATE")]
    pub state: State,
    #[tabled(rename = "TASKS")]
    pub tasks: usize,
    #[tabled(rename = "WORKER_ID")]
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
    type Err = anyhow::Error;

    fn from_str(input: &str) -> anyhow::Result<State, Self::Err> {
        match input {
            "RUNNING" => Ok(State::Running),
            "PAUSED" => Ok(State::Paused),
            "UNASSIGNED" => Ok(State::Unassigned),
            "FAILED" => Ok(State::Failed),
            _ => Err(anyhow!(
                "unimplemneted state, valid values are: RUNNING, PAUSED, UNASSIGNED and FAILED"
            )),
        }
    }
}

impl Display for ConnectorName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "{}", "RUNNING"),
            Self::Failed => write!(f, "{}", "FAILED"),
            Self::Paused => write!(f, "{}", "PAUSED"),
            Self::Unassigned => write!(f, "{}", "UNASSIGNED"),
        }
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
