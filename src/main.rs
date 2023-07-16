use serde::Deserialize;
use std::env;

fn main() -> Result<(), ureq::Error> {
    let uri = env::var("CONNECT_URI").expect("env var CONNECT_URI not found");
    let connectors = ureq::get(&uri)
        .set("Accept", "application/json")
        .call()?
        .into_string()?;

    println!("{}", connectors);

    Ok(())
}

#[derive(Deserialize)]
struct Connector {
    name: String,
}
