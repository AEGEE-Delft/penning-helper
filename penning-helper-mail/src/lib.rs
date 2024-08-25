use askama::Template;
use lettre::{
    message::{Attachment, Mailbox, MultiPart, SinglePart},
    transport::smtp::{authentication::Mechanism, PoolConfig},
    Message, SmtpTransport, Transport,
};
use penning_helper_types::{Date, Euro};

#[derive(Debug, Template)]
#[template(path = "email.html")]
struct EmailTemplate<'a> {
    name: &'a str,
    amount: Euro,
    date: Date,
    no_details: bool,
    company_name: &'a str,
    company_iban: &'a str,
    board_line: &'a str,
    treasurer: &'a str,
}
impl<'a> EmailTemplate<'a> {
    fn new(
        name: &'a str,
        amount: Euro,
        date: Date,
        no_details: bool,
        company_name: &'a str,
        company_iban: &'a str,
        board_line: &'a str,
        treasurer: &'a str,
    ) -> Self {
        Self {
            name,
            amount,
            date,
            no_details,
            company_name,
            company_iban,
            board_line,
            treasurer,
        }
    }
}

mod filters {
    use penning_helper_types::Euro;

    pub fn abs_euro(e: &Euro) -> ::askama::Result<String> {
        Ok(format!("{:-}", e))
    }

    pub fn too_much_result(e: &Euro) -> ::askama::Result<String> {
        Ok(format!("{:-}", *e - 100.0))
    }

    pub fn owes_or_not(e: &Euro) -> ::askama::Result<bool> {
        let e = *e;
        Ok(e > Euro::default())
    }

    pub fn too_large(e: &Euro) -> ::askama::Result<bool> {
        let e = *e;
        Ok(e > Euro::new(100, 0))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error("TransportError: {0}")]
    TransportError(#[from] lettre::transport::smtp::Error),
    #[error("MailContentError: {0}")]
    MailContentError(#[from] lettre::error::Error),
}

#[derive(Clone)]
pub struct MailServer {
    sender: SmtpTransport,
    from: Mailbox,
    reply_to: Mailbox,
    iban: String,
    name: String,
}

impl std::fmt::Debug for MailServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailServer")
            .field("from", &self.from)
            .field("reply_to", &self.reply_to)
            .field("iban", &self.iban)
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl MailServer {
    pub fn new(
        config: &penning_helper_config::MailConfig,
        sepa: &penning_helper_config::SEPAConfig,
    ) -> Result<Self, MailError> {
        let sender = SmtpTransport::starttls_relay(&config.smtp_server)?
            .port(config.smtp_port)
            .authentication(vec![Mechanism::Plain])
            .pool_config(PoolConfig::new().max_size(20))
            .credentials(config.credentials.as_pair().into());

        Ok(Self {
            sender: sender.build(),
            from: config.from.as_pair().try_into().unwrap(),
            reply_to: config.reply_to.as_pair().try_into().unwrap(),
            iban: sepa.company_iban.clone(),
            name: sepa.company_name.clone(),
        })
    }

    pub fn send_mail(
        &self,
        name: &str,
        email: &str,
        pdf_file: Vec<u8>,
        amount: Euro,
        date: Date,
        no_details: bool,
        board: &str,
        treasurer: &str,
    ) -> Result<(), MailError> {
        let mail_content =
            EmailTemplate::new(&name, amount, date, no_details, &self.name, &self.iban, board, treasurer)
                .render()
                .unwrap();
        let email = Message::builder()
            .from(self.from.clone())
            .reply_to(self.reply_to.clone())
            .to((name, email).try_into().unwrap())
            .subject(format!("AEGEE-Delft invoice {}", date))
            .multipart(
                MultiPart::mixed()
                    .multipart(MultiPart::related().singlepart(SinglePart::html(mail_content)))
                    .singlepart(Attachment::new_inline("logo".to_string()).body(
                        include_bytes!("../logo.png").to_vec(),
                        "image/png".parse().unwrap(),
                    ))
                    .singlepart(
                        Attachment::new("invoice.pdf".to_string())
                            .body(pdf_file, "application/pdf".parse().unwrap()),
                    ),
            )?;
        self.sender.send(&email)?;
        Ok(())
    }
}
