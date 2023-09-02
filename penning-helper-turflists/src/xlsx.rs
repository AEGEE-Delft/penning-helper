use std::path::Path;

use calamine::Reader;
use penning_helper_types::Euro;

use crate::turflist::{TurfList, TurfListRow};

#[derive(Debug, Clone, Default)]
struct RowIndices {
    first_name: usize,
    last_name: usize,
    email: usize,
    member: usize,
    iban: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum XlsxError {
    #[error("Calamine error: {0}")]
    Calamine(#[from] calamine::Error),
    #[error("Deserialize error: {0}")]
    Deserialize(#[from] calamine::DeError),
    #[error("Xlsx error: {0}")]
    Xlsx(#[from] calamine::XlsxError),
}

pub fn read_excel(p: impl AsRef<Path>, cost: Euro) -> Result<TurfList, XlsxError> {
    let mut workbook = calamine::open_workbook_auto(p)?;

    let Some(x) = workbook.worksheet_range("Sheet1") else {
        return Err(XlsxError::Xlsx(calamine::XlsxError::Unexpected(
            "No Sheet1 found",
        )));
    };
    let data = x?;
    let mut list = vec![];

    // let res = vec![];
    let mut rows = data.rows();
    let header = rows.next().unwrap();
    let mut indices = RowIndices::default();
    for (idx, content) in header.into_iter().enumerate() {
        let content = content.to_string();
        match content.as_str() {
            "First Name" => indices.first_name = idx,
            "Last Name" => indices.last_name = idx,
            "email" => indices.email = idx,
            "Member" => indices.member = idx,
            "IBAN" => indices.iban = idx,
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
        list.push(TurfListRow::new(
            format!(
                "{} {}",
                item.get(indices.first_name).unwrap().to_string(),
                item.get(indices.last_name).unwrap().to_string()
            ),
            item.get(indices.email).unwrap().to_string(),
            cost,
            iban,
        ))
    }

    Ok(TurfList::new(list))
}
