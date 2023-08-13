use std::env;
use std::path::PathBuf;
use std::time::Duration;
use ureq::Agent;

mod error;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use kofr::{Client, Config, CreateConnector};

fn main() -> Result<()> {
    let uri = env::var("CONNECT_URI").expect("env var CONNECT_URI not found");

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
        Commands::List(list) => list.run(client)?,
        Commands::ConnectorCmd(connector_command) => match connector_command {
            ConnectorCmd::Create(create) => create.run(client)?,
        },
    }

    Ok(())
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// lists active connectors in current cluster
    #[clap(alias = "ls")]
    List(List),
    /// operate on connectors
    #[command(subcommand)]
    #[clap(name = "connector", alias = "cn")]
    ConnectorCmd(ConnectorCmd),
}

#[derive(Args, Debug)]
struct List {}

#[derive(Subcommand, Debug)]
enum ConnectorCmd {
    /// creates a connector
    Create(Create),
}

#[derive(Args, Debug)]
struct Create {
    #[arg(short = 'f', long = "file")]
    config: PathBuf,
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
        let create_connector = std::fs::read_to_string(self.config)?;
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
