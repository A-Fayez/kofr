use std::env;
use std::time::Duration;
use ureq::Agent;

mod config;
mod connect;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use clap_stdin::FileOrStdin;
use connect::{Client, Config, CreateConnector};

fn main() -> Result<()> {
    let uri = env::var("CONNECT_URI").expect("env var CONNECT_URI not found");

    let config_dir = config::ClusterConfig::from_file("/Users/fayez/.kofr/config")?;

    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let client = Client::from_config(Config {
        http_agent: (agent),
        connect_uri: (uri.to_owned()),
    });

    let cli = Cli::parse();

    match cli.command {
        Action::List(list) => list.run(client)?,
        Action::ConnectorAction(connector_command) => match connector_command {
            ConnectorAction::Create(create) => create.run(client)?,
        },
    }

    Ok(())
}

/// Kafka Connect CLI for connect cluster management
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "git")]
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
}

#[derive(Args, Debug)]
struct List {}

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
    fn run(self, connect_client: Client) -> Result<()> {
        let connectors = connect_client.list_connectors()?;
        for c in &connectors {
            println!("{}", c.0)
        }
        Ok(())
    }
}

impl Create {
    fn run(self, connect_client: Client) -> Result<()> {
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
