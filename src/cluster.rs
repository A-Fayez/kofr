#[derive(tabled::Tabled, Debug)]
pub struct UriStatus {
    #[tabled(rename = "HOST")]
    pub uri: String,
    #[tabled(rename = "STATE")]
    pub state: UriState,
    #[tabled(skip)]
    pub id: String,
}

#[derive(Debug)]
pub enum UriState {
    Online,
    Offline,
}

impl std::fmt::Display for UriState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Online => write!(f, "Online"),
            Self::Offline => write!(f, "Offline"),
        }
    }
}

pub fn get_uri_status(host: &str) -> UriStatus {
    match ureq::get(host).set("Accept", "application/json").call() {
        Ok(response) => {
            let response: serde_json::Value = response.into_json().unwrap();
            let id = response.get("kafka_cluster_id").unwrap().to_string();
            UriStatus {
                uri: host.to_string(),
                state: UriState::Online,
                id,
            }
        }
        Err(_) => UriStatus {
            uri: host.to_string(),
            state: UriState::Offline,
            id: "".to_string(),
        },
    }
}
