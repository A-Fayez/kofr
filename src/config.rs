use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ClusterConfig {
    #[serde(rename = "current-cluster")]
    pub current_cluster: String,
    pub clusters: Vec<Cluster>,
}

impl ClusterConfig {
    pub fn from_file<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let config = std::fs::read_to_string(path)?;
        let deserialized_config: ClusterConfig = serde_yaml::from_str(&config)?;

        dbg!(&deserialized_config);
        Ok(ClusterConfig {
            current_cluster: (String::from("dev")),
            clusters: (Vec::new()),
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Cluster {
    pub name: String,
    pub hosts: Vec<String>,
}
