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
        "2223".to_string()
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

    pub fn sepa(&self) -> &v1::sepa::SEPAConfig {
        match self {
            Self::V1 { sepa, .. } => sepa,
        }
    }

    pub fn conscribo(&self) -> &v1::conscribo::ConscriboConfig {
        match self {
            Self::V1 { conscribo, .. } => conscribo,
        }
    }

    pub fn year_format(&self) -> &str {
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
