use std::collections::HashMap;

use chrono::NaiveDate;
use penning_helper_types::{Date, Euro};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use thiserror::Error;

use crate::ApiCall;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transactions<'l> {
    pub filters: Filters<'l>,
    pub limit: i64,
    pub offset: i64,
}

impl<'l> Transactions<'l> {
    pub fn new(limit: i64, offset: i64) -> Self {
        let limit = limit.min(100);
        Self {
            filters: Filters::default(),
            limit,
            offset,
        }
    }

    pub fn transaction_ids(mut self, transaction_ids: Vec<&'l str>) -> Self {
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

    pub fn references(mut self, references: Vec<&'l str>) -> Self {
        self.filters.references = references;
        self
    }

    pub fn relations(mut self, relations: Vec<&'l str>) -> Self {
        self.filters.relations = relations;
        self
    }

    pub fn accounts(mut self, accounts: Vec<&'l str>) -> Self {
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

impl ApiCall for Transactions<'_> {
    type Response = TransactionsResponse;

    const PATH: &'static str = "financial/transactions/filters";

    const METHOD: reqwest::Method = reqwest::Method::POST;
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Filters<'l> {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_ids: Vec<&'l str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_start: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_end: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<&'l str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<&'l str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<&'l str>,
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
    pub date: Date,
    pub description: String,
    pub transaction_nr: String,
    pub transaction_rows: HashMap<String, TransactionRow>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRow {
    pub account_nr: String,
    pub amount: Euro,
    pub side: Side,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub relation_nr: Option<String>,
    #[serde(default)]
    pub vat_code: Option<String>,
    #[serde(default)]
    pub vat_amount: Option<Euro>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    #[default]
    Debet,
    Credit,
}

impl Transaction {
    pub fn unify(self) -> Result<Vec<UnifiedTransaction>, TransactionConvertError> {
        self.try_into()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UnifiedTransaction {
    pub unique_id: String,
    pub date: Date,
    pub code: String,
    pub description: String,
    pub reference: String,
    pub cost: Euro,
}

impl UnifiedTransaction {
    pub fn create_new_mock(date: Date, description: String, cost: Euro) -> Self {
        Self {
            unique_id: "123312".to_string(),
            date,
            code: "123321".to_string(),
            description,
            reference: "123321".to_string(),
            cost,
        }
    }
}

impl TryFrom<Transaction> for Vec<UnifiedTransaction> {
    type Error = TransactionConvertError;

    fn try_from(value: Transaction) -> Result<Self, Self::Error> {
        let date = value.date;

        let mut rows = HashMap::new();

        for row in value.transaction_rows.values() {
            println!("{:?}", row);
            if row.account_nr != "1001" && row.account_nr != "1002" {
                continue;
            }
            if let Some(r) = &row.relation_nr {
                let urow = rows.entry(r).or_insert_with(|| UnifiedTransaction {
                    unique_id: format!(
                        "{}-{}-{}-{}-{}",
                        row.reference.as_ref().map(String::as_str).unwrap_or("????"),
                        r,
                        row.amount,
                        row.description
                            .as_ref()
                            .map(String::as_str)
                            .unwrap_or("????"),
                        row.account_nr,
                    ),
                    date,
                    code: r.clone(),
                    description: row
                        .description
                        .clone()
                        .unwrap_or_else(|| "????".to_string()),
                    reference: row.reference.clone().unwrap_or_else(|| "????".to_string()),
                    cost: Default::default(),
                });
                match row.side {
                    Side::Debet => urow.cost += row.amount,
                    Side::Credit => urow.cost -= row.amount,
                }
            }
        }
        Ok(rows.into_iter().map(|(_, v)| v).collect())
    }
}

#[derive(Debug, Error)]
pub enum TransactionConvertError {
    #[error("Multiple Relations found in transaction: {0:?}")]
    MultipleRelations(Vec<u32>),
}
