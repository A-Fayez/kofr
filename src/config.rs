use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use home::home_dir;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    #[serde(rename = "current-cluster")]
    pub current_cluster: Option<String>,
    pub clusters: Vec<ClusterContext>,
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl Config {
    pub fn from_file() -> Result<Self> {
        let mut path = home_dir().context("could not get user's home dir")?;
        path.push(".kofr/config");

        let config = std::fs::read_to_string(&path)?;
        let mut deserialized_config: Config = serde_yaml::from_str(&config)?;
        deserialized_config.file_path = path;

        dbg!(&deserialized_config);
        Ok(deserialized_config)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ClusterContext {
    pub name: String,
    pub hosts: Vec<String>,
}
