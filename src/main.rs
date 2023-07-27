use std::env;
use std::time::Duration;
use ureq::Agent;

mod error;
use error::{KofrError, Result};

use kofr::get_all_connectors;

fn main() -> Result<()> {
    let uri = env::var("CONNECT_URI").expect("env var CONNECT_URI not found");

    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let connectors_vec =
        get_all_connectors(&agent, &uri).map_err(|_| KofrError::DeserializeConnectorError);

    for c in &connectors_vec {
        dbg!(c);
    }

    Ok(())
}
