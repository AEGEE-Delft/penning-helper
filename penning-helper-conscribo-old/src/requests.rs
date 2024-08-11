use std::collections::HashMap;

use crate::results::*;
use crate::LoginResult;
use crate::Side;
use penning_helper_macros::set_command;
use penning_helper_types::{Date, Euro};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr, EnumMap};

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

    type Response: DeserializeOwned;

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
#[set_command(authenticateWithUserAndPass -> LoginResult)]
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
#[set_command(listFieldDefinitions -> FieldRes)]
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
#[set_command(listRelations -> Relations<R>)]
pub struct ListRelations<R: DeserializeOwned + Serialize> {
    entity_type: String,
    requested_fields: RequestFields,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<R>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestFields {
    field_name: Vec<String>,
}

impl<R: DeserializeOwned + Serialize> ListRelations<R> {
    pub fn new(entity_type: impl ToString, fields: Vec<impl ToString>) -> Self {
        Self {
            entity_type: entity_type.to_string(),
            requested_fields: RequestFields {
                field_name: fields.into_iter().map(|f| f.to_string()).collect(),
            },
            _phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[set_command(replaceRelations -> ReplaceRelationsResult)]
pub struct UpdateRelation {
    code: String,
    fields: HashMap<String, Value>,
}

impl UpdateRelation {
    pub fn new(code: String) -> Self {
        Self {
            code,
            fields: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, field: impl ToString, value: impl Serialize) {
        self.fields
            .insert(field.to_string(), serde_json::to_value(value).unwrap());
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[set_command(listTransactions -> Transactions)]
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
#[set_command(addChangeTransaction -> TransactionResult)]
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

    pub fn add_merch(
        self,
        relation_nr: u32,
        merch_rekening: String,
        merch_verkoop_rekening: String,
        total_amount: Euro,
        merch_price: Euro,
        reference: String,
    ) -> Self {
        self.with_row(
            merch_rekening,
            merch_price,
            Side::Credit,
            reference.clone(),
            relation_nr,
        )
        .with_row(
            merch_verkoop_rekening,
            total_amount - merch_price,
            Side::Credit,
            reference.clone(),
            relation_nr,
        )
        .with_row(
            "1001".to_string(),
            total_amount,
            Side::Debet,
            reference,
            relation_nr,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[set_command(listAccounts -> AccountResult)]
pub struct ListAccounts {
    date: Date,
}

impl ListAccounts {
    pub fn new(date: Date) -> Self {
        Self { date }
    }

    pub fn today() -> Self {
        Self::new(Date::today())
    }
}
