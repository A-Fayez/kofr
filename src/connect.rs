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
            .with_context(|| format!("Failed sending request to \"{}\"", &_endpoint))?;

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

            let connector_type = status
                .get("type")
                .with_context(|| {
                    format!(
                        r#"no field named "type" in response json struct: {}"#,
                        &response_body
                    )
                })?
                .as_str()
                .unwrap();
            let connector_type = ConnectorType::from_str(connector_type)?;

            _vec.push(VerboseConnector {
                name: ConnectorName(String::from(entry.0)),
                tasks,
                state,
                connector_type,
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
                .context("could not parse response returned"),
            Err(Error::Status(_, response)) => Err(anyhow!(" {:?}", response.into_string()?)),
            Err(err) => Err(anyhow!("{}", err)),
        }
    }

    pub fn desribe_connector(&self, name: &str) -> Result<DescribeConnector> {
        let status: ConnectorStatus = self.get_connector_status(name)?;
        let config: ConnectorConfig = self.get_connector_config(name)?;

        Ok(DescribeConnector {
            name: ConnectorName(String::from(name)),
            connector_type: status.connector_type,
            config,
            state: status.connector_state,
            tasks: status.tasks,
        })
    }

    // updates a connector's config wrapping PUT request to /connectors/<name>/config
    pub fn put_connector(self, name: &str, config: ConnectorConfig) -> Result<Connector> {
        let uri = &self.config.connect_uri;
        let config_endpoint = format!("{}/{}/config", self.valid_uri(uri), name);
        match self
            .config
            .http_agent
            .put(&config_endpoint)
            .set("Accept", "application/json")
            .set("Content-Type", "application/json")
            .send_json(config)
        {
            Ok(response) => match response.into_json::<Connector>() {
                Ok(response) => Ok(response),
                Err(e) => Err(anyhow!("{e}")),
            },
            Err(ureq::Error::Status(404, _)) => {
                Err(anyhow!("connector: \"{}\" was not found", name))
            }
            Err(ureq::Error::Status(_, response)) => {
                let response = response
                    .into_string()
                    .context("resposne was larger than 10MBs")?;
                Err(anyhow!("{response}"))
            }
            Err(ureq::Error::Transport(transport)) => {
                let message = transport.message().unwrap_or_default();
                Err(anyhow!("unexpected transport error:\n{message}"))
            }
        }
    }

    pub fn get_connector_config(&self, name: &str) -> Result<ConnectorConfig> {
        let uri = &self.config.connect_uri;
        let config_endpoint = format!("{}/{}/config", self.valid_uri(uri), name);
        match self
            .config
            .http_agent
            .get(&config_endpoint)
            .set("Accept", "application/json")
            .call()
        {
            Ok(response) => response
                .into_json()
                .context("failed parsing connector config's json"),

            Err(ureq::Error::Status(404, _)) => {
                Err(anyhow!("connector: \"{}\" was not found", name))
            }
            Err(err) => Err(anyhow!("{}", err)),
        }
    }

    pub fn get_connector_status(&self, name: &str) -> Result<ConnectorStatus> {
        let uri = &self.config.connect_uri;
        let status_endpoint = format!("{}/{}/status", self.valid_uri(uri), name);
        match self
            .config
            .http_agent
            .get(&status_endpoint)
            .set("Accept", "application/json")
            .call()
        {
            Ok(response) => response
                .into_json()
                .context("failed parsing connector status's json"),
            Err(ureq::Error::Status(404, _)) => {
                Err(anyhow!("connector: \"{}\" was not found", name))
            }
            Err(err) => Err(anyhow!("{}", err)),
        }
    }

    pub fn restart_connector(
        self,
        name: &str,
        include_tasks: bool,
        only_failed: bool,
    ) -> Result<()> {
        let uri = &self.config.connect_uri;
        let restart_endpoint = format!("{}/{}/restart", self.valid_uri(uri), name);
        match self
            .config
            .http_agent
            .post(&restart_endpoint)
            .set("Accept", "application/json")
            .query("includeTasks", &include_tasks.to_string())
            .query("onlyFailed", &only_failed.to_string())
            .call()
        {
            Ok(response) => match response.status() {
                200 | 204 => {
                    println!("connector: \"{}\" restarted sucessfully", name);
                    Ok(())
                }
                202 => {
                    let response: ConnectorStatus = response.into_json()?;
                    let response = serde_json::to_string_pretty(&response)?;
                    println!("{}", response);
                    Ok(())
                }
                _ => {
                    let response = response
                        .into_string()
                        .expect("response was larger than 10MBs");
                    println!("{response}");
                    Ok(())
                }
            },
            Err(ureq::Error::Status(_, response)) => {
                let response = response
                    .into_string()
                    .context("response was larger than 10MBs")?;
                Err(anyhow!("{response}"))
            }
            Err(err) => Err(anyhow!("{err}")),
        }
    }

    pub fn pause_connector(self, name: &str) -> Result<()> {
        let uri = &self.config.connect_uri;
        let pause_endpoint = format!("{}/{}/pause", self.valid_uri(uri), name);
        match self.config.http_agent.put(&pause_endpoint).call() {
            Ok(_) => {
                println!("connector: \"{}\" paused successfully", name);
                Ok(())
            }
            Err(ureq::Error::Status(_, r)) => Err(anyhow!("{}", r.into_string()?)),
            Err(err) => Err(anyhow!("{}", err)),
        }
    }

    pub fn resume_connector(self, name: &str) -> Result<()> {
        let uri = &self.config.connect_uri;
        let resume_endpoint = format!("{}/{}/resume", self.valid_uri(uri), name);
        match self.config.http_agent.put(&resume_endpoint).call() {
            Ok(_) => {
                println!("connector: \"{}\" resumed successfully", name);
                Ok(())
            }
            Err(ureq::Error::Status(_, r)) => Err(anyhow!("{}", r.into_string()?)),
            Err(err) => Err(anyhow!("{}", err)),
        }
    }

    pub fn delete_connector(self, name: &str) -> Result<()> {
        let uri = &self.config.connect_uri;
        let delete_endpoint = format!("{}/{}/", self.valid_uri(uri), name);
        match self.config.http_agent.delete(&delete_endpoint).call() {
            Ok(_) => {
                println!("connector: \"{}\" deleted", name);
                Ok(())
            }
            Err(ureq::Error::Status(_, r)) => Err(anyhow!("{}", r.into_string()?)),
            Err(err) => Err(anyhow!("{err}")),
        }
    }

    fn valid_uri(&self, uri: &str) -> String {
        if uri.ends_with('/') {
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
    #[tabled(rename = "TYPE")]
    pub connector_type: ConnectorType,
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
    Restarting,
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
            "RESTARTING" => Ok(State::Restarting),
            _ => Err(anyhow!(
                "unimplemneted state, valid values are: RUNNING, PAUSED, UNASSIGNED, RESTARTING and FAILED"
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
            Self::Running => write!(f, "RUNNING"),
            Self::Failed => write!(f, "FAILED"),
            Self::Paused => write!(f, "PAUSED"),
            Self::Unassigned => write!(f, "UNASSIGNED"),
            Self::Restarting => write!(f, "RESTARTING"),
        }
    }
}

impl Display for ConnectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Sink => write!(f, "SINK"),
            Self::Source => write!(f, "SOURCE"),
        }
    }
}

