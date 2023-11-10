use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use anyhow::{anyhow, Context, Result};

pub fn list_topics(host: &str, connector_name: &str) -> Result<Topic> {
    let endpoint = crate::tasks::valid_uri(host);
    let endpoint = format!("{}/{}/topics", &endpoint, connector_name);
    match ureq::get(&endpoint)
        .set("Accept", "application/json")
        .call()
    {
        Ok(response) => response
            .into_json::<Topic>()
            .context("invalid json returned from api"),
        Err(ureq::Error::Status(_, response)) => Err(anyhow!("{}", response.into_string()?)),
        Err(err) => Err(anyhow!("{}", err)),
    }
}

pub fn reset(host: &str, connector_name: &str) -> Result<()> {
    let endpoint = crate::tasks::valid_uri(host);
    let endpoint = format!("{}/{}/topics/reset", &endpoint, connector_name);
    match ureq::put(&endpoint)
        .set("Accept", "application/json")
        .call()
    {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(_, response)) => Err(anyhow!("{}", response.into_string()?)),
        Err(err) => Err(anyhow!("{}", err)),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topic {
    #[serde(flatten)]
    pub topics: HashMap<String, TopicsList>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicsList {
    pub topics: Vec<String>,
}
