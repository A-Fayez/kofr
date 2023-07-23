use serde::{Deserialize, Serialize};
use serde_json::Value;
use ureq::Agent;

pub fn get_connectors(agent: &Agent, uri: &str) -> Result<Vec<Connector>, ureq::Error> {
    let connectors: Value = agent
        .get(uri)
        .set("Accept", "application/json")
        .call()?
        .into_json()?;

    let connectors = connectors.as_array().unwrap();
    let mut connectors_vec: Vec<Connector> = Vec::new();
    for (i, c) in connectors.into_iter().enumerate() {
        let connector_name: String = serde_json::from_value(c.to_owned()).unwrap();
        connectors_vec.push(Connector {
            name: (connector_name),
        });
    }

    Ok(connectors_vec)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Connector {
    name: String,
}
