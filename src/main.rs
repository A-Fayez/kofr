use std::env;
use std::time::Duration;
use ureq::Agent;

mod error;
use error::Result;

use kofr::get_all_connectors;

fn main() -> Result<()> {
    let uri = env::var("CONNECT_URI").expect("env var CONNECT_URI not found");

    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let connectors_vec =
        get_all_connectors(&agent, &uri).unwrap();

    for c in &connectors_vec {
        dbg!(c);
    }

    Ok(())
}
