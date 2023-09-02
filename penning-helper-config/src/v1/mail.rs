use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub credentials: Credentials,
    pub from: MailAddress,
    pub reply_to: MailAddress,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn as_pair(&self) -> (&str, &str) {
        (&self.username, &self.password)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MailAddress {
    pub name: String,
    pub address: String,
}

impl MailAddress {
    pub fn as_pair(&self) -> (&str, &str) {
        (&self.name, &self.address)
    }
}
