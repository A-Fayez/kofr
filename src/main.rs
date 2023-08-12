use std::env;
use std::time::Duration;
use ureq::Agent;

mod error;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use kofr::{Client, Config};

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

    match &cli.command {
        Commands::List(list) => list.run(client)?,
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
}

#[derive(Args, Debug)]
struct List {}

impl List {
    fn run(&self, connect_client: Client) -> Result<()> {
        let connectors = connect_client.list_connectors()?;
        for c in &connectors {
            println!("{}", c.0)
        }
        Ok(())
    }
}
