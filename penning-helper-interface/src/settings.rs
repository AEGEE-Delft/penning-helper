use egui::{vec2, Align2, TextEdit, Ui, Window};
use penning_helper_config::Config;

#[derive(Clone, Debug, Default)]
pub struct SettingsWindow {
    pub open: bool,
    pub config: Config,
}

impl SettingsWindow {
    pub fn show(&mut self, ctx: &egui::Context) {
        let mut open = self.open;
        Window::new("Settings")
            .open(&mut open)
            .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
            .default_size(vec2(512.0, 512.0))
            .collapsible(false)
            .resizable(false)
            .scroll([false, true])
            .show(ctx, |ui| self.actual_show(ui));
        if open && !self.open {
            self.open = false;
        } else if !open && self.open {
            self.open = false;
        }
    }

    fn actual_show(&mut self, ui: &mut Ui) {
        ui.heading("Current Year");
        labelled_row(ui, "Current year:", self.config.year_format_mut(), "2324");
        ui.heading("SEPA");
        labelled_row(
            ui,
            "IBAN",
            &mut self.config.sepa_mut().company_iban,
            "NL12ABCD0123456789",
        );
        labelled_row(
            ui,
            "BIC",
            &mut self.config.sepa_mut().company_bic,
            "ABCDNL2A",
        );
        labelled_row(
            ui,
            "Name",
            &mut self.config.sepa_mut().company_name,
            "AEGEE-Delft",
        );
        labelled_row(
            ui,
            "Incasso ID",
            &mut self.config.sepa_mut().company_id,
            "NL00ZZZ404840000000",
        );
        ui.heading("Conscribo");
        labelled_row(
            ui,
            "Username",
            &mut self.config.conscribo_mut().username,
            "admin",
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
            "Merch Sale Account",
            &mut self.config.conscribo_mut().merch_winst_rekening,
            "0000-00",
        );
        labelled_row(
            ui,
            "Account Name",
            &mut self.config.conscribo_mut().account_name,
            "aegee-delft",
        );
        ui.heading("Mail");
        labelled_row(
            ui,
            "SMTP Server",
            &mut self.config.mail_mut().smtp_server,
            "smtp.gmail.com",
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
            "testkees@me.org",
        );
        ui.vertical(|ui| {
            ui.label("SMTP Password");
            ui.add(
                TextEdit::singleline(&mut self.config.mail_mut().credentials.password)
                    .hint_text("hunter2")
                    .password(true),
            );
        });
        labelled_row(
            ui,
            "From",
            &mut self.config.mail_mut().from.name,
            "AEGEE-Delft",
        );
        labelled_row(
            ui,
            "From Email",
            &mut self.config.mail_mut().from.address,
            "invoices@aegee-delft.nl",
        );

        labelled_row(
            ui,
            "Reply-To",
            &mut self.config.mail_mut().reply_to.name,
            "AEGEE-Delft",
        );

        labelled_row(
            ui,
            "Reply-To Email",
            &mut self.config.mail_mut().reply_to.address,
            "treasurer@aegee-delft.nl",
        );

        labelled_row(
            ui,
            "Board Line",
            &mut self.config.mail_mut().board_line,
            "XLIth Board of AEGEE-Delft 'Wervelwind'",
        );

        labelled_row(
            ui,
            "Name",
            &mut self.config.mail_mut().name,
            "Piet Pieterse",
        );

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

fn labelled_row(ui: &mut Ui, name: &str, line: &mut String, hint: &'static str) {
    ui.vertical(|ui| {
        ui.label(name);
        TextEdit::singleline(line).hint_text(hint).show(ui);
    });
}
