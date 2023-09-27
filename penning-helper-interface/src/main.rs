// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    ffi::OsStr,
    sync::{
        mpsc::{channel, Receiver, Sender},
        OnceLock,
    },
    time::SystemTime,
};

use eframe::egui::{self, Ui};
use egui::{
    plot::Text, CentralPanel, Color32, ColorImage, RichText, TextEdit, TopBottomPanel, Vec2,
    Visuals,
};
use egui_dock::{NodeIndex, Style, Tree};
use egui_extras::{Column, TableBuilder};
use file_receiver::{FileReceievers, FileReceiverResult, FileReceiverSource};
use penning_helper_config::{Config, ConscriboConfig};
use penning_helper_conscribo::{
    AddChangeTransaction, ConscriboClient, ConscriboResult, Relation, TransactionResult,
    UnifiedTransaction,
};
use penning_helper_turflists::{matched_turflist::MatchedTurflist, turflist::TurfList};
use penning_helper_types::{Date, Euro};
use popup::{ErrorThing, Popup};

use settings::SettingsWindow;

mod file_receiver;
mod popup;
mod settings;
mod background_requester;

static ERROR_STUFF: OnceLock<Sender<String>> = OnceLock::new();

fn main() {
    let (s, r) = channel();
    ERROR_STUFF.set(s).unwrap();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Penning Helper",
        native_options,
        Box::new(|cc| Box::new(PenningHelperApp::new(cc, r))),
    )
    .unwrap();
}

#[derive(Default)]
struct PenningHelperApp {
    visuals: Visuals,
    tabs: MyTabs,
    settings_window: SettingsWindow,
    file_channels: FileReceievers,
    popups: HashMap<String, Popup>,
    conscribo_client: ConscriboConnector,
    r: Option<Receiver<String>>,
    members: Vec<Relation>,
}

struct FooBar<'t> {
    popups: &'t mut HashMap<String, Popup>,
    conscribo: &'t ConscriboConnector,
    files: &'t mut FileReceievers,
    cfg: &'t Config,
    members: &'t [Relation],
}

impl<'t> FooBar<'t> {
    pub fn from_app(app: &'t mut PenningHelperApp) -> Self {
        Self {
            popups: &mut app.popups,
            conscribo: &app.conscribo_client,
            files: &mut app.file_channels,
            cfg: &app.settings_window.config,
            members: &app.members,
        }
    }
}

#[derive(Default)]
struct ConscriboConnector {
    client: Option<ConscriboClient>,
    username: String,
    password: String,
    n: u32,
}

impl ConscriboConnector {
    fn connect(&mut self, cfg: &ConscriboConfig) -> bool {
        // don't try to log in when already logged in
        if self.client.is_some() {
            return true;
        }
        let username = cfg.username.clone();
        let password = cfg.password.clone();
        // don't try to log in when username or password is same as stored (and potentially not working)
        if self.username == username && self.password == password {
            return false;
        }
        // don't try to log in when username or password is empty
        if username.is_empty() || password.is_empty() {
            return false;
        }

        self.username = username;
        self.password = password;
        println!("Attempting actual login");
        if self.n > 2 {
            println!("Too many attempts, not trying again");
            return false;
        }
        self.client = {
            match ConscriboClient::new(&self.username, &self.password, &cfg.url) {
                Ok(o) => Some(o),
                Err(e) => {
                    println!("Error logging in: {}", e);
                    if let Some(s) = ERROR_STUFF.get() {
                        s.send(format!("Error logging in: {}", e)).unwrap();
                    }
                    None
                }
            }
        };
        if self.client.is_some() {
            println!("Connected to Conscribo");
            return true;
        }
        self.n += 1;
        false
    }

    pub fn run<F: FnOnce(&ConscriboClient) -> R, R>(&self, f: F) -> Option<R> {
        self.client.as_ref().map(f)
    }
}

