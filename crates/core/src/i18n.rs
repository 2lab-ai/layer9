//! Internationalization (i18n) Support - L5/L6
//! Multi-language support with dynamic locale switching
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

// Type aliases to simplify complex types
type TranslateFn = Box<dyn Fn(&str, Option<&HashMap<String, String>>) -> String>;
type PluralFn = Box<dyn Fn(&str, i32, Option<&HashMap<String, String>>) -> String>;
type SetLocaleFn = Box<dyn Fn(Locale)>;
type SimpleTFn = Box<dyn Fn(&str) -> String>;

/// Supported locales
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Locale {
    EnUS,
    EnGB,
    ZhCN,
    ZhTW,
    JaJP,
    KoKR,
    DeDE,
    FrFR,
    EsES,
    ItIT,
    PtBR,
    RuRU,
}

impl Locale {
    pub fn code(&self) -> &'static str {
        match self {
            Locale::EnUS => "en-US",
            Locale::EnGB => "en-GB",
            Locale::ZhCN => "zh-CN",
            Locale::ZhTW => "zh-TW",
            Locale::JaJP => "ja-JP",
            Locale::KoKR => "ko-KR",
            Locale::DeDE => "de-DE",
            Locale::FrFR => "fr-FR",
            Locale::EsES => "es-ES",
            Locale::ItIT => "it-IT",
            Locale::PtBR => "pt-BR",
            Locale::RuRU => "ru-RU",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en-US" | "en" => Some(Locale::EnUS),
            "en-GB" => Some(Locale::EnGB),
            "zh-CN" | "zh" => Some(Locale::ZhCN),
            "zh-TW" => Some(Locale::ZhTW),
            "ja-JP" | "ja" => Some(Locale::JaJP),
            "ko-KR" | "ko" => Some(Locale::KoKR),
            "de-DE" | "de" => Some(Locale::DeDE),
            "fr-FR" | "fr" => Some(Locale::FrFR),
            "es-ES" | "es" => Some(Locale::EsES),
            "it-IT" | "it" => Some(Locale::ItIT),
            "pt-BR" | "pt" => Some(Locale::PtBR),
            "ru-RU" | "ru" => Some(Locale::RuRU),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Locale::EnUS => "English (US)",
            Locale::EnGB => "English (UK)",
            Locale::ZhCN => "简体中文",
            Locale::ZhTW => "繁體中文",
            Locale::JaJP => "日本語",
            Locale::KoKR => "한국어",
            Locale::DeDE => "Deutsch",
            Locale::FrFR => "Français",
            Locale::EsES => "Español",
            Locale::ItIT => "Italiano",
            Locale::PtBR => "Português (Brasil)",
            Locale::RuRU => "Русский",
        }
    }

    pub fn rtl(&self) -> bool {
        // Add RTL languages here if needed (Arabic, Hebrew, etc.)
        false
    }
}

/// Translation value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TranslationValue {
    Text(String),
    Plural {
        zero: Option<String>,
        one: String,
        few: Option<String>,
        many: Option<String>,
        other: String,
    },
}

/// Translation messages
pub type Messages = HashMap<String, TranslationValue>;

/// Translation catalog
pub struct TranslationCatalog {
    locales: HashMap<Locale, Messages>,
}

impl Default for TranslationCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationCatalog {
    pub fn new() -> Self {
        TranslationCatalog {
            locales: HashMap::new(),
        }
    }

    pub fn add_locale(mut self, locale: Locale, messages: Messages) -> Self {
        self.locales.insert(locale, messages);
        self
    }

    pub fn get(&self, locale: Locale, key: &str) -> Option<&TranslationValue> {
        self.locales.get(&locale)?.get(key)
    }

    pub fn merge(&mut self, other: TranslationCatalog) {
        for (locale, messages) in other.locales {
            self.locales
                .entry(locale)
                .or_default()
                .extend(messages);
        }
    }
}

/// i18n context
pub struct I18nContext {
    current_locale: Rc<RefCell<Locale>>,
    catalog: Rc<TranslationCatalog>,
    fallback_locale: Locale,
}

impl I18nContext {
    pub fn new(catalog: TranslationCatalog) -> Self {
        // Detect browser locale
        let browser_locale = detect_browser_locale().unwrap_or(Locale::EnUS);

        I18nContext {
            current_locale: Rc::new(RefCell::new(browser_locale)),
            catalog: Rc::new(catalog),
            fallback_locale: Locale::EnUS,
        }
    }

    pub fn locale(&self) -> Locale {
        *self.current_locale.borrow()
    }

    pub fn set_locale(&self, locale: Locale) {
        *self.current_locale.borrow_mut() = locale;

        // Persist to localStorage
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item("layer9-locale", locale.code());
        }

