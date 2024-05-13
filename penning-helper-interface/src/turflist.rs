use std::ffi::OsStr;

use crate::file_receiver::{FileReceiverResult, FileReceiverSource};
use crate::popup::Popup;
use crate::rekening_selector::Selector;
use crate::{FooBar, Relations, ERROR_STUFF};
use eframe::egui::{self, Ui};
use egui::TextEdit;
use egui_extras::{Column, TableBuilder};
use penning_helper_conscribo::{AddChangeTransaction, ConscriboResult, TransactionResult};
use penning_helper_turflists::{matched_turflist::MatchedTurflist, turflist::TurfList};
use penning_helper_types::{Date, Euro};

#[derive(Clone, Debug, Default)]
pub struct TurflistImport {
    rekening: Selector<String>,
    description: String,
    reference: String,
    turflist: Option<TurfList>,
    matched: Option<MatchedTurflist>,
    price: Euro,
    last_len: usize,
}

impl TurflistImport {
    pub fn ui(&mut self, ui: &mut Ui, foobar: &mut FooBar, members: &Relations) {
        // TopBottomPanel::top(ui.next_auto_id()).show(ui.ctx(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Rekening");
            self.rekening.ui_convert(ui, foobar.accounts.iter(), |c| {
                foobar.accounts.find_by_name(c).map(|a| a.account_nr.clone())
            });
            if let Some(c) = self.rekening.get() {
                ui.label(format!("{}", c));
            } else {
                ui.label("No account selected");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Description");
            TextEdit::singleline(&mut self.description)
                .hint_text("Description")
                .show(ui);
        });
        ui.horizontal(|ui| {
            ui.label("Reference");
            TextEdit::singleline(&mut self.reference)
                .hint_text("T0011-00")
                .show(ui);
        });
        ui.horizontal(|ui| {
            if ui.button("Open Turflist").clicked() {
                self.turflist = None;
                foobar.files.new_receiver(FileReceiverSource::TurfList);
                self.price = Default::default();
            };
            if let Some(list) = foobar.files.get_receiver(FileReceiverSource::TurfList) {
                // println!("Receiver exists!");
                match list.get_file() {
                    FileReceiverResult::File(f) => {
                        ui.label(format!("File: {:?}", f));
                        if self.turflist.is_none() {
                            println!("Turflist was none");
                            let ext = f.extension().and_then(OsStr::to_str).unwrap_or("");
                            match ext {
                                "csv" => {
                                    ui.label("csv");
                                    match penning_helper_turflists::csv::read_csv(f) {
                                        Ok(mut l) => {
                                            l.shrink();
                                            self.turflist = Some(l);
                                            self.matched = None;
                                        }
                                        Err(e) => {
                                            if let Some(s) = ERROR_STUFF.get() {
                                                s.send(e.to_string()).unwrap();
                                            }
                                        }
                                    }
                                }
                                "xlsx" | "xls" => {
                                    if self.price == (0, 0).into() {
                                        let res =
                                            foobar.popups.entry("Price".to_string()).or_insert(
                                                Popup::new_default::<(String, f64)>("Price"),
                                            );
                                        if let Some(v) = res.value::<String>() {
                                            self.price = v.parse().unwrap_or((0, 0).into());
                                            foobar.popups.remove("Price");
                                        }
                                    } else {
                                        ui.label(format!("Price: {}", self.price));
                                        foobar.popups.remove("Price");

                                        match penning_helper_turflists::xlsx::read_excel(
                                            f, self.price,
                                        )
                                        .map(|mut l| {
                                            l.shrink();
                                            l
                                        })
                                        .map_err(|e| e.to_string())
                                        {
                                            Ok(mut o) => {
                                                o.shrink();
                                                self.turflist = Some(o);
                                                self.matched = None;
                                            }
                                            Err(e) => {
                                                if let Some(s) = ERROR_STUFF.get() {
                                                    s.send(e).unwrap();
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    ui.label("Invalid file type");
                                }
                            }
                            // if ext == "csv" {
                            //     ui.label("CSV");
                            // } else if ext == "xlsx" {
                            //     ui.label("Excel");

                            // } else {
                            //     ui.label("Invalid file type");
                            // }
                        }
                    }
                    FileReceiverResult::NoFile => {
                        ui.label("No file selected.");
                    }
                    FileReceiverResult::Waiting => {
                        ui.label("Waiting for file");
                    }
                };
            }
            if let Some(t) = &self.matched {
                if ui.button("Append to Conscribo").clicked() {
                    let transactions = t
                        .iter()
                        .flat_map(|m| m.idx().map(|idx| (&members[idx], m.amount)))
                        .map(|(r, eur)| {
                            let a =
                                AddChangeTransaction::new(Date::today(), self.description.clone());
                            let a = if eur > Euro::default() {
                                a.add_debet(
                                    self.rekening.get().unwrap().clone(),
                                    eur,
                                    self.reference.clone(),
                                    r.code,
                                )
                            } else {
                                a.add_credit(
                                    self.rekening.get().unwrap().clone(),
                                    eur,
                                    self.reference.clone(),
                                    r.code,
                                )
                            };
                            a
                        })
                        .collect::<Vec<_>>();
                    let res: Option<ConscriboResult<Vec<TransactionResult>>> =
                        foobar.conscribo.run(|c| c.do_multi_request(transactions));
                    if let Some(res) = res {
                        match res {
                            Ok(o) => {
                                let mut s = String::new();
                                for r in o {
                                    s.push_str(&format!("{:?}\n", r));
                                }
                                if let Some(se) = ERROR_STUFF.get() {
                                    se.send(s).unwrap();
                                }
                            }
                            Err(e) => {
                                if let Some(s) = ERROR_STUFF.get() {
                                    s.send(format!("Error: {}", e)).unwrap();
                                }
                            }
                        }
                    }
                }
                if ui.button("Save PDF").clicked() {
                    let pdf = penning_helper_pdf::generate_turflist_pdf(
                        self
                            .matched
                            .as_ref()
                            .unwrap()
                            .iter()
                            .map(|m| {
                                let name = match m.idx() {
                                    Some(idx) => members[idx].naam.as_str(),
                                    None => m.name.as_str(),
                                };
                                (name, m.row())
                            })
                            .collect::<Vec<_>>(),
                        &self.description,
                        &self.reference,
                    );
                    if let Err(e) = open::that_detached(pdf) {
                        if let Some(s) = ERROR_STUFF.get() {
                            s.send(format!("Error: {}", e)).unwrap();
                        }
                    }
                }
            } else {
                ui.add_enabled_ui(false, |ui| {
                    ui.button("Append to Conscribo").changed();
                    ui.button("Save PDF").clicked();
                });
            }
        });
        // });
        // TopBottomPanel::bottom(ui.next_auto_id()).show(ui.ctx(), |ui| {
        if self
            .matched
            .as_ref()
            .is_some_and(|l| l.iter().any(|r| r.idx().is_none()))
        {
            ui.label("The list contains IBANs (aka externals), you need to add these manually to conscribo!");
        } else {
            ui.label("");
        }

        if let Some(o) = &self.turflist {
            if self.last_len != members.len() || self.matched.is_none() {
                println!("{} != {}", self.last_len, members.len());
                let names = members.iter().map(|m| m.naam.clone()).collect::<Vec<_>>();
                let emails = members
                    .iter()
                    .map(|m| m.email_address.clone())
                    .collect::<Vec<_>>();
                let mut matches = o.get_matches(&names, &emails);
                matches.remove_zero_cost();
                self.matched = Some(matches);
                self.last_len = members.len();
            }
        }
        // });
        // CentralPanel::default().show(ui.ctx(), |ui| {
        TableBuilder::new(ui)
            .columns(Column::remainder().at_least(50.0), 5)
            .header(20.0, |mut r| {
                r.col(|ui| {
                    ui.strong("Name");
                });
                r.col(|ui| {
                    ui.strong("Original Name");
                });
                r.col(|ui| {
                    ui.strong("Email");
                });
                r.col(|ui| {
                    ui.strong("Amount");
                });
                r.col(|ui| {
                    ui.strong("Is Member or IBAN");
                });
            })
            .body(|mut b| {
                for row in self.matched.iter().flat_map(|l| l.iter()) {
                    let (name, email, amount, member) = if let Some(idx) = row.idx() {
                        let member = &members[idx];
                        (
                            member.naam.clone(),
                            if member.email_address.is_empty() {
                                row.row().email.clone().unwrap_or_else(|| String::new())
                            } else {
                                member.email_address.clone()
                            },
                            row.amount,
                            Some(member),
                        )
                    } else {
                        (
                            row.name.clone(),
                            row.email.clone().unwrap_or_else(|| String::new()),
                            row.amount,
                            None,
                        )
                    };

                    b.row(20.0, |mut r| {
                        r.col(|ui| {
                            let mut s = name.as_str();
                            let t = TextEdit::singleline(&mut s);
                            if member.is_none() {
                                t.text_color(ui.visuals().warn_fg_color)
                            } else {
                                t
                            }
                            .show(ui);
                        });
                        r.col(|ui| {
                            ui.label(&row.name);
                        });
                        r.col(|ui| {
                            let mut s = email.as_str();
                            ui.text_edit_singleline(&mut s);
                        });
                        r.col(|ui| {
                            ui.label(amount.to_string());
                        });
                        r.col(|ui| {
                            if member.is_some() {
                                ui.label("Member");
                            } else {
                                let mut iban = row.iban.as_ref().map(String::as_str).unwrap_or("");
                                ui.text_edit_singleline(&mut iban);
                            }
                        });
                    });
                }
            });
        // });
    }
}
