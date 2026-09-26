//! # `findquery` — preparing a typed query for the engine
//!
//! One subject, one file: the single decision about what the operator typed
//! versus what is handed to `EditSession::search_text`. It exists because that
//! decision now has a preference attached to it and a disclosure that has to
//! agree with both, and three things that must agree are three things that
//! drift when they live in three files.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/findquery.md`.

/// **Does the raw query have whitespace at either end?**
#[must_use]
pub fn has_edge_whitespace(raw: &str) -> bool {
    raw.trim() != raw
}

/// **The needle the engine is given**, for a raw query and the current
/// preference.
#[must_use]
pub fn for_search(raw: &str, trim: bool) -> &str {
    if trim { raw.trim() } else { raw }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Excel paste, which is the case that was reported.
    #[test]
    fn a_trailing_space_is_removed_when_the_setting_is_on() {
        assert_eq!(for_search("TR-0180 ", true), "TR-0180");
        assert!(has_edge_whitespace("TR-0180 "));
    }

    /// The reader who MEANT the space keeps it, and is the reason this is a
    /// setting rather than an unconditional trim.
    #[test]
    fn the_setting_off_hands_the_query_over_exactly_as_typed() {
        assert_eq!(for_search(" lot ", false), " lot ");
    }

    /// Tabs and non-breaking spaces, because he wrote *"spaces/tabs/etc"* and a
    /// clipboard supplies all three.
    #[test]
    fn a_tab_and_a_non_breaking_space_count_as_whitespace() {
        assert!(has_edge_whitespace("part\t"));
        assert!(has_edge_whitespace("\u{a0}part"));
        assert_eq!(for_search("part\t", true), "part");
    }

    /// Interior whitespace survives, deliberately — see the module header.
    #[test]
    fn two_spaces_in_the_middle_are_left_alone() {
        assert_eq!(for_search("PART  NUMBER", true), "PART  NUMBER");
        assert!(!has_edge_whitespace("PART  NUMBER"));
    }

    /// A box holding one space is an empty query, not a failed search.
    #[test]
    fn a_query_that_is_all_whitespace_trims_to_nothing() {
        assert_eq!(for_search("   ", true), "");
    }
}
