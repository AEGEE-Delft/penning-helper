use std::collections::HashMap;

use chrono::NaiveDate;
use penning_helper_types::Euro;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::ApiCall;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transactions {
    pub filters: Filters,
    pub limit: i64,
    pub offset: i64,
}

impl Transactions {
    pub fn new(limit: i64, offset: i64) -> Self {
        let limit = limit.min(100);
        Self {
            filters: Filters::default(),
            limit,
            offset,
        }
    }

    pub fn transaction_ids(mut self, transaction_ids: Vec<String>) -> Self {
        self.filters.transaction_ids = transaction_ids;
        self
    }

    pub fn date_start(mut self, date_start: NaiveDate) -> Self {
        self.filters.date_start = Some(date_start);
        self
    }

    pub fn date_end(mut self, date_end: NaiveDate) -> Self {
        self.filters.date_end = Some(date_end);
        self
    }

    pub fn references(mut self, references: Vec<String>) -> Self {
        self.filters.references = references;
        self
    }

    pub fn relations(mut self, relations: Vec<String>) -> Self {
        self.filters.relations = relations;
        self
    }

    pub fn accounts(mut self, accounts: Vec<String>) -> Self {
        self.filters.accounts = accounts;
        self
    }

    /// Works opposite of api documentation, it it's false it will show all, 
    /// and if it's true it will show only non-settled transactions.
    pub fn settled(mut self, settled: bool) -> Self {
        self.filters.settled = Some((!settled) as i64);
        self
    }
}

impl ApiCall for Transactions {
    type Response = TransactionsResponse;

    const PATH: &'static str = "financial/transactions/filters";

    const METHOD: reqwest::Method = reqwest::Method::POST;
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_start: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_end: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled: Option<i64>,
}

#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionsResponse {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub nr_transactions: i64,
    pub transactions: HashMap<String, Transaction>,
}

impl TransactionsResponse {
    pub fn from_json(s: &str) -> TransactionsResponse {
        serde_json::from_str(s).unwrap()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub transaction_id: i64,
    pub date: String,
    pub description: String,
    pub transaction_nr: String,
    pub transaction_rows: HashMap<String, TransactionRow>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRow {
    pub account_nr: String,
    pub amount: Euro,
    pub side: String,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub relation_nr: Option<String>,
    #[serde(default)]
    pub vat_code: Option<String>,
    #[serde(default)]
    pub vat_amount: Option<Euro>,
}
