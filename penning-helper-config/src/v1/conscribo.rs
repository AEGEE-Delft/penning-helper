use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ConscriboConfig {
    pub username: String,
    pub password: String,
    pub url: String,
}