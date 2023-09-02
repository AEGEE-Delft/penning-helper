use xml::writer::XmlEvent;

use crate::ToXml;

pub struct DocumentString {
    pub(super) header: HeaderString,
    pub(super) payment_information: Vec<PaymentInformationString>,
}

impl ToXml for DocumentString {
    fn to_xml(&self) -> Vec<XmlEvent> {
        let mut v = vec![
            XmlEvent::start_element("Document")
                .default_ns("urn:iso:std:iso:20022:tech:xsd:pain.008.001.02")
                .ns("xsi", "http://www.w3.org/2001/XMLSchema-instance")
                .into(),
            XmlEvent::start_element("CstmrDrctDbtInitn").into(),
        ];
        v.extend(self.header.to_xml());
        v.extend(self.payment_information.iter().flat_map(|p| p.to_xml()));
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
                .map(|p| p.into())
                .collect(),
        }
    }
}

pub struct HeaderString {
    /// date-randomhash
    pub(super) message_id: String,
    pub(super) creation_date_time: String,
    pub(super) number_of_transactions: String,
    pub(super) control_sum: String,
    pub(super) name: String,
    pub(super) id: String,
}

impl ToXml for HeaderString {
    fn to_xml(&self) -> Vec<XmlEvent> {
        vec![
            XmlEvent::start_element("GrpHdr").into(),
            XmlEvent::start_element("MsgId").into(),
            XmlEvent::characters(&self.message_id).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CreDtTm").into(),
            XmlEvent::characters(&self.creation_date_time).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("NbOfTxs").into(),
            XmlEvent::characters(&self.number_of_transactions).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CtrlSum").into(),
            XmlEvent::characters(&self.control_sum).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("InitgPty").into(),
            XmlEvent::start_element("Nm").into(),
            XmlEvent::characters(&self.name).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("Id").into(),
            XmlEvent::start_element("OrgId").into(),
            XmlEvent::start_element("Othr").into(),
            XmlEvent::start_element("Id").into(),
            XmlEvent::characters(&self.id).into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
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
            creation_date_time: value.creation_date_time.to_string(),
            number_of_transactions: value.number_of_transactions.to_string(),
            control_sum: value.control_sum.xml_string(),
            name: value.name,
            id: value.id,
        }
    }
}

pub struct PaymentInformationString {
    pub(super) payment_information_id: String,
    /// AEGEE-Delft
    pub(super) creditor_name: String,
    /// AEGEE-Delft IBAN
    pub(super) creditor_iban: String,
    /// AEGEE-Delft BIC
    pub(super) creditor_bic: String,
    /// today + 3 days
    pub(super) collection_date: String,
    /// Total amount of all invoices
    pub(super) control_sum: String,
    pub(super) num_transactions: String,
    /// same as id from header
    pub(super) creditor_id: String,
    pub(super) debtors: Vec<DebtorString>,
}

