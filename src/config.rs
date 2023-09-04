use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    #[serde(rename = "current-cluster")]
    pub current_cluster: Option<String>,
    pub clusters: Vec<ClusterContext>,
}

impl Config {
    pub fn from_file<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let config = std::fs::read_to_string(path)?;
        let deserialized_config: Config = serde_yaml::from_str(&config)?;

        dbg!(&deserialized_config);
        Ok(deserialized_config)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ClusterContext {
    pub name: String,
    pub hosts: Vec<String>,
}
