// {
//     "accountNr": "accountNr",
//     "accountName": "accountName",
//     "type": "balance",
//     "usage": "generic",
//     "usedForCredit": false,
//     "usedForDebit": false,
//     "parent": "parent"
//     }

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::ApiCall;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRequest {
    date: NaiveDate,
}

impl AccountRequest {
    pub fn new(date: NaiveDate) -> Self {
        Self { date }
    }

    pub fn today() -> Self {
        Self::new(Local::now().date_naive())
    }
}

impl ApiCall for AccountRequest {
    type Response = AccountResponse;

    const PATH: &'static str = "financial/accounts";

    const METHOD: reqwest::Method = reqwest::Method::GET;
}

#[derive(Debug, Deserialize, Default)]
pub struct AccountResponse {
    accounts: Vec<Account>,
}

impl AccountResponse {
    pub fn accounts(&self) -> &[Account] {
        &self.accounts
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub account_nr: String,
    pub account_name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub usage: AccountUsage,
    pub used_for_credit: bool,
    pub used_for_debit: bool,
    pub parent: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountType {
    Balance,
    Result,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountUsage {
    Generic,
    Transactional,
    Financial,
    Bank,
    Savings,
    Vat,
}
