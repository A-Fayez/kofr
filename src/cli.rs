use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, ensure, Context, Ok, Result};
use clap::{Args, Parser, Subcommand};
use clap_stdin::FileOrStdin;
use tabled::{settings::Style, Table};

use crate::{
    config::ClusterContext,
    connect::{ConnectorConfig, CreateConnector, DescribeConnector, HTTPClient},
};

/// Kafka Connect CLI for connect cluster management
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Action,
    /// config file (default is $HOME/.kofr/config)
    #[arg(long = "config-file")]
    pub config_file: Option<PathBuf>,
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

    /// check cluster status
    #[command(subcommand)]
    Cluster(Cluster),

    /// operate on connector tasks
    #[command(subcommand)]
    #[clap(alias = "tasks")]
    Task(Task),

    /// operate on connector's topics
    #[command(subcommand)]
    #[clap(alias = "topics")]
    Topic(Topic),

    /// operate on connectors plugins
    #[command(subcommand)]
    #[clap(alias = "plugins")]
    Plugin(Plugin),
}

#[derive(Args, Debug)]
pub struct List {}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Sets a current cluster context in the configuration
    #[clap(name = "use-cluster")]
    UseCluster(UseCluster),

    /// Displays the current context
    #[clap(name = "current-context")]
    CurrentContext,

    /// Displays available clusters in the configuration file
    #[clap(name = "get-clusters")]
    GetClusters,

    /// Add a new cluster
    #[clap(name = "add-cluster")]
    AddCluster(AddCluster),

    /// Remove cluster
    #[clap(name = "remove-cluster")]
    RemoveCluster(RemoveCluster),
}

#[derive(Args, Debug)]
pub struct UseCluster {
    pub cluster: String,
}

#[derive(Args, Debug)]
pub struct AddCluster {
    /// cluster name
    pub name: String,

    /// Comma seperated list of valid http kafka connect hosts
    #[arg(long = "hosts")]
    pub hosts: String,
}

#[derive(Args, Debug)]
pub struct RemoveCluster {
    pub name: String,
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
    /// pause the connector and its tasks
    Pause(Pause),
    /// resume a paused connector or do nothing of the connector is not paused
    Resume(Resume),
    /// restar the connector, you may use --include-tasks and/or --only-failed to restart any combination of the Connector and/or Task instances for the connector.
    Restart(Restart),
    /// delete a connector, halting all tasks and deleting its configuration.
    Delete(Delete),
    /// patches a connector configuration with provided
    Patch(Patch),
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

#[derive(Args, Debug)]
pub struct Pause {
    pub name: String,
}

#[derive(Args, Debug)]
pub struct Resume {
    pub name: String,
}

#[derive(Args, Debug)]
pub struct Delete {
    pub name: String,
}

#[derive(Args, Debug)]
pub struct Patch {
    pub name: String,
    #[arg(short = 'd', long = "data")]
    pub data: String,
}

#[derive(Args, Debug)]
pub struct Restart {
    pub name: String,
    #[arg(long = "include-tasks")]
    pub include_tasks: bool,
    #[arg(long = "only-failed")]
    pub only_failed: bool,
}

#[derive(Subcommand, Debug)]
pub enum Cluster {
    Status,
}

#[derive(Subcommand, Debug)]
pub enum Task {
    /// list active tasks of a connector
    #[clap(alias = "ls")]
    List(TaskList),

    /// restart an indicidual connector's task
    Restart(TaskRestart),

    /// get a task’s status and config.
    Status(TaskStatus),
}

#[derive(Args, Debug)]
pub struct TaskList {
    pub connector_name: String,
}

#[derive(Args, Debug)]
pub struct TaskRestart {
    pub connector_name: String,
    pub task_id: usize,
}

#[derive(Args, Debug)]
pub struct TaskStatus {
    pub connector_name: String,
    pub task_id: usize,
}

#[derive(Subcommand, Debug)]
pub enum Topic {
    /// list connector's topics
    #[clap(alias = "ls")]
    List(TopicList),

