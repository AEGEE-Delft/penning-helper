#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use eframe::{
    egui::{self, Window},
    epaint::vec2,
};
use egui::Visuals;
use egui_extras::{Column, TableBuilder};
use penning_helper_turflists::turflist::TurfList;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    )
    .unwrap();
}

#[derive(Default)]
struct MyEguiApp {
    visuals: Visuals,
    loaded_turflist: Option<TurfList>,
    settings_shown: bool,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    self.visuals.light_dark_radio_buttons(ui);
                    ctx.set_visuals(self.visuals.clone());
                    if ui.button("Settings").clicked() {
                        self.settings_shown = true;
                        Window::new("Settings")
                            .open(&mut self.settings_shown)
                            .default_size(vec2(512.0, 512.0))
                            .show(ctx, |ui| {
                                ui.label("Settings");
                            });
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }

                    egui::warn_if_debug_build(ui);
                });
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Penning Helper");

            if ui
                .button("Load Turflist")
                .on_hover_text("Load a turflist from a CSV file")
                .clicked()
            {
                let path = rfd::FileDialog::new()
                    .add_filter("CSV", &["csv"])
                    .pick_file();
                if let Some(path) = path {
                    let list = std::fs::File::open(path).unwrap();
                    self.loaded_turflist = penning_helper_turflists::csv::read_csv(list)
                        .map(|mut l| {
                            l.shrink();
                            l
                        })
                        .map_err(|e| e.to_string())
                        .ok();
                }
            }

            if ui
                .button("Load Members portal list")
                .on_hover_text("Load an Excel turf list")
                .clicked()
            {
                let path = rfd::FileDialog::new()
                    .add_filter("Excel File", &["xlsx", "xls", "xlsm", "xlsb"])
                    .pick_file();
                if let Some(path) = path {
                    self.loaded_turflist =
                        penning_helper_turflists::xlsx::read_excel(path, (1, 0).into())
                            .map(|mut l| {
                                l.shrink();
                                l
                            })
                            .map_err(|e| e.to_string())
                            .ok();
                }
            }
            TableBuilder::new(ui)
                .striped(true)
                .columns(Column::remainder(), 4)
                .header(20.0, |mut r| {
                    r.col(|ui| {
                        ui.strong("Name");
                    });
                    r.col(|ui| {
                        ui.strong("Email");
                    });
                    r.col(|ui| {
                        ui.strong("Amount");
                    });
                    r.col(|ui| {
                        ui.strong("IBAN");
                    });
                })
                .body(|mut body| {
                    if let Some(t) = &self.loaded_turflist {
                        for row in t.iter() {
                            body.row(20.0, |mut r| {
                                r.col(|ui| {
                                    ui.label(&row.name);
                                });
                                r.col(|ui| {
                                    ui.label(*&row.email.as_ref().unwrap_or(&"".to_string()));
                                });
                                r.col(|ui| {
                                    ui.label(&row.amount.to_string());
                                });
                                r.col(|ui| {
                                    ui.label(*&row.iban.as_ref().unwrap_or(&"".to_string()));
                                });
                            });
                        }
                    }
                });

            // egui::ScrollArea::new([true, true]).show(ui, |ui| {
            //     egui::Grid::new("grid")
            //         .striped(true)
            //         .num_columns(4)
            //         .show(ui, |ui| {
            //             ui.label("Name");
            //             ui.label("Email");
            //             ui.label("Amount");
            //             ui.label("IBAN");
            //             ui.end_row();
            //             if let Some(t) = &self.loaded_turflist {
            //                 for row in t.iter() {
            //                     ui.label(&row.name);
            //                     ui.label(*&row.email.as_ref().unwrap_or(&"".to_string()));
            //                     ui.label(&row.amount.to_string());
            //                     ui.label(*&row.iban.as_ref().unwrap_or(&"".to_string()));
            //                     ui.end_row();
            //                 }
            //             }
            //         });
            // });
        });
    }
}
