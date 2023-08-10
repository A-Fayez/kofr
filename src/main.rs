use std::env;
use std::time::Duration;
use ureq::Agent;

mod error;
use anyhow::Result;

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

    let connectors_vec = client.list_connectors()?;

    for c in &connectors_vec {
        dbg!(c);
    }

    Ok(())
}
