use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::connect::{ConnectorName, State};

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
        Err(err) => Err(anyhow!("{err}")),
    }
}

fn valid_uri(uri: &str) -> String {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatus {
    pub id: usize,
    pub state: State,
    pub worker_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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
