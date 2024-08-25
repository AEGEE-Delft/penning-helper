use serde::{Deserialize, Serialize};
use crate::{Describe, Type};

#[derive(Debug, Clone, Deserialize, Serialize, Default, Describe)]
pub struct MailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub credentials: Credentials,
    pub from: MailAddress,
    pub reply_to: MailAddress,
    #[serde(default = "board_default")]
    pub board_line: String,
    #[serde(default)]
    pub name: String,
}

fn board_default() -> String {
    "XLIth Board of AEGEE-Delft 'Wervelwind'".to_string()
}



#[derive(Debug, Clone, Deserialize, Serialize, Default, Describe)]
pub struct Credentials {
    #[describe(email)]
    pub username: String,
    #[describe(password)]
    pub password: String,
}

impl Credentials {
    pub fn as_pair(&self) -> (&str, &str) {
        (&self.username, &self.password)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, Describe)]
pub struct MailAddress {
    pub name: String,
    #[describe(email)]
    pub address: String,
}

impl MailAddress {
    pub fn as_pair(&self) -> (&str, &str) {
        (&self.name, &self.address)
    }
}
