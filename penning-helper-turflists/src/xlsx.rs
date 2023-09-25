use std::path::Path;

use calamine::Reader;
use penning_helper_types::Euro;

use crate::turflist::{TurfList, TurfListRow};

#[derive(Debug, Clone)]
struct ColumnIndices {
    first_name: usize,
    last_name: usize,
    naam: usize,
    email: usize,
    member: usize,
    iban: usize,
    price: usize,
}

impl Default for ColumnIndices {
    fn default() -> Self {
        Self {
            first_name: 999,
            last_name: 999,
            naam: 999,
            email: 999,
            member: 999,
            iban: 999,
            price: 999,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum XlsxError {
    #[error("Calamine error: {0}")]
    Calamine(#[from] calamine::Error),
    #[error("Deserialize error: {0}")]
    Deserialize(#[from] calamine::DeError),
    #[error("Xlsx error: {0}")]
    Xlsx(#[from] calamine::XlsxError),
    #[error("Other error: {0}")]
    Other(String),
}

pub fn read_excel(p: impl AsRef<Path>, cost: Euro) -> Result<TurfList, XlsxError> {
    let mut workbook = calamine::open_workbook_auto(p)?;
    let sheet = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| XlsxError::Other("Workbook does not have any sheets!".to_string()))?;
    let Some(x) = workbook.worksheet_range(&sheet) else {
        return Err(XlsxError::Xlsx(calamine::XlsxError::Unexpected(
            "No Sheet1 found",
        )));
    };
    let data = x?;
    let mut list = vec![];

    // let res = vec![];
    let mut rows = data.rows();
    let header = rows.next().unwrap();
    let mut indices = ColumnIndices::default();
    for (idx, content) in header.into_iter().enumerate() {
        let content = content.to_string().to_lowercase();
        match content.as_str() {
            "first name" => indices.first_name = idx,
            "naam" => indices.naam = idx,
            "last name" => indices.last_name = idx,
            "email" => indices.email = idx,
            "member" => indices.member = idx,
            "iban" => indices.iban = idx,
            "price" => indices.price = idx,
            "prijs" => indices.price = idx,
            "name" => indices.naam = idx,
            _ => {}
        }
    }

    for item in rows {
        let iban = item.get(indices.iban).map(|x| x.to_string());
        let iban = if iban == Some("".to_string()) {
            None
        } else {
            iban
        };
        let cost = if indices.price != 999 {
            item.get(indices.price).unwrap().as_f64().map(|f| Euro::from(f)).unwrap_or(cost)
        } else {
            cost
        };
        list.push(TurfListRow::new(
            if indices.naam != 999 {
                item.get(indices.naam).unwrap().to_string()
            } else {
                format!(
                    "{} {}",
                    item.get(indices.first_name).unwrap().to_string(),
                    item.get(indices.last_name).unwrap().to_string()
                )
            },
            item.get(indices.email).unwrap().to_string(),
            cost,
            iban,
        ))
    }

    Ok(TurfList::new(list))
}