        // Update document lang attribute
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(html) = document.document_element() {
                let _ = html.set_attribute("lang", locale.code());
            }
        }
    }

    pub fn t(&self, key: &str) -> String {
        self.translate(key, None)
    }

    pub fn translate(&self, key: &str, args: Option<&HashMap<String, String>>) -> String {
        let locale = self.locale();

        // Try current locale
        if let Some(value) = self.catalog.get(locale, key) {
            return self.format_translation(value, args);
        }

        // Try fallback locale
        if let Some(value) = self.catalog.get(self.fallback_locale, key) {
            return self.format_translation(value, args);
        }

        // Return key if not found
        key.to_string()
    }

    pub fn plural(&self, key: &str, count: i32, args: Option<&HashMap<String, String>>) -> String {
        let locale = self.locale();

        if let Some(value) = self.catalog.get(locale, key) {
            match value {
                TranslationValue::Plural {
                    zero,
                    one,
                    few,
                    many,
                    other,
                } => {
                    let text = match count {
                        0 if zero.is_some() => zero.as_ref().unwrap(),
                        1 => one,
                        2..=4 if few.is_some() => few.as_ref().unwrap(),
                        _ if count > 20 && many.is_some() => many.as_ref().unwrap(),
                        _ => other,
                    };

                    let mut final_args = args.cloned().unwrap_or_default();
                    final_args.insert("count".to_string(), count.to_string());

                    self.interpolate(text, &final_args)
                }
                TranslationValue::Text(text) => {
                    let mut final_args = args.cloned().unwrap_or_default();
                    final_args.insert("count".to_string(), count.to_string());

                    self.interpolate(text, &final_args)
                }
            }
        } else {
            format!("{} ({})", key, count)
        }
    }

    fn format_translation(
        &self,
        value: &TranslationValue,
        args: Option<&HashMap<String, String>>,
    ) -> String {
        match value {
            TranslationValue::Text(text) => {
                if let Some(args) = args {
                    self.interpolate(text, args)
                } else {
                    text.clone()
                }
            }
            TranslationValue::Plural { other, .. } => other.clone(),
        }
    }

    fn interpolate(&self, text: &str, args: &HashMap<String, String>) -> String {
        let mut result = text.to_string();

        for (key, value) in args {
            result = result.replace(&format!("{{{}}}", key), value);
            result = result.replace(&format!("{{{{ {} }}}}", key), value);
        }

        result
    }
}

thread_local! {
    static I18N: RefCell<Option<I18nContext>> = const { RefCell::new(None) };
}

/// Initialize i18n
pub fn init_i18n(catalog: TranslationCatalog) {
    let ctx = I18nContext::new(catalog);
    I18N.with(|i18n| {
        *i18n.borrow_mut() = Some(ctx);
    });
}

/// i18n hook
pub fn use_i18n() -> I18n {
    // First check if initialized
    let locale = I18N.with(|i18n| {
        let borrowed = i18n.borrow();
        borrowed.as_ref().map(|ctx| ctx.locale()).unwrap_or_else(|| {
            panic!("i18n not initialized. Call init_i18n() first.");
        })
    });
    
    I18n {
        locale,
        set_locale: Box::new(move |locale| {
            I18N.with(|i18n| {
                if let Some(ctx) = i18n.borrow().as_ref() {
                    ctx.set_locale(locale);
                }
            });
        }),
        t: Box::new(move |key| {
            I18N.with(|i18n| {
                i18n.borrow().as_ref().map(|ctx| ctx.t(key)).unwrap_or_else(|| key.to_string())
            })
        }),
        translate: Box::new(move |key, args| {
            I18N.with(|i18n| {
                i18n.borrow().as_ref().map(|ctx| ctx.translate(key, args)).unwrap_or_else(|| key.to_string())
            })
        }),
        plural: Box::new(move |key, count, args| {
            I18N.with(|i18n| {
                i18n.borrow().as_ref().map(|ctx| ctx.plural(key, count, args)).unwrap_or_else(|| key.to_string())
            })
        }),
    }
}

/// i18n hook result
pub struct I18n {
    pub locale: Locale,
    pub set_locale: SetLocaleFn,
    pub t: SimpleTFn,
    pub translate: TranslateFn,
    pub plural: PluralFn,
}

/// Detect browser locale
fn detect_browser_locale() -> Option<Locale> {
    // Check localStorage first
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        if let Ok(Some(stored)) = storage.get_item("layer9-locale") {
            if let Some(locale) = Locale::from_code(&stored) {
                return Some(locale);
            }
        }
    }

    // Check navigator language
    if let Some(window) = web_sys::window() {
        if let Some(navigator) = window.navigator().language() {
            if let Some(locale) = Locale::from_code(&navigator) {
                return Some(locale);
            }
        }
    }

    None
}

