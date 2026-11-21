//! # `text::insertoutline` — the bookmarks and named destinations an insert brought
//!
//! The sentence for `InsertOutcome::outline_items_carried`, `dests_carried` and
//! `dests_renamed`. A source whose bookmarks all stayed behind is worded by
//! `text::pages::inserted`.

/// What came across; `None` when nothing did, so a plain insert says nothing.
#[must_use]
pub fn carried(items: usize, dests: usize, renamed: usize) -> Option<String> {
    let mut parts = Vec::new();
    match items {
        0 => {}
        1 => parts
            .push("1 bookmark came across, at the end of this document's bookmarks.".to_owned()),
        n => parts.push(format!(
            "{n} bookmarks came across, at the end of this document's bookmarks."
        )),
    }
    if dests > 0 {
        let mut line = match dests {
            1 => "1 named destination came across".to_owned(),
            n => format!("{n} named destinations came across"),
        };
        match renamed {
            0 => {}
            1 => line.push_str("; 1 was renamed because this document already uses that name"),
            n => line.push_str(&format!(
                "; {n} were renamed because this document already uses those names"
            )),
        }
        line.push('.');
        parts.push(line);
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::carried;

    #[test]
    fn nothing_carried_says_nothing() {
        assert_eq!(carried(0, 0, 0), None);
    }

    #[test]
    fn bookmarks_and_renamed_destinations_are_both_said() {
        assert_eq!(
            carried(3, 2, 1).as_deref(),
            Some(
                "3 bookmarks came across, at the end of this document's bookmarks. 2 named \
                 destinations came across; 1 was renamed because this document already uses \
                 that name."
            )
        );
    }

    #[test]
    fn one_bookmark_is_singular() {
        assert_eq!(
            carried(1, 0, 0).as_deref(),
            Some("1 bookmark came across, at the end of this document's bookmarks.")
        );
    }
}
