use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JobView {
    pub id: String,
    pub url: String,
    pub http_method: String,
    pub repetition_seconds: Option<u32>,
    pub state: String
}
