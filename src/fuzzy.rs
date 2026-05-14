//! fzf-style fuzzy matching, wrapping [`nucleo_matcher`]. Used to filter
//! interactive `inquire` prompts and to rank tab-completion suggestions.
//!
//! We go through `nucleo_matcher::pattern::Pattern` rather than the raw
//! matcher because the pattern API handles case folding and unicode
//! normalization on the needle — which the low-level matcher leaves to the
//! caller. `CaseMatching::Smart` matches fzf's default (case-insensitive
//! unless the needle has uppercase).

use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32Str,
};

fn build_matcher() -> Matcher {
    Matcher::new(Config::DEFAULT)
}

fn build_pattern(needle: &str) -> Pattern {
    Pattern::parse(needle, CaseMatching::Smart, Normalization::Smart)
}

fn score_one(matcher: &mut Matcher, pattern: &Pattern, haystack: &str) -> Option<u32> {
    let mut buf = Vec::new();
    let h = Utf32Str::new(haystack, &mut buf);
    pattern.score(h, matcher)
}

/// Whether `needle` fuzzy-matches `haystack`. Empty needles always match.
pub fn matches(needle: &str, haystack: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let pattern = build_pattern(needle);
    let mut matcher = build_matcher();
    score_one(&mut matcher, &pattern, haystack).is_some()
}

/// Sort `items` descending by their fuzzy score against `needle`, dropping
/// non-matches. `selector` returns the searchable text for each item.
pub fn sort_by_score<T, F>(needle: &str, items: Vec<T>, selector: F) -> Vec<T>
where
    F: Fn(&T) -> &str,
{
    if needle.is_empty() {
        return items;
    }
    let pattern = build_pattern(needle);
    let mut matcher = build_matcher();
    let mut scored: Vec<(u32, T)> = items
        .into_iter()
        .filter_map(|item| {
            let s = score_one(&mut matcher, &pattern, selector(&item))?;
            Some((s, item))
        })
        .collect();
    // Stable sort: equal-score items keep their original order.
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, t)| t).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_needle_matches_anything() {
        assert!(matches("", "build:dev"));
    }

    #[test]
    fn substring_still_matches() {
        assert!(matches("dev", "build:dev"));
    }

    #[test]
    fn non_contiguous_characters_match() {
        // The whole point of fuzzy — characters in order but not adjacent.
        assert!(matches("bld", "build:dev"));
        assert!(matches("bdv", "build:dev"));
    }

    #[test]
    fn smart_case() {
        // Lowercase needle → case-insensitive (matches anywhere).
        assert!(matches("dev", "BUILD:DEV"));
        assert!(matches("dev", "build:dev"));
        // Uppercase in needle → case-sensitive (fzf's smart-case rule).
        assert!(matches("DEV", "BUILD:DEV"));
        assert!(!matches("DEV", "build:dev"));
    }

    #[test]
    fn missing_character_rejects() {
        assert!(!matches("xyz", "build:dev"));
        assert!(!matches("zdev", "build:dev"));
    }

    #[test]
    fn ranks_tighter_matches_higher() {
        let items = vec![
            "build:dev".to_string(),
            "dev".to_string(),
            "build".to_string(),
        ];
        let ranked = sort_by_score("dev", items, |s| s.as_str());
        // "dev" (whole haystack matches) should outrank "build:dev".
        assert_eq!(ranked.first().unwrap(), "dev");
        // "build" has no 'v', so it should be dropped entirely.
        assert!(!ranked.iter().any(|s| s == "build"));
    }

    #[test]
    fn empty_needle_preserves_order() {
        let items = vec!["b".to_string(), "a".to_string(), "c".to_string()];
        let out = sort_by_score("", items.clone(), |s| s.as_str());
        assert_eq!(out, items);
    }
}
