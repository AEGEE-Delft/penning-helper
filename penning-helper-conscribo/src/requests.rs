use penning_helper_macros::set_command;
use penning_helper_types::{Date, Euro};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, EnumMap};

use crate::Side;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConscriboRequest<T> {
    pub(crate) request: T,
}

impl<T> ConscriboRequest<T> {
    pub fn new(request: T) -> Self {
        Self { request }
    }
}

#[derive(Debug, Serialize)]
pub struct ConscriboMultiRequest<T> {
    requests: ConscriboRequest<Vec<ReqHolder<T>>>,
}

impl<T: ToRequest> ConscriboMultiRequest<T> {
    pub fn new(requests: Vec<T>) -> Self {
        Self {
            requests: ConscriboRequest {
                request: requests
                    .into_iter()
                    .map(|r| r.to_request().request)
                    .collect(),
            },
        }
    }
}

impl<T> ConscriboRequest<ReqHolder<T>> {
    pub fn get_command(&self) -> &'static str {
        self.request.command
    }
}

pub trait ToRequest: Sized + Serialize {
    const COMMAND: &'static str;

    fn to_request(self) -> ConscriboRequest<ReqHolder<Self>> {
        ConscriboRequest {
            request: ReqHolder {
                command: Self::COMMAND,
                request: self,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReqHolder<T> {
    command: &'static str,
    #[serde(flatten)]
    request: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[set_command(authenticateWithUserAndPass)]
pub struct LoginRequest {
    user_name: String,
    pass_phrase: String,
}

impl LoginRequest {
    pub fn new(user_name: String, pass_phrase: String) -> Self {
        Self {
            user_name,
            pass_phrase,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[set_command(listFieldDefinitions)]
pub struct FieldReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_type: Option<String>,
}

impl FieldReq {
    pub fn new(entity_type: String) -> Self {
        Self {
            entity_type: Some(entity_type),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[set_command(listRelations)]
pub struct ListRelations {
    entity_type: String,
    requested_fields: RequestFields,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestFields {
    field_name: Vec<String>,
}

impl ListRelations {
    pub fn new(entity_type: String, fields: Vec<String>) -> Self {
        Self {
            entity_type,
            requested_fields: RequestFields { field_name: fields },
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[set_command(listTransactions)]
pub struct ListTransactions {
    #[serde_as(as = "EnumMap")]
    filters: Vec<TransactionFilter>,
}

impl ListTransactions {
    pub fn new(filters: Vec<TransactionFilter>) -> Self {
        Self { filters }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionFilter {
    DateStart(Date),
    DateEnd(Date),
    Relations {
        #[serde(rename = "relationNr")]
        #[serde_as(as = "Vec<DisplayFromStr>")]
        relation_nr: Vec<u32>,
    },
}

impl TransactionFilter {
    pub fn relations(relation_nr: Vec<u32>) -> Self {
        Self::Relations { relation_nr }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[set_command(addChangeTransaction)]
pub struct AddChangeTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    transaction_id: Option<i32>,
    date: Date,
    description: String,
    transaction_rows: TransactionRows,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransactionRows {
    transaction_row: Vec<TransactionRow>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransactionRow {
    account_nr: String,
    amount: Euro,
    side: Side,
    reference: String,
    relation_nr: u32,
}

impl AddChangeTransaction {
    pub fn new(date: Date, description: String) -> Self {
        Self {
            transaction_id: None,
            date,
            description,
            transaction_rows: TransactionRows {
                transaction_row: Vec::new(),
            },
        }
    }

    pub fn with_row(
        mut self,
        account_nr: String,
        amount: Euro,
        side: Side,
        reference: String,
        relation_nr: u32,
    ) -> Self {
        self.transaction_rows.transaction_row.push(TransactionRow {
            account_nr,
            amount,
            side,
            reference,
            relation_nr,
        });
        self
    }

    pub fn add_debet(
        self,
        rekening: String,
        amount: Euro,
        reference: String,
        relation_nr: u32,
    ) -> Self {
        self.with_row(
            rekening,
            amount,
            Side::Credit,
            reference.clone(),
            relation_nr.clone(),
        )
        .with_row(
            "1001".to_string(),
            amount,
            Side::Debet,
            reference,
            relation_nr,
        )
    }

    pub fn add_credit(
        self,
        rekening: String,
        amount: Euro,
        reference: String,
        relation_nr: u32,
    ) -> Self {
        self.with_row(
            rekening,
            amount,
            Side::Debet,
            reference.clone(),
            relation_nr.clone(),
        )
        .with_row(
            "1002".to_string(),
            amount,
            Side::Credit,
            reference,
            relation_nr,
        )
    }
}
