// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    ops::Index,
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        OnceLock,
    },
    time::SystemTime,
};

use eframe::egui::{self, Ui};
use egui::{Color32, Visuals};
use egui_dock::{DockState, NodeIndex, Style, SurfaceIndex};

use file_receiver::{FileReceievers, FileReceiverResult, FileReceiverSource};
use member_info::MemberInfo;
use merch_sales::MerchSales;
use penning_helper_config::{Config, ConscriboConfig};
use penning_helper_conscribo::{
    accounts::{AccountRequest, AccountResponse},
    entities::{filters::Filter, Entities, Entity},
    field_definitions::FieldDefs,
    multirequest::{MultiRequest, MultiRequestElementResponse},
    session::Credentials,
    ConscriboClient,
};

use popup::{ErrorThing, Popup};

use sepa_stuff::SepaGen;
use settings::SettingsWindow;
use turflist::TurflistImport;

mod file_receiver;
mod member_info;
mod merch_sales;
mod popup;
mod rekening_selector;
mod sepa_stuff;
mod settings;
mod turflist;

static ERROR_STUFF: OnceLock<Sender<String>> = OnceLock::new();

fn main() {
    let (s, r) = channel();
    ERROR_STUFF.set(s).unwrap();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Penning Helper",
        native_options,
        Box::new(|cc| Ok(Box::new(PenningHelperApp::new(cc, r)))),
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
    rekeningen: AccountResponse,
}

#[derive(Debug, Clone, Default)]
struct Relations {
    remapper: HashMap<String, String>,
    members: Vec<Entity>,
}

impl Relations {
    pub fn new(member_lists: &[Vec<Entity>]) -> Self {
        let mut remapper = HashMap::new();
        let mut members: Vec<Entity> = vec![];
        for l in member_lists {
            for m in l {
                if let Some(r) = members.iter().find(|&mem| {
                    mem.display_name == m.display_name
                        && mem.email.to_lowercase() == m.email.to_lowercase()
                }) {
                    remapper.insert(m.code.clone(), r.code.clone());
                } else {
                    remapper.insert(m.code.clone(), m.code.clone());
                    members.push(m.clone());
                }
            }
        }
        members.sort_by(|a, b| a.display_name.cmp(&b.display_name));

        Self { remapper, members }
    }

    pub fn find_member(&self, code: &str) -> Option<&Entity> {
        let actual_code = self.remapper.get(code)?.as_str();
        self.members.iter().find(|m| m.code == actual_code)
    }

    pub fn find_member_by_name(&self, name: &str) -> Option<&Entity> {
        self.find_member(&self.members.iter().find(|m| m.display_name == name)?.code)
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.members.iter()
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }
}

impl Index<usize> for Relations {
    type Output = Entity;

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
    accounts: &'t AccountResponse,
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
            accounts: &app.rekeningen,
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
            // match
            // {
            //     Ok(o) => Some(o),
            //     Err(e) => {
            //         println!("Error logging in: {}", e);
            //         if let Some(s) = ERROR_STUFF.get() {
            //             s.send(format!("Error logging in: {}", e)).unwrap();
            //         }
            //         None
            //     }
            // }

