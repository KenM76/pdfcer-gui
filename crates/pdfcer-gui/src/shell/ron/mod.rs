//! # shell::ron — the built-in manifest, on disk
//!
//! [`built_in_ron`] is the text of `built_in.ron`, compiled in with
//! `include_str!`, and [`parse_built_in`] is that text parsed back into an
//! `egui_shell::Shell`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/ron/mod.md`.
// The operator-customization path, not yet wired into start-up.
//
// `PdfcerApp::new` merges only the built-in layer today, so nothing calls
// these at runtime. They are exercised by this module's own tests, which is
// what keeps `built_in.ron` honest — a drift between the RON and the Rust
// fails the suite. The runtime consumer arrives at **S3**, when layout and
// manifest persistence land and the three-layer merge (built-in →
// application override → operator) gets its outer two layers.
//
// `allow` rather than deletion because deleting them would delete the
// round-trip test with them, and that test is the only thing proving the
// format a customizing operator will hand-edit actually parses.
#![allow(dead_code)]

use egui_shell::Shell;
use egui_shell::manifest::ManifestError;

/// The built-in manifest as RON text.
///
/// Compiled in rather than read at run time: this is the **built-in
/// layer**, the one that is always available as the reset target and can
/// never be missing or malformed on an operator's machine. A layer read
/// from disk is layer two or three.
#[must_use]
pub fn built_in_ron() -> &'static str {
    include_str!("built_in.ron")
}

/// Parse the built-in manifest from its RON text.
///
/// # Errors
///
/// [`ManifestError::Parse`], carrying RON's line and column. Unreachable
/// in a shipped build — the text is compiled in and a test parses it — but
/// returned rather than unwrapped so that the same function can be pointed
/// at an operator's file by a tool that wants the span.
pub fn parse_built_in() -> Result<Shell, ManifestError> {
    Shell::from_ron(built_in_ron())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::manifest;

    /// **The `.ron` file and `manifest::built_in()` are the same shell.**
    #[test]
    fn the_ron_file_and_the_rust_agree() {
        let from_file = parse_built_in().expect("the checked-in manifest must parse");
        assert_eq!(
            from_file,
            manifest::built_in(),
            "built_in.ron is out of date. Regenerate it with:\n    \
             cargo test -p pdfcer-gui rewrite_built_in_ron -- --ignored"
        );
    }

    /// The checked-in file is a *complete, valid* manifest on its own.
    #[test]
    fn the_ron_file_is_a_complete_manifest() {
        parse_built_in()
            .expect("parses")
            .validate()
            .expect("the built-in layer must validate on its own");
    }

    /// **A hand-written snippet parses — the `IMPLICIT_SOME` check.**
    #[test]
    fn a_hand_written_snippet_parses() {
        let typed_by_a_person = r#"
            Shell(
                // Put Tools first, and give the batch commands a chord.
                tabs: [
                    Tab(id: "tools"),
                    Tab(id: "file", label: "Document", groups: [
                        Group(id: "file", caption: "Document", items: [
                            Command(id: "file.open"),
                            Separator,
                            Command(id: "file.close"),
                        ]),
                    ]),
                ],
                keymap: { "Ctrl+B": "tools.merge_files" },
            )
        "#;

        let layer = Shell::from_ron(typed_by_a_person).expect(
            "a hand-written layer must parse — no Some() wrappers, comments and trailing \
             commas allowed. If this fails with `ExpectedOption`, the IMPLICIT_SOME \
             extension has been lost from egui-shell's ron::Options.",
        );

        assert_eq!(layer.tabs().len(), 2);
        assert_eq!(layer.tabs()[0].id, "tools");
        assert!(
            layer.tabs()[0].groups.is_none(),
            "a bare `Tab(id: …)` is a REFERENCE to a tab — used to reorder it — and must \
             not come back as an instruction to empty it"
        );
        assert_eq!(layer.tabs()[1].label.as_deref(), Some("Document"));
        assert_eq!(
            layer.keymap.as_ref().and_then(|k| k.get("Ctrl+B")),
            Some("tools.merge_files")
        );
    }

    /// **No `Some(` appears anywhere in the file.**
    #[test]
    fn the_generated_file_carries_no_option_wrappers() {
        assert!(
            !built_in_ron().contains("Some("),
            "the generated manifest must not be full of Option wrappers — that is the \
             difference between a file an operator can edit and one they cannot"
        );
    }

    /// The file is recognisably the ribbon when read by a person.
    #[test]
    fn the_ron_file_reads_as_a_ribbon() {
        let text = built_in_ron();
        for needle in [
            // Was `Command(id: "file.open")` until 2026-09-04. `file.open`
            // is now a **Large** item — the mockup draws it as one of the File
            // group's two big controls — so it serializes with its size and no
            // longer matches a needle that was really asserting *"a
            // default-sized command elides its size"*.
            //
            // Both halves of that property are now asserted, which is stronger
            // than what was here before: `file.new_from_template` is the plain
            // form (size omitted because `Medium` is the default) and
            // `file.new` is the qualified one. A serializer that started
            // emitting `size: Medium` everywhere, or that stopped emitting
            // `size:` at all, fails on one needle or the other rather than
            // slipping past a single example.
            "Command(id: \"file.new_from_template\")",
            "Command(id: \"file.new\", size: Large)",
            "caption: \"Page display\"",
            "id: \"review\"",
            "\"Ctrl+1\": \"mode.read\"",
            // The contextual Format tab's condition, and it is
            // `selection.formattable` rather than `selection.any` since
            // 2026-08-27 — the tab now carries controls for two kinds of
            // selection, so its condition is the union rather than either
            // operand. Kept in this list because the needle it is here to
            // prove is *"a condition round-trips into the file legibly"*, and
            // that is exactly as true of the new name.
            "visible_when: \"selection.formattable\"",
            "kind: \"colour_swatch\"",
            "Custom(kind: \"font_face\", visible_when: \"mode.edit_content\")",
        ] {
            assert!(text.contains(needle), "the file should contain {needle}");
        }
    }

    /// Rewrite `built_in.ron` from the Rust manifest.
    ///
    /// Not a test — a generator that lives here because it needs the same
    /// types and the same path. Ignored so `cargo test` never modifies the
    /// source tree; run it deliberately when the manifest changes:
    ///
    /// ```text
    /// cargo test -p pdfcer-gui rewrite_built_in_ron -- --ignored
    /// ```
    ///
    /// `CARGO_MANIFEST_DIR` rather than a relative path because the
    /// working directory of a test binary is the workspace root under
    /// `cargo test` and the crate root under some IDE runners, and writing
    /// the file to whichever one happened to be current is how a
    /// regenerated manifest ends up somewhere nobody looks.
    #[test]
    #[ignore = "generator: writes to the source tree; run deliberately"]
    fn rewrite_built_in_ron() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("shell")
            .join("ron")
            .join("built_in.ron");
        let text = manifest::built_in()
            .to_ron_pretty()
            .expect("the manifest serializes");
        std::fs::write(&path, text).expect("the source tree is writable");
        // Prove the file that was just written is the one the tests will
        // read: a generator that emits something its own parser rejects
        // would otherwise be discovered on the next run, by someone else.
        let written = std::fs::read_to_string(&path).expect("readable");
        assert_eq!(
            Shell::from_ron(&written).expect("the generated file parses"),
            manifest::built_in()
        );
    }
}