    /// Resets the set of topic names that the connector has been using since its creation or since the last time its set of active topics was reset.
    Reset(TopicReset),
}

#[derive(Args, Debug)]
pub struct TopicList {
    pub connector_name: String,
}

#[derive(Args, Debug)]
pub struct TopicReset {
    pub connector_name: String,
}

#[derive(Subcommand, Debug)]
pub enum Plugin {
    /// list connector plugins
    #[clap(alias = "ls")]
    List(PluginList),

    /// Validate the provided configuration values against the configuration definition.
    ValidateConfig(ValidateConfig),
}

#[derive(Args, Debug)]
pub struct PluginList {}

#[derive(Args, Debug)]
pub struct ValidateConfig {
    /// The class name of the connector plugin
    #[arg(short = 'n', long = "name")]
    pub class_name: Option<String>,

    /// user provided configuration, read from file or stdin
    #[arg(short = 'f', long = "file")]
    pub config: FileOrStdin,
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

impl Patch {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        let new_config = self.data;
        let new_config = serde_json::from_str(&new_config).context("invalid config format")?;
        connect_client.put_connector(&self.name, new_config)?;
        println!("successfully patched connector: '{}'", &self.name);
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
        std::fs::write(file.path(), old_config).context("failed writing data to tempfile")?;
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

impl Pause {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        connect_client.pause_connector(&self.name)?;
        println!("connector: \"{}\" paused successfully", &self.name);
        Ok(())
    }
}

impl Resume {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        connect_client.resume_connector(&self.name)?;
        println!("connector: \"{}\" resumed successfully", &self.name);
        Ok(())
    }
}

impl Restart {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        connect_client.restart_connector(&self.name, self.include_tasks, self.only_failed)?;
        println!("connector: \"{}\" restarted sucessfully", &self.name);
        Ok(())
    }
}

impl Delete {
    pub fn run(self, connect_client: HTTPClient) -> Result<()> {
        connect_client.delete_connector(&self.name)?;
        println!("connector: \"{}\" deleted", &self.name);
        Ok(())
    }
}

impl UseCluster {
    pub fn run(&self, current_config: &mut crate::config::Config) -> Result<()> {
        let clusters: Vec<&String> = current_config.clusters.iter().map(|c| &c.name).collect();

        ensure!(
            clusters.contains(&&self.cluster),
            format!("Cluster with name \"{}\" could not be found", &self.cluster)
        );

        current_config.current_cluster = Some(self.cluster.clone());

        let updated_config_yaml =
            serde_yaml::to_string(&current_config).context("invalid config yaml format")?;
        std::fs::write(&current_config.file_path, updated_config_yaml)
            .context("failed writing file to filesystem")?;
        println!("Switched to cluster \"{}\"", self.cluster);
        Ok(())
    }
}

impl AddCluster {
    pub fn run(&self, current_config: &mut crate::config::Config) -> Result<()> {
        let cluster_name = &self.name;
        let hosts: Vec<String> = self
            .hosts
            .split(',')
            .collect::<Vec<&str>>()
            .iter()
            .map(|&s| s.to_string())
            .collect();

        if current_config
            .clusters
            .iter()
            .any(|c| c.name == *cluster_name)
        {
            return Err(anyhow!("Cluster \"{}\" already exists.", cluster_name));
        }

        current_config.clusters.push(ClusterContext {
            name: cluster_name.to_string(),
            hosts,
        });
        current_config.current_cluster = Some(cluster_name.to_string());

        let updated_config_yaml =
            serde_yaml::to_string(&current_config).context("invalid config yaml format")?;
        std::fs::write(&current_config.file_path, updated_config_yaml)
            .context("failed writing file to filesystem")?;
        println!("Added cluster \"{}\"", cluster_name);
        Ok(())
    }
}

impl RemoveCluster {
    pub fn run(&self, current_config: &mut crate::config::Config) -> Result<()> {
        let index = current_config
            .clusters
            .iter()
            .position(|c| c.name == self.name)
            .with_context(|| {
                format!(
                    "Could not delete cluster: cluster with name '{}' does not exists",
                    self.name
                )
            })?;
        current_config.clusters.remove(index);
        let updated_config_yaml =
            serde_yaml::to_string(&current_config).context("invalid config yaml format")?;
        std::fs::write(&current_config.file_path, updated_config_yaml)
            .context("failed writing file to filesystem")?;
        println!("Removed cluster \"{}\"", self.name);
        Ok(())
    }
}

