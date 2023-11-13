use std::path::PathBuf;

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
    pub fn new() -> Result<Self> {
        let mut default_config_path = home_dir().context("could not get user's home dir")?;
        default_config_path.push(".kofr/config");
        let config = Self {
            current_cluster: None,
            clusters: Vec::new(),
            file_path: PathBuf::new(),
        };
        if !default_config_path.exists() {
            std::fs::create_dir_all(default_config_path.parent().unwrap())?;
            let config_yaml =
                serde_yaml::to_string(&config).context("invalid config yaml format")?;
            std::fs::write(&default_config_path, config_yaml)
                .context("failed writing file to filesystem")?;
        }
        Ok(config)
    }

    pub fn with_file(mut self, path: PathBuf) -> Result<Self> {
        let config = std::fs::read_to_string(&path).with_context(|| {
            format!("error reading config file \"{}\"", &path.to_string_lossy())
        })?;
        let deserialized_config: Self =
            serde_yaml::from_str(&config).context("invalid config file format")?;

        self.current_cluster = deserialized_config.current_cluster;
        self.clusters = deserialized_config.clusters;
        self.file_path = path;

        Ok(self)
    }

    pub fn current_context(&self) -> Result<&ClusterContext> {
        let cluster_name = self.current_cluster.as_deref().ok_or(anyhow::anyhow!(
            "No current context was set\n consider using command: kofr config use-cluster <CLUSTER>"
        ))?;
        self.clusters
            .iter()
            .find(|&c| c.name == cluster_name)
            .ok_or(anyhow::anyhow!(format!(
                "Cluster with name: \"{}\" could not be found\nConsider setting one with command: kofr config use-cluster <CLUSTER>",
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
            if ureq::get(host).call().is_ok() {
                return Ok(host.to_string());
            }
        }
        Err(anyhow!(
            "client has run out of available hosts to talk to for cluster: \"{}\"",
            self.name
        ))
    }
}
