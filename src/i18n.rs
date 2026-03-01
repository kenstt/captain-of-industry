// The i18n!() macro is invoked in main.rs (crate root).
// This module provides locale management helpers.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Locale {
    En,
    ZhTW,
}

impl Locale {
    pub fn code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::ZhTW => "zh-TW",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::ZhTW => "中文",
        }
    }

    pub fn all() -> &'static [Locale] {
        &[Locale::En, Locale::ZhTW]
    }
}

pub fn set_locale(locale: Locale) {
    rust_i18n::set_locale(locale.code());
}
