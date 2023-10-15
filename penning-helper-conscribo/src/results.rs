use std::{collections::HashMap, fmt::Display, ops::Deref};

use penning_helper_types::Euro;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, DisplayFromStr, KeyValueMap};
use thiserror::Error;

use crate::{ConscriboError, ConscriboResult, Date};

#[derive(Debug, Serialize, Deserialize)]
pub struct RootResult<T> {
    result: ConscriboResultE<T>,
}

impl<T> RootResult<T> {
    pub fn to_result(self) -> ConscriboResult<T> {
        match self.result {
            ConscriboResultE::Ok { result, success: _ } => Ok(result),
            ConscriboResultE::Err {
                notifications,
                success: _,
            } => Err(ConscriboError::ErrorMessages(notifications.notification)),
        }
    }
}

impl<T> From<RootResult<T>> for ConscriboResult<T> {
    fn from(root_result: RootResult<T>) -> Self {
        root_result.to_result()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiRootResult<T> {
    results: MultiResult<T>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MultiResult<T> {
    result: Vec<ConscriboResultE<T>>,
}

impl<T> From<MultiRootResult<T>> for ConscriboResult<Vec<T>> {
    fn from(value: MultiRootResult<T>) -> Self {
        value
            .results
            .result
            .into_iter()
            .map(|r| match r {
                ConscriboResultE::Ok { result, .. } => Ok(result),
                ConscriboResultE::Err { notifications, .. } => {
                    Err(ConscriboError::ErrorMessages(notifications.notification))
                }
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ConscriboResultE<T> {
    Ok {
        success: Success<1>,
        #[serde(flatten)]
        result: T,
    },
    Err {
        success: Success<0>,
        notifications: Notification,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Success<const V: u8>;

impl<const V: u8> Serialize for Success<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(V)
    }
}

impl<'de, const V: u8> Deserialize<'de> for Success<V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        if value == V {
            Ok(Success::<V>)
        } else {
            Err(serde::de::Error::custom("Invalid Success value"))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    notification: Vec<String>,
}

impl Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for notification in &self.notification {
            writeln!(f, "{}", notification)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldRes {
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    field_name: String,
    entity_type: String,
    label: String,
    description: String,
    #[serde(rename = "type")]
    field_type: String,
    // required: Option<bool>,
    read_only: bool,
    shared_field_name: Option<String>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relations {
    result_count: String,
    #[serde_as(as = "KeyValueMap<_>")]
    relations: Vec<Relation>,
}

impl From<Relations> for Vec<Relation> {
    fn from(relations: Relations) -> Self {
        relations.relations
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    #[serde(rename = "$key$")]
    internal_id: String,
    #[serde_as(as = "DisplayFromStr")]
    pub code: u32,
    pub naam: String,
    #[serde(alias = "e_mailadres", alias = "email")]
    pub email_address: String,
    #[serde(default, alias = "bankrekeningnummer")]
    pub rekening: Option<Account>,
    #[serde(default, rename = "membership_started")]
    pub membership_started: Option<Date>,
    #[serde(skip)]
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub iban: String,
    pub bic: String,
    pub name: String,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transactions {
    nr_transactions: String,
    #[serde_as(as = "KeyValueMap<_>")]
    transactions: Vec<Transaction>,
}

impl Transactions {
    pub fn into_transactions(self) -> Vec<Transaction> {
        self.transactions
    }
}

impl IntoIterator for Transactions {
    type Item = Transaction;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.transactions.into_iter()
    }
}

impl Deref for Transactions {
    type Target = Vec<Transaction>;

    fn deref(&self) -> &Self::Target {
        &self.transactions
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    #[serde(rename = "$key$")]
    internal_id: String,
    pub date: Date,
    pub description: String,
    pub transaction_id: i32,
    pub transaction_nr: String,
    #[serde_as(as = "KeyValueMap<_>")]
    pub transaction_rows: Vec<TransactionRow>,
}

impl Transaction {
    pub fn unify(self) -> Result<Vec<UnifiedTransaction>, TransactionConvertError> {
        self.try_into()
    }
}

fn default_account() -> String {
    "99999".to_string()
}

fn nullable_account<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_else(default_account))
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRow {
    #[serde(rename = "$key$")]
    internal_id: String,
    #[serde(default = "default_account", deserialize_with = "nullable_account")]
    pub account_nr: String,
    pub amount: Euro,
    pub description: String,
    pub reference: Option<String>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub relation_nr: Option<u32>,
    pub side: Side,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Debet,
    Credit,
}

#[derive(Debug, Error)]
pub enum TransactionConvertError {
    #[error("Multiple Relations found in transaction: {0:?}")]
    MultipleRelations(Vec<u32>),
}

impl TryFrom<Transaction> for Vec<UnifiedTransaction> {
    type Error = TransactionConvertError;

    fn try_from(value: Transaction) -> Result<Self, Self::Error> {
        let description = value.description;
        let date = value.date;

        let mut rows = HashMap::new();

        for row in &value.transaction_rows {
            println!("{:?}", row);
            if row.account_nr != "1001" && row.account_nr != "1002" {
                continue;
            }
            if let Some(r) = row.relation_nr {
                let urow = rows.entry(r).or_insert_with(|| UnifiedTransaction {
                    date,
                    code: r,
                    description: description.clone(),
                    reference: row.reference.clone().unwrap_or_else(|| "????".to_string()),
                    cost: Default::default(),
                });
                match row.side {
                    Side::Debet => urow.cost += row.amount,
                    Side::Credit => urow.cost -= row.amount,
                }
            }
        }
        // let relations: HashSet<u32> = value
        //     .transaction_rows
        //     .iter()
        //     .filter_map(|row| row.relation_nr)
        //     .collect();
        // if relations.len() > 1 {
        //     println!(
        //         "Multiple relations in transaction: {:?}",
        //         value.transaction_nr
        //     );
        //     return Err(TransactionConvertError::MultipleRelations(
        //         relations.into_iter().collect(),
        //     ));
        // }
        // let reference = value
        //     .transaction_rows
        //     .iter()
        //     .find_map(|row| row.reference.clone())
        //     .unwrap_or_default();

        // let cost = value
        //     .transaction_rows
        //     .iter()
        //     .filter(|r| r.account_nr == "1001" || r.account_nr == "1002")
        //     .fold(Euro::default(), |acc, row| match row.side {
        //         Side::Debet => acc + row.amount,
        //         Side::Credit => acc - row.amount,
        //     });
        Ok(rows.into_iter().map(|(_, v)| v).collect())
    }
}

#[derive(Debug, Clone)]
pub struct UnifiedTransaction {
    pub date: Date,
    pub code: u32,
    pub description: String,
    pub reference: String,
    pub cost: Euro,
}

impl UnifiedTransaction {
    pub fn create_new_mock(date: Date, description: String, cost: Euro) -> Self {
        Self {
            date,
            code: 0,
            description,
            reference: "????".to_string(),
            cost,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResult {
    transaction_id: i32,
    transaction_nr: String,
}
