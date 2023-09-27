// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    cmp::min,
    collections::HashMap,
    ffi::OsStr,
    fs::File,
    ops::{Add, Deref, DerefMut, Index},
    sync::{
        mpsc::{channel, Receiver, Sender},
        OnceLock,
    },
    time::{Duration, Instant, SystemTime},
};

use eframe::egui::{self, Ui};
use egui::{Color32, ColorImage, RichText, TextEdit, Vec2, Visuals};
use egui_dock::{NodeIndex, Style, Tree};
use egui_extras::{Column, TableBuilder};
use file_receiver::{FileReceievers, FileReceiverResult, FileReceiverSource};
use penning_helper_config::{Config, ConscriboConfig};
use penning_helper_conscribo::{
    AddChangeTransaction, ConscriboClient, ConscriboResult, Relation, TransactionResult,
    UnifiedTransaction,
};
use penning_helper_mail::MailServer;
use penning_helper_turflists::{matched_turflist::MatchedTurflist, turflist::TurfList};
use penning_helper_types::{Date, Euro};
use popup::{ErrorThing, Popup};

use settings::SettingsWindow;

mod background_requester;
mod file_receiver;
mod popup;
mod settings;

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
    members: Relations,
    sepa_stuff: penning_helper_sepa::SEPAConfig,
}

#[derive(Debug, Clone, Default)]
struct Relations {
    remapper: HashMap<u32, u32>,
    members: Vec<Relation>,
}

impl Relations {
    pub fn new(member_lists: &[Vec<Relation>]) -> Self {
        let mut remapper = HashMap::new();
        let mut members: Vec<Relation> = vec![];
        for l in member_lists {
            for m in l {
                if let Some(r) = members
                    .iter()
                    .find(|&mem| mem.naam == m.naam && mem.email_address == m.email_address)
                {
                    remapper.insert(m.code, r.code);
                } else {
                    remapper.insert(m.code, m.code);
                    members.push(m.clone());
                }
            }
        }

        Self { remapper, members }
    }

    pub fn find_member(&self, code: u32) -> Option<&Relation> {
        let actual_code = *self.remapper.get(&code)?;
        self.members.iter().find(|m| m.code == actual_code)
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Relation> {
        self.members.iter()
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }
}

impl Index<usize> for Relations {
    type Output = Relation;

    fn index(&self, index: usize) -> &Self::Output {
        &self.members[index]
    }
}

struct FooBar<'t> {
    popups: &'t mut HashMap<String, Popup>,
    conscribo: &'t ConscriboConnector,
    files: &'t mut FileReceievers,
    cfg: &'t Config,
    members: &'t Relations,
    sepa: &'t penning_helper_sepa::SEPAConfig,
}

impl<'t> FooBar<'t> {
    pub fn from_app(app: &'t mut PenningHelperApp) -> Self {
        Self {
            popups: &mut app.popups,
            conscribo: &app.conscribo_client,
            files: &mut app.file_channels,
            cfg: &app.settings_window.config,
            members: &app.members,
            sepa: &app.sepa_stuff,
        }
    }
}

