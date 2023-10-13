use std::any::Any;

use egui::{vec2, Align2, Key, TextEdit, Ui, Window};
use regex::Regex;

static ID_GEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub struct Popup {
    open: bool,
    title: String,
    data: Box<dyn DataStuff>,
}

impl Popup {
    pub fn new(title: impl Into<String>, data: impl DataStuff + 'static) -> Self {
        Self {
            open: true,
            title: title.into(),
            data: Box::new(data),
        }
    }

    pub fn new_default<D: DataStuff + Default + 'static>(title: impl Into<String>) -> Self {
        Self::new(title, D::default())
    }

    /// The sad part about this is that you can't really get the original type back.
    /// So we have to use Any::downcast_ref() to get the value back.
    /// This is not ideal, but it works.
    pub fn value<V: Any>(&self) -> Option<&V> {
        if self.open {
            return None;
        }
        self.data.as_any().downcast_ref()
    }

    // /// See [`Popup::value()`], but this one copies the value.
    // pub fn value_owned<V: Any + Copy>(&self) -> Option<V> {
    //     self.value().copied()
    // }

    pub fn show(&mut self, ctx: &egui::Context) {
        let mut open = self.open;
        let r = Window::new(&self.title)
            .id(egui::Id::new(ID_GEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)))
            .open(&mut open)
            .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
            .default_size(vec2(512.0, 512.0))
            .collapsible(false)
            .show(ctx, |ui| self.actual_show(ui));
        if let Some(r) = r {
            let r = r.inner.unwrap();
            match r {
                PopupState::Close => {
                    self.open = false;
                    return;
                }
                PopupState::Open => {}
                PopupState::OpenHide => {}
                PopupState::CantClose => {
                    self.open = true;
                    return;
                }
            }
        }
        if open && !self.open {
            self.open = false;
            ctx.request_repaint();
        } else if !open && self.open {
            self.open = false;
            ctx.request_repaint();
        }
    }

    fn actual_show(&mut self, ui: &mut Ui) -> PopupState {
        let res = self.data.show_part(ui);
        if matches!(res, PopupState::Open | PopupState::CantClose) {
            ui.horizontal(|ui| {
                if ui.button("Ok").clicked() {
                    self.open = false;
                }
                if ui.button("Cancel").clicked() {
                    self.open = false;
                }
            });
        }

        res
    }
}

pub enum PopupState {
    /// The popup should be closed
    Close,
    /// The popup should stay open
    Open,

    /// The popup should stay open, and the default buttons should be hidden.
    OpenHide,

    /// The popup should stay open, but the user should not be able to close it.
    CantClose,
}

impl From<bool> for PopupState {
    fn from(value: bool) -> Self {
        if value {
            PopupState::Close
        } else {
            PopupState::OpenHide
        }
    }
}

pub trait DataStuff {
    fn show_part(&mut self, ui: &mut Ui) -> PopupState;

    fn as_any(&self) -> &dyn Any;
}

impl DataStuff for bool {
    fn show_part(&mut self, ui: &mut Ui) -> PopupState {
        ui.horizontal(|ui| {
            if ui.button("Yes").clicked() {
                *self = true;
                true
            } else if ui.button("No").clicked() {
                *self = false;
                true
            } else {
                false
            }
        })
        .inner
        .into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DataStuff for String {
    fn show_part(&mut self, ui: &mut Ui) -> PopupState {
        let mut s = (self.clone(), ());
        let state = <(String, ()) as DataStuff>::show_part(&mut s, ui);
        *self = s.0;
        state
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait StringValidator {
    fn validate(&self, s: &str) -> bool;
}

impl StringValidator for () {
    fn validate(&self, _: &str) -> bool {
        true
    }
}

impl StringValidator for &'static str {
    fn validate(&self, s: &str) -> bool {
        s == *self
    }
}

impl StringValidator for Regex {
    fn validate(&self, s: &str) -> bool {
        self.is_match(s)
    }
}

macro_rules! number_validators {
    ($num:ty) => {
        impl StringValidator for $num {
            fn validate(&self, s: &str) -> bool {
                s.parse::<$num>().is_ok()
            }
        }
    };
    ($($num:ty),*) => {
        $(number_validators!($num);)*
    };
}

number_validators!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

impl<V: StringValidator + Any> DataStuff for (String, V) {
    fn show_part(&mut self, ui: &mut Ui) -> PopupState {
        let valid = self.1.validate(&self.0);
        let res = TextEdit::singleline(&mut self.0)
            .text_color(if valid {
                egui::Color32::GREEN
            } else {
                egui::Color32::RED
            })
            .show(ui);
        if !valid {
            ui.memory_mut(|m| m.request_focus(res.response.id));
            PopupState::CantClose
        } else {
            if res.response.lost_focus() && ui.input(|s| s.key_pressed(Key::Enter)) {
                PopupState::Close
            } else {
                if !res.response.has_focus() {
                    ui.memory_mut(|m| m.request_focus(res.response.id));
                }
                PopupState::Open
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }
}

pub struct ErrorThing(String);

impl ErrorThing {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl DataStuff for ErrorThing {
    fn show_part(&mut self, ui: &mut Ui) -> PopupState {
        ui.label(&self.0);
        PopupState::Open
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }
}