impl Cluster {
    pub fn run(&self, current_config: &crate::config::Config) -> Result<()> {
        use crate::cluster::*;

        let mut hosts_status = Vec::<UriStatus>::new();
        for host in &current_config.current_context()?.hosts {
            hosts_status.push(get_uri_status(host));
        }
        let mut _id = "";
        let cluster_id = hosts_status.iter().find(|&h| !h.id.is_empty());
        if let Some(cluster_id) = cluster_id {
            _id = &cluster_id.id;
        }
        println!(
            r#" Current Cluster: {}
 id : {}
 ..........................................."#,
            current_config.current_cluster.as_ref().unwrap(),
            _id
        );
        let status_table = Table::new(hosts_status).with(Style::blank()).to_string();
        println!("{}", status_table);
        Ok(())
    }
}

impl TaskList {
    pub fn run(self, connect_host: &str) -> Result<()> {
        let tasks_status: Result<Vec<crate::tasks::TaskStatus>> =
            crate::tasks::list_tasks(connect_host, &self.connector_name)?
                .iter()
                .map(|t| crate::tasks::task_status(connect_host, &self.connector_name, t.id.task))
                .collect();

        let tasks_table = Table::new(tasks_status?).with(Style::blank()).to_string();
        println!("Active tasks of connector: '{}'", &self.connector_name);
        println!("{}", tasks_table);
        Ok(())
    }
}

impl TaskRestart {
    pub fn run(self, connect_host: &str) -> Result<()> {
        crate::tasks::restart_task(connect_host, &self.connector_name, self.task_id)?;
        println!(
            "restarted task: '{}/{}'",
            &self.connector_name, self.task_id
        );
        Ok(())
    }
}

impl TaskStatus {
    pub fn run(self, connect_host: &str) -> Result<()> {
        let binding = crate::tasks::list_tasks(connect_host, &self.connector_name)?;
        let task_response = binding
            .iter()
            .find(|t| t.id.task == self.task_id && t.id.connector == self.connector_name)
            .ok_or(anyhow!(
                "No status found for task {}-{}",
                &self.connector_name,
                &self.task_id
            ))?;

        let task_status =
            crate::tasks::task_status(connect_host, &self.connector_name, self.task_id)?;

        let task_status = serde_json::json!({
            "status": task_status,
            "config": task_response.config,
        });
        let task_status = serde_json::to_string_pretty(&task_status)?;
        println!("{}", task_status);
        Ok(())
    }
}

impl TopicList {
    pub fn run(self, connect_host: &str) -> Result<()> {
        let topics = crate::topics::list_topics(connect_host, &self.connector_name)?;
        let topics = serde_json::to_string_pretty(&topics)?;
        println!("{}", topics);
        Ok(())
    }
}

impl TopicReset {
    pub fn run(self, connect_host: &str) -> Result<()> {
        crate::topics::reset(connect_host, &self.connector_name)?;
        println!(
            "resetted topics successfully of connector: '{}'",
            self.connector_name
        );
        Ok(())
    }
}

impl PluginList {
    pub fn run(self, connect_host: &str) -> Result<()> {
        let plugins = crate::connector_plugins::list_plugins(connect_host)?;
        let plugins = serde_json::to_string_pretty(&plugins)?;
        println!("{}", plugins);
        Ok(())
    }
}

impl ValidateConfig {
    pub fn run(self, connect_host: &str) -> Result<()> {
        let config = self.config;
        let config: HashMap<String, String> = serde_json::from_str(&config)?;

        let class_name: String = match self.class_name {
            Some(name) => name,
            None => config
                .get("connector.class")
                .ok_or(anyhow!("no connector.class was provided"))?
                .to_string(),
        };

        let response =
            crate::connector_plugins::validate_config(connect_host, &class_name, config)?;
        let response = serde_json::to_string_pretty(&response)?;
        println!("{}", response);
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
