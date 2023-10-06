mod cli;
mod config;
mod connect;

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use ureq::Agent;

use cli::*;
use connect::HTTPClient;

fn main() -> Result<()> {
    let mut cluster_config = config::Config::from_file()?;
    let current_context = cluster_config.current_context()?;
    let uri = &cluster_config.current_context()?.hosts[0];

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
            ConnectorAction::Describe(describe) => describe.run(client)?,
        },
        Action::ConfigAction(config_command) => match config_command {
            ConfigAction::UseCluster(cluster) => cluster.run(&mut cluster_config)?,
            ConfigAction::CurrentContext => println!("{}", current_context.name),
            ConfigAction::GetClusters => {
                for cluster in &cluster_config.clusters {
                    println!("{}", cluster.name)
                }
            }
        },
    }

    Ok(())
}