impl PenningHelperApp {
    fn new(cc: &eframe::CreationContext<'_>, r: Receiver<String>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let mut s = Self::with_config(Config::load_from_file());
        s.r = Some(r);
        s
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

    fn login_conscribo(&mut self) -> bool {
        if self.settings_window.open {
            return false;
        }
        let conscribo_cfg = self.settings_window.config.conscribo();
        self.conscribo_client.connect(conscribo_cfg)
    }
}

impl eframe::App for PenningHelperApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.settings_window.show(ctx);
        for (_, p) in &mut self.popups {
            p.show(ctx);
        }
        self.file_channels.receive_all();
        if self.login_conscribo() {
            if self.members.is_empty() {
                let relations = self.conscribo_client.run(|c| {
                    let members = c.get_relations("lid").unwrap();
                    let others = c.get_relations("onbekend").unwrap();
                    let others = others
                        .into_iter()
                        .filter(|o| !members.iter().any(|m| m.naam == o.naam))
                        .collect::<Vec<_>>();
                    let all_relations = members
                        .into_iter()
                        .chain(others.into_iter())
                        // .filter(|r| r.naam == "Julius de Jeu")
                        .collect::<Vec<_>>();
                    all_relations
                });
                if let Some(relations) = relations {
                    self.members = relations;
                }
            }
        }
        if let Some(r) = &self.r {
            if let Ok(r) = r.try_recv() {
                self.popups.insert(
                    format!("{:?}", SystemTime::now()),
                    Popup::new("Msg", ErrorThing::new(r)),
                );
            }
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    self.visuals.light_dark_radio_buttons(ui);
                    ctx.set_visuals(self.visuals.clone());
                    if ui.button("Settings").clicked() {
                        self.settings_window.open = true;
                        ui.close_menu();
                    }
                    if ui.button("Refresh memberlist").clicked() {
                        self.members = vec![];
                        ui.close_menu();
                    }
                    if ui.button("Load File").clicked() {
                        ui.close_menu();
                        self.file_channels
                            .new_receiver(FileReceiverSource::TurfList);
                    }
                    if ui.button("show popup").clicked() {
                        ui.close_menu();
                        self.popups.insert(
                            "test".to_string(),
                            Popup::new_default::<(String, u16)>("Port number time"),
                        );
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }

                    egui::warn_if_debug_build(ui);
                });
            })
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.colored_label(Color32::LIGHT_GRAY, "Made by Julius de Jeu");
                if let Some(ch) = self
                    .file_channels
                    .get_receiver(FileReceiverSource::TurfList)
                {
                    match ch.get_file() {
                        FileReceiverResult::File(file) => {
                            ui.colored_label(Color32::LIGHT_GRAY, "File recieved");
                            ui.colored_label(Color32::LIGHT_GRAY, format!("{:?}", file));
                        }
                        FileReceiverResult::Waiting => {
                            ui.colored_label(Color32::LIGHT_GRAY, "Waiting for file");
                        }
                        FileReceiverResult::NoFile => {
                            ui.colored_label(Color32::LIGHT_GRAY, "No file");
                        }
                    }
                } else {
                    ui.colored_label(Color32::LIGHT_GRAY, "No file channel");
                }

                if let Some(p) = self.popups.get("Price") {
                    ui.colored_label(Color32::LIGHT_GRAY, "Popup open");
                    ui.colored_label(Color32::LIGHT_GRAY, format!("{:?}", p.value::<String>()));
                } else {
                    ui.colored_label(Color32::LIGHT_GRAY, "No popup");
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.settings_window.open, |ui| {
                let mut tabs = self.tabs.clone();
                let foobar = FooBar::from_app(self);
                tabs.ui(ui, foobar);
                self.tabs = tabs;
            })
        });
    }
}

#[derive(Clone, Debug)]
struct MyTabs {
    tree: Tree<ContentThing>,
}

impl Default for MyTabs {
    fn default() -> Self {
        Self::new()
    }
}

