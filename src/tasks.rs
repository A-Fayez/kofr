use std::collections::HashMap;
use std::fmt::Display;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::connect::ConnectorName;

pub fn list_tasks(host: &str, connector_name: &str) -> Result<Vec<TaskResponse>> {
    let endpoint = valid_uri(host);
    let endpoint = format!("{}/{}/tasks", &endpoint, connector_name);

    match ureq::get(&endpoint)
        .set("Accept", "application/json")
        .call()
    {
        Ok(response) => response
            .into_json::<Vec<TaskResponse>>()
            .context("invalid json returned from api"),
        Err(ureq::Error::Status(404, _)) => {
            Err(anyhow!("connector: \"{}\" was not found", connector_name))
        }
        Err(ureq::Error::Status(_, r)) => Err(anyhow!("{}", r.into_string()?)),
        Err(err) => Err(anyhow!("{}", err)),
    }
}

pub fn task_status(host: &str, connector_name: &str, task_id: usize) -> Result<TaskStatus> {
    let endpoint = valid_uri(host);
    let endpoint = format!("{}/{}/tasks/{}/status", &endpoint, connector_name, task_id);

    match ureq::get(&endpoint)
        .set("Accept", "application/json")
        .call()
    {
        Ok(response) => response
            .into_json::<TaskStatus>()
            .context("invalid json returned from api"),
        Err(ureq::Error::Status(404, _)) => Err(anyhow!(
            "No status found for task {}-{}",
            connector_name,
            task_id
        )),
        Err(ureq::Error::Status(_, r)) => Err(anyhow!("{}", r.into_string()?)),
        Err(err) => Err(anyhow!("{}", err)),
    }
}

pub fn restart_task(host: &str, connector_name: &str, task_id: usize) -> Result<()> {
    let endpoint = valid_uri(host);
    let endpoint = format!("{}/{}/tasks/{}/restart", &endpoint, connector_name, task_id);
    match ureq::post(&endpoint)
        .set("Accept", "application/json")
        .set("Content-Type", "application/json")
        .call()
    {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(404, _)) => {
            Err(anyhow!("task: '{}/{}' not found", connector_name, task_id))
        }
        Err(ureq::Error::Status(_, r)) => Err(anyhow!("{}", r.into_string()?)),
        Err(err) => Err(anyhow!("{err}")),
    }
}

pub fn valid_uri(uri: &str) -> String {
    if uri.ends_with('/') {
        return format!("{}connectors", uri);
    }
    format!("{}/connectors", uri)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub connector: ConnectorName,
    #[serde(rename = "task")]
    pub id: usize,
}

#[derive(Debug, Serialize, Deserialize, tabled::Tabled)]
pub struct TaskStatus {
    #[tabled(rename = "ID")]
    pub id: usize,
    #[tabled(rename = "STATE")]
    pub state: TaskState,
    #[tabled(rename = "WORKER_ID")]
    pub worker_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tabled(display_with = "display_option")]
    #[tabled(rename = "TRACE")]
    pub trace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: TaskID,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskID {
    pub connector: String,
    pub task: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskState {
    Running,
    Failed,
    Paused,
    Restarting,
    Lost,
    Created,
    Dead,
}

impl std::str::FromStr for TaskState {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> anyhow::Result<TaskState, Self::Err> {
        match input {
            "RUNNING" => Ok(TaskState::Running),
            "FAILED" => Ok(TaskState::Failed),
            "RESTARTING" => Ok(TaskState::Restarting),
            "LOST" => Ok(TaskState::Lost),
            "CREATED" => Ok(TaskState::Created),
            "DEAD" => Ok(TaskState::Dead),
            _ => Err(anyhow!("unimplemneted state")),
        }
    }
}

impl Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "RUNNING"),
            Self::Failed => write!(f, "FAILED"),
            Self::Paused => write!(f, "PAUSED"),
            Self::Lost => write!(f, "LOST"),
            Self::Restarting => write!(f, "RESTARTING"),
            Self::Created => write!(f, "CREATED"),
            Self::Dead => write!(f, "DEAD"),
        }
    }
}

fn display_option(o: &Option<String>) -> String {
    match o {
        Some(s) => s.to_string(),
        None => "-".to_string(),
    }
}
