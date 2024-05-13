use egui::TextEdit;

use egui_extras::{Column, TableBuilder};
use penning_helper_conscribo::AddChangeTransaction;
use penning_helper_pdf::{generate_turflist_pdf, SimpleTurfRow};
use penning_helper_types::{Date, Euro};

use crate::{rekening_selector::Selector, ERROR_STUFF};

#[derive(Debug, Default, Clone)]
struct Row {
    price_text: String,
    price: Option<Euro>,
    total_price_text: String,
    total_price: Option<Euro>,
    member_name_selector: Selector<u32>,
    rekening_selector: Selector<String>,
}

impl<'r> From<&'r Row> for SimpleTurfRow<'r> {
    fn from(value: &'r Row) -> Self {
        let name = value.member_name_selector.as_str();
        let what = {
            &value
                .rekening_selector
                .as_str()
                .split_once("(")
                .map(|(l, _)| l)
                .unwrap_or(value.rekening_selector.as_str())
        };
        let amount = value.total_price.unwrap_or_default();

        Self::new(name, what, amount)
    }
}

impl Row {
    fn all_optionals(&self) -> Option<(&str, u32, Euro, Euro)> {
        self.rekening_selector
            .get()
            .as_ref()
            .map(|f| f.as_str())
            .and_then(|r| {
                self.member_name_selector.get().as_ref().and_then(|m| {
                    self.price
                        .and_then(|p| self.total_price.map(|t| (r, *m, p, t)))
                })
            })
            .map(|(r, m, p, t)| (r, *m, p, t))
    }
}

#[derive(Debug, Default, Clone)]
pub struct MerchSales {
    rows: Vec<Row>,
    rekening: Selector<String>,
    reference: String,
    setup: bool,
}