impl MyTabs {
    pub fn new() -> Self {
        let mut tree = Tree::new(vec![ContentThing::Info]);
        tree.set_focused_node(NodeIndex::root());
        Self { tree }
    }

    fn ui(&mut self, ui: &mut egui::Ui, foobar: FooBar) {
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
                    members: &foobar.members,
                    foobar,
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
    MemberInfo(MemberInfo),
    TurflistImport(TurflistImport),
    SepaGen(SepaGen),
}

impl ContentThing {
    pub fn title(&self) -> &'static str {
        match self {
            ContentThing::Info => "Info",
            ContentThing::MemberInfo(_) => "Member Info",
            ContentThing::TurflistImport(_) => "Turflist Import",
            ContentThing::SepaGen(_) => "Invoice Generator",
        }
    }

    pub fn modified(&self) -> bool {
        match self {
            ContentThing::Info => true,
            ContentThing::MemberInfo(_) => false,
            ContentThing::TurflistImport(_) => false,
            ContentThing::SepaGen(_) => false,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, cfg: &mut FooBar, members: &[Relation]) {
        match self {
            ContentThing::Info => InfoTab::ui(ui, cfg),
            ContentThing::MemberInfo(mi) => mi.ui(ui, cfg, members),
            ContentThing::TurflistImport(tli) => tli.ui(ui, cfg, members),
            ContentThing::SepaGen(sg) => sg.ui(ui, cfg, members),
        }
    }

    pub fn file_handle(&self) -> Option<FileReceiverSource> {
        match self {
            ContentThing::TurflistImport(_) => Some(FileReceiverSource::TurfList),
            _ => None,
        }
    }
}

struct InfoTab;

impl InfoTab {
    pub fn ui(ui: &mut Ui, foobar: &FooBar) {
        ui.heading("Penning Helper");
        ui.label("Penning Helper is a tool to help with the administration of AEGEE-Delft.");
        ui.label("It can be used to import turflists from the members portal and to import borrels from loyverse.");
        ui.label("But really as long as you give it an excel file with some specific columns it'll happily work with it.");
        ui.label("It can also be used to generate SEPA files for the bank,");
        ui.label("and can send automated emails to the members that have an open balance to inform them that they have to pay.");
        egui_extras::RetainedImage::from_color_image("memes", ColorImage::example())
            .show_max_size(ui, Vec2::new(ui.available_width(), 256.0));
        let errors = foobar.cfg.config_errors();
        if errors.len() > 0 {
            ui.heading("Config Errors:");
            for error in errors {
                ui.label(error);
            }
        } else {
            ui.heading("Config is valid");
        }
    }
}

#[derive(Clone, Debug, Default)]
struct MemberInfo {
    search: String,
}

impl MemberInfo {
    pub fn ui(&mut self, ui: &mut Ui, foobar: &FooBar, members: &[Relation]) {
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
                            ui.label(&member.source);
                        });
                    });
                }
            });
    }
}

#[derive(Clone, Debug, Default)]
struct TurflistImport {
    rekening: String,
    description: String,
    reference: String,
    turflist: Option<TurfList>,
    path: Option<std::path::PathBuf>,
    matched: Option<MatchedTurflist>,
    price: Euro,
    last_len: usize,
}