#[derive(Default, Clone)]
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
    fn new(_cc: &eframe::CreationContext<'_>, r: Receiver<String>) -> Self {
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
            self.sepa_stuff =
                penning_helper_sepa::SEPAConfig::from_config(self.settings_window.config.sepa());
            if self.members.is_empty() {
                let relations = self.conscribo_client.run(|c| {
                    let members = c.get_relations("lid").unwrap();
                    let others = c.get_relations("onbekend").unwrap();
                    // let others = others
                    //     .into_iter()
                    //     .filter(|o| !members.iter().any(|m| m.naam == o.naam))
                    //     .collect::<Vec<_>>();
                    // let all_relations = members
                    //     .into_iter()
                    //     .chain(others.into_iter())
                    //     // .filter(|r| r.naam == "Julius de Jeu")
                    //     .collect::<Vec<_>>();
                    vec![members, others]
                });
                if let Some(relations) = relations {
                    self.members = Relations::new(&relations);
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
                        self.members = Default::default();
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

    pub fn show(&mut self, ui: &mut Ui, cfg: &mut FooBar, members: &Relations) {
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
    matched: Option<MatchedTurflist>,
    price: Euro,
    last_len: usize,
}

impl TurflistImport {
    pub fn ui(&mut self, ui: &mut Ui, foobar: &mut FooBar, members: &Relations) {
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
                            let ext = f.extension().and_then(OsStr::to_str).unwrap_or("");
                            match ext {
                                "csv" => {
                                    ui.label("csv");
                                    match File::open(f) {
                                        Ok(f) => match penning_helper_turflists::csv::read_csv(f) {
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
                                        },
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
                if ui.button("Save PDF").clicked() {
                    let pdf = penning_helper_pdf::generate_turflist_pdf(
                        &self
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
    code: u32,
    membership_date: Date,
    iban: String,
    bic: String,
    email: String,
}

impl RelationTransaction {
    pub fn total_cost(&self) -> Euro {
        self.t.iter().map(|t| t.cost).sum()
    }

    pub fn all_after(&self, date: Date) -> impl Iterator<Item = &UnifiedTransaction> {
        self.t.iter().filter(move |t| t.date >= date)
    }

    pub fn cost_after(&self, date: Date) -> Euro {
        self.all_after(date).map(|t| t.cost).sum()
    }

    pub fn previous_invoices_left(&self, date: Date) -> Euro {
        self.t
            .iter()
            .filter(|t| t.date < date)
            .map(|t| t.cost)
            .sum()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum SendMode {
    #[default]
    Test,
    Real,
}

#[derive(Clone, Debug, Default)]
struct SepaGen {
    /// These are the transactions that are good for us
    transactions: Vec<RelationTransaction>,
    /// These ones miss an iban or bic (or both)
    invalid_transactions: Vec<RelationTransaction>,
    /// These ones have a total cost of over 100 euros
    /// They will only have 100 euros deducted from their account
    /// and will have to pay the rest manually
    too_high_transactions: Vec<RelationTransaction>,
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
    fn ui(&mut self, ui: &mut Ui, foobar: &mut FooBar, members: &Relations) {
        if !self.has_tried_mail {
            self.has_tried_mail = true;
            self.email_client = MailServer::new(foobar.cfg.mail(), foobar.cfg.sepa()).ok();
        }
        let done = if !self.unifieds_grabbed {
            ui.label(format!("Getting transactions{}", ".".repeat(self.idx / 50)));
            self.idx += 1;
            self.idx %= 200;
            let r = foobar
                .conscribo
                .run(|c| c.get_transactions())
                .transpose()
                .map(Option::flatten);
            ui.ctx().request_repaint();
            match r {
                Ok(r) => {
                    if let Some(r) = r {
                        self.unifieds_grabbed = true;
                        self.unifieds = r;
                    }
                }
                Err(e) => {
                    if let Some(s) = ERROR_STUFF.get() {
                        s.send(format!("Error: {}", e)).unwrap();
                    }
                }
            }
            false
        } else if !self.unifieds.is_empty() {
            ui.label("Calculating, numbers will change for a bit");
            ui.ctx().request_repaint();
            for _ in 0..min(self.unifieds.len(), 10) {
                let t = self.unifieds.remove(0);
                // since we generate the list transactions from the same list of members we use we can just unwrap here
                let Some(rel) = members.find_member(t.code) else {
                    println!("No relation found for {}", t.code);
                    continue;
                };
                let name = rel.naam.as_str();
                let iban = rel
                    .rekening
                    .as_ref()
                    .map(|r| r.iban.as_str())
                    .unwrap_or_default();
                let bic = rel
                    .rekening
                    .as_ref()
                    .map(|r| r.bic.as_str())
                    .unwrap_or_default();
                let email = rel.email_address.as_str();
                let membership_date = rel.membership_started.unwrap_or_else(|| Date::today());
                let code = rel.code;

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
                        code,
                        membership_date,
                    });
                }
            }
            false
        } else if !self.sorted {
            let mut transactions = vec![];
            let mut invalid = vec![];
            let mut big_money = vec![];
            for t in self.transactions.clone() {
                if t.total_cost() == Euro::default() {
                    continue;
                }
                if t.bic.is_empty() || t.iban.is_empty() {
                    invalid.push(t);
                    continue;
                }
                let total = t.total_cost();
                if total > 100.into() {
                    big_money.push(t);
                    continue;
                }

                transactions.push(t);
            }

            self.transactions = transactions;
            self.invalid_transactions = invalid;
            self.too_high_transactions = big_money;
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
                            for t in &self.transactions {
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
                                    if total > 100.into() {}
                                    // it's a debtor
                                    let d = foobar.sepa.new_debtor(
                                        total,
                                        t.name.clone(),
                                        t.bic.clone(),
                                        t.iban.clone(),
                                        t.code,
                                        t.membership_date,
                                        "Invoice of open AEGEE-Delft balance".to_string(),
                                    );
                                    debtors.push(d);
                                } else {
                                    // nothing
                                }
                            }
                            // can pay max 100 euros per invoice, so we need to make this one 99.99 euros
                            // and the rest will be paid manually
                            for t in &self.too_high_transactions {
                                let total = 99.99.into();
                                let d = foobar.sepa.new_debtor(
                                    total,
                                    t.name.clone(),
                                    t.bic.clone(),
                                    t.iban.clone(),
                                    t.code,
                                    t.membership_date,
                                    "Partial invoice of open AEGEE-Delft balance".to_string(),
                                );
                                debtors.push(d);
                            }
                            let debtors = foobar
                                .sepa
                                .new_invoice_payment_information(Date::in_some_days(3), debtors);
                            let creditors = foobar
                                .sepa
                                .new_transfer_payment_information(Date::in_some_days(6), creditors);
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
                                .chain(self.too_high_transactions.iter().take(1).cloned())
                                .chain(self.invalid_transactions.iter().take(1).cloned())
                                .collect();
                        }
                        SendMode::Real => {
                            self.to_send = self
                                .transactions
                                .iter()
                                .filter(|t| t.total_cost() != Euro::default())
                                .cloned()
                                .chain(self.too_high_transactions.iter().cloned())
                                .chain(self.invalid_transactions.iter().cloned())
                                .collect()
                        }
                    }
                }
                if let Some(mail_client) = &self.email_client {
                    if !self.to_send.is_empty()
                        && (self.last_send + Duration::from_secs(5 * 60)) <= Instant::now()
                    {
                        self.last_send = TimeThing::now();
                        println!("Sending emails");
                        let today = Date::today();
                        for r in self.to_send.drain(0..(20.min(self.to_send.len()))) {
                            let total = r.total_cost();
                            let previous = r.previous_invoices_left(self.last_invoice_date);
                            let t = UnifiedTransaction::create_new_mock(
                                self.last_invoice_date,
                                "Open costs of previous invoice".to_string(),
                                previous,
                            );
                            let to_show = r.all_after(self.last_invoice_date);
                            let t = std::iter::once(&t)
                                .chain(to_show)
                                .map(|t| {
                                    penning_helper_pdf::SimpleTransaction::new(
                                        t.cost,
                                        &t.description,
                                        t.date,
                                    )
                                })
                                .collect::<Vec<_>>();
                            let pdf = penning_helper_pdf::create_invoice_pdf(t, &r.name);
                            let email_address = if matches!(self.send_mode, SendMode::Test) {
                                "test@asraphiel.dev"
                            } else {
                                r.email.as_str()
                            };
                            println!(
                                "Sending email for {} to {}, with total {}",
                                r.name, email_address, total
                            );
                            mail_client
                                .send_mail(
                                    &r.name,
                                    email_address,
                                    pdf,
                                    total,
                                    today,
                                    r.iban.is_empty() || r.bic.is_empty(),
                                )
                                .unwrap();
                        }
                    } else {
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
                for t in self.transactions.iter() {
                    let amount = t.total_cost();
                    if amount == Euro::default() {
                        continue;
                    }
                    b.row(20.0, |mut r| {
                        r.col(|ui| {
                            ui.label(&t.name);
                        });
                        r.col(|ui| {
                            ui.label(amount.to_string());
                        });
                    });
                }
                for t in self.too_high_transactions.iter() {
                    let amount = t.total_cost();
                    b.row(20.0, |mut r| {
                        r.col(|ui| {
                            let text = RichText::new(&t.name).color(ui.visuals().warn_fg_color);
                            ui.label(text);
                        });
                        r.col(|ui| {
                            let text = RichText::new(&amount.to_string())
                                .color(ui.visuals().warn_fg_color);

                            ui.label(text);
                        });
                    });
                }
                for t in self.invalid_transactions.iter() {
                    let amount = t.total_cost();
                    if amount == Euro::default() {
                        continue;
                    }
                    b.row(20.0, |mut r| {
                        r.col(|ui| {
                            let text = RichText::new(&t.name).color(Color32::RED);
                            ui.label(text);
                        });
                        r.col(|ui| {
                            let text = RichText::new(&amount.to_string()).color(Color32::RED);

                            ui.label(text);
                        });
                    });
                }
            });
    }
}

struct TabViewer<'a> {
    added_nodes: &'a mut Vec<(NodeIndex, ContentThing)>,
    foobar: FooBar<'a>,
    members: &'a Relations,
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
