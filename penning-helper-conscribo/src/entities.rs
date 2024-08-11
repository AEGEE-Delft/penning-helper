use std::collections::HashMap;

use chrono::NaiveDate;
use filters::Filter;
use penning_helper_types::Euro;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;

use crate::ApiCall;

pub mod filters;

#[derive(Debug, Serialize)]
pub struct Entities {
    filters: Vec<Filter>,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }
}

impl ApiCall for Entities {
    type Response = EntityResponse;

    const PATH: &'static str = "relations/entities/filters";

    const METHOD: reqwest::Method = reqwest::Method::POST;
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityResponse {
    pub result_count: String,
    pub entities: HashMap<String, Entity>,
}

#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub code_sort: String,
    pub id: String,
    pub entity_type: String,
    pub naam: Value,
    pub code: String,
    pub lidmaatschap_gestart: Option<NaiveDate>,
    #[serde(rename = "lidmaatschap_be__indigd")]
    pub lidmaatschap_be_indigd: Option<NaiveDate>,
    pub gesproken_taal: String,
    pub ere_lid: i64,
    pub alumni_lidmaatschap_gestart: Option<NaiveDate>,
    #[serde(rename = "alumni_lidmaatschap_be__indigd")]
    pub alumni_lidmaatschap_be_indigd: Option<NaiveDate>,
    #[serde_as(as = "serde_with::DefaultOnError")]
    pub alumni_contributie: Euro,
    pub geen_invoice: i64,
    pub leeftijd: String,
    pub voornaam: Option<String>,
    pub achternaam: Option<String>,
    pub display_name: String,
    pub email: String,
    pub account: Option<Account>,
    pub postal_address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub nr: String,
    pub country: String,
    pub name: String,
    pub city: String,
    pub iban: String,
    pub bic: String,
}
