use anyhow::{ensure, Context, Result};
use clap::{Args, Parser, Subcommand};
use clap_stdin::FileOrStdin;
use tabled::{settings::Style, Table};

use crate::connect::{ConnectorConfig, CreateConnector, DescribeConnector, HTTPClient};

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
    ///  update the configuration for an existing connector.
    Edit(Edit),
    /// get connector's status
    Status(Status),
    /// get connector's configuration
    Config(Config),
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

#[derive(Args, Debug)]
pub struct Edit {
    name: String,
}

#[derive(Args, Debug)]
pub struct Status {
    pub name: String,
}

#[derive(Args, Debug)]
pub struct Config {
    pub name: String,
}

impl List {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        let connectors = connect_client.list_connectors_status()?;
        let connectors_table = Table::new(connectors).with(Style::blank()).to_string();
        println!("{}", connectors_table);
        Ok(())
    }
}

impl Create {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        let create_connector = self.config;
        let create_connector: CreateConnector = serde_json::from_str(&create_connector)?;
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
        println!("{pretty_json}");
        Ok(())
    }
}

impl Edit {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        let old_config_json: ConnectorConfig = connect_client.get_connector_config(&self.name)?;
        let old_config = serde_json::to_string_pretty(&old_config_json)?;
        let file = tempfile::Builder::new()
            .prefix(&format!("{}-edit-", &self.name))
            .suffix(".json")
            .tempfile()
            .context("could not create tempfile for editing")?;

        let editor = Editor::new();
        std::fs::write(file.path(), &old_config).context("failed writing data to tempfile")?;
        std::process::Command::new(&editor.name)
            .arg(file.path())
            .spawn()
            .with_context(|| format!("unable to launch the editor: {}", editor.name))?
            .wait()?;

        let new_config = std::fs::read_to_string(file.path())?;
        let new_config_json: ConnectorConfig = serde_json::from_str(&new_config)?;
        if old_config_json == new_config_json {
            println!("Edit cancelled, no changes were made");
            return Ok(());
        }
        connect_client.put_connector(&self.name, new_config_json)?;
        println!("connector: {} edited.", &self.name);
        Ok(())
    }
}

impl Status {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        let status = connect_client.get_connector_status(&self.name)?;
        let status = serde_json::to_string_pretty(&status)?;
        println!("{status}");
        Ok(())
    }
}

impl Config {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        let config = connect_client.get_connector_config(&self.name)?;
        let config = serde_json::to_string_pretty(&config)?;
        println!("{config}");
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
        println!("Switched to cluster \"{}\"", self.cluster);
        Ok(())
    }
}

struct Editor {
    name: String,
}

impl Editor {
    fn new() -> Self {
        Default::default()
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            name: std::env::var("EDITOR").unwrap_or(String::from("vi")),
        }
    }
}
