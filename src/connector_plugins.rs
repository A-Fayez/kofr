use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

pub fn list_plugins(host: &str) -> Result<serde_json::Value> {
    let endpoint = plugins_endpoint(host);
    match ureq::get(&endpoint)
        .set("Accept", "application/json")
        .call()
    {
        Ok(response) => response
            .into_json()
            .context("invalid json returned from api"),
        Err(ureq::Error::Status(_, r)) => Err(anyhow!("{}", r.into_string()?)),
        Err(err) => Err(anyhow!("{}", err)),
    }
}

pub fn validate_config(
    host: &str,
    name: &str,
    config: HashMap<String, String>,
) -> Result<serde_json::Value> {
    let endpoint = plugins_endpoint(host);
    let endpoint = format!("{}/{}/config/validate", endpoint, name);
    // let config = serde_json::to_value(config)?;
    dbg!(&endpoint);
    match ureq::put(&endpoint)
        .set("Accept", "application/json")
        .set("Content-Type", "application/json")
        .send_json(config)
    {
        Ok(response) => response
            .into_json()
            .context("invalid json returned from api"),
        Err(ureq::Error::Status(_, r)) => Err(anyhow!("{}", r.into_string()?)),
        Err(err) => Err(anyhow!("{}", err)),
    }
}

fn plugins_endpoint(uri: &str) -> String {
    if uri.ends_with('/') {
        return format!("{}connector-plugins", uri);
    }
    format!("{}/connector-plugins", uri)
}
