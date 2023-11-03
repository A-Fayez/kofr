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
    let mut cluster_config = config::Config::new();
    let cli = Cli::parse();

    match &cli.command {
        Action::ConfigAction(config_command) => match &config_command {
            ConfigAction::UseCluster(use_cluster) => {
                cluster_config = cluster_config.with_file(".kofr/config")?;
                use_cluster.run(&mut cluster_config)?;
                std::process::exit(exitcode::OK);
            }
            ConfigAction::CurrentContext => {
                let current_context = cluster_config.current_context()?;
                println!("{}", current_context.name);
                std::process::exit(exitcode::OK);
            }
            ConfigAction::GetClusters => {
                for cluster in &cluster_config.clusters {
                    println!("{}", cluster.name);
                    std::process::exit(exitcode::OK);
                }
            }
        },
        _ => (),
    }

    cluster_config = cluster_config.with_file(".kofr/config")?;
    // TODO: implement retry logic
    let uri = &cluster_config.current_context()?.hosts[0];

    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let client = HTTPClient::from_config(connect::HTTPClientConfig {
        http_agent: (agent),
        connect_uri: (uri.to_owned()),
    });

    match cli.command {
        Action::List(list) => list.run(client)?,
        Action::ConnectorAction(connector_command) => match connector_command {
            ConnectorAction::Create(create) => create.run(client)?,
            ConnectorAction::Describe(describe) => describe.run(client)?,
            ConnectorAction::Edit(edit) => edit.run(client)?,
            ConnectorAction::Status(status) => status.run(client)?,
            ConnectorAction::Config(config) => config.run(client)?,
            ConnectorAction::Pause(pause) => pause.run(client)?,
            ConnectorAction::Resume(resume) => resume.run(client)?,
            ConnectorAction::Restart(restart) => restart.run(client)?,
            ConnectorAction::Delete(delete) => delete.run(client)?,
        },
        _ => (),
    }

    Ok(())
}
