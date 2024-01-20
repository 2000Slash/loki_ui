use serde::{Serialize, Deserialize};


/// The response from GET /loki/api/v1/labels
#[derive(Serialize, Deserialize, Debug)]
pub struct LokiLabels {
    pub status: String,
    pub data: Vec<String>
}