            Some(
                ConscriboClient::new(cfg.account_name.clone())
                    .with_credentials(Credentials::new(cfg.username.clone(), cfg.password.clone())),
            )
        };
        if let Some(c) = &self.client {
            println!("Connected to Conscribo");
            let fields = c.execute(FieldDefs::new("lid".to_string()));
            if let Ok(_fields) = fields {
                // println!("Fields: {:?}", fields);
            } else {
                return false;
            }
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    let res = c.execute(
                        MultiRequest::new()
                            .push("lid", Entities::new().filter(Filter::entity_type("lid")))
                            .push(
                                "onbekend",
                                Entities::new().filter(Filter::entity_type("onbekend")),
                            ),
                    );

                    let r = if let Ok(res) = res {
                        let resses = res.responses_owned_unsafe();
                        let lid = resses.get("lid").unwrap();
                        let onbekend = resses.get("onbekend").unwrap();
                        let mut relations = vec![];
                        if let MultiRequestElementResponse::EntityRequest(leden) =
                            lid.content_unsafe()
                        {
                            relations.push(leden.entities.values().cloned().collect());
                        }
                        if let MultiRequestElementResponse::EntityRequest(onbekenden) =
                            onbekend.content_unsafe()
                        {
                            relations.push(onbekenden.entities.values().cloned().collect());
                        }
                        relations
                    } else {
                        vec![]
                    };
                    r
                });
                if let Some(relations) = relations {
                    self.members = Relations::new(&relations);
                }
            }

            if self.rekeningen.accounts().is_empty() {
                if let Some(res) = self.conscribo_client.run(|c| {
                    let res = c.execute(AccountRequest::today());
                    res
                }) {
                    match res {
                        Ok(res) => {
                            self.rekeningen = res.response_unsafe_owned();
                        }
                        Err(e) => {
                            eprintln!("Error getting accounts: {}", e);
                        }
                    }
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
                    if ui
                        .button("Refresh accounts")
                        .on_hover_text("Accounts as in what it shows in conscribo")
                        .clicked()
                    {
                        self.rekeningen = Default::default();
                        ui.close_menu();
                    }
                    if ui.button("Load File").clicked() {
                        ui.close_menu();
                        self.file_channels
                            .new_receiver(FileReceiverSource::TurfList);
                    }
                    if ui
                        .button("Delete Invoice cache")
                        .on_hover_text("Use this if you have changed a transaction in the past")
                        .clicked()
                    {
                        ui.close_menu();
                        let file = dirs::data_local_dir()
                            .unwrap_or(PathBuf::from("."))
                            .join("penning-helper")
                            .join("clientcache.bin");
                        if let Err(e) = std::fs::remove_file(file) {
                            eprintln!("Error deleting cache: {}", e);
                        }
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
    dock: DockState<ContentThing>,
}

impl Default for MyTabs {
    fn default() -> Self {
        Self::new()
    }
}

impl MyTabs {
    pub fn new() -> Self {
        let dock = DockState::new(vec![ContentThing::Info]);
        Self { dock }
    }

    fn ui(&mut self, ui: &mut egui::Ui, foobar: FooBar) {
        let mut nodes = vec![];
        egui_dock::DockArea::new(&mut self.dock)
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
        nodes
            .drain(..)
            .for_each(|(_node, content)| self.dock.push_to_focused_leaf(content))
    }
}

#[derive(Clone, Debug)]
enum ContentThing {
    Info,
    MemberInfo(MemberInfo),
    TurflistImport(TurflistImport),
    SepaGen(SepaGen),
    MerchSales(MerchSales),
}

impl ContentThing {
    pub fn title(&self) -> &'static str {
        match self {
            ContentThing::Info => "Info",
            ContentThing::MemberInfo(_) => "Member Info",
            ContentThing::TurflistImport(_) => "Turflist Import",
            ContentThing::SepaGen(_) => "Invoice Generator",
            ContentThing::MerchSales(_) => "Merch Sales",
        }
    }

    pub fn modified(&self) -> bool {
        match self {
            ContentThing::Info => true,
            ContentThing::MemberInfo(_) => false,
            ContentThing::TurflistImport(_) => false,
            ContentThing::SepaGen(_) => false,
            ContentThing::MerchSales(_) => false,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, cfg: &mut FooBar, members: &Relations) {
        match self {
            ContentThing::Info => InfoTab::ui(ui, cfg),
            ContentThing::MemberInfo(mi) => mi.ui(ui, cfg, members),
            ContentThing::TurflistImport(tli) => tli.ui(ui, cfg, members),
            ContentThing::SepaGen(sg) => sg.ui(ui, cfg, members),
            ContentThing::MerchSales(ms) => ms.ui(ui, cfg, members),
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

    fn add_popup(&mut self, ui: &mut egui::Ui, _surface: SurfaceIndex, node: NodeIndex) {
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
            if ui.button("Merch Sales").clicked() {
                self.added_nodes
                    .push((node, ContentThing::MerchSales(Default::default())));
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
