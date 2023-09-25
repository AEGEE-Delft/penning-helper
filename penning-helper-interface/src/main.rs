#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::io::{self, Write};

// hide console window on Windows in release
use eframe::{
    egui::{self, Slider, TextBuffer, TextEdit, Ui, Window},
    emath::Align2,
    epaint::vec2,
    Storage,
};
use egui::{Color32, ColorImage, Label, Vec2, Visuals};
use egui_dock::{NodeIndex, Style, Tree};
use egui_extras::{Column, TableBuilder};
use penning_helper_config::Config;
use penning_helper_conscribo::Relation;
use penning_helper_turflists::turflist::TurfList;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Penning Helper",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    )
    .unwrap();
}

#[derive(Default)]
struct MyEguiApp {
    visuals: Visuals,
    tabs: MyTabs,
    settings_window: SettingsWindow,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        Self::with_config(Config::load_from_file())
    }

    fn with_config(config: Config) -> Self {
        Self {
            settings_window: SettingsWindow {
                config,
                ..Default::default()
            },
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
struct SettingsWindow {
    open: bool,
    config: Config,
}

impl SettingsWindow {
    pub fn show(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut open = self.open;
        Window::new("Settings")
            .open(&mut open)
            .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
            .default_size(vec2(512.0, 512.0))
            .scroll2([true, false])
            .show(ctx, |ui| self.actual_show(ui, frame));
        if open && !self.open {
            self.open = false;
        } else if !open && self.open {
            self.open = false;
        }
    }

    fn actual_show(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        ui.heading("Current Year");
        labelled_row(
            ui,
            "Current year:",
            self.config.year_format_mut(),
            Some("2324"),
        );
        ui.heading("SEPA");
        labelled_row(
            ui,
            "IBAN",
            &mut self.config.sepa_mut().company_iban,
            Some("NL12ABCD0123456789"),
        );
        labelled_row(
            ui,
            "BIC",
            &mut self.config.sepa_mut().company_bic,
            Some("ABCDNL2A"),
        );
        labelled_row(
            ui,
            "Name",
            &mut self.config.sepa_mut().company_name,
            Some("AEGEE-Delft"),
        );
        labelled_row(
            ui,
            "Incasso ID",
            &mut self.config.sepa_mut().company_id,
            Some("NL00ZZZ404840000000"),
        );
        ui.heading("Conscribo");
        labelled_row(
            ui,
            "Username",
            &mut self.config.conscribo_mut().username,
            Some("admin"),
        );
        ui.vertical(|ui| {
            ui.label("Password");
            ui.add(
                TextEdit::singleline(&mut self.config.conscribo_mut().password)
                    .hint_text("hunter2")
                    .password(true),
            );
        });
        labelled_row(
            ui,
            "URL",
            &mut self.config.conscribo_mut().url,
            Some("https://secure.conscribo.nl/aegee-delft/request.json"),
        );
        ui.heading("Mail");
        labelled_row(
            ui,
            "SMTP Server",
            &mut self.config.mail_mut().smtp_server,
            Some("smtp.gmail.com"),
        );
        let t = self.config.mail_mut().smtp_port;
        let mut s = if t == 0 {
            "".to_string()
        } else {
            t.to_string()
        };
        ui.vertical(|ui| {
            ui.label("SMTP Port");
            ui.add(TextEdit::singleline(&mut s).char_limit(5).hint_text("587"));
        });
        self.config.mail_mut().smtp_port = if s.is_empty() {
            0
        } else {
            s.parse().unwrap_or(t)
        };

        labelled_row(
            ui,
            "SMTP Username",
            &mut self.config.mail_mut().credentials.username,
            Some("testkees@me.org"),
        );
        ui.vertical(|ui| {
            ui.label("SMTP Password");
            ui.add(
                TextEdit::singleline(&mut self.config.mail_mut().credentials.password)
                    .hint_text("hunter2")
                    .password(true),
            );
        });
        // labelled_row(ui, "From", &mut self.config.mail_mut().from);
        // labelled_row(ui, "To", &mut self.config.mail_mut().to);
        // labelled_row(ui, "Subject", &mut self.config.mail_mut().subject);
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                self.config.save_to_file();
                self.open = false;
            }
            if ui.button("Cancel").clicked() {
                self.open = false;
            }
        });
    }
}

