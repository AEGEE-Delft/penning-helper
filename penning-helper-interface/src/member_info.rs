use egui::Ui;
use egui_extras::{Column, TableBuilder};

use crate::{FooBar, Relations};

#[derive(Clone, Debug, Default)]
pub struct MemberInfo {
    search: String,
}

impl MemberInfo {
    pub fn ui(&mut self, ui: &mut Ui, _foobar: &FooBar, members: &Relations) {
        ui.horizontal(|ui| {
            ui.label("filter:");
            ui.text_edit_singleline(&mut self.search);
        });
        TableBuilder::new(ui)
            .column(Column::auto().at_least(50.0))
            .column(Column::remainder())
            .column(Column::remainder())
            .column(Column::remainder())
            .column(Column::auto().at_least(50.0))
            .header(20.0, |mut r| {
                r.col(|ui| {
                    ui.label("id");
                });
                r.col(|ui| {
                    ui.label("Name");
                });

                r.col(|ui| {
                    ui.label("Email");
                });
                r.col(|ui| {
                    ui.label("IBAN");
                });

                r.col(|ui| {
                    ui.label("Source");
                });
            })
            .body(|mut b| {
                for member in members.iter().filter(|m| {
                    self.search.is_empty()
                        || m.naam.to_lowercase().contains(&self.search.to_lowercase())
                }) {
                    b.row(20.0, |mut r| {
                        r.col(|ui| {
                            ui.label(&member.code.to_string());
                        });
                        r.col(|ui| {
                            ui.label(&member.naam);
                        });
                        r.col(|ui| {
                            ui.label(&member.email_address);
                        });
                        r.col(|ui| {
                            ui.label(
                                &member
                                    .rekening
                                    .as_ref()
                                    .map(|a| a.iban.clone())
                                    .unwrap_or("".to_string()),
                            );
                        });
                        r.col(|ui| {
                            ui.label(member.source);
                        });
                    });
                }
            });
    }
}
