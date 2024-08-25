use std::{
    cmp::min,
    fs::File,
    io::Write,
    ops::{Add, Deref, DerefMut},
    time::{Duration, Instant},
};

use crate::popup::Popup;
use crate::{
    file_receiver::{FileReceiverResult, FileReceiverSource},
    FooBar, Relations, ERROR_STUFF,
};
use chrono::{Days, Local, NaiveDate};
use eframe::egui::{self, Ui};
use egui::RichText;
use egui_extras::{Column, TableBuilder};

use penning_helper_conscribo::{transactions::UnifiedTransaction, GetTransactionResult};
use penning_helper_mail::MailServer;
use penning_helper_types::{Date, Euro};
use rand::Rng;

#[derive(Clone, Debug)]
struct RelationTransaction {
    t: Vec<UnifiedTransaction>,
    name: String,
    code: String,
    membership_date: NaiveDate,
    iban: String,
    bic: String,
    email: String,
    membership: bool,
    alumni_contributie: Euro,
}

impl RelationTransaction {
    pub fn total_cost(&self) -> Euro {
        self.t.iter().map(|t| t.cost).sum()
    }

    pub fn all_after(&self, date: Date) -> impl Iterator<Item = &UnifiedTransaction> {
        self.t.iter().filter(move |t| t.date >= date)
    }

    pub fn previous_invoices_left(&self, date: Date) -> Euro {
        self.t
            .iter()
            .filter(|t| t.date < date)
            .map(|t| t.cost)
            .sum()
    }

