use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
    ops::{Deref, DerefMut},
};

use penning_helper_types::Euro;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, DisplayFromStr, KeyValueMap};
use thiserror::Error;

use crate::{ConscriboError, ConscriboResult, Date, RelationType};

#[derive(Debug, Serialize, Deserialize)]
pub struct RootResult<T> {
    result: ConscriboResultE<T>,
}

impl<T> RootResult<T> {
    pub fn to_result(self) -> ConscriboResult<T> {
        match self.result {
            ConscriboResultE::Ok { result, ..} => Ok(result),
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
    pub field_name: String,
    entity_type: String,
    pub label: String,
    description: String,
    #[serde(rename = "type")]
    field_type: String,
    // required: Option<bool>,
    read_only: bool,
    pub shared_field_name: Option<String>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relations<R: DeserializeOwned + Serialize> {
    result_count: String,
    #[serde_as(as = "KeyValueMap<_>")]
    relations: Vec<R>,
}

impl<R: DeserializeOwned + Serialize> From<Relations<R>> for Vec<R> {
    fn from(relations: Relations<R>) -> Self {
        relations.relations
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    #[serde(rename = "$key$")]
    internal_id: String,
    #[serde_as(as = "DisplayFromStr")]
    pub code: u32,
    #[serde(alias = "weergavenaam")]
    pub naam: String,
    #[serde(alias = "e_mailadres", alias = "email")]
    pub email_address: String,
    #[serde(default, alias = "bankrekeningnummer")]
    pub rekening: Option<Account>,
    #[serde(default, rename = "membership_started")]
    pub membership_started: Option<Date>,
    #[serde_as(as = "serde_with::BoolFromInt")]
    #[serde(default, alias = "geen_invoice")]
    pub no_invoice: bool,

    #[serde(skip, default = "Member::entity_type")]
    pub source: &'static str,

    #[serde(alias = "alumni_lidmaatschap_gestart")]
    pub alumni_lidmaatschap_gestart: Option<Date>,
    #[serde(alias = "alumni_lidmaatschap_be__indigt")]
    pub alumni_lidmaatschap_beeindigd: Option<Date>,
    #[serde(alias = "alumni_contributie")]
    pub alumni_contributie: Euro,
}

impl RelationType for Member {
    const ENTITY_TYPE: &'static str = "lid";

    fn fields() -> Vec<&'static str> {
        vec![
            "code",
            "weergavenaam",
            "email",
            "rekening",
            "membership_started",
            "geen_invoice",
            "alumni_lidmaatschap_gestart",
            "alumni_lidmaatschap_be__indigt",
            "alumni_contributie",
        ]
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NonMember {
    #[serde(rename = "$key$")]
    internal_id: String,
    #[serde_as(as = "DisplayFromStr")]
    pub code: u32,
    #[serde(alias = "weergavenaam")]
    pub naam: String,
    #[serde(alias = "e_mailadres", alias = "email")]
    pub email_address: String,
    #[serde(default, alias = "bankrekeningnummer")]
    pub rekening: Option<Account>,
    #[serde(default, rename = "membership_started")]
    pub membership_started: Option<Date>,
}

impl RelationType for NonMember {
    const ENTITY_TYPE: &'static str = "onbekend";

    fn fields() -> Vec<&'static str> {
        vec![
            "code",
            "weergavenaam",
            "email",
            "rekening",
            "membership_started",
            "geen_invoice",
        ]
    }
}

impl From<NonMember> for Member {
    fn from(value: NonMember) -> Self {
        Self {
            internal_id: value.internal_id,
            code: value.code,
            naam: value.naam,
            email_address: value.email_address,
            rekening: value.rekening,
            membership_started: value.membership_started,
            no_invoice: false,
            source: Self::ENTITY_TYPE,
            alumni_lidmaatschap_gestart: None,
            alumni_lidmaatschap_beeindigd: None,
            alumni_contributie: Euro::default(),
        }
    }
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
        let date = value.date;

        let mut rows = HashMap::new();

        for row in &value.transaction_rows {
            println!("{:?}", row);
            if row.account_nr != "1001" && row.account_nr != "1002" {
                continue;
            }
            if let Some(r) = row.relation_nr {
                let urow = rows.entry(r).or_insert_with(|| UnifiedTransaction {
                    unique_id: format!(
                        "{}-{}-{}-{}-{}",
                        row.reference.as_ref().map(String::as_str).unwrap_or("????"),
                        r,
                        row.amount,
                        row.description,
                        row.account_nr,
                    ),
                    date,
                    code: r,
                    description: row.description.clone(),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UnifiedTransaction {
    pub unique_id: String,
    pub date: Date,
    pub code: u32,
    pub description: String,
    pub reference: String,
    pub cost: Euro,
}

impl UnifiedTransaction {
    pub fn create_new_mock(date: Date, description: String, cost: Euro) -> Self {
        Self {
            unique_id: "mock".to_string(),
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceRelationsResult {
    #[serde(default)]
    relation_nr: Option<String>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResult {
    pub accounts: HashMap<String, Rekening>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rekening {
    pub account_nr: String,
    pub account_name: String,
    pub parent: Option<usize>,
    #[serde(rename = "type")]
    pub account_type: String,
    pub used_for_debit: bool,
    pub used_for_credit: bool,
    #[serde(default)]
    pub transactional: bool,
}

impl AccountResult {
    pub fn to_rekening_maps(self) -> RekeningMap {
        let rekeningen = self.accounts;

        let mut stack = VecDeque::new();
        let mut rekening_maps = RekeningMap::new();
        for (nr, rek) in rekeningen {
            let rm = RekeningMapEntry {
                nr: nr.parse().unwrap(),
                account_nr: rek.account_nr,
                account_name: rek.account_name,
                children: Default::default(),
            };
            if let Some(parent) = rek.parent {
                if let Some(parent) = rekening_maps.find_recursive_mut(parent) {
                    parent.children.push(rm);
                } else {
                    stack.push_front((parent, rm));
                }
            } else {
                rekening_maps.push(rm);
            }
        }

        while let Some((parent, child)) = stack.pop_back() {
            if let Some(parent) = rekening_maps.find_recursive_mut(parent) {
                parent.children.push(child);
            } else {
                stack.push_front((parent, child));
            }
        }

        rekening_maps
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RekeningMap(Vec<RekeningMapEntry>);

impl RekeningMap {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn find_recursive(&self, nr: usize) -> Option<&RekeningMapEntry> {
        for entry in &self.0 {
            if entry.nr == nr {
                return Some(entry);
            }
            if let Some(entry) = entry.find_recursive(nr) {
                return Some(entry);
            }
        }
        None
    }

    pub fn find_recursive_mut(&mut self, nr: usize) -> Option<&mut RekeningMapEntry> {
        for entry in &mut self.0 {
            if entry.nr == nr {
                return Some(entry);
            }
            if let Some(entry) = entry.find_recursive_mut(nr) {
                return Some(entry);
            }
        }
        None
    }

    fn push(&mut self, entry: RekeningMapEntry) {
        self.0.push(entry);
    }

    pub fn find_closest_match(&self, name: &str) -> Option<&RekeningMapEntry> {
        // uses textdistance::str::damerau_levenshtein(s1, s2), and works recursively
        let best = f64::INFINITY;
        self.find_closest_match_internal(&name.to_lowercase(), best)
            .map(|(_, e)| e)
    }

    fn find_closest_match_internal(
        &self,
        name: &str,
        mut best: f64,
    ) -> Option<(f64, &RekeningMapEntry)> {
        let mut best_entry = None;
        for entry in &self.0 {
            let account_name = &entry.account_name.to_lowercase();
            let idx = account_name.find('(');
            let score = if let Some(idx) = idx {
                textdistance::nstr::damerau_levenshtein(
                    &(account_name.split_at(idx).0.trim()),
                    name,
                )
            } else {
                textdistance::nstr::damerau_levenshtein(&account_name, name)
            };

            if score < best {
                best = score;
                best_entry = Some(entry);
            }
            if let Some((score, entry)) = entry.find_closest_match_internal(name, best) {
                if score < best {
                    best = score;
                    best_entry = Some(entry);
                }
            }
        }
        best_entry.map(|e| (best, e))
    }

    pub fn iter(&self) -> RekeningMapRefIterator<'_> {
        let mut stack = VecDeque::new();
        stack.extend(self.0.iter());
        RekeningMapRefIterator { stack }
    }

    pub fn find_by_name(&self, name: &str) -> Option<&RekeningMapEntry> {
        self.iter().find(|e| e.account_name == name)
    }
}

impl Deref for RekeningMap {
    type Target = Vec<RekeningMapEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RekeningMapEntry {
    pub nr: usize,
    pub account_nr: String,
    pub account_name: String,
    pub children: RekeningMap,
}

impl Deref for RekeningMapEntry {
    type Target = RekeningMap;

    fn deref(&self) -> &Self::Target {
        &self.children
    }
}

impl DerefMut for RekeningMapEntry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.children
    }
}

impl RekeningMapEntry {
    pub fn number(&self) -> usize {
        self.nr
    }
}

impl AsRef<str> for RekeningMapEntry {
    fn as_ref(&self) -> &str {
        &self.account_name
    }
}

pub struct RekeningMapIterator {
    stack: VecDeque<RekeningMapEntry>,
}

pub struct RekeningMapRefIterator<'m> {
    stack: VecDeque<&'m RekeningMapEntry>,
}

impl<'m> Iterator for RekeningMapRefIterator<'m> {
    type Item = &'m RekeningMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.stack.pop_front()?;
        self.stack.extend(entry.children.0.iter());
        Some(entry)
    }
}

impl Iterator for RekeningMapIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.stack.pop_front()?;
        self.stack.extend(entry.children.0.into_iter());
        Some(entry.account_name)
    }
}

impl RekeningMapIterator {
    pub fn new(rekening_map: RekeningMap) -> Self {
        let mut stack = VecDeque::new();
        stack.extend(rekening_map.0.into_iter());
        Self { stack }
    }
}

impl IntoIterator for RekeningMap {
    type Item = String;

    type IntoIter = RekeningMapIterator;

    fn into_iter(self) -> Self::IntoIter {
        RekeningMapIterator::new(self)
    }
}

impl<'a> IntoIterator for &'a RekeningMap {
    type Item = &'a RekeningMapEntry;

    type IntoIter = RekeningMapRefIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
