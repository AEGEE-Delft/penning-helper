use serde::{Deserialize, Serialize};
use crate::{Describe, Type};

#[derive(Debug, Clone, Deserialize, Serialize, Default, Describe)]
pub struct MailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub credentials: Credentials,
    pub from: MailAddress,
    pub reply_to: MailAddress,
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
