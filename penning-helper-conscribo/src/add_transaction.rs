use chrono::{Local, NaiveDate};
use penning_helper_types::Euro;
use serde::{Deserialize, Serialize};

use crate::{transactions::Side, ApiCall};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTransaction {
    pub date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_nr: Option<String>,
    pub transaction_rows: Vec<TransactionRow>,
    #[serde(skip)]
    pub reference: Option<String>,
    #[serde(skip)]
    pub description: Option<String>,
    #[serde(skip)]
    pub relation_nr: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRow {
    account_nr: String,
    amount: f64,
    side: Side,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relation_nr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vat_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vat_amount: Option<Euro>,
}

impl AddTransaction {
    pub fn new() -> Self {
        Self {
            date: Local::now().date_naive(),
            transaction_nr: None,
            transaction_rows: Vec::new(),
            reference: None,
            description: None,
            relation_nr: None,
        }
    }

    pub fn with_date(mut self, date: NaiveDate) -> Self {
        self.date = date;
        self
    }

    pub fn with_transaction_nr(mut self, transaction_nr: String) -> Self {
        self.transaction_nr = Some(transaction_nr);
        self
    }

    pub fn with_reference(mut self, reference: String) -> Self {
        self.reference = Some(reference);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_relation_nr(mut self, relation_nr: String) -> Self {
        self.relation_nr = Some(relation_nr);
        self
    }

    pub fn with_row(mut self, account_nr: String, amount: Euro, side: Side) -> Self {
        self.transaction_rows.push(TransactionRow {
            account_nr,
            amount: amount.into(),
            side,
            reference: self.reference.clone(),
            description: self.description.clone(),
            relation_nr: self.relation_nr.clone(),
            vat_code: None,
            vat_amount: None,
        });
        self
    }

    pub fn add_debet(
        self,
        rekening: String,
        amount: Euro,
    ) -> Self {
        self.with_row(
            rekening,
            amount,
            Side::Credit,
        )
        .with_row(
            "1001".to_string(),
            amount,
            Side::Debet,
        )
    }

    pub fn add_credit(
        self,
        rekening: String,
        amount: Euro,
    ) -> Self {
        self.with_row(
            rekening,
            amount,
            Side::Debet,
        )
        .with_row(
            "1002".to_string(),
            amount,
            Side::Credit,
        )
    }

    pub fn add_merch(
        self,
        merch_rekening: String,
        merch_verkoop_rekening: String,
        total_amount: Euro,
        merch_price: Euro,
    ) -> Self {
        self.with_row(
            merch_rekening,
            merch_price,
            Side::Credit,
        )
        .with_row(
            merch_verkoop_rekening,
            total_amount - merch_price,
            Side::Credit,
        )
        .with_row(
            "1001".to_string(),
            total_amount,
            Side::Debet,
        )
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AddTransactionResult {
    pub transaction_id: i64,
    pub transaction_nr: String,
}

impl ApiCall for AddTransaction {
    type Response = AddTransactionResult;

    const PATH: &'static str = "financial/transactions";

    const METHOD: reqwest::Method = reqwest::Method::POST;
}
