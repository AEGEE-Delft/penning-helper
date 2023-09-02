use std::io::Write;

use penning_helper_types::{Date, Euro};
use xml::EventWriter;

use crate::ToXml;

use self::invoice_gen::DocumentString;

mod invoice_gen;

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

    pub fn write<W: Write>(self, writer: &mut EventWriter<W>) -> xml::writer::Result<()> {
        let doc = self.to_xml_doc();
        let xml = doc.to_xml();
        for event in xml {
            writer.write(event)?;
        }
        Ok(())
    }
}

pub struct Header {
    /// date-randomhash
    message_id: String,
    creation_date_time: String,
    number_of_transactions: u32,
    control_sum: Euro,
    name: String,
    id: String,
}

impl Header {
    pub(crate) fn new(
        message_id: String,
        creation_date_time: String,
        number_of_transactions: u32,
        control_sum: Euro,
        name: String,
        id: String,
    ) -> Self {
        Self {
            message_id,
            creation_date_time,
            number_of_transactions,
            control_sum,
            name,
            id,
        }
    }
}

#[derive(Debug)]
pub struct PaymentInformation {
    payment_information_id: String,
    creditor_name: String,
    creditor_iban: String,
    creditor_bic: String,
    collection_date: Date,
    control_sum: Euro,
    num_transactions: u32,
    creditor_id: String,
    debtors: Vec<Debtor>,
}

impl PaymentInformation {
    pub(crate) fn new(
        payment_information_id: String,
        creditor_name: String,
        creditor_iban: String,
        creditor_bic: String,
        collection_date: Date,
        control_sum: Euro,
        num_transactions: u32,
        creditor_id: String,
        debtors: Vec<Debtor>,
    ) -> Self {
        Self {
            payment_information_id,
            creditor_name,
            creditor_iban,
            creditor_bic,
            collection_date,
            control_sum,
            num_transactions,
            creditor_id,
            debtors,
        }
    }

    pub fn control_sum(&self) -> Euro {
        self.control_sum
    }

    pub fn num_transactions(&self) -> u32 {
        self.num_transactions
    }
}

#[derive(Debug)]
pub struct Debtor {
    invoice_id: String,
    amount: Euro,
    name: String,
    bic: String,
    iban: String,
    code: u32,
    membership_date: Date,
    description: String,
}

impl Debtor {
    pub(crate) fn new(
        invoice_id: String,
        amount: Euro,
        name: String,
        bic: String,
        iban: String,
        code: u32,
        membership_date: Date,
        description: String,
    ) -> Self {
        Self {
            invoice_id,
            amount,
            name,
            bic,
            iban,
            code,
            membership_date,
            description,
        }
    }

    pub fn amount(&self) -> Euro {
        self.amount
    }
}
