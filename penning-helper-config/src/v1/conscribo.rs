use crate::{Describe, Type};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default, Describe)]
pub struct ConscriboConfig {
    pub username: String,
    #[describe(password)]
    pub password: String,
    #[serde(alias="url")]
    pub account_name: String,
    #[serde(default)]
    pub merch_winst_rekening: String,
}