    fn is_valid(&self) -> bool {
        !self.iban.is_empty() && !self.bic.is_empty() && !self.email.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum SendMode {
    #[default]
    Test,
    Real,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Show {
    #[default]
    All,
    OwesUsALot,
    IsOwedByUs,
    Contributie,
    AlumniContributie,
}

impl Show {
    fn filter(&self, t: &RelationTransaction) -> bool {
        match self {
            Show::All => true,
            Show::OwesUsALot => t.total_cost() > Euro::from(100),
            Show::IsOwedByUs => t.total_cost() < Euro::from(-10),
            Show::Contributie => t.total_cost() > Euro::from(0) && t.membership,
            Show::AlumniContributie => t.alumni_contributie > Euro::from(0),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SepaGen {
    transactions: Vec<RelationTransaction>,
    unifieds: Vec<UnifiedTransaction>,
    unifieds_grabbed: bool,
    sorted: bool,
    idx: usize,
    done: bool,
    send_mode: SendMode,
    to_send: Vec<RelationTransaction>,
    last_send: TimeThing,
    email_client: Option<MailServer>,
    has_tried_mail: bool,
    last_invoice_date: Date,
    show: Show,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct TimeThing(Instant);

impl TimeThing {
    pub fn now() -> Self {
        Self(Instant::now())
    }
}

impl Default for TimeThing {
    fn default() -> Self {
        Self(
            Instant::now()
                .checked_sub(Duration::from_secs(100 * 60))
                .unwrap_or_else(|| Instant::now()),
        )
    }
}

impl Deref for TimeThing {
    type Target = Instant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add<Duration> for TimeThing {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl PartialEq<Instant> for TimeThing {
    fn eq(&self, other: &Instant) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<Instant> for TimeThing {
    fn partial_cmp(&self, other: &Instant) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl SepaGen {
    pub fn ui(&mut self, ui: &mut Ui, foobar: &mut FooBar, members: &Relations) {
        if !self.has_tried_mail {
            self.has_tried_mail = true;
            self.email_client = MailServer::new(foobar.cfg.mail(), foobar.cfg.sepa()).ok();
        }
        let done = if !self.unifieds_grabbed {
            ui.label(format!("Getting transactions{}", ".".repeat(self.idx / 50)));
            // ui.label("This will take a while the first time");
            self.idx += 1;
            self.idx %= 200;

            let r = foobar
                .conscribo
                .run(|c| c.get_transactions_faster())
                .transpose();
            match r {
                Ok(r) => {
                    if let Some(r) = r {
                        match r {
                            GetTransactionResult::Done(r) => {
                                self.unifieds_grabbed = true;
                                self.unifieds = r;
                            }
                            GetTransactionResult::NotDone {
                                total,
                                count,
                                from_cache,
                            } => {
                                ui.label(format!(
                                    "Got {} out of {} transactions, with {} from cache",
                                    count, total, from_cache
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(s) = ERROR_STUFF.get() {
                        s.send(format!("Error: {}", e)).unwrap();
                    }
                }
            }
            ui.ctx().request_repaint();

            false
        } else if !self.unifieds.is_empty() {
            ui.label("Calculating, numbers will change for a bit");
            ui.ctx().request_repaint();
            for _ in 0..min(self.unifieds.len(), 1000) {
                let t = self.unifieds.remove(0);

                let Some(rel) = members.find_member(&t.code) else {
                    println!("No relation found for {}", t.code);
                    continue;
                };
                if rel.geen_invoice == 1 {
                    continue;
                }
                let name = rel.display_name.as_str();
                let iban = rel
                    .account
                    .as_ref()
                    .map(|r| r.iban.as_str())
                    .unwrap_or_default();
                let bic = rel
                    .account
                    .as_ref()
                    .map(|r| r.bic.as_str())
                    .unwrap_or_default();
                let email = rel.email.as_str();
                let membership_date = rel
                    .lidmaatschap_gestart
                    .unwrap_or_else(|| Local::now().date_naive());
                let code = &rel.code;
                // let membership = rel.source == "lid";

                let alumni_contributie = if let Some((start, eind, amount)) =
                    rel.alumni_lidmaatschap_gestart.and_then(|s| {
                        let e = rel.alumni_lidmaatschap_be_indigd.unwrap_or_else(|| {
                            Local::now()
                                .date_naive()
                                .checked_add_days(Days::new(1024))
                                .unwrap()
                        });
                        Some((s, e, rel.alumni_contributie))
                    }) {
                    if start <= Local::now().date_naive() && eind >= Local::now().date_naive() {
                        amount
                    } else {
                        Euro::from(0)
                    }
                } else {
                    Euro::from(0)
                };

                if let Some(r) = self
                    .transactions
                    .iter_mut()
                    .find(|t| t.name == name && t.iban == iban && t.email == email)
                {
                    r.t.push(t)
                } else {
                    self.transactions.push(RelationTransaction {
                        t: vec![t],
                        name: name.to_string(),
                        iban: iban.to_string(),
                        bic: bic.to_string(),
                        email: email.to_string(),
                        code: code.to_string(),
                        membership_date,
                        membership: true,
                        alumni_contributie,
                    });
                }
            }
            false
        } else if !self.sorted {
            let mut transactions = vec![];

            for t in self.transactions.clone() {
                if t.total_cost() == Euro::default() {
                    continue;
                }
                if t.email.is_empty() {
                    println!("No email for {}", t.name);
                    continue;
                }

                transactions.push(t);
            }

            transactions.sort_by_cached_key(|t| t.name.clone());

            self.transactions = transactions;
            self.sorted = true;
            false
        } else {
            ui.label("Done calculating");
            true
        };
        ui.add_enabled_ui(done, |ui| {
            if ui.button("Create SEPA file").clicked() {
                foobar.files.new_receiver(FileReceiverSource::SepaSaveLoc);
                self.done = false;
            }
            if let Some(r) = foobar.files.get_receiver(FileReceiverSource::SepaSaveLoc) {
                match r.get_file() {
                    FileReceiverResult::File(f) => {
                        ui.label(format!("File: {:?}", f));
                        if !self.done {
                            let mut creditors = vec![];
                            let mut debtors = vec![];
                            for t in self
                                .transactions
                                .iter()
                                .filter(|t| t.total_cost() != Euro::default())
                                .filter(|t| t.is_valid())
                                .filter(|t| self.show.filter(t))
                            {
                                let total = t.total_cost();
                                if total < Euro::default() {
                                    // it's a creditor
                                    let c = foobar.sepa.new_creditor(
                                        -total,
                                        t.name.clone(),
                                        t.bic.clone(),
                                        t.iban.clone(),
                                        "Payment of positive balance".to_string(),
                                    );
                                    creditors.push(c);
                                } else if total > Euro::default() {
                                    // it's a debtor
                                    let d = if self.show == Show::AlumniContributie {
                                        foobar.sepa.new_debtor(
                                            t.alumni_contributie,
                                            t.name.clone(),
                                            t.bic.clone(),
                                            t.iban.clone(),
                                            t.code.clone(),
                                            t.membership_date.into(),
                                            "Alumni Contributie".to_string(),
                                        )
                                    } else if self.show == Show::Contributie {
                                        foobar.sepa.new_debtor(
                                            total.clamp(Euro::from(0), Euro::from(50)),
                                            t.name.clone(),
                                            t.bic.clone(),
                                            t.iban.clone(),
                                            t.code.clone(),
                                            t.membership_date.into(),
                                            "Contributie".to_string(),
                                        )
                                    } else if total >= 100.into() {
                                        let total = 100.into();
                                        foobar.sepa.new_debtor(
                                            total,
                                            t.name.clone(),
                                            t.bic.clone(),
                                            t.iban.clone(),
                                            t.code.clone(),
                                            t.membership_date.into(),
                                            "Partial invoice of open AEGEE-Delft balance"
                                                .to_string(),
                                        )
                                    } else {
                                        foobar.sepa.new_debtor(
                                            total,
                                            t.name.clone(),
                                            t.bic.clone(),
                                            t.iban.clone(),
                                            t.code.clone(),
                                            t.membership_date.into(),
                                            "Invoice of open AEGEE-Delft balance".to_string(),
                                        )
                                    };
                                    debtors.push(d);
                                } else {
                                    // nothing
                                }
                            }
                            let debtors = foobar
                                .sepa
                                .new_invoice_payment_information(Date::in_some_days(2), debtors);
                            let creditors = foobar
                                .sepa
                                .new_transfer_payment_information(Date::in_some_days(2), creditors);
                            let debtors = foobar.sepa.new_invoice_document(debtors);
                            let creditors = foobar.sepa.new_transfer_document(creditors);

                            let mut debtors_file = f.to_path_buf();
                            debtors_file.set_extension("invoice.xml");
                            let debtors_file = File::create(debtors_file).unwrap();
                            debtors.write(debtors_file).unwrap();

                            let mut creditors_file = f.to_path_buf();
                            creditors_file.set_extension("transfer.xml");
                            let creditors_file = File::create(creditors_file).unwrap();
                            creditors.write(creditors_file).unwrap();
                            self.done = true;
                        }
                    }
                    FileReceiverResult::NoFile => {
                        ui.label("No file selected.");
                    }
                    FileReceiverResult::Waiting => {
                        ui.label("Waiting for file");
                    }
                }
            }
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.send_mode, SendMode::Test, "Test")
                    .on_hover_text("Send the emails to the test email address");
                if ui
                    .radio_value(&mut self.send_mode, SendMode::Real, "Real")
                    .on_hover_text("Send the emails to the real email addresses")
                    .clicked()
                    && !matches!(self.send_mode, SendMode::Real)
                {
                    foobar.popups.insert(
                        "SEPAShouldChange".to_string(),
                        Popup::new_default::<bool>(
                            "Are you sure you want to send the mail to the real email addresses?",
                        ),
                    );
                    self.send_mode = SendMode::Test;
                }
                ui.add(egui_extras::DatePickerButton::new(
                    self.last_invoice_date.deref_mut(),
                ));
                if let Some(res) = foobar.popups.get("SEPAShouldChange") {
                    if let Some(v) = res.value::<bool>() {
                        if *v {
                            self.send_mode = SendMode::Real;
                        } else {
                            self.send_mode = SendMode::Test;
                        }
                        foobar.popups.remove("SEPAShouldChange");
                    }
                }
                if ui.button("Send Emails").clicked() {
                    match self.send_mode {
                        SendMode::Test => {
                            self.to_send = self
                                .transactions
                                .iter()
                                .filter(|t| t.total_cost() != Euro::default())
                                .take(1)
                                .cloned()
                                .filter(|t| self.show.filter(t))
                                .collect();
                        }
                        SendMode::Real => {
                            self.to_send = self
                                .transactions
                                .iter()
                                .filter(|t| t.total_cost() != Euro::default())
                                .cloned()
                                .filter(|t| self.show.filter(t))
                                .collect()
                        }
                    }
                }

                ui.collapsing("Filter", |ui| {
                    ui.vertical(|ui| {
                        ui.radio_value(&mut self.show, Show::All, "All");
                        ui.radio_value(
                            &mut self.show,
                            Show::OwesUsALot,
                            "Owes us more than 100 euros",
                        );
                        ui.radio_value(
                            &mut self.show,
                            Show::IsOwedByUs,
                            "We owe them more than 10 euros",
                        );
                        ui.radio_value(
                            &mut self.show,
                            Show::Contributie,
                            "Contributie, meer dan 0 euro, maar max 50 euro geind",
                        );
                        ui.radio_value(
                            &mut self.show,
                            Show::AlumniContributie,
                            "Alumni Contributie (houdt rekening met start en eind datum)",
                        );
                    })
                });
                if let Some(mail_client) = &self.email_client {
                    if !self.to_send.is_empty()
                        && (self.last_send + Duration::from_secs(5 * 60)) <= Instant::now()
                    {
                        self.last_send = TimeThing::now();
                        println!("Sending emails");
                        let today = Date::today();
                        for r in self.to_send.drain(0..(20.min(self.to_send.len()))) {
                            if r.email.is_empty() {
                                println!("No email for {}", r.name);
                                continue;
                            }
                            let total = r.total_cost();

                            let pdf = Self::get_pdf(self.last_invoice_date, &r);

                            let email_address = if matches!(self.send_mode, SendMode::Test) {
                                foobar.cfg.mail().reply_to.address.as_str()
                            } else {
                                r.email.as_str()
                            };
                            println!(
                                "Sending email for {} to {}, with total {}",
                                r.name, email_address, total
                            );
                            if let Err(e) = mail_client.send_mail(
                                &r.name,
                                email_address,
                                pdf,
                                total,
                                today,
                                r.iban.is_empty() || r.bic.is_empty(),
                                &foobar.cfg.mail().board_line,
                                &foobar.cfg.mail().name,
                            ) {
                                if let Some(s) = ERROR_STUFF.get() {
                                    s.send(format!("Error sending mail: {}", e)).unwrap();
                                }
                            }
                        }
                        ui.ctx().request_repaint_after(Duration::from_secs(1));
                        self.last_send = TimeThing::now();
                    } else {
                        if matches!(self.send_mode, SendMode::Test) {
                            self.last_send.0 -= Duration::from_secs(5 * 60);
                        }
                        ui.label(format!(
                            "Waiting, {} emails remaining, {} time remaning",
                            self.to_send.len(),
                            (self.last_send + Duration::from_secs(5 * 60))
                                .duration_since(Instant::now())
                                .as_secs()
                        ));
                        ui.ctx().request_repaint_after(Duration::from_secs(1));
                    }
                } else {
                    ui.label("Mail machine broke :(");
                }
            });
        });

        TableBuilder::new(ui)
            .columns(Column::remainder(), 3)
            .header(20.0, |mut r| {
                r.col(|ui| {
                    ui.label("Name");
                });
                r.col(|ui| {
                    ui.label("Amount");
                });
                r.col(|ui| {
                    ui.label("Get PDF");
                });
            })
            .body(|mut b| {
                for t in self
                    .transactions
                    .iter()
                    .filter(|t| t.total_cost() != Euro::default())
                    .filter(|t| self.show.filter(t))
                {
                    let amount = t.total_cost();
                    b.row(20.0, |mut r| {
                        r.col(|ui| {
                            let text = RichText::new(&t.name);
                            let text = if amount > Euro::from(100) {
                                text.color(ui.visuals().warn_fg_color)
                            } else if !t.is_valid() {
                                text.color(ui.visuals().error_fg_color)
                            } else {
                                text
                            };
                            ui.label(text);
                        });
                        r.col(|ui| {
                            let text = RichText::new(amount.to_string());
                            let text = if amount > Euro::from(100) {
                                text.color(ui.visuals().warn_fg_color)
                            } else if !t.is_valid() {
                                text.color(ui.visuals().error_fg_color)
                            } else {
                                text
                            };
                            ui.label(text);
                        });
                        r.col(|ui| {
                            if ui.button("Open PDF").clicked() {
                                let pdf = Self::get_pdf(self.last_invoice_date, t);
                                let mut temp_file = std::env::temp_dir();
                                let mut rng = rand::thread_rng();
                                let random_name: String = std::iter::repeat(())
                                    .map(|()| rng.sample(rand::distributions::Alphanumeric) as char)
                                    .take(10)
                                    .collect();
                                temp_file.push(random_name);
                                temp_file.set_extension("pdf");
                                let mut f = File::create(&temp_file).unwrap();
                                f.write_all(&pdf).unwrap();
                                open::that_detached(temp_file).unwrap();
                            }
                        });
                    });
                }
            });
    }

    fn get_pdf(last_invoice_date: Date, r: &RelationTransaction) -> Vec<u8> {
        let previous = r.previous_invoices_left(last_invoice_date);
        let t = UnifiedTransaction::create_new_mock(
            last_invoice_date,
            "Open costs of previous invoice".to_string(),
            previous,
        );
        let to_show = r.all_after(last_invoice_date);
        let t = std::iter::once(&t)
            .chain(to_show)
            .map(|t| penning_helper_pdf::SimpleTransaction::new(t.cost, &t.description, t.date))
            .collect::<Vec<_>>();
        penning_helper_pdf::create_invoice_pdf(t, &r.name)
    }
}