fn labelled_row(ui: &mut Ui, name: &str, line: &mut String, hint: Option<&'static str>) {
    ui.vertical(|ui| {
        ui.label(name);
        TextEdit::singleline(line)
            .hint_text(hint.unwrap_or(""))
            .show(ui);
    });
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.settings_window.show(ctx, frame);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    self.visuals.light_dark_radio_buttons(ui);
                    ctx.set_visuals(self.visuals.clone());
                    if ui.button("Settings").clicked() {
                        self.settings_window.open = true;
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }

                    egui::warn_if_debug_build(ui);
                });
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.settings_window.open, |ui| self.tabs.ui(ui))
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.colored_label(Color32::LIGHT_GRAY, "Made by Julius de Jeu");
            });
        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.heading("Penning Helper");

        //     if ui
        //         .button("Load Turflist")
        //         .on_hover_text("Load a turflist from a CSV file")
        //         .clicked()
        //     {
        //         let path = rfd::FileDialog::new()
        //             .add_filter("CSV", &["csv"])
        //             .pick_file();
        //         if let Some(path) = path {
        //             let list = std::fs::File::open(path).unwrap();
        //             self.loaded_turflist = penning_helper_turflists::csv::read_csv(list)
        //                 .map(|mut l| {
        //                     l.shrink();
        //                     l
        //                 })
        //                 .map_err(|e| e.to_string())
        //                 .ok();
        //         }
        //     }

        //     if ui
        //         .button("Load Members portal list")
        //         .on_hover_text("Load an Excel turf list")
        //         .clicked()
        //     {
        //         let path = rfd::FileDialog::new()
        //             .add_filter("Excel File", &["xlsx", "xls", "xlsm", "xlsb"])
        //             .pick_file();
        //         if let Some(path) = path {
        //             self.loaded_turflist =
        //                 penning_helper_turflists::xlsx::read_excel(path, (1, 0).into())
        //                     .map(|mut l| {
        //                         l.shrink();
        //                         l
        //                     })
        //                     .map_err(|e| e.to_string())
        //                     .ok();
        //         }
        //     }
        //     TableBuilder::new(ui)
        //         .striped(true)
        //         .columns(Column::remainder(), 4)
        //         .header(20.0, |mut r| {
        //             r.col(|ui| {
        //                 ui.strong("Name");
        //             });
        //             r.col(|ui| {
        //                 ui.strong("Email");
        //             });
        //             r.col(|ui| {
        //                 ui.strong("Amount");
        //             });
        //             r.col(|ui| {
        //                 ui.strong("IBAN");
        //             });
        //         })
        //         .body(|mut body| {
        //             if let Some(t) = &self.loaded_turflist {
        //                 for row in t.iter() {
        //                     body.row(20.0, |mut r| {
        //                         r.col(|ui| {
        //                             ui.label(&row.name);
        //                         });
        //                         r.col(|ui| {
        //                             ui.label(*&row.email.as_ref().unwrap_or(&"".to_string()));
        //                         });
        //                         r.col(|ui| {
        //                             ui.label(&row.amount.to_string());
        //                         });
        //                         r.col(|ui| {
        //                             ui.label(*&row.iban.as_ref().unwrap_or(&"".to_string()));
        //                         });
        //                     });
        //                 }
        //             }
        //         });

        //     // egui::ScrollArea::new([true, true]).show(ui, |ui| {
        //     //     egui::Grid::new("grid")
        //     //         .striped(true)
        //     //         .num_columns(4)
        //     //         .show(ui, |ui| {
        //     //             ui.label("Name");
        //     //             ui.label("Email");
        //     //             ui.label("Amount");
        //     //             ui.label("IBAN");
        //     //             ui.end_row();
        //     //             if let Some(t) = &self.loaded_turflist {
        //     //                 for row in t.iter() {
        //     //                     ui.label(&row.name);
        //     //                     ui.label(*&row.email.as_ref().unwrap_or(&"".to_string()));
        //     //                     ui.label(&row.amount.to_string());
        //     //                     ui.label(*&row.iban.as_ref().unwrap_or(&"".to_string()));
        //     //                     ui.end_row();
        //     //                 }
        //     //             }
        //     //         });
        //     // });
        // });
    }
}

