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
    #[serde(default = "default_last_closed_year")]
    pub last_closed_year: u16,
}

fn default_last_closed_year() -> u16 {
    2022
}