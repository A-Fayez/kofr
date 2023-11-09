use serde::{Deserialize, Serialize};

use crate::connect::{ConnectorName, State};

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
