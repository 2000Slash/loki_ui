use serde::{Deserialize, Serialize};

/// Contains all the code for interacting with loki
pub mod loki;
/// Contains all the code for the user interface
pub mod ui;

/// The configuration for the Loki client
#[derive(Serialize, Deserialize)]
pub struct LokiConfig {
    pub loki_url: String,
}

impl Default for LokiConfig {
    fn default() -> Self {
        Self {
            loki_url: String::from("http://localhost:3100"),
        }
    }
}
