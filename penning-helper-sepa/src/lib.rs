mod invoice;
mod transfer;

use invoice::Debtor;
// use invoice_gen::{DebtorString, DocumentString, HeaderString, PaymentInformationString};

use penning_helper_types::{Date, Euro};
use rand::{thread_rng, Rng};
use xml::writer::XmlEvent;

#[derive(Debug, Clone, Default)]
pub struct SEPAConfig {
    pub company_name: String,
    pub company_iban: String,
    pub company_bic: String,
    pub company_id: String,
}

impl SEPAConfig {
    fn new(
        creditor_name: impl ToString,
        creditor_iban: impl ToString,
        creditor_bic: impl ToString,
        creditor_id: impl ToString,
    ) -> Self {
        Self {
            company_name: creditor_name.to_string(),
            company_iban: creditor_iban.to_string(),
            company_bic: creditor_bic.to_string(),
            company_id: creditor_id.to_string(),
        }
    }

    pub fn from_config(cfg: &penning_helper_config::SEPAConfig) -> Self {
        Self::new(
            &cfg.company_name,
            &cfg.company_iban,
            &cfg.company_bic,
            &cfg.company_id,
        )
    }

    pub fn new_debtor(
        &self,
        amount: Euro,
        name: String,
        bic: String,
        iban: String,
        code: u32,
        membership_date: Date,
        description: String,
    ) -> Debtor {
        let id = thread_rng().gen::<u64>();
        let invoice_id = format!(
            "{}-{:0>16x}",
            self.company_name.to_uppercase().replace('-', ""),
            id
        );
        Debtor::new(
            invoice_id,
            amount,
            name,
            bic,
            iban,
            code,
            membership_date,
            description,
        )
    }

    pub fn new_invoice_payment_information(
        &self,
        collection_date: Date,
        debtors: Vec<Debtor>,
    ) -> invoice::PaymentInformation {
        let id = thread_rng().gen::<u64>();
        let payment_information_id = format!(
            "{}-{:0>16x}",
            self.company_name.to_uppercase().replace('-', ""),
            id
        );
        let costs = debtors.iter().map(|d| d.amount()).sum::<Euro>();
        invoice::PaymentInformation::new(
            payment_information_id,
            self.company_name.clone(),
            self.company_iban.clone(),
            self.company_bic.clone(),
            collection_date,
            costs,
            debtors.len() as u32,
            self.company_id.clone(),
            debtors,
        )
    }

    pub fn new_invoice_document(
        &self,
        payment_info: invoice::PaymentInformation,
    ) -> invoice::Document {
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
        let now = now.to_string();
        let id = thread_rng().gen::<u64>();
        let message_id = format!(
            "{}-{:0>16x}",
            Date::today().to_string().replace('-', ""),
            id
        );
        let header = invoice::Header::new(
            message_id,
            now,
            payment_info.num_transactions(),
            payment_info.control_sum(),
            self.company_name.clone(),
            self.company_id.clone(),
        );
        invoice::Document::new(header, vec![payment_info])
    }

    pub fn new_creditor(
        &self,
        amount: Euro,
        name: String,
        bic: String,
        iban: String,
        description: String,
    ) -> transfer::Creditor {
        let id = thread_rng().gen::<u64>();
        let invoice_id = format!(
            "{}-{:0>16x}",
            self.company_name.to_uppercase().replace('-', ""),
            id
        );
        transfer::Creditor::new(invoice_id, amount, bic, name, iban, description)
    }

    pub fn new_transfer_payment_information(
        &self,
        execution_date: Date,
        creditors: Vec<transfer::Creditor>,
    ) -> transfer::PaymentInformation {
        let id = thread_rng().gen::<u64>();
        let payment_information_id = format!(
            "{}-{:0>16x}",
            self.company_name.to_uppercase().replace('-', ""),
            id
        );
        let costs = creditors.iter().map(|d| d.amount()).sum::<Euro>();
        transfer::PaymentInformation::new(
            payment_information_id,
            creditors.len() as u32,
            costs,
            execution_date,
            self.company_name.clone(),
            self.company_iban.clone(),
            self.company_bic.clone(),
            creditors,
        )
    }

    pub fn new_transfer_document(
        &self,
        payment_info: transfer::PaymentInformation,
    ) -> transfer::Document {
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
        let now = now.to_string();
        let id = thread_rng().gen::<u64>();
        let message_id = format!(
            "{}-{:0>16x}",
            Date::today().to_string().replace('-', ""),
            id
        );
        let header = transfer::Header::new(
            message_id,
            now,
            payment_info.number_of_transactions(),
            payment_info.control_sum(),
            self.company_name.clone(),
        );
        transfer::Document::new(header, vec![payment_info])
    }
}

trait ToXml {
    fn to_xml(&self) -> Vec<XmlEvent>;
}
