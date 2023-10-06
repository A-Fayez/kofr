use anyhow::{ensure, Result};
use clap::{Args, Parser, Subcommand};
use clap_stdin::FileOrStdin;
use tabled::{settings::Style, Table};

use crate::connect::{CreateConnector, DescribeConnector, HTTPClient};

/// Kafka Connect CLI for connect cluster management
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Action,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// lists active connectors in current cluster
    #[clap(alias = "ls")]
    List(List),

    /// operate on connectors
    #[command(subcommand)]
    #[clap(name = "connector", alias = "cn")]
    ConnectorAction(ConnectorAction),

    /// Handle kofr configuration
    #[command(subcommand)]
    #[clap(name = "config")]
    ConfigAction(ConfigAction),
}

#[derive(Args, Debug)]
pub struct List {}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    #[clap(name = "use-cluster")]
    UseCluster(UseCluster),
    #[clap(name = "current-context")]
    CurrentContext,
    #[clap(name = "get-clusters")]
    GetClusters,
}

#[derive(Args, Debug)]
pub struct UseCluster {
    pub cluster: String,
}

#[derive(Subcommand, Debug)]
pub enum ConnectorAction {
    /// creates a connector
    Create(Create),
    /// describes a connector's config and status
    Describe(Describe),
}

#[derive(Args, Debug)]
pub struct Create {
    #[arg(short = 'f', long = "file")]
    pub config: FileOrStdin,
}

#[derive(Args, Debug)]
pub struct Describe {
    pub name: String,
}

impl List {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        let connectors = connect_client.list_connectors_status()?;
        let connectors_table = Table::new(connectors).with(Style::blank()).to_string();
        println!("{}\n", connectors_table);
        Ok(())
    }
}

impl Create {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        dbg!(&self.config);
        let create_connector = self.config;
        let create_connector: CreateConnector = serde_json::from_str(&create_connector)?;
        dbg!(&create_connector);
        dbg!(&create_connector.name.0);
        let response = connect_client.create_connector(&create_connector)?;
        let response = serde_json::to_string_pretty(&response)?;
        println!(
            "successfully created connector: {}",
            &create_connector.name.0
        );
        println!("{}", response);
        Ok(())
    }
}

impl Describe {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        let describe_connector: DescribeConnector = connect_client.desribe_connector(&self.name)?;
        let pretty_json = serde_json::to_string_pretty(&describe_connector)?;
        print!("{pretty_json}\n");
        Ok(())
    }
}

impl UseCluster {
    pub fn run(self, current_config: &mut crate::config::Config) -> Result<()> {
        let clusters: Vec<&String> = current_config.clusters.iter().map(|c| &c.name).collect();

        ensure!(
            clusters.contains(&&self.cluster),
            format!("Cluster with name {} could not be found", &self.cluster)
        );

        current_config.current_cluster = Some(self.cluster.clone());

        let updated_config_yaml = serde_yaml::to_string(&current_config)?;
        std::fs::write(&current_config.file_path, updated_config_yaml)?;
        print!("Switched to cluster \"{}\"\n", self.cluster);
        Ok(())
    }
}
