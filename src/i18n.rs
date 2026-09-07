use std::collections::HashMap;

const LANG_EN_US: &str = include_str!("../lang/en_us.json");
const LANG_DE_DE: &str = include_str!("../lang/de_de.json");

pub type Lang = HashMap<String, String>;

pub fn detect_locale() -> &'static str {
    let env_lang = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_default();
    if !env_lang.is_empty() {
        return locale_from(&env_lang);
    }
    let conf = std::fs::read_to_string("/etc/locale.conf").unwrap_or_default();
    for line in conf.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "LANG" {
                return locale_from(value.trim());
            }
        }
    }
    "en_us"
}

fn locale_from(value: &str) -> &'static str {
    if value.to_ascii_lowercase().starts_with("de") {
        "de_de"
    } else {
        "en_us"
    }
}

pub fn load() -> Lang {
    let raw = match detect_locale() {
        "de_de" => LANG_DE_DE,
        _ => LANG_EN_US,
    };
    serde_json::from_str(raw).unwrap_or_default()
}

pub fn t(lang: &Lang, key: &str, vars: &[(&str, &str)]) -> String {
    let template = lang.get(key).map(String::as_str).unwrap_or(key);
    let mut out = template.to_string();
    for (name, value) in vars {
        out = out.replace(&format!("{{{}}}", name), value);
    }
    out
}
