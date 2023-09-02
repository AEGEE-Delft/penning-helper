use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SEPAConfig {
    pub company_name: String,
    pub company_iban: String,
    pub company_bic: String,
    pub company_id: String,
}
