//! Rank ladders, one per game. An account stores a template id and a value
//! ("Diamond 3", or "21340" for the CS2 Premier rating).

use std::sync::OnceLock;

/// How the values of a template are entered and shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankKind {
    /// A list of named ranks ("Global Elite", "Platinum 3").
    Tiered,
    /// A number the player types in, as in CS2 Premier (0 to 50 000).
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

/// One selectable rank of a tiered template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankEntry {
    /// Stored value and shown label ("Diamond 3").
    pub value: String,
    /// Icon file stem inside the template folder ("diamond"), empty when the
    /// ladder has no artwork for that tier.
    pub family: String,
}

/// A rank ladder: Marvel Rivals, Overwatch, CS2 competitive or CS2 Premier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankTemplate {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: RankKind,
    pub ranks: Vec<RankEntry>,
    /// Highest rating accepted by a `Rating` template.
    pub max_rating: Option<u32>,
}

/// The ladder used when an account has never been given a template, which is
/// also the one the previous versions of this application stored.
pub const DEFAULT_TEMPLATE: &str = "marvel-rivals";

/// CS2 Premier ratings run from 0 to 50 000.
pub const MAX_PREMIER_RATING: u32 = 50_000;

/// Unranked is available everywhere and is what an unknown value falls back to.
pub const UNRANKED: &str = "Unranked";

/// Old spellings kept for compatibility with the previous rank list.
const RANK_ALIASES: [(&str, &str); 7] = [
    ("Platine 3", "Platinum 3"),
    ("Platine 2", "Platinum 2"),
    ("Platine 1", "Platinum 1"),
    ("Grand Master 3", "Grandmaster 3"),
    ("Grand Master 2", "Grandmaster 2"),
    ("Grand Master 1", "Grandmaster 1"),
    ("One Above All", "Top 500"),
];

/// Marvel Rivals: seven tiers with three divisions each, then Eternity and the
/// Top 500 (One Above All).
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

/// Overwatch: seven tiers with five divisions, then Top 500.
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

/// Counter-Strike 2 competitive: the eighteen classic ranks.
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

/// Every ladder the interface offers, in menu order.
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

/// Rank of `value` inside `template_id`, falling back to Unranked.
pub fn normalize(template_id: &str, value: &str) -> String {
    let collapsed = collapse(value);
    let resolved = RANK_ALIASES
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(&collapsed))
        .map(|(_, canonical)| (*canonical).to_string())
        .unwrap_or(collapsed);

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

/// Icon file stem for a stored value, empty when the tier has no artwork.
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

/// Parses a Premier rating, clamped to the accepted range.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_template_is_a_full_ladder() {
        for template in templates() {
            match template.kind {
                RankKind::Tiered => {
                    assert_eq!(template.ranks[0].value, UNRANKED, "{}", template.id);
                    assert!(
                        template.ranks.len() > 10,
                        "{} should list a full ladder",
                        template.id
                    );
                }
                RankKind::Rating => assert!(template.ranks.is_empty()),
            }
        }
    }

    #[test]
    fn ladders_are_unique_and_ordered() {
        for template in templates() {
            let mut seen = HashSet::new();
            for entry in &template.ranks {
                assert!(
                    seen.insert(entry.value.clone()),
                    "duplicate {} in {}",
                    entry.value,
                    template.id
                );
            }
        }

        let cs2 = template("cs2").expect("cs2 template");
        assert_eq!(cs2.ranks[1].value, "Silver I");
        assert_eq!(cs2.ranks.last().expect("last rank").value, "Global Elite");
        assert_eq!(cs2.ranks.len(), 19, "Unranked plus the 18 ranks");
    }

    #[test]
    fn unknown_values_fall_back_to_unranked() {
        assert_eq!(normalize("marvel-rivals", "nonsense"), UNRANKED);
        assert_eq!(normalize("does-not-exist", "Gold 3"), UNRANKED);
        assert_eq!(normalize("cs2", "Global Elite"), "Global Elite");
    }

    #[test]
    fn old_spellings_and_spacing_are_normalised() {
        assert_eq!(normalize("marvel-rivals", "  Diamond   2 "), "Diamond 2");
        assert_eq!(normalize("marvel-rivals", "One Above All"), "Top 500");
        assert_eq!(normalize("marvel-rivals", "Platine 1"), "Platinum 1");
    }

    #[test]
    fn rating_values_are_clamped_to_the_premier_range() {
        assert_eq!(normalize("cs2-premier", "21 340"), "21340");
        assert_eq!(normalize("cs2-premier", "999999"), "50000");
        assert_eq!(normalize("cs2-premier", "unranked"), UNRANKED);
        assert_eq!(rating("0"), Some(0));
    }

    #[test]
    fn families_match_the_asset_names() {
        assert_eq!(family("marvel-rivals", "Diamond 3"), "diamond");
        assert_eq!(family("marvel-rivals", "Top 500"), "one-above-all");
        assert_eq!(family("overwatch", "Emerald 4"), "emerald");
        assert_eq!(family("cs2", "Global Elite"), "global-elite");
        assert_eq!(family("cs2-premier", "21340"), "");
    }
}