impl TurflistImport {
    pub fn ui(&mut self, ui: &mut Ui, foobar: &mut FooBar, members: &[Relation]) {
        // TopBottomPanel::top(ui.next_auto_id()).show(ui.ctx(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Rekening");
            TextEdit::singleline(&mut self.rekening)
                .hint_text("0000-00")
                .show(ui);
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
                println!("Receiver exists!");
                match list.get_file() {
                    FileReceiverResult::File(f) => {
                        ui.label(format!("File: {:?}", f));
                        if self.turflist.is_none() {
                            println!("Turflist was none");
                            let ext = f.extension().unwrap_or_else(|| OsStr::new(""));
                            if ext == "csv" {
                                ui.label("CSV");
                            } else if ext == "xlsx" {
                                ui.label("Excel");
                                if self.price == (0, 0).into() {
                                    let res = foobar
                                        .popups
                                        .entry("Price".to_string())
                                        .or_insert(Popup::new_default::<(String, f64)>("Price"));
                                    if let Some(v) = res.value::<String>() {
                                        self.price = v.parse().unwrap_or((0, 0).into());
                                        foobar.popups.remove("Price");
                                    }
                                } else {
                                    ui.label(format!("Price: {}", self.price));
                                    foobar.popups.remove("Price");

                                    match penning_helper_turflists::xlsx::read_excel(f, self.price)
                                        .map(|mut l| {
                                            l.shrink();
                                            l
                                        })
                                        .map_err(|e| e.to_string())
                                    {
                                        Ok(mut o) => {
                                            // let names = members
                                            //     .iter()
                                            //     .map(|m| m.naam.clone())
                                            //     .collect::<Vec<_>>();
                                            // let emails = members
                                            //     .iter()
                                            //     .map(|m| m.email_address.clone())
                                            //     .collect::<Vec<_>>();
                                            // self.matched = Some(o.get_matches(&names, &emails));
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
                            } else {
                                ui.label("Invalid file type");
                            }
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
                                    self.rekening.clone(),
                                    eur,
                                    self.reference.clone(),
                                    r.code,
                                )
                            } else {
                                a.add_credit(
                                    self.rekening.clone(),
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
            } else {
                ui.add_enabled_ui(false, |ui| ui.button("Append to Conscribo"));
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
                self.matched = Some(o.get_matches(&names, &emails));
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

#[derive(Clone, Debug)]
struct RelationTransaction {
    t: Vec<UnifiedTransaction>,
    name: String,
    iban: String,
    email: String,
}

#[derive(Clone, Debug, Default)]
struct SepaGen {
    transactions: Option<Vec<RelationTransaction>>,
}

impl SepaGen {
    fn ui(&mut self, ui: &mut Ui, foobar: &mut FooBar, members: &[Relation]) {
        if self.transactions.is_none() {}
        TableBuilder::new(ui)
            .columns(Column::remainder(), 2)
            .header(20.0, |mut r| {
                r.col(|ui| {
                    ui.label("Name");
                });
                r.col(|ui| {
                    ui.label("Amount");
                });
            })
            .body(|mut b| {
                if let Some(t) = &self.transactions {
                    for t in t {
                        b.row(20.0, |mut r| {
                            r.col(|ui| {
                                ui.label(&t.name);
                            });
                            r.col(|ui| {
                                ui.label(
                                    &t.t.iter()
                                        .map(|t| t.cost)
                                        .fold(Euro::default(), |a, b| a + b)
                                        .to_string(),
                                );
                            });
                        });
                    }
                }
            });
    }
}

struct TabViewer<'a> {
    added_nodes: &'a mut Vec<(NodeIndex, ContentThing)>,
    foobar: FooBar<'a>,
    members: &'a [Relation],
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = ContentThing;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.show(ui, &mut self.foobar, self.members);
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, node: NodeIndex) {
        ui.vertical(|ui| {
            ui.set_min_width(128.0);
            if ui.button("Member Info").clicked() {
                self.added_nodes
                    .push((node, ContentThing::MemberInfo(Default::default())));
            }
            if ui.button("Turflist Import").clicked() {
                self.added_nodes.push((
                    node,
                    ContentThing::TurflistImport(TurflistImport::default()),
                ));
            }
            if ui.button("Generate Invoice").clicked() {
                self.added_nodes
                    .push((node, ContentThing::SepaGen(Default::default())));
            }
        });
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        if !tab.modified() {
            if let Some(handle) = tab.file_handle() {
                self.foobar.files.remove_receiver(handle);
            }
            true
        } else {
            false
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        if tab.modified() && !matches!(tab, ContentThing::Info) {
            format!("{}*", tab.title()).into()
        } else {
            tab.title().into()
        }
    }
}
