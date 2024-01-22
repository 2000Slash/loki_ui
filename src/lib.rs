use serde::{Deserialize, Serialize};

pub mod loki;
pub mod ui;


#[derive(Serialize, Deserialize)]
pub struct LokiConfig {
    url: String
}

impl Default for LokiConfig {
    fn default() -> Self {
        Self { url: String::from("http://localhost:3100") }
    }
}