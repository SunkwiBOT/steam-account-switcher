use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankKind {
    Tiered,
    Rating,
}

impl RankKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RankKind::Tiered => "tiered",
            RankKind::Rating => "rating",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankEntry {
    pub value: String,
    /// Icon file stem inside the template folder ("diamond"), empty when the
    /// ladder has no artwork for that tier.
    pub family: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankTemplate {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: RankKind,
    pub ranks: Vec<RankEntry>,
    pub max_rating: Option<u32>,
}

pub const DEFAULT_TEMPLATE: &str = "marvel-rivals";

pub const MAX_PREMIER_RATING: u32 = 50_000;

pub const UNRANKED: &str = "Unranked";

fn marvel_ranks() -> Vec<RankEntry> {
    let mut ranks = vec![unranked()];
    for family in [
        "Bronze",
        "Silver",
        "Gold",
        "Platinum",
        "Diamond",
        "Grandmaster",
        "Celestial",
    ] {
        for division in [3, 2, 1] {
            ranks.push(entry(
                &format!("{family} {division}"),
                &family.to_lowercase(),
            ));
        }
    }
    ranks.push(entry("Eternity", "eternity"));
    ranks.push(entry("Top 500", "one-above-all"));
    ranks
}

fn overwatch_ranks() -> Vec<RankEntry> {
    let mut ranks = vec![unranked()];
    for family in [
        "Bronze",
        "Silver",
        "Gold",
        "Platinum",
        "Emerald",
        "Diamond",
        "Grandmaster",
    ] {
        for division in [5, 4, 3, 2, 1] {
            ranks.push(entry(
                &format!("{family} {division}"),
                &family.to_lowercase(),
            ));
        }
    }
    ranks.push(entry("Top 500", "top-500"));
    ranks
}

fn cs2_ranks() -> Vec<RankEntry> {
    let mut ranks = vec![unranked()];
    for (label, family) in [
        ("Silver I", "silver-1"),
        ("Silver II", "silver-2"),
        ("Silver III", "silver-3"),
        ("Silver IV", "silver-4"),
        ("Silver Elite", "silver-elite"),
        ("Silver Elite Master", "silver-elite-master"),
        ("Gold Nova I", "gold-nova-1"),
        ("Gold Nova II", "gold-nova-2"),
        ("Gold Nova III", "gold-nova-3"),
        ("Gold Nova Master", "gold-nova-master"),
        ("Master Guardian I", "master-guardian-1"),
        ("Master Guardian II", "master-guardian-2"),
        ("Master Guardian Elite", "master-guardian-elite"),
        (
            "Distinguished Master Guardian",
            "distinguished-master-guardian",
        ),
        ("Legendary Eagle", "legendary-eagle"),
        ("Legendary Eagle Master", "legendary-eagle-master"),
        ("Supreme Master First Class", "supreme-master-first-class"),
        ("Global Elite", "global-elite"),
    ] {
        ranks.push(entry(label, family));
    }
    ranks
}

fn unranked() -> RankEntry {
    entry(UNRANKED, "")
}

fn entry(value: &str, family: &str) -> RankEntry {
    RankEntry {
        value: value.to_string(),
        family: family.to_string(),
    }
}

pub fn templates() -> &'static [RankTemplate] {
    static TEMPLATES: OnceLock<Vec<RankTemplate>> = OnceLock::new();
    TEMPLATES.get_or_init(|| {
        vec![
            RankTemplate {
                id: DEFAULT_TEMPLATE,
                label: "Marvel Rivals",
                kind: RankKind::Tiered,
                ranks: marvel_ranks(),
                max_rating: None,
            },
            RankTemplate {
                id: "overwatch",
                label: "Overwatch",
                kind: RankKind::Tiered,
                ranks: overwatch_ranks(),
                max_rating: None,
            },
            RankTemplate {
                id: "cs2",
                label: "Counter-Strike 2",
                kind: RankKind::Tiered,
                ranks: cs2_ranks(),
                max_rating: None,
            },
            RankTemplate {
                id: "cs2-premier",
                label: "Counter-Strike 2 (Premier)",
                kind: RankKind::Rating,
                ranks: Vec::new(),
                max_rating: Some(MAX_PREMIER_RATING),
            },
        ]
    })
}

pub fn template(id: &str) -> Option<&'static RankTemplate> {
    templates().iter().find(|template| template.id == id)
}

pub fn normalize(template_id: &str, value: &str) -> String {
    let resolved = collapse(value);

    let Some(template) = template(template_id) else {
        return UNRANKED.to_string();
    };

    match template.kind {
        RankKind::Tiered => template
            .ranks
            .iter()
            .find(|entry| entry.value.eq_ignore_ascii_case(&resolved))
            .map(|entry| entry.value.clone())
            .unwrap_or_else(|| UNRANKED.to_string()),
        RankKind::Rating => match rating(&resolved) {
            Some(rating) => rating.to_string(),
            None => UNRANKED.to_string(),
        },
    }
}

pub fn family(template_id: &str, value: &str) -> String {
    template(template_id)
        .and_then(|template| {
            template
                .ranks
                .iter()
                .find(|entry| entry.value.eq_ignore_ascii_case(value))
        })
        .map(|entry| entry.family.clone())
        .unwrap_or_default()
}

pub fn rating(value: &str) -> Option<u32> {
    let digits: String = value
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return None;
    }
    digits
        .parse::<u32>()
        .ok()
        .map(|number| number.min(MAX_PREMIER_RATING))
}

fn collapse(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
