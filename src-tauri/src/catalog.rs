// ! Generated at build time from `messages/*.json`, the same catalogue the interface uses, so there is a single translation source.

include!(concat!(env!("OUT_DIR"), "/catalog.rs"));

const FALLBACK: &str = "en";

fn lookup(locale: &str, key: &'static str) -> &'static str {
    CATALOG
        .iter()
        .find(|(tag, _)| *tag == locale)
        .and_then(|(_, entries)| entries.iter().find(|(name, _)| *name == key))
        .or_else(|| {
            CATALOG
                .iter()
                .find(|(tag, _)| *tag == FALLBACK)
                .and_then(|(_, entries)| entries.iter().find(|(name, _)| *name == key))
        })
        .map(|(_, text)| *text)
        .unwrap_or(key)
}

pub fn text(locale: &str, key: &'static str) -> &'static str {
    lookup(locale, key)
}

pub fn account_label(locale: &str, index: usize) -> String {
    lookup(locale, "tray_account").replacen("{index}", &index.to_string(), 1)
}
