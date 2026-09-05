//! # shell::ron — the built-in manifest, on disk
//!
//! [`built_in_ron`] is the text of `built_in.ron`, compiled in with
//! `include_str!`, and [`parse_built_in`] is that text parsed back into an
//! `egui_shell::Shell`.
//!
//! # Why a file exists at all when the Rust already builds the manifest
//!
//! Because otherwise the claim at the centre of `SHELL_FRAMEWORK.md` would
//! be untested. The claim is:
//!
//! > **The shell is data.** Tabs, groups, commands, panels, layouts, modes
//! > and key bindings are a serializable document that the application
//! > *supplies* and the operator *edits* — not code that has to be
//! > recompiled to change.
//!
//! A manifest that exists only as a `Shell` value built by a Rust function
//! satisfies the *types* and none of the point. The operator's
//! customization layer is a file; the application-override layer is a
//! file; a saved workspace is a file. If the format cannot express the
//! real ribbon — all eight tabs, thirty-three groups, three modes, the QAT
//! and twenty key bindings — then "the shell is data" is a description
//! of a data structure rather than of a product, and nobody finds out
//! until an operator opens `userdata/shell.ron` and it does not round
//! trip.
//!
//! So the built-in manifest is emitted as RON, checked in, and
//! [`the_ron_file_and_the_rust_agree`] asserts the two are the same shell.
//! That test is the proof, and it is deliberately an equality of *parsed
//! values* rather than of text: comparing strings would fail on
//! whitespace and would say nothing about whether the document means the
//! same thing.
//!
//! # ★ `IMPLICIT_SOME`, and why the round trip alone would not have caught
//! the defect
//!
//! `egui-shell` reads and writes with RON's `IMPLICIT_SOME` extension
//! enabled on the [`ron::Options`] used for **both** directions. Without
//! it, every `Option` field — which is nearly every field in the manifest,
//! because `None` is what "this layer does not mention this" means — has
//! to be written `tabs: Some([…])`, and the obvious spelling
//!
//! ```ron
//! Shell(tabs: [ Tab(id: "tools") ])
//! ```
//!
//! fails to parse with `ExpectedOption` — a message naming a Rust type the
//! operator has never heard of, at a position, with no hint that the fix
//! is four characters.
//!
//! **The trap is that a round-trip test cannot see this.** `to_string` →
//! `from_str` passes either way, because the serializer emits `Some(…)`
//! and the deserializer accepts it. The writer and the reader agree by
//! construction; the population that breaks is the one that never goes
//! through the writer — a file hand-authored from scratch, a snippet
//! pasted out of documentation, a customization one operator shared with
//! another.
//!
//! Hence [`a_hand_written_snippet_parses`], whose input is a string
//! literal written by a person and never produced by any serializer. That
//! is the only test in this module that is about the *format* rather than
//! about pdfcer's manifest, and it is the one that would have failed.
//!
//! # Regenerating the file
//!
//! `built_in.ron` is generated, not hand-maintained. When the manifest
//! changes, run the ignored test that rewrites it:
//!
//! ```text
//! cargo test -p pdfcer-gui rewrite_built_in_ron -- --ignored
//! ```
//!
//! and commit the result. Ignored rather than automatic because a test
//! that writes to the source tree as a side effect of `cargo test` makes
//! every run a potential working-copy change, and because the diff is the
//! most reviewable artefact this module produces: it is the ribbon,
//! stated as data, in a form that shows up in a pull request.
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

    /// **★ The `.ron` file and `manifest::built_in()` are the same shell.**
    ///
    /// The proof that the format is genuinely the manifest rather than a
    /// Rust-only fiction with a file beside it. Equality of parsed values,
    /// not of text — see the module header.
    ///
    /// When this fails it is almost always because the Rust changed and
    /// the file was not regenerated, so the message says how.
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
    ///
    /// Distinct from the equality test above, and not implied by it. A
    /// layer is not required to validate; the built-in layer is, because
    /// it is what every other layer patches and what a reset restores. If
    /// this file could only be understood as a diff against something
    /// else, it would not be the built-in layer.
    #[test]
    fn the_ron_file_is_a_complete_manifest() {
        parse_built_in()
            .expect("parses")
            .validate()
            .expect("the built-in layer must validate on its own");
    }

    /// **★ A hand-written snippet parses — the `IMPLICIT_SOME` check.**
    ///
    /// Deliberately **not** produced by the serializer. Every string here
    /// was typed: no `Some(…)` wrappers, a comment, a trailing comma, and
    /// the shape the documentation shows. This is the input class an
    /// operator-editable format exists to accept, and the one a round-trip
    /// test structurally cannot cover.
    ///
    /// It is written as a *customization layer* rather than a whole
    /// manifest, because that is what an operator actually writes: a small
    /// file that patches the built-in one per item. It therefore also
    /// checks that an incomplete layer parses without validating — which
    /// is the property the whole three-layer design rests on.
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

    /// **★ No `Some(` appears anywhere in the file.**
    ///
    /// This is the observable consequence of `IMPLICIT_SOME` on the
    /// *writer*, and it is the property that makes the generated file a
    /// usable template: an operator who copies three lines out of it and
    /// pastes them into `userdata/shell.ron` gets a fragment in the same
    /// dialect their own file is read in. If the extension were ever lost
    /// from `egui-shell`'s `ron::Options`, this file would fill up with
    /// `question: Some("…")` — the round trip would still pass, and the
    /// format would have quietly stopped being hand-editable.
    ///
    /// Note what is *not* asserted: the file carries **no**
    /// `#![enable(implicit_some)]` header and **no** struct names —
    /// `egui-shell`'s `PrettyConfig` sets the extension but ron 0.8
    /// emits neither, so the file opens with a bare `(` rather than
    /// `Shell(`. Neither costs correctness: the reader defaults the
    /// extension on independently of any header (which is the whole
    /// finding recorded in `D:/dev/rag/rust/`), and RON accepts both the
    /// named and the anonymous struct spelling, so the documented
    /// `Shell(tabs: [ Tab(id: "tools") ])` form still parses — see
    /// [`a_hand_written_snippet_parses`], which uses it. Both would make
    /// the generated file more legible and both are `egui-shell`'s to
    /// change, not this crate's.
    #[test]
    fn the_generated_file_carries_no_option_wrappers() {
        assert!(
            !built_in_ron().contains("Some("),
            "the generated manifest must not be full of Option wrappers — that is the \
             difference between a file an operator can edit and one they cannot"
        );
    }

    /// The file is recognisably the ribbon when read by a person.
    ///
    /// A weak assertion on purpose: it is not checking the layout, which
    /// the equality test covers exactly. It is checking that the *file*
    /// contains the words an operator would search for — that the
    /// serialized form is legible enough to edit, which is the property
    /// the whole format choice was made for.
    #[test]
    fn the_ron_file_reads_as_a_ribbon() {
        let text = built_in_ron();
        for needle in [
            // ★ Was `Command(id: "file.open")` until 2026-09-04. `file.open`
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
            // ★ The contextual Format tab's condition, and it is
            // `selection.formattable` rather than `selection.any` since
            // 2026-08-27 — the tab now carries controls for two kinds of
            // selection, so its condition is the union rather than either
            // operand. Kept in this list because the needle it is here to
            // prove is *"a condition round-trips into the file legibly"*, and
            // that is exactly as true of the new name.
            "visible_when: \"selection.formattable\"",
            "kind: \"colour_swatch\"",
            // ★ A custom item carrying a `visible_when`, which the format
            // could not express before 2026-08-27. It is what makes the whole
            // Font group vanish in Read and Review rather than drawing three
            // controls into a mode that cannot use them, and a serialization
            // that dropped it would look correct in every test but this one.
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
