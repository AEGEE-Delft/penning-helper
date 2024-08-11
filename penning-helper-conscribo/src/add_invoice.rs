use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddInvoice {
    invoice_date: NaiveDate,
    relation_nr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    invoice_nr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invoice_expiry_date: Option<NaiveDate>,
    extra_comments: String,
    internal_comments: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    create_payment_page: Option<bool>,
}