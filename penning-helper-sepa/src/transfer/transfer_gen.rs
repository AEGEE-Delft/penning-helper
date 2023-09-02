use xml::writer::XmlEvent;

use crate::ToXml;

pub struct DocumentString {
    header: HeaderString,
    payment_information: Vec<PaymentInformationString>,
}

impl ToXml for DocumentString {
    fn to_xml(&self) -> Vec<XmlEvent> {
        let mut v = vec![
            XmlEvent::start_element("Document")
                .default_ns("urn:iso:std:iso:20022:tech:xsd:pain.001.001.03")
                .ns("xsi", "http://www.w3.org/2001/XMLSchema-instance")
                .into(),
            XmlEvent::start_element("CstmrCdtTrfInitn").into(),
        ];
        v.extend(self.header.to_xml());
        for payment_information in &self.payment_information {
            v.extend(payment_information.to_xml());
        }
        v.push(XmlEvent::end_element().into());
        v.push(XmlEvent::end_element().into());
        v
    }
}

impl From<super::Document> for DocumentString {
    fn from(value: super::Document) -> Self {
        Self {
            header: value.header.into(),
            payment_information: value
                .payment_information
                .into_iter()
                .map(|payment_information| payment_information.into())
                .collect(),
        }
    }
}

struct HeaderString {
    /// date-randomhash
    message_id: String,
    creation_date_time: String,
    number_of_transactions: String,
    control_sum: String,
    name: String,
}

impl ToXml for HeaderString {
    fn to_xml(&self) -> Vec<XmlEvent> {
        vec![
            XmlEvent::start_element("GrpHdr").into(),
            XmlEvent::start_element("MsgId").into(),
            XmlEvent::characters(&self.message_id),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CreDtTm").into(),
            XmlEvent::characters(&self.creation_date_time),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("NbOfTxs").into(),
            XmlEvent::characters(&self.number_of_transactions),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CtrlSum").into(),
            XmlEvent::characters(&self.control_sum),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("InitgPty").into(),
            XmlEvent::start_element("Nm").into(),
            XmlEvent::characters(&self.name),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
        ]
    }
}

impl From<super::Header> for HeaderString {
    fn from(value: super::Header) -> Self {
        Self {
            message_id: value.message_id,
            creation_date_time: value.creation_date_time,
            number_of_transactions: value.number_of_transactions.to_string(),
            control_sum: value.control_sum.xml_string(),
            name: value.name,
        }
    }
}

struct PaymentInformationString {
    payment_information_id: String,
    number_of_transactions: String,
    control_sum: String,
    execution_date: String,
    debtor_name: String,
    debtor_iban: String,
    debtor_bic: String,
    creditors: Vec<CreditorString>,
}

impl ToXml for PaymentInformationString {
    fn to_xml(&self) -> Vec<XmlEvent> {
        let mut v = vec![
            XmlEvent::start_element("PmtInf").into(),
            XmlEvent::start_element("PmtInfId").into(),
            XmlEvent::characters(&self.payment_information_id),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("PmtMtd").into(),
            XmlEvent::characters("TRF"),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("BtchBookg").into(),
            XmlEvent::characters("true"),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("NbOfTxs").into(),
            XmlEvent::characters(&self.number_of_transactions),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CtrlSum").into(),
            XmlEvent::characters(&self.control_sum),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("PmtTpInf").into(),
            XmlEvent::start_element("SvcLvl").into(),
            XmlEvent::start_element("Cd").into(),
            XmlEvent::characters("SEPA"),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("ReqdExctnDt").into(),
            XmlEvent::characters(&self.execution_date),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("Dbtr").into(),
            XmlEvent::start_element("Nm").into(),
            XmlEvent::characters(&self.debtor_name),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("DbtrAcct").into(),
            XmlEvent::start_element("Id").into(),
            XmlEvent::start_element("IBAN").into(),
            XmlEvent::characters(&self.debtor_iban),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("DbtrAgt").into(),
            XmlEvent::start_element("FinInstnId").into(),
            XmlEvent::start_element("BIC").into(),
            XmlEvent::characters(&self.debtor_bic),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("ChrgBr").into(),
            XmlEvent::characters("SLEV"),
            XmlEvent::end_element().into(),
        ];
        for creditor in &self.creditors {
            v.extend(creditor.to_xml());
        }
        v.push(XmlEvent::end_element().into());
        v
    }
}

impl From<super::PaymentInformation> for PaymentInformationString {
    fn from(value: super::PaymentInformation) -> Self {
        Self {
            payment_information_id: value.payment_information_id,
            number_of_transactions: value.number_of_transactions.to_string(),
            control_sum: value.control_sum.xml_string(),
            execution_date: value.execution_date.to_string(),
            debtor_name: value.debtor_name,
            debtor_iban: value.debtor_iban,
            debtor_bic: value.debtor_bic,
            creditors: value
                .creditors
                .into_iter()
                .map(|creditor| creditor.into())
                .collect(),
        }
    }
}

struct CreditorString {
    id: String,
    amount: String,
    bic: String,
    name: String,
    iban: String,
    description: String,
}

impl ToXml for CreditorString {
    fn to_xml(&self) -> Vec<xml::writer::XmlEvent> {
        vec![
            XmlEvent::start_element("CdtTrfTxInf").into(),
            XmlEvent::start_element("PmtId").into(),
            XmlEvent::start_element("EndToEndId").into(),
            XmlEvent::characters(&self.id),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("Amt").into(),
            XmlEvent::start_element("InstdAmt")
                .attr("Ccy", "EUR")
                .into(),
            XmlEvent::characters(&self.amount),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CdtrAgt").into(),
            XmlEvent::start_element("FinInstnId").into(),
            XmlEvent::start_element("BIC").into(),
            XmlEvent::characters(&self.bic),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("Cdtr").into(),
            XmlEvent::start_element("Nm").into(),
            XmlEvent::characters(&self.name),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CdtrAcct").into(),
            XmlEvent::start_element("Id").into(),
            XmlEvent::start_element("IBAN").into(),
            XmlEvent::characters(&self.iban),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("RmtInf").into(),
            XmlEvent::start_element("Ustrd").into(),
            XmlEvent::characters(&self.description),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
        ]
    }
}

impl From<super::Creditor> for CreditorString {
    fn from(value: super::Creditor) -> Self {
        Self {
            id: value.id,
            amount: value.amount.xml_string(),
            bic: value.bic,
            name: value.name,
            iban: value.iban,
            description: value.description,
        }
    }
}
