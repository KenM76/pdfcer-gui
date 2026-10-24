//! # `text::structlinks` — what a page copy did with links it could not keep as they were
//!
//! A tagged source's structure tree never comes across with its pages
//! (`InsertOutcome::source_tags_dropped`, `MergeOutcome::source_tags_dropped`,
//! `AssembleReport::struct_tree_dropped`). The engine re-points every link
//! that named a structure element at the page that element marks, or at the
//! source's first page when it marks none (ISO 32000-2 §12.3.2.3), and drops
//! the destination of a link to a page that was not copied. Each is a change
//! to what a click on the copied pages does, so each is said.

/// The sentence for one copy; `None` when the source was untagged and every
/// link arrived as it was.
#[must_use]
pub fn repointed(
    to_pages: usize,
    to_first: usize,
    left_behind: usize,
    tags_dropped: bool,
) -> Option<String> {
    let mut parts = Vec::new();
    if tags_dropped {
        parts.push(
            "The copied pages came from a tagged PDF; its tags did not come across, so they \
             are untagged here."
                .to_owned(),
        );
    }
    if to_pages > 0 {
        let mut line = format!(
            "{to_pages} link(s) that pointed into those tags now go to the page they marked"
        );
        if to_first > 0 {
            line.push_str(&format!(
                "; {to_first} marked no page and now go to the first page of the file they came from"
            ));
        }
        line.push('.');
        parts.push(line);
    }
    if left_behind > 0 {
        parts.push(format!(
            "{left_behind} link(s) to pages that were not copied lost their destination."
        ));
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::repointed;

    #[test]
    fn an_untagged_copy_with_every_link_kept_says_nothing() {
        assert_eq!(repointed(0, 0, 0, false), None);
    }

    #[test]
    fn re_pointed_links_name_the_first_page_fallback() {
        let line = repointed(3, 1, 0, true).unwrap_or_default();
        assert!(
            line.starts_with("The copied pages came from a tagged PDF"),
            "{line}"
        );
        assert!(
            line.contains("3 link(s) that pointed into those tags"),
            "{line}"
        );
        assert!(line.contains("1 marked no page"), "{line}");
    }

    #[test]
    fn a_link_left_behind_is_said_without_tags() {
        let line = repointed(0, 0, 2, false).unwrap_or_default();
        assert_eq!(
            line,
            "2 link(s) to pages that were not copied lost their destination."
        );
    }
}
