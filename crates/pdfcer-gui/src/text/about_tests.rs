//! Tests of `text::about` that read the application crate.

#[cfg(test)]
mod tests {
    /// **The build stamp is present and is not a placeholder.**
    #[test]
    fn the_build_stamp_is_populated() {
        let stamp = env!("PDFCER_BUILD_TIME");
        assert!(!stamp.trim().is_empty(), "build.rs set no build time");
        assert!(
            stamp.contains('-') && stamp.contains(':'),
            "a build stamp should carry a date and a time, got {stamp:?}"
        );
        let rev = env!("PDFCER_GUI_REV");
        assert!(!rev.trim().is_empty(), "build.rs set no revision");
    }

    /// The engine is REALLY named, because this is the row that identifies
    /// which pdfcer is inside a given executable.
    #[test]
    fn the_engine_reports_a_version_and_a_revision() {
        assert!(
            !env!("PDFCER_ENGINE_VERSION").is_empty(),
            "the engine version was not read out of Cargo.lock"
        );
        assert!(
            !env!("PDFCER_ENGINE_REV").is_empty(),
            "the engine revision was not read out of Cargo.lock"
        );
    }

    /// A component line reads as a sentence with and without a commit date.
    #[test]
    fn a_component_line_survives_a_missing_commit_date() {
        let with = component_line("pdfcer", "0.7.0", "6af5655", "2026-08-18 14:02");
        assert!(with.contains("0.7.0") && with.contains("6af5655"));
        assert!(with.contains("committed 2026-08-18 14:02"));

        let without = component_line("pdfcer", "0.7.0", "6af5655", "");
        assert!(
            !without.contains("committed"),
            "an empty date must leave the word out too, got {without:?}"
        );
        assert!(without.ends_with("6af5655"));
    }

    /// An absent component says so, and says it about itself.
    #[test]
    fn an_absent_component_names_itself() {
        let line = component_absent("iccce");
        assert!(line.starts_with("iccce"));
        assert!(line.contains("not in this build"));
    }

    use crate::text::about::*;
    use std::path::PathBuf;

    /// The shipped notice file, read from the repository root.
    fn notice() -> String {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        std::fs::read_to_string(root.join("THIRD_PARTY_LICENSES.md"))
            .expect("THIRD_PARTY_LICENSES.md must exist at the workspace root")
    }

    /// Every field of every attribution is populated.
    #[test]
    fn every_attribution_says_all_five_things() {
        let list = attributions();
        assert!(
            !list.is_empty(),
            "an empty attribution list would make this test vacuous, and the dialog pointless"
        );
        for a in list {
            assert!(!a.component.is_empty(), "component is empty");
            assert!(
                !a.creator.is_empty(),
                "creator is empty for {}",
                a.component
            );
            assert!(!a.origin.is_empty(), "origin is empty for {}", a.component);
            assert!(
                !a.licence.is_empty(),
                "licence is empty for {}",
                a.component
            );
            assert!(
                a.changes.ends_with('.'),
                "the indication of changes is a statement and needs its punctuation: {:?}",
                a.changes
            );
        }
    }

    /// The dialog and the shipped notice file cannot disagree.
    #[test]
    fn the_shipped_notice_carries_every_attribution_this_dialog_makes() {
        let notice = notice();
        for a in attributions() {
            assert!(
                notice.contains(a.licence),
                "{} is attributed to {} under {} in the About dialog, and \
                 THIRD_PARTY_LICENSES.md never mentions that licence. \
                 Regenerate it: cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md",
                a.component,
                a.creator,
                a.licence
            );
        }
    }

    /// The notice file carries the licence texts, not just their names.
    #[test]
    fn the_shipped_notice_is_a_real_notice_and_not_a_stub() {
        let notice = notice();
        assert!(
            notice.len() > 100_000,
            "THIRD_PARTY_LICENSES.md is {} bytes, which is too small to be \
             carrying the licence texts of the crates this binary links",
            notice.len()
        );
        assert!(
            notice.contains("Full licence texts"),
            "the generated notice is missing its licence-text section"
        );
    }

    /// pdfcer's own licence line agrees with the file that ships beside it.
    #[test]
    fn the_licence_line_matches_the_shipped_licence_file() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        let licence = std::fs::read_to_string(root.join("LICENSE")).expect("LICENSE");
        assert!(licence.contains("MIT License"), "LICENSE is not MIT");
        assert!(
            licence.contains("Copyright (c) 2026 Ken Mantle"),
            "LICENSE's copyright line has changed; about's licence_line() must follow it"
        );
        assert!(licence_line().contains("MIT"));
        assert!(licence_line().contains("Copyright (c) 2026 Ken Mantle"));
    }

    /// No two entries describe the same work.
    #[test]
    fn no_two_attributions_share_a_component() {
        let list = attributions();
        for (i, a) in list.iter().enumerate() {
            for b in &list[i + 1..] {
                assert_ne!(
                    a.component, b.component,
                    "two entries describe the same work"
                );
            }
        }
    }
}
