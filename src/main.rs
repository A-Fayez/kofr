use kofr::get_connectors;
use std::env;
use std::time::Duration;
use ureq::Agent;

fn main() -> Result<(), ureq::Error> {
    let uri = env::var("CONNECT_URI").expect("env var CONNECT_URI not found");

    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let connectors_vec = get_connectors(&agent, &uri);

    for c in &connectors_vec {
        dbg!(c);
    }

    Ok(())
}
