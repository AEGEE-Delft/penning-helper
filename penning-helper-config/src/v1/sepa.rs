use serde::{Serialize, Deserialize};

use crate::{Describe, Type};


#[derive(Debug, Clone, Deserialize, Serialize, Default, Describe)]
pub struct SEPAConfig {
    pub company_name: String,
    pub company_iban: String,
    pub company_bic: String,
    pub company_id: String,
}