impl ToXml for PaymentInformationString {
    fn to_xml(&self) -> Vec<XmlEvent> {
        let mut xml = vec![
            XmlEvent::start_element("PmtInf").into(),
            XmlEvent::start_element("PmtInfId").into(),
            XmlEvent::characters(&self.payment_information_id).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("PmtMtd").into(),
            XmlEvent::characters("DD").into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("BtchBookg").into(),
            XmlEvent::characters("true").into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("NbOfTxs").into(),
            XmlEvent::characters(&self.num_transactions).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CtrlSum").into(),
            XmlEvent::characters(&self.control_sum).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("PmtTpInf").into(),
            XmlEvent::start_element("SvcLvl").into(),
            XmlEvent::start_element("Cd").into(),
            XmlEvent::characters("SEPA").into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("LclInstrm").into(),
            XmlEvent::start_element("Cd").into(),
            XmlEvent::characters("CORE").into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("SeqTp").into(),
            XmlEvent::characters("RCUR").into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("ReqdColltnDt").into(),
            XmlEvent::characters(&self.collection_date).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("Cdtr").into(),
            XmlEvent::start_element("Nm").into(),
            XmlEvent::characters(&self.creditor_name).into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CdtrAcct").into(),
            XmlEvent::start_element("Id").into(),
            XmlEvent::start_element("IBAN").into(),
            XmlEvent::characters(&self.creditor_iban).into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CdtrAgt").into(),
            XmlEvent::start_element("FinInstnId").into(),
            XmlEvent::start_element("BIC").into(),
            XmlEvent::characters(&self.creditor_bic).into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("ChrgBr").into(),
            XmlEvent::characters("SLEV").into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("CdtrSchmeId").into(),
            XmlEvent::start_element("Id").into(),
            XmlEvent::start_element("PrvtId").into(),
            XmlEvent::start_element("Othr").into(),
            XmlEvent::start_element("Id").into(),
            XmlEvent::characters(&self.creditor_id).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("SchmeNm").into(),
            XmlEvent::start_element("Prtry").into(),
            XmlEvent::characters("SEPA").into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
        ];

        xml.extend(self.debtors.iter().flat_map(|d| d.to_xml()));
        // xml.push(XmlEvent::end_element().into());
        xml.push(XmlEvent::end_element().into());
        xml
    }
}

impl From<super::PaymentInformation> for PaymentInformationString {
    fn from(value: super::PaymentInformation) -> Self {
        Self {
            payment_information_id: value.payment_information_id,
            creditor_name: value.creditor_name,
            creditor_iban: value.creditor_iban,
            creditor_bic: value.creditor_bic,
            collection_date: value.collection_date.to_string(),
            control_sum: value.control_sum.xml_string(),
            num_transactions: value.num_transactions.to_string(),
            creditor_id: value.creditor_id,
            debtors: value.debtors.into_iter().map(|d| d.into()).collect(),
        }
    }
}

pub struct DebtorString {
    /// AEGEEDELFT-random-hash
    pub(super) invoice_id: String,
    pub(super) amount: String,
    pub(super) name: String,
    pub(super) bic: String,
    pub(super) iban: String,
    /// Member code
    pub(super) mandate_id: String,
    /// Date membership started
    pub(super) mandate_date: String,
    pub(super) description: String,
}

impl ToXml for DebtorString {
    fn to_xml(&self) -> Vec<XmlEvent> {
        vec![
            XmlEvent::start_element("DrctDbtTxInf").into(),
            XmlEvent::start_element("PmtId").into(),
            XmlEvent::start_element("EndToEndId").into(),
            XmlEvent::characters(&self.invoice_id).into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("InstdAmt")
                .attr("Ccy", "EUR")
                .into(),
            XmlEvent::characters(&self.amount).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("DrctDbtTx").into(),
            XmlEvent::start_element("MndtRltdInf").into(),
            XmlEvent::start_element("MndtId").into(),
            XmlEvent::characters(&self.mandate_id).into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("DtOfSgntr").into(),
            XmlEvent::characters(&self.mandate_date).into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("DbtrAgt").into(),
            XmlEvent::start_element("FinInstnId").into(),
            XmlEvent::start_element("BIC").into(),
            XmlEvent::characters(&self.bic).into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("Dbtr").into(),
            XmlEvent::start_element("Nm").into(),
            XmlEvent::characters(&self.name).into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("DbtrAcct").into(),
            XmlEvent::start_element("Id").into(),
            XmlEvent::start_element("IBAN").into(),
            XmlEvent::characters(&self.iban).into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::start_element("RmtInf").into(),
            XmlEvent::start_element("Ustrd").into(),
            XmlEvent::characters(&self.description).into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
            XmlEvent::end_element().into(),
        ]
    }
}


impl From<super::Debtor> for DebtorString {
    fn from(value: super::Debtor) -> Self {
        Self {
            invoice_id: value.invoice_id,
            amount: value.amount.xml_string(),
            name: value.name,
            bic: value.bic,
            iban: value.iban,
            mandate_id: value.code.to_string(),
            mandate_date: value.membership_date.to_string(),
            description: value.description,
        }
    }
}