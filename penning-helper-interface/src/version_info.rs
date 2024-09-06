use egui::Ui;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
const URL: &str = "https://github.com/AEGEE-Delft/penning-helper/releases/latest";

pub struct VersionInfo {
    pub local_version: semver::Version,
    pub remote_version: semver::Version,
    pub outdated: bool,
}

impl VersionInfo {
    pub fn new() -> Self {
        let res = reqwest::blocking::get(URL);
        match res {
            Ok(res) => {
                let url_version = res.url().path().split_once("tag/v").unwrap().1;
                let remote_version = semver::Version::parse(url_version).unwrap();
                let local_version = semver::Version::parse(VERSION).unwrap();
                let outdated = local_version < remote_version;
                Self {
                    local_version,
                    remote_version,
                    outdated,
                }
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }
    }

    pub fn render(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("Current: {}", self.local_version));
            ui.label(format!("Latest: {}", self.remote_version));
            let color = if self.outdated {
                egui::Color32::RED
            } else {
                egui::Color32::GREEN
            };
            ui.colored_label(color, format!("Up to date: {}", !self.outdated));
            ui.hyperlink(URL);
        });
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self::new()
    }
}
