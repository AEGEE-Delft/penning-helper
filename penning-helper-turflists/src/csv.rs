use std::{io::Read, num::ParseFloatError};

use penning_helper_types::Euro;
use serde::{Deserialize, Serialize};

use crate::turflist::{TurfList, TurfListRow};

#[derive(Debug, Deserialize, Serialize)]
struct CsvEntry {
    // Datum,Bon nummer,Bon soort,Bruto-omzet,Kortingen,Netto-omzet,Btw,Fooien,Totaal verzameld,Kosten van goederen,Bruto winst,Betaalwijzen,Omschrijving,POS,Winkel,Naam medewerker,Naam klant,Klant contacten,Status
    #[serde(rename = "Totaal verzameld", alias = "Total collected")]
    total: String,
    #[serde(rename = "Naam klant", alias = "Customer name")]
    name: String,
    #[serde(rename = "Klant contacten", alias = "Customer contacts")]
    email: String,
    #[serde(rename = "Payment type")]
    payment_type: String,

    #[serde(rename = "Description")]
    description: String,
}

impl TryFrom<CsvEntry> for TurfListRow {
    type Error = ParseFloatError;

    fn try_from(value: CsvEntry) -> Result<Self, Self::Error> {
        let total = value.total.parse::<f64>()?;
        let mut e = Self::new(value.name, value.email, Euro::from(total), None);
        e.set_what(value.description);
        Ok(e)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CsvReadError {
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("Parse error: {0}")]
    EuroParseError(#[from] ParseFloatError),
}

pub fn read_csv(r: impl Read) -> Result<TurfList, CsvReadError> {
    let mut rdr = csv::Reader::from_reader(r);
    let mut list = vec![];
    for result in rdr.deserialize() {
        let record: CsvEntry = result?;
        if record.payment_type != "AEGEE-DELFT" {
            continue;
        }
        list.push(record.try_into()?);
    }
    Ok(TurfList::new(list))
}
