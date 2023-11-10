mod cli;
mod cluster;
mod config;
mod connect;
mod tasks;

use std::{path::PathBuf, time::Duration};

use anyhow::{Context, Result};
use clap::Parser;
use home::home_dir;
use ureq::Agent;

use cli::*;
use connect::HTTPClient;

fn main() -> Result<()> {
    let mut cluster_config = config::Config::new();
    let cli = Cli::parse();
    let mut default_config_path = home_dir().context("could not get user's home dir")?;
    default_config_path.push(".kofr/config");

    let binding = cli.config_file.unwrap_or(default_config_path);
    let user_config_file = binding.to_string_lossy();

    let user_config_file = shellexpand::tilde(&user_config_file);

    cluster_config = cluster_config.with_file(PathBuf::from(user_config_file.into_owned()))?;
    match &cli.command {
        Action::ConfigAction(config_command) => match &config_command {
            ConfigAction::UseCluster(use_cluster) => {
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
                }
                std::process::exit(exitcode::OK);
            }
            ConfigAction::AddCluster(add_cluster) => {
                add_cluster.run(&mut cluster_config)?;
                std::process::exit(exitcode::OK);
            }
            ConfigAction::RemoveCluster(remove) => {
                remove.run(&mut cluster_config)?;
                std::process::exit(exitcode::OK);
            }
        },
        Action::Cluster(status) => {
            status.run(&cluster_config)?;
            std::process::exit(exitcode::OK);
        }
        _ => (),
    }

    let uri = &cluster_config.current_context()?.available_host()?;

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
        Action::Task(task) => match task {
            Task::List(list) => list.run(&uri)?,
            Task::Restart(restart) => unimplemented!(),
            Task::Status(status) => unimplemented!(),
        },
        _ => (),
    }

    Ok(())
}
