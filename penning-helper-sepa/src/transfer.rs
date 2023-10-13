use std::io::Write;

use penning_helper_types::{Date, Euro};
use xml::EventWriter;

use crate::ToXml;

use self::transfer_gen::DocumentString;

mod transfer_gen;

pub struct Document {
    header: Header,
    payment_information: Vec<PaymentInformation>,
}

impl Document {
    pub(crate) fn new(header: Header, payment_information: Vec<PaymentInformation>) -> Document {
        Document {
            header,
            payment_information,
        }
    }

    pub fn to_xml_doc(self) -> DocumentString {
        DocumentString::from(self)
    }

    pub fn write<W: Write>(self, writer: W) -> xml::writer::Result<()> {
        let mut writer = EventWriter::new(writer);
        self.write_xml(&mut writer)?;
        writer.inner_mut().flush()?;
        Ok(())
    }

    fn write_xml<W: Write>(self, writer: &mut EventWriter<W>) -> xml::writer::Result<()> {
        let doc = self.to_xml_doc();
        let xml = doc.to_xml();
        for event in xml {
            writer.write(event)?;
        }
        Ok(())
    }
}

pub struct Header {
    message_id: String,
    creation_date_time: String,
    number_of_transactions: u32,
    control_sum: Euro,
    name: String,
}

impl Header {
    pub(crate) fn new(
        message_id: String,
        creation_date_time: String,
        number_of_transactions: u32,
        control_sum: Euro,
        name: String,
    ) -> Self {
        Self {
            message_id,
            creation_date_time,
            number_of_transactions,
            control_sum,
            name,
        }
    }
}

pub struct PaymentInformation {
    payment_information_id: String,
    number_of_transactions: u32,
    control_sum: Euro,
    execution_date: Date,
    debtor_name: String,
    debtor_iban: String,
    debtor_bic: String,
    creditors: Vec<Creditor>,
}

impl PaymentInformation {
    pub(crate) fn new(
        payment_information_id: String,
        number_of_transactions: u32,
        control_sum: Euro,
        execution_date: Date,
        debtor_name: String,
        debtor_iban: String,
        debtor_bic: String,
        creditors: Vec<Creditor>,
    ) -> Self {
        Self {
            payment_information_id,
            number_of_transactions,
            control_sum,
            execution_date,
            debtor_name,
            debtor_iban,
            debtor_bic,
            creditors,
        }
    }

    pub fn number_of_transactions(&self) -> u32 {
        self.number_of_transactions
    }

    pub fn control_sum(&self) -> Euro {
        self.control_sum
    }
}

#[derive(Debug)]
pub struct Creditor {
    id: String,
    amount: Euro,
    bic: String,
    name: String,
    iban: String,
    description: String,
}

impl Creditor {
    pub(crate) fn new(
        id: String,
        amount: Euro,
        bic: String,
        name: String,
        iban: String,
        description: String,
    ) -> Self {
        Self {
            id,
            amount,
            bic,
            name,
            iban,
            description,
        }
    }

    pub fn amount(&self) -> Euro {
        self.amount
    }
}