impl std::str::FromStr for ConnectorType {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> anyhow::Result<Self, Self::Err> {
        match input {
            "sink" => Ok(Self::Sink),
            "source" => Ok(Self::Source),
            _ => Err(anyhow!(
                "unimplemneted type, valid values are: sink, source"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kcmockserver::KcTestServer;
    use std::time::Duration;

    #[test]
    fn test_listing_connectors_should_return_empty_vec() {
        let server = KcTestServer::new();
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        let client = HTTPClient::from_config(HTTPClientConfig {
            http_agent: (agent),
            connect_uri: (server.base_url().to_string()),
        });

        let connectors_vec = client.list_connectors_status().unwrap();
        assert!(connectors_vec.is_empty());
    }

    #[test]
    fn test_creating_a_connector_should_return_the_right_connector() {
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

        let returned_connector = client.create_connector(&c).unwrap();
        assert_eq!(returned_connector, expected_connector);
    }

    #[test]
    fn test_listing_mutliple_connectors() {
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

        client.create_connector(&a).unwrap();
        client.create_connector(&b).unwrap();
        let response = client.list_connectors_status().unwrap();

        assert_eq!(response.len(), 2);
    }

    #[test]
    fn test_listing_connector_status() {
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

        client.create_connector(&c).unwrap();
        let connectors_vec = client.list_connectors_status().unwrap();

        assert_eq!(
            connectors_vec[0].name,
            ConnectorName(String::from("sink-connector"))
        );
        assert_eq!(connectors_vec[0].tasks, 1);
        assert_eq!(connectors_vec[0].state, State::Running);
    }

    #[test]
    fn test_listing_empty_connector_status() {
        let server = KcTestServer::new();
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        let client = HTTPClient::from_config(HTTPClientConfig {
            http_agent: (agent),
            connect_uri: (server.base_url().to_string()),
        });

        let connectors: Vec<VerboseConnector> = client.list_connectors_status().unwrap();

        assert_eq!(connectors.len(), 0);
    }
}
