use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fmt;

fn main() -> Result<(), ureq::Error> {
    let uri = env::var("CONNECT_URI").expect("env var CONNECT_URI not found");
    let connectors: serde_json::Value = ureq::get(&uri)
        .set("Accept", "application/json")
        .call()?
        .into_json()?;

    let mut connectors_vec: Vec<Connector> = Vec::new();
    let test = connectors.as_array().unwrap();

    for (i, c) in connectors.as_array().unwrap().into_iter().enumerate() {
        let connector_name: String = serde_json::from_value(*c.get(i).unwrap()).unwrap();
        connectors_vec.push(Connector {
            name: connector_name,
        });
    }

    for c in connectors_vec {
        println!("{}", c);
    }

    Ok(())
}

#[derive(Deserialize, Serialize)]
struct Connector {
    name: String,
}

impl fmt::Display for Connector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
