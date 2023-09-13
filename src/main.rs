use std::env;
use std::time::Duration;
use ureq::Agent;

mod config;
mod connect;
use anyhow::{ensure, Result};
use clap::{Args, Parser, Subcommand};
use clap_stdin::FileOrStdin;
use connect::{CreateConnector, HTTPClient};

fn main() -> Result<()> {
    let uri = env::var("CONNECT_URI").expect("env var CONNECT_URI not found");

    let mut cluster_config = config::Config::from_file()?;

    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let client = HTTPClient::from_config(connect::HTTPClientConfig {
        http_agent: (agent),
        connect_uri: (uri.to_owned()),
    });

    let cli = Cli::parse();

    match cli.command {
        Action::List(list) => list.run(client)?,
        Action::ConnectorAction(connector_command) => match connector_command {
            ConnectorAction::Create(create) => create.run(client)?,
        },
        Action::ConfigAction(config_command) => match config_command {
            ConfigAction::UseCluster(cluster) => cluster.run(&mut cluster_config)?,
        },
    }

    Ok(())
}

/// Kafka Connect CLI for connect cluster management
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
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
struct List {}

#[derive(Subcommand, Debug)]
#[clap(name = "use-cluster")]
enum ConfigAction {
    UseCluster(UseCluster),
}

#[derive(Args, Debug)]
struct UseCluster {
    cluster: String,
}

#[derive(Subcommand, Debug)]
enum ConnectorAction {
    /// creates a connector
    Create(Create),
}

#[derive(Args, Debug)]
struct Create {
    #[arg(short = 'f', long = "file")]
    config: FileOrStdin,
}

impl List {
    fn run(self, connect_client: HTTPClient) -> Result<()> {
        let connectors = connect_client.list_connectors()?;
        for c in &connectors {
            println!("{}", c.0)
        }
        Ok(())
    }
}

impl Create {
    fn run(self, connect_client: HTTPClient) -> Result<()> {
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

impl UseCluster {
    fn run(self, current_config: &mut config::Config) -> Result<()> {
        let clusters: Vec<&String> = current_config.clusters.iter().map(|c| &c.name).collect();

        ensure!(
            clusters.contains(&&self.cluster),
            format!("Cluster with name {} could not be found", &self.cluster)
        );

        current_config.current_cluster = Some(self.cluster);

        let updated_config_yaml = serde_yaml::to_string(&current_config)?;
        std::fs::write(&current_config.file_path, updated_config_yaml)?;

        dbg!(clusters);
        Ok(())
    }
}
