use std::io::{BufWriter, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

mod version;
pub use version::Version;

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

pub const CURRENT_VERSION: Version<1> = Version;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Config {
    V1 {
        version: Version<1>,
        mail: v1::mail::MailConfig,
        sepa: v1::sepa::SEPAConfig,
        conscribo: v1::conscribo::ConscriboConfig,
        #[serde(default = "v1::default_year_format")]
        year_format: String,
    },
}

impl Config {
    pub fn mail(&self) -> &v1::mail::MailConfig {
        match self {
            Self::V1 { mail, .. } => mail,
        }
    }

    pub fn mail_mut(&mut self) -> &mut v1::mail::MailConfig {
        match self {
            Self::V1 { mail, .. } => mail,
        }
    }

    pub fn sepa(&self) -> &v1::sepa::SEPAConfig {
        match self {
            Self::V1 { sepa, .. } => sepa,
        }
    }

    pub fn sepa_mut(&mut self) -> &mut v1::sepa::SEPAConfig {
        match self {
            Self::V1 { sepa, .. } => sepa,
        }
    }

    pub fn conscribo(&self) -> &v1::conscribo::ConscriboConfig {
        match self {
            Self::V1 { conscribo, .. } => conscribo,
        }
    }

    pub fn conscribo_mut(&mut self) -> &mut v1::conscribo::ConscriboConfig {
        match self {
            Self::V1 { conscribo, .. } => conscribo,
        }
    }

    pub fn year_format(&self) -> &str {
        match self {
            Self::V1 { year_format, .. } => year_format,
        }
    }

    pub fn year_format_mut(&mut self) -> &mut String {
        match self {
            Self::V1 { year_format, .. } => year_format,
        }
    }

    pub fn needs_upgrade(&self) -> bool {
        match self {
            Self::V1 { version, .. } => version < &CURRENT_VERSION,
        }
    }

    pub fn upgrade_to_latest(self) -> Self {
        match self {
            Self::V1 {
                mail,
                sepa,
                conscribo,
                year_format,
                ..
            } => Self::V1 {
                mail,
                sepa,
                conscribo,
                year_format,
                version: CURRENT_VERSION,
            },
        }
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
        Self::V1 {
            version: CURRENT_VERSION,
            mail: v1::mail::MailConfig::default(),
            sepa: v1::sepa::SEPAConfig::default(),
            conscribo: v1::conscribo::ConscriboConfig::default(),
            year_format: v1::default_year_format(),
        }
    }
}
