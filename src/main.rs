use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;

fn main() -> Result<(), ureq::Error> {
    let uri = env::var("CONNECT_URI").expect("env var CONNECT_URI not found");
    let connectors = ureq::get(&uri)
        .set("Accept", "application/json")
        .call()?
        .into_json::<serde_json::Value>()?;

    let connectors = connectors.as_array().unwrap();

    // println!("{}", connectors[0]);

    let mut connectors_vec: Vec<Connector> = Vec::new();
    for (i, c) in connectors.into_iter().enumerate() {
        let connector_name: String = serde_json::from_value(c.to_owned()).unwrap();
        connectors_vec.push(Connector {
            name: (connector_name),
        });
    }

    for c in &connectors_vec {
        println!("{}", *c);
    }

    Ok(())
}

#[derive(Deserialize, Serialize)]
struct Connector {
    name: String,
}

impl fmt::Display for Connector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type -> {}", self.name)
    }
}
