use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Filter {
    String(StringNumberFilter),
    Date(DateFilter),
}

impl Filter {
    pub fn entity_type(is: impl ToString) -> Self {
        Self::String(StringNumberFilter {
            field_name: "entity_type".to_string(),
            operator: Operator::Equal,
            value: is.to_string(),
        })
    }

    pub fn new_string(field_name: String, operator: Operator, value: String) -> Self {
        Self::String(StringNumberFilter {
            field_name,
            operator,
            value,
        })
    }

    pub fn new_number(field_name: String, value: String) -> Self {
        Self::String(StringNumberFilter {
            field_name,
            operator: Operator::Equal,
            value,
        })
    }

    pub fn new_date(field_name: String, operator: DateOperator, start: NaiveDate, end: NaiveDate) -> Self {
        Self::Date(DateFilter {
            field_name,
            operator,
            value: DateValue { start, end },
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StringNumberFilter {
    field_name: String,
    operator: Operator,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Operator {
    #[serde(rename = "=")]
    Equal,
    #[serde(rename = "~")]
    Search,
    #[serde(rename = "!~")]
    SearchNot,
    #[serde(rename = "|=")]
    StartsWith,
    #[serde(rename = "+")]
    NotEmpty,
    #[serde(rename = "-")]
    Empty,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateFilter {
    field_name: String,
    operator: DateOperator,
    value: DateValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DateOperator {
    #[serde(rename = "><")]
    Between,
    #[serde(rename = ">=")]
    FromStart,
    #[serde(rename = "<=")]
    ToEnd,
}

#[derive(Debug, Serialize, Deserialize)]
struct DateValue {
    start: NaiveDate,
    end: NaiveDate,
}