use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Ok, Result};
use home::home_dir;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    #[serde(rename = "current-cluster")]
    pub current_cluster: Option<String>,
    pub clusters: Vec<ClusterContext>,
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        Self {
            current_cluster: None,
            clusters: Vec::new(),
            file_path: PathBuf::new(),
        }
    }

    pub fn from_file() -> Result<Self> {
        let mut path = home_dir().context("could not get user's home dir")?;
        path.push(".kofr/config");

        let config = std::fs::read_to_string(&path)?;
        let mut deserialized_config: Self = serde_yaml::from_str(&config)?;
        deserialized_config.file_path = path;

        Ok(deserialized_config)
    }

    pub fn with_file<P>(mut self, path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut config_path = home_dir().context("could not get user's home dir")?;
        config_path.push(&path);

        let config = std::fs::read_to_string(&config_path)?;
        let deserialized_config: Self = serde_yaml::from_str(&config).context("invalid config file format")?;

        self.current_cluster = deserialized_config.current_cluster;
        self.clusters = deserialized_config.clusters;
        self.file_path = config_path;

        Ok(self)
    }

    // TODO:
    // get a ClusterContext, implement http and trailing slash here, refactor ClusterContext's hosts to valid http Uri
    pub fn current_context(&self) -> Result<&ClusterContext> {
        let cluster_name = self.current_cluster.as_deref().ok_or(anyhow::anyhow!(
            "No current context was set\n consider using command: kofr config use-cluster <CLUSTER>"
        ))?;
        self.clusters
            .iter()
            .find(|&c| c.name == cluster_name)
            .ok_or(anyhow::anyhow!(format!(
                "Cluster with name {} could not be found\nConsider setting one with command: kofr config use-cluster <CLUSTER>",
                &cluster_name
            )))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ClusterContext {
    pub name: String,
    pub hosts: Vec<String>,
}

impl ClusterContext {
    pub fn available_host(&self) -> Result<String> {
        for host in &self.hosts {
            if ureq::get(&host).call().is_ok() {
                return Ok(host.to_string());
            }
        }
        Err(anyhow!(
            "client has run out of available hosts to talk to for cluster \"{}\"",
            self.name
        ))
    }
}
