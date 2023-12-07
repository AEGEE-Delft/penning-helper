use std::{collections::HashMap, io::Read, num::ParseFloatError, path::Path};

use csv::Reader;
use penning_helper_types::Euro;
use serde::{Deserialize, Serialize};

use crate::turflist::{TurfList, TurfListRow};

#[derive(Debug, Deserialize, Serialize)]
struct CsvEntry {
    // Datum,Bon nummer,Bon soort,Bruto-omzet,Kortingen,Netto-omzet,Btw,Fooien,Totaal verzameld,Kosten van goederen,Bruto winst,Betaalwijzen,Omschrijving,POS,Winkel,Naam medewerker,Naam klant,Klant contacten,Status
    #[serde(alias = "Totaal verzameld", alias = "Total collected", alias = "Total", alias = "Totaal")]
    total: String,
    #[serde(alias = "Naam klant", alias = "Customer name", alias = "Name", alias = "Naam")]
    name: String,
    #[serde(alias = "Klant contacten", alias = "Customer contacts", alias = "Email", alias = "E-mail", default = "default_email")]
    email: String,
    #[serde(alias = "Payment type", default = "payment_type_default")]
    payment_type: String,

    #[serde(alias = "Description", alias = "Omschrijving", default)]
    description: String,
}

fn payment_type_default() -> String {
    "AEGEE-DELFT".to_string()
}

fn default_email() -> String {
    "This Is a Very long string that will probably never match to a valid email address".to_string()
}

impl TryFrom<CsvEntry> for TurfListRow {
    type Error = ParseFloatError;

    fn try_from(value: CsvEntry) -> Result<Self, Self::Error> {
        let total = value.total.parse::<f64>()?;
        println!("{}: {}", total, Euro::from(total));
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

pub fn try_loyverse<R: Read>(mut rdr: Reader<R>) -> Result<TurfList, CsvReadError> {
    let mut list = vec![];
    let mut t = 0.0;
    for result in rdr.deserialize() {
        let record: CsvEntry = result?;
        if record.payment_type != "AEGEE-DELFT" {
            continue;
        }
        t += record.total.parse::<f64>()?;
        list.push(record.try_into()?);
    }
    println!("Total: {}", t);
    Ok(TurfList::new(list))
}

#[derive(Debug, Deserialize, Serialize)]
struct TurffEntry {
    #[serde(rename = "Naam")]
    name: String,
    #[serde(flatten)]
    data: HashMap<String, serde_json::Value>,
}

impl TryFrom<TurffEntry> for TurfListRow {
    type Error = ParseFloatError;

    fn try_from(value: TurffEntry) -> Result<Self, Self::Error> {
        let mut e = Self::new(value.name, "This Is a Very long string that will probably never match to a valid email address, i really hope this works :p".to_string(), Euro::from(0.0), None);
        let mut acc = vec![];
        for (k, v) in value.data {
            let price = if let Some(v) = v.as_f64() {
                Euro::from(v)
            } else if let Some(v) = v.as_i64() {
                Euro::from(v)
            } else if let Some(v) = v.as_str() {
                
                Euro::from(v.replace(',', ".").parse::<f64>()?)
            } else {
                println!("Skipping {}", k);
                continue;
            };

            e.amount += price;
            if price != Euro::default() {
                acc.push(format!("{}: {}", k, v));
            }
        }
        e.set_what(acc.join(", "));
        Ok(e)
    }
}

pub fn try_turff<R: Read>(mut rdr: Reader<R>) -> Result<TurfList, CsvReadError> {
    let mut list = vec![];
    let mut t = Euro::default();
    for result in rdr.deserialize() {
        let record: TurffEntry = result?;
        let converted: TurfListRow = record.try_into()?;
        if converted.amount != Euro::default() {
            t += converted.amount;
            list.push(converted);
        }
    }
    println!("Total: {}", t);
    Ok(TurfList::new(list))
}

pub fn read_csv(r: impl AsRef<Path>) -> Result<TurfList, CsvReadError> {
    let r = r.as_ref();
    if let Ok(l) = try_loyverse(csv::ReaderBuilder::new().from_path(r)?) {
        return Ok(l);
    } else {
        return try_turff(csv::ReaderBuilder::new().delimiter(b';').from_path(r)?);
    }
}