impl MerchSales {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        foobar: &mut crate::FooBar,
        members: &crate::Relations,
    ) {
        if !self.setup {
            self.rows.push(Row::default());
            self.setup = true;
        }
        ui.horizontal(|ui| {
            ui.label("Rekening");
            self.rekening.ui_convert(ui, foobar.accounts.iter(), |r| {
                foobar
                    .accounts
                    .find_by_name(r)
                    .map(|r| r.account_nr.clone())
            });
            if let Some(r) = self.rekening.get() {
                ui.label(r);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Reference");
            TextEdit::singleline(&mut self.reference)
                .hint_text("T0011-00")
                .show(ui);
        });
        ui.horizontal(|ui| {
            if ui.button("Add to conscribo").clicked() {
                if self.rekening.get().is_none()
                    || self.reference.is_empty() | self.effectively_empty()
                {
                    if let Some(e) = ERROR_STUFF.get() {
                        e.send("Missing rekening or reference!".to_string())
                            .unwrap();
                    }
                } else {
                    let mut transactions = vec![];
                    for row in &self.rows {
                        if let Some((rekening_num, member_num, price, total_price)) =
                            row.all_optionals()
                        {
                            let rek = self.rekening.get().unwrap();
                            let desc = {
                                let rek = row
                                    .rekening_selector
                                    .as_str()
                                    .split('(')
                                    .collect::<Vec<_>>();
                                let len = if rek.len() > 1 {
                                    rek.len() - 1
                                } else {
                                    rek.len()
                                };
                                rek[..len].join("(")
                            };
                            let t = AddChangeTransaction::new(Date::today(), desc).add_merch(
                                member_num,
                                rekening_num.to_string(),
                                rek.clone(),
                                total_price,
                                price,
                                self.reference.clone(),
                            );
                            transactions.push(t);
                        } else {
                            println!("Missing data for row: {:?}", row);
                        }
                    }
                    if let Some(res) = foobar.conscribo.run(|c| c.do_multi_request(transactions)) {
                        match res {
                            Ok(v) => {
                                let s = format!("Added {} transactions", v.len());
                                if let Some(se) = ERROR_STUFF.get() {
                                    se.send(s).unwrap();
                                }
                            }
                            Err(e) => {
                                let s = format!("Error: {:?}", e);
                                if let Some(se) = ERROR_STUFF.get() {
                                    se.send(s).unwrap();
                                }
                            }
                        }
                    }
                }
            }
            if ui.button("Save PDF").clicked() {
                if self.rekening.get().is_none()
                    || self.reference.is_empty() | self.effectively_empty()
                {
                    if let Some(e) = ERROR_STUFF.get() {
                        e.send("Missing rekening or reference!".to_string())
                            .unwrap();
                    }
                } else {
                    let mut rows = self
                        .rows
                        .iter()
                        .map(|r| r.into())
                        .collect::<Vec<SimpleTurfRow>>();
                    rows.sort();
                    let res = generate_turflist_pdf(rows, "Merch", &self.reference);
                    open::that_detached(res).unwrap();
                }
            }
        });

        TableBuilder::new(ui)
            .auto_shrink([true, false])
            .striped(true)
            .columns(Column::remainder(), 10)
            .header(20.0, |mut r| {
                r.col(|ui| {
                    ui.label("Name");
                });
                r.col(|ui| {
                    ui.label("id");
                });
                r.col(|ui| {
                    ui.label("Rekening");
                });
                r.col(|ui| {
                    ui.label("rekening id");
                });
                r.col(|ui| {
                    ui.label("Kostprijs");
                });
                r.col(|ui| {
                    ui.label("Kostprijs");
                });
                r.col(|ui| {
                    ui.label("Verkoopprijs");
                });
                r.col(|ui| {
                    ui.label("Verkoopprijs");
                });
                r.col(|ui| {
                    ui.label("Winst");
                });
                r.col(|ui| {
                    ui.label("Delete");
                });
            })
            .body(|b| {
                b.rows(20.0, self.rows.len() + 1, |mut r| {
                    let idx = r.index();
                    if idx >= self.rows.len() {
                        r.col(|ui| {
                            let button = ui.button("Add new row");
                            if button.clicked() {
                                self.rows.push(Row::default());
                            }
                            ui.memory_mut(|m| m.interested_in_focus(button.id));
                        });
                        for _ in 0..10 {
                            r.col(|ui| {
                                ui.label("");
                            });
                        }
                    } else {
                        let row = &mut self.rows[idx];
                        r.col(|ui| {
                            row.member_name_selector.ui_convert(
                                ui,
                                members.members.iter().map(|m| m.naam.as_str()),
                                |m| members.find_member_by_name(m).map(|m| m.code),
                            );
                        });

                        r.col(|ui| {
                            if let Some(code) = row.member_name_selector.get() {
                                ui.label(format!("{}", code));
                            } else {
                                ui.label("-");
                            }
                        });

                        r.col(|ui| {
                            row.rekening_selector
                                .ui_convert(ui, foobar.accounts.iter(), |r| {
                                    foobar
                                        .accounts
                                        .find_by_name(r)
                                        .map(|r| r.account_nr.clone())
                                });
                        });

                        r.col(|ui| {
                            if let Some(account_nr) = row.rekening_selector.get() {
                                ui.label(format!("{}", account_nr));
                                if row.price_text.is_empty() {
                                    if let Some((_, r)) =
                                        row.rekening_selector.as_str().split_once("(")
                                    {
                                        row.price_text = r.replace(")", "").replace(",", ".");
                                    }
                                }
                            } else {
                                ui.label("-");
                            }
                        });

                        r.col(|ui| {
                            let color = if row.price_text.is_empty() {
                                row.price = None;
                                None
                            } else {
                                if let Ok(c) = row.price_text.parse::<Euro>() {
                                    row.price = Some(c);
                                    Some(egui::Color32::GREEN)
                                } else {
                                    row.price = None;
                                    Some(egui::Color32::RED)
                                }
                            };
                            let res = ui.add(
                                TextEdit::singleline(&mut row.price_text)
                                    .hint_text("Price")
                                    .text_color_opt(color),
                            );
                            ui.memory_mut(|m| m.interested_in_focus(res.id));
                        });

                        r.col(|ui| {
                            if let Some(price) = row.price {
                                ui.label(format!("{}", price));
                            } else {
                                ui.label("-");
                            }
                        });

                        r.col(|ui| {
                            let color = if row.total_price_text.is_empty() {
                                row.total_price = None;
                                None
                            } else {
                                if let Ok(c) = row.total_price_text.parse::<Euro>() {
                                    row.total_price = Some(c);
                                    Some(egui::Color32::GREEN)
                                } else {
                                    row.total_price = None;
                                    Some(egui::Color32::RED)
                                }
                            };
                            let res = ui.add(
                                TextEdit::singleline(&mut row.total_price_text)
                                    .hint_text("Price")
                                    .text_color_opt(color),
                            );
                            ui.memory_mut(|m| m.interested_in_focus(res.id));
                        });

                        r.col(|ui| {
                            if let Some(price) = row.total_price {
                                ui.label(format!("{}", price));
                            } else {
                                ui.label("-");
                            }
                        });
                        r.col(|ui| {
                            if let Some((price, total_price)) = row
                                .price
                                .as_ref()
                                .and_then(|p| row.total_price.as_ref().map(|t| (*p, *t)))
                            {
                                ui.label(format!("{}", total_price - price));
                            } else {
                                ui.label("-");
                            }
                        });
                        r.col(|ui| {
                            if ui.button("Delete").clicked() {
                                self.rows.remove(idx);
                            }
                        });
                    }
                });
            });
    }

    fn effectively_empty(&self) -> bool {
        self.rows.iter().all(|r| {
            r.rekening_selector.as_str().is_empty()
                && r.member_name_selector.as_str().is_empty()
                && r.price_text.is_empty()
                && r.rekening_selector.get().is_none()
                && r.member_name_selector.get().is_none()
                && r.price.is_none()
        })
    }
}