#[derive(Clone, Debug)]
struct MyTabs {
    tree: Tree<ContentThing>,
    members: Vec<Relation>,
}

impl Default for MyTabs {
    fn default() -> Self {
        Self::new(&[])
    }
}

impl MyTabs {
    pub fn new(members: &[Relation]) -> Self {
        let mut tree = Tree::new(vec![ContentThing::Info]);
        tree.set_focused_node(NodeIndex::root());
        Self {
            tree,
            members: members.to_vec(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let mut nodes = vec![];
        egui_dock::DockArea::new(&mut self.tree)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_close_buttons(true)
            .show_add_buttons(true)
            .show_add_popup(true)
            .show_inside(
                ui,
                &mut TabViewer {
                    added_nodes: &mut nodes,
                },
            );
        nodes.drain(..).for_each(|(node, content)| {
            self.tree.set_focused_node(node);
            self.tree.push_to_focused_leaf(content)
        })
    }
}

#[derive(Clone, Debug)]
enum ContentThing {
    Info,
    TurflistImport(TurflistImport),
    DrinkImport(DrinkImport),
}

impl ContentThing {
    pub fn title(&self) -> &'static str {
        match self {
            ContentThing::Info => "Info",
            ContentThing::TurflistImport(_) => "Turflist Import",
            ContentThing::DrinkImport(_) => "Borrel Import",
        }
    }

    pub fn modified(&self) -> bool {
        match self {
            ContentThing::Info => true,
            ContentThing::TurflistImport(_) => false,
            ContentThing::DrinkImport(_) => false,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        match self {
            ContentThing::Info => InfoTab::ui(ui),
            ContentThing::TurflistImport(tli) => todo!(),
            ContentThing::DrinkImport(tli) => todo!(),
        }
    }
}

struct InfoTab;

impl InfoTab {
    pub fn ui(ui: &mut Ui) {
        ui.heading("Penning Helper");
        ui.label("Penning Helper is a tool to help with the administration of AEGEE-Delft.");
        ui.label("It can be used to import turflists from the members portal and to import borrels from loyverse.");
        ui.label("But really as long as you give it an excel file with some specific columns it'll happily work with it.");
        ui.label("It can also be used to generate SEPA files for the bank,");
        ui.label("and can send automated emails to the members that have an open balance to inform them that they have to pay.");
        egui_extras::RetainedImage::from_color_image("memes", ColorImage::example())
            .show_max_size(ui, Vec2::new(ui.available_width(), 256.0));
    }
}

#[derive(Clone, Debug, Default)]
struct TurflistImport {
    turflist: Option<TurfList>,
    path: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug, Default)]
struct DrinkImport {
    turflist: Option<TurfList>,
    path: Option<std::path::PathBuf>,
}

struct TabViewer<'a> {
    added_nodes: &'a mut Vec<(NodeIndex, ContentThing)>,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = ContentThing;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.show(ui);
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, node: NodeIndex) {
        ui.vertical(|ui| {
            ui.set_min_width(128.0);
            if ui.button("Turflist Import").clicked() {
                self.added_nodes.push((
                    node,
                    ContentThing::TurflistImport(TurflistImport::default()),
                ));
            }
            if ui.button("Borrel Import").clicked() {
                self.added_nodes
                    .push((node, ContentThing::DrinkImport(DrinkImport::default())));
            }
        });
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        !tab.modified()
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        if tab.modified() && !matches!(tab, ContentThing::Info) {
            format!("{}*", tab.title()).into()
        } else {
            tab.title().into()
        }
    }
}