/// Number formatting
#[allow(dead_code)]
pub struct NumberFormat {
    locale: Locale,
}

impl NumberFormat {
    pub fn new(locale: Locale) -> Self {
        NumberFormat { locale }
    }

    pub fn format(&self, number: f64) -> String {
        // Use Intl.NumberFormat
        format!("{:.2}", number) // Simplified for now
    }

    pub fn currency(&self, amount: f64, currency: &str) -> String {
        match currency {
            "USD" => format!("${:.2}", amount),
            "EUR" => format!("€{:.2}", amount),
            "GBP" => format!("£{:.2}", amount),
            "JPY" => format!("¥{:.0}", amount),
            "CNY" => format!("¥{:.2}", amount),
            "KRW" => format!("₩{:.0}", amount),
            _ => format!("{} {:.2}", currency, amount),
        }
    }

    pub fn percent(&self, value: f64) -> String {
        format!("{:.1}%", value * 100.0)
    }
}

/// Date/time formatting
#[allow(dead_code)]
pub struct DateTimeFormat {
    locale: Locale,
}

impl DateTimeFormat {
    pub fn new(locale: Locale) -> Self {
        DateTimeFormat { locale }
    }

    pub fn date(&self, timestamp: f64) -> String {
        // Use Intl.DateTimeFormat
        let date = js_sys::Date::new(&JsValue::from_f64(timestamp));
        date.to_locale_date_string(self.locale.code(), &JsValue::NULL).into()
    }

    pub fn time(&self, timestamp: f64) -> String {
        let date = js_sys::Date::new(&JsValue::from_f64(timestamp));
        date.to_locale_time_string(self.locale.code()).into()
    }

    pub fn date_time(&self, timestamp: f64) -> String {
        let date = js_sys::Date::new(&JsValue::from_f64(timestamp));
        date.to_locale_string(self.locale.code(), &JsValue::NULL).into()
    }

    pub fn relative(&self, timestamp: f64) -> String {
        let now = js_sys::Date::now();
        let diff = (now - timestamp) / 1000.0; // seconds

        match diff {
            d if d < 60.0 => "just now".to_string(),
            d if d < 3600.0 => format!("{} minutes ago", (d / 60.0) as i32),
            d if d < 86400.0 => format!("{} hours ago", (d / 3600.0) as i32),
            d if d < 604800.0 => format!("{} days ago", (d / 86400.0) as i32),
            _ => self.date(timestamp),
        }
    }
}

/// Format hooks
pub fn use_number_format() -> NumberFormat {
    let i18n = use_i18n();
    NumberFormat::new(i18n.locale)
}

pub fn use_date_format() -> DateTimeFormat {
    let i18n = use_i18n();
    DateTimeFormat::new(i18n.locale)
}

/// Translation macros
#[macro_export]
macro_rules! t {
    ($key:expr) => {{
        use_i18n().t($key)
    }};
    ($key:expr, $($arg_name:ident = $arg_value:expr),*) => {{
        let mut args = std::collections::HashMap::new();
        $(
            args.insert(stringify!($arg_name).to_string(), $arg_value.to_string());
        )*
        use_i18n().translate($key, Some(&args))
    }};
}

#[macro_export]
macro_rules! plural {
    ($key:expr, $count:expr) => {{
        use_i18n().plural($key, $count, None)
    }};
    ($key:expr, $count:expr, $($arg_name:ident = $arg_value:expr),*) => {{
        let mut args = std::collections::HashMap::new();
        $(
            args.insert(stringify!($arg_name).to_string(), $arg_value.to_string());
        )*
        use_i18n().plural($key, $count, Some(&args))
    }};
}

/// Example translations builder
pub struct TranslationsBuilder;

impl TranslationsBuilder {
    pub fn en_us() -> Messages {
        let mut messages = HashMap::new();

        messages.insert(
            "app.title".to_string(),
            TranslationValue::Text("My Application".to_string()),
        );

        messages.insert(
            "welcome.message".to_string(),
            TranslationValue::Text("Welcome, {name}!".to_string()),
        );

        messages.insert(
            "items.count".to_string(),
            TranslationValue::Plural {
                zero: Some("No items".to_string()),
                one: "1 item".to_string(),
                few: None,
                many: None,
                other: "{count} items".to_string(),
            },
        );

        messages
    }

    pub fn ko_kr() -> Messages {
        let mut messages = HashMap::new();

        messages.insert(
            "app.title".to_string(),
            TranslationValue::Text("내 애플리케이션".to_string()),
        );

        messages.insert(
            "welcome.message".to_string(),
            TranslationValue::Text("{name}님, 환영합니다!".to_string()),
        );

        messages.insert(
            "items.count".to_string(),
            TranslationValue::Text("항목 {count}개".to_string()),
        );

        messages
    }
}
