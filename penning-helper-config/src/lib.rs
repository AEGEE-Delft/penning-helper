use std::io::{BufWriter, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use penning_helper_macros::Describe;
pub use v1::conscribo::ConscriboConfig;
pub use v1::mail::{Credentials, MailAddress, MailConfig};
pub use v1::sepa::SEPAConfig;

mod v1 {
    /// Email Config
    pub mod mail;

    /// SEPA Config
    pub mod sepa;

    /// Conscribo Config
    pub mod conscribo;

    pub fn default_year_format() -> String {
        "2324".to_string()
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    String,
    Email,
    Integer,
    Password,
    Struct(Vec<(&'static str, Type)>),
}

pub enum TypeStuff {
    Header(&'static str),
    Field(&'static str, Type),
}

impl Type {
    fn is_struct(&self) -> bool {
        matches!(self, Type::Struct(_))
    }

    pub fn to_type_stuff(&self) -> Vec<TypeStuff> {
        let mut v = vec![];
        match self {
            Type::Struct(s) => {
                for (k, t) in s {
                    if t.is_struct() {
                        v.push(TypeStuff::Header(k));
                        v.extend(t.to_type_stuff());
                    } else {
                        v.push(TypeStuff::Field(k, t.clone()));
                    }
                }
            }
            _ => {}
        }
        v
    }
}

pub trait Describe {
    fn describe_fields() -> Vec<(&'static str, Type)> {
        vec![]
    }

    fn describe_self() -> Type {
        Type::Struct(Self::describe_fields())
    }
}

impl Describe for String {
    fn describe_self() -> Type {
        Type::String
    }
}

impl Describe for u16 {
    fn describe_self() -> Type {
        Type::Integer
    }
}

pub const CURRENT_VERSION: usize = 1;

#[derive(Debug, Clone, Deserialize, Serialize, Describe)]
pub struct Config {
    #[serde(default = "v1::default_year_format")]
    year_format: String,
    mail: v1::mail::MailConfig,
    sepa: v1::sepa::SEPAConfig,
    conscribo: v1::conscribo::ConscriboConfig,
    #[describe(skip)]
    version: usize,
}

impl Config {
    pub fn mail(&self) -> &v1::mail::MailConfig {
        &self.mail
    }

    pub fn mail_mut(&mut self) -> &mut v1::mail::MailConfig {
        &mut self.mail
    }

    pub fn sepa(&self) -> &v1::sepa::SEPAConfig {
        &self.sepa
    }

    pub fn sepa_mut(&mut self) -> &mut v1::sepa::SEPAConfig {
        &mut self.sepa
    }

    pub fn conscribo(&self) -> &v1::conscribo::ConscriboConfig {
        &self.conscribo
    }

    pub fn conscribo_mut(&mut self) -> &mut v1::conscribo::ConscriboConfig {
        &mut self.conscribo
    }

    pub fn year_format(&self) -> &str {
        &self.year_format
    }

    pub fn year_format_mut(&mut self) -> &mut String {
        &mut self.year_format
    }

    pub fn needs_upgrade(&self) -> bool {
        self.version < CURRENT_VERSION
    }

    pub fn upgrade_to_latest(self) -> Self {
        self
    }

    pub fn from_toml(toml: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    pub fn load_from_file() -> Self {
        let config_file = config_location();
        if !config_file.exists() {
            Default::default()
        } else {
            let config = std::fs::read_to_string(config_file).expect("Could not read config file");
            Self::from_toml(&config).expect("Could not parse config file")
        }
    }

    pub fn save_to_file(&self) {
        let config_file = config_location();
        let config_file = std::fs::File::create(config_file).unwrap();
        let toml = self.to_toml().expect("Could not serialize config");
        let mut buf = BufWriter::new(config_file);
        buf.write_all(toml.as_bytes())
            .expect("Could not write config file");
        buf.flush().expect("Could not flush config file");
    }

    /// get a list of all things potentially wrong with the config
    pub fn config_errors(&self) -> Vec<&str> {
        let mut errors = Vec::new();
        if self.mail().smtp_server.is_empty() {
            errors.push("SMTP server is empty");
        }
        if self.mail().smtp_port == 0 {
            errors.push("SMTP port is 0");
        }
        if self.mail().credentials.username.is_empty() {
            errors.push("SMTP username is empty");
        }
        if self.mail().credentials.password.is_empty() {
            errors.push("SMTP password is empty");
        }
        if self.mail().from.name.is_empty() || self.mail().from.address.is_empty() {
            errors.push("SMTP from is empty");
        }
        if self.mail().reply_to.name.is_empty() || self.mail().reply_to.address.is_empty() {
            errors.push("SMTP reply-to is empty");
        }

        if self.sepa().company_id.is_empty() {
            errors.push("SEPA company id is empty");
        }
        if self.sepa().company_name.is_empty() {
            errors.push("SEPA company name is empty");
        }
        if self.sepa().company_iban.is_empty() {
            errors.push("SEPA company IBAN is empty");
        }
        if self.sepa().company_bic.is_empty() {
            errors.push("SEPA company BIC is empty");
        }

        if self.conscribo().username.is_empty() {
            errors.push("Conscribo username is empty");
        }
        if self.conscribo().password.is_empty() {
            errors.push("Conscribo password is empty");
        }
        if self.conscribo().account_name.is_empty() {
            errors.push("Conscribo URL is empty");
        }

        errors
    }
}

fn config_location() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        let dir = config_dir.join("penning-helper");
        std::fs::create_dir_all(&dir).expect("Could not create config directory");
        dir.join("config.toml")
    } else {
        PathBuf::from("penning-helper.toml")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            mail: v1::mail::MailConfig::default(),
            sepa: v1::sepa::SEPAConfig::default(),
            conscribo: v1::conscribo::ConscriboConfig::default(),
            year_format: v1::default_year_format(),
        }
    }
}
