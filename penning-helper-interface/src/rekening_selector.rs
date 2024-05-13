use egui_autocomplete::AutoCompleteTextEdit;

#[derive(Debug, Clone)]
pub struct Selector<T> {
    current_account: String,
    current_account_num: Option<T>,
    hint: String,
}

impl<T> Default for Selector<T> {
    fn default() -> Self {
        Self {
            current_account: Default::default(),
            current_account_num: Default::default(),
            hint: Default::default(),
        }
    }
}

impl<T> Selector<T> {
    pub fn ui_convert(
        &mut self,
        ui: &mut egui::Ui,
        options: impl Iterator<Item = impl AsRef<str>>,
        f: impl FnOnce(&str) -> Option<T>,
    ) {
        self.ui_inner(ui, options, Some(f));
    }

    fn ui_inner(
        &mut self,
        ui: &mut egui::Ui,
        options: impl Iterator<Item = impl AsRef<str>>,
        f: Option<impl FnOnce(&str) -> Option<T>>,
    ) {
        let hint = self.hint.clone();
        ui.add(
            AutoCompleteTextEdit::new(&mut self.current_account, options)
                .highlight_matches(true)
                .max_suggestions(5)
                .set_text_edit_properties(|e| e.hint_text(hint)),
        );
        if let Some(f) = f {
            self.current_account_num = f(&self.current_account);
        }
    }

    pub fn get(&self) -> Option<&T> {
        self.current_account_num.as_ref()
    }

    pub fn as_str(&self) -> &str {
        &self.current_account
    }
}
