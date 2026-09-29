//! # `app::settings` — the configuration funnel, re-exported, and its checks
//!
//! The funnel lives in `pdfcer_gui_base::settings`. Its tests stay here
//! because they sweep this crate and reach `app` and `render` to do it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settings.md`.

pub use pdfcer_gui_base::settings::SettingsExt;

#[cfg(test)]
use pdfcer_core::settings::Settings;
#[cfg(test)]
use pdfcer_core::text_extract::ExtractOptions;
#[cfg(test)]
use pdfcer_core::writer::SaveOptions;
#[cfg(test)]
mod tests {
    use super::*;

    /// **A fresh install opens on *Match other PDF viewers*.**
    #[test]
    fn a_fresh_install_matches_other_viewers() {
        use pdfcer_core::settings::CmykIntent;
        assert_eq!(Settings::default().cmyk_intent, CmykIntent::Calibrated);
    }

    use pdfcer_core::settings::{
        ActualTextPrecedence, CmykIntent, CmykJpegPolarity, MaskResample, MinifyFilter,
        MissingAppearanceState, TrailingEol, UnmappableCode, XrefEntryEol,
    };
    use std::path::Path;

    /// A `Settings` whose every funnelled field differs from its default.
    fn every_field_moved() -> Settings {
        let mut s = Settings::default();
        s.word_gap_ratio = 0.42;
        s.unmappable_code = UnmappableCode::Omit;
        s.actual_text = ActualTextPrecedence::Glyphs;
        s.cmyk_intent = CmykIntent::NeutralBlack;
        s.mask_resample = MaskResample::Bilinear;
        s.image_minify = MinifyFilter::Smooth;
        s.cmyk_jpeg_polarity = CmykJpegPolarity::InvertOnApp14;
        s.missing_as = MissingAppearanceState::FirstEntry;
        s.xref_entry_eol = XrefEntryEol::CrLf;
        s.trailing_eol = TrailingEol::None;
        s.max_cmyk_buffer_bytes = Some(777_000_000);
        s
    }

    /// **The session funnel applies the operator's quad-point order.**
    #[test]
    fn the_session_funnel_applies_the_operators_quad_point_order() {
        use pdfcer_core::settings::QuadPointOrder;

        for order in [
            QuadPointOrder::ReadingOrder,
            QuadPointOrder::Counterclockwise,
        ] {
            let mut settings = Settings::default();
            settings.quad_point_order = order;
            let (doc, _pages) = crate::app::blank::document().expect("the template parses");
            let session = settings.open_session(doc);
            assert_eq!(
                session.quad_point_order(),
                order,
                "the session took the engine's default instead of the operator's choice — \
                 which is what `EditSession::new` at a call site does, and why the funnel check \
                 forbids it"
            );
        }
    }

    /// **The regression test for the defect this module exists to prevent.**
    #[test]
    fn every_setting_reaches_the_options_it_configures() {
        let s = every_field_moved();

        let extract = s.extract_options();
        assert!((extract.word_gap_ratio - 0.42).abs() < f32::EPSILON);
        assert_eq!(extract.unmappable_code, UnmappableCode::Omit);
        assert_eq!(extract.actual_text, ActualTextPrecedence::Glyphs);

        let save = s.save_options();
        assert_eq!(save.xref_entry_eol, XrefEntryEol::CrLf);
        assert_eq!(save.trailing_eol, TrailingEol::None);

        // `RenderOptions` has no `PartialEq` and its fields are read through
        // the renderer rather than compared here; what is assertable from
        // outside is that the builder chain is total. A default-valued render
        // options and a fully-moved one must not be the same picture, and the
        // cheapest honest statement of that is the debug rendering, which
        // names every field.
        let moved = format!("{:?}", s.render_options());
        let plain = format!("{:?}", Settings::default().render_options());
        assert_ne!(
            moved, plain,
            "render options ignore every setting fed to them"
        );
        for expected in [
            "NeutralBlack",
            "Bilinear",
            "Smooth",
            "InvertOnApp14",
            "FirstEntry",
            // The CMYK ceiling, and this is the assertion that stops the
            // Colour group's control being a number that changes nothing.
            //
            // The setting reaches `CmykBuffer::new` through
            // `with_max_cmyk_buffer_bytes` and through NO other route, so a
            // build that shipped the control and forgot the builder call would
            // present the operator with a field that accepts a size, saves it,
            // shows it back — and leaves the colours exactly as they were. That
            // is worse than not offering it, because they would conclude the
            // problem cannot be fixed.
            //
            // It is asserted as a raw digit string rather than through a getter
            // because `RenderOptions` publishes none; the debug rendering is
            // what is available from outside, which the block above already
            // explains.
            "777000000",
        ] {
            assert!(
                moved.contains(expected),
                "render options dropped {expected}: {moved}"
            );
        }
    }

    /// The default settings produce the engine's own defaults, unchanged.
    #[test]
    fn default_settings_change_nothing_about_the_engines_own_defaults() {
        let s = Settings::default();
        let extract = s.extract_options();
        let plain = ExtractOptions::default();
        assert!((extract.word_gap_ratio - plain.word_gap_ratio).abs() < f32::EPSILON);
        assert_eq!(extract.unmappable_code, plain.unmappable_code);
        assert_eq!(extract.actual_text, plain.actual_text);

        let save = s.save_options();
        let identity = SaveOptions::identity();
        assert_eq!(save.xref_entry_eol, identity.xref_entry_eol);
        assert_eq!(save.trailing_eol, identity.trailing_eol);
    }

    /// **No call site in this crate builds its own option struct.**
    ///
    /// The rule that keeps the funnel from being a suggestion. Without it, one
    /// new `ExtractOptions::default()` written in good faith next year silently
    /// restores the defect for whichever surface it is on — and, being correct
    /// in isolation, survives review.
    ///
    /// # Why the AST and not a grep
    ///
    /// The identifier `ExtractOptions::default()` appears in a dozen **doc
    /// comments** in this crate, several of them in this module's own header
    /// explaining why it must not be called. A grep counts those and reports
    /// violations that are prose, or is loosened past the point where it
    /// catches the real ones. A syntax tree contains no comments.
    ///
    /// This is the third such check in the crate — `shell::commands::reach`
    /// parses dispatch arms, `redact::sealed` counts one call — and they share
    /// the argument and the `syn` dev-dependency.
    ///
    /// # The exemptions, restated where they are enforced
    ///
    /// - **`app/settings.rs`** — this file. It is the funnel.
    /// - **`ocr/fixture.rs`** — a synthetic-document generator, not a surface.
    /// - **`pdfcer-gui-base/src/blank.rs`** — the sized-New path serializes and re-parses a
    ///   443-byte template, and **no operator-visible byte of that rewrite
    ///   survives**: two of `SaveOptions`' three fields spell the written
    ///   file, which is discarded in the same statement, and the third writes
    ///   `/Producer` into an `/Info` the template does not have. It uses
    ///   `identity()`, which promises to change nothing. The full argument is
    ///   on `blank::document_sized`; this line exists so a reader who finds
    ///   the exemption first is not left guessing.
    /// - **`redact/`** — see [`SettingsExt::save_options`]. The proof must run
    ///   over bytes no setting can vary.
    /// - **`#[cfg(test)]` modules anywhere** — a test pinning the engine's own
    ///   default behaviour must be able to name it, or it is testing the
    ///   operator's configuration instead of the engine's contract.
    ///
    /// # The fourth constructor, and the finding that added it
    ///
    ///
    /// ⇒ The lesson is not about the field. **A guard shaped around one
    /// delivery mechanism cannot see a second one**, and the way to find the
    /// second is to ask what the engine offers rather than to re-read the
    /// guard. `tools/verb-coverage.py` is the instrument that asks that
    /// question mechanically, and it is what surfaced this.
    ///
    /// `blank.rs` is exempt for its existing reason extended: its session
    /// rewrites a 443-byte template whose bytes are discarded in the same
    /// statement, and it authors no annotation, so no `/QuadPoints` array
    /// exists for the order to govern.
    #[test]
    fn no_call_site_builds_its_own_options() {
        use syn::visit::Visit;

        /// Constructors that discard the operator's configuration.
        const FORBIDDEN: &[(&str, &str)] = &[
            ("ExtractOptions", "default"),
            ("RenderOptions", "default"),
            ("SaveOptions", "default"),
            ("SaveOptions", "identity"),
            //
            // `EditSession::new` takes the engine's defaults, so every session
            // opened through it discarded `Settings::quad_point_order` — for
            // the whole life of this shell, with this very check green
            // throughout. `SettingsExt::open_session` is the funnel.
            ("EditSession", "new"),
        ];

        struct Finder {
            hits: Vec<String>,
        }

        impl<'ast> Visit<'ast> for Finder {
            /// Skip `#[cfg(test)]` modules whole.
            fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
                let is_test_mod = node.attrs.iter().any(|a| {
                    a.path().is_ident("cfg") && a.to_token_stream_string().contains("test")
                });
                if !is_test_mod {
                    syn::visit::visit_item_mod(self, node);
                }
            }

            /// Skip a `#[cfg(test)]` **function**, for the module rule's
            /// reason and not as a widening of it.
            fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
                let is_test_only = node.attrs.iter().any(|a| {
                    a.path().is_ident("cfg") && a.to_token_stream_string().contains("test")
                });
                if !is_test_only {
                    syn::visit::visit_item_fn(self, node);
                }
            }

            fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
                if let syn::Expr::Path(path) = &*node.func {
                    let segs: Vec<String> = path
                        .path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect();
                    if segs.len() >= 2 {
                        let ty = &segs[segs.len() - 2];
                        let func = &segs[segs.len() - 1];
                        if FORBIDDEN.iter().any(|(t, f)| t == ty && f == func) {
                            self.hits.push(format!("{ty}::{func}()"));
                        }
                    }
                }
                syn::visit::visit_expr_call(self, node);
            }
        }

        /// `syn`'s `Attribute` has no direct "text of the tokens" accessor, so
        /// this trait supplies the one thing the module filter needs. Kept
        /// local because it is a detail of this test and not a facility.
        trait TokensAsString {
            fn to_token_stream_string(&self) -> String;
        }
        impl TokensAsString for syn::Attribute {
            fn to_token_stream_string(&self) -> String {
                match &self.meta {
                    syn::Meta::List(list) => list.tokens.to_string(),
                    _ => String::new(),
                }
            }
        }

        /// Is `path` a module its own parent gates out of release builds?
        ///
        /// Answers *"does the file that declares this one say `#[cfg(test)] mod
        /// <stem>;`"*, which is the out-of-line spelling of the `#[cfg(test)]
        /// mod tests { … }` that `visit_item_mod` already skips. The two are
        /// the same fact about the same code; only the file boundary differs,
        /// and a file boundary is exactly what R2 moves.
        ///
        /// # How the declaring file is located
        ///
        /// For `…/foo/bar.rs` the declaring module is the module for directory
        /// `foo`, which Rust spells either `…/foo/mod.rs` (this crate's
        /// convention) or `…/foo.rs` (the 2018 style). Both are tried, in that
        /// order, and a miss returns `false` — **the safe direction**, because
        /// a false `false` costs a spurious violation somebody reads, while a
        /// false `true` would silently exempt shipped code from the whole
        /// check.
        ///
        /// `mod.rs` itself is never test-only by this route: it is declared by
        /// its *grandparent* under the directory's name, and a whole directory
        /// gated out of release builds would be spelled `#![cfg(test)]` inside
        /// it — which the caller already handles.
        fn declared_test_only(path: &Path) -> bool {
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                return false;
            };
            if stem == "mod" || stem == "lib" || stem == "main" {
                return false;
            }
            let Some(dir) = path.parent() else {
                return false;
            };
            let candidates = [dir.join("mod.rs"), dir.with_extension("rs")];
            for parent in candidates {
                let Ok(text) = std::fs::read_to_string(&parent) else {
                    continue;
                };
                let Ok(parsed) = syn::parse_file(&text) else {
                    continue;
                };
                for item in &parsed.items {
                    let syn::Item::Mod(m) = item else { continue };
                    // `semi.is_some()` is the out-of-line form — `mod x;` with
                    // no braces. An inline `mod x { … }` cannot be this file.
                    if m.semi.is_none() || m.ident != stem {
                        continue;
                    }
                    if m.attrs.iter().any(|a| {
                        a.path().is_ident("cfg") && a.to_token_stream_string().contains("test")
                    }) {
                        return true;
                    }
                }
            }
            false
        }

        fn walk(dir: &Path, out: &mut Vec<(String, Vec<String>)>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let name = path.to_string_lossy().replace('\\', "/");
                // The four exempt files, matched on their path suffix so the
                // check works from any working directory.
                if name.ends_with("app/settings.rs")
                    || name.ends_with("pdfcer-gui-base/src/settings.rs")
                    || name.ends_with("pdfcer-gui-base/src/blank.rs")
                    || name.ends_with("ocr/fixture.rs")
                    || name.contains("/redact/")
                    //
                    // Why a session is built here at all, rather than reusing
                    // the open one: `prepare` answers *"was this file opened
                    // with the OWNER password"* against the document on disk,
                    // and the engine's own version of that answer requires a
                    // session. The alternative is to ask the engine and get a
                    // second, differently-worded refusal for a question already
                    // answered — see the comment at the call.
                    || name.contains("/protect/")
                {
                    continue;
                }
                // A file whose PARENT declares it `#[cfg(test)] mod x;`
                // is test-only, and the scan cannot see that from inside the
                // file.
                //
                // This blind spot was found on 2026-09-10 by an **R2 split**,
                // not by a report: `canvas/forms/boxes.rs` reached 1,501 lines
                // and its `mod tests { … }` moved out into
                // `canvas/forms/boxes/tests.rs`. Not one line of test code
                // changed — but the `#[cfg(test)]` that had exempted those
                // tests stayed behind in the parent, as `#[cfg(test)] mod
                // tests;`, and three call sites that had been invisible for
                // months became violations of a rule they do not break. The
                // check went red on a refactor that could not possibly have
                // introduced the defect it reports, which is the signature of
                // an instrument measuring the wrong thing.
                //
                // ⇒ It is resolved the way the `#![cfg(test)]` case above is
                // resolved: by asking what makes the exemption TRUE — *this
                // code is not in the shipped binary* — rather than by adding
                // `boxes/tests.rs` to the path list. A path would have to be
                // added again by the next R2 split, and every one of those
                // splits arrives as a red check in an unrelated module, which
                // is the most expensive possible way to be told.
                if declared_test_only(&path) {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(parsed) = syn::parse_file(&text) else {
                    continue;
                };
                // A whole file gated out of release builds is exempt, and it
                // must be recognised from the AST rather than from the path.
                //
                // `#![cfg(test)]` as an INNER attribute is how this crate marks
                // a module that compiles to nothing in a release build —
                // `canvas::textedit::proof` and `canvas::textedit::cost` both
                // use it, and both must be able to name the engine's own
                // defaults, because what they exist to measure is the *engine's*
                // behaviour and not the operator's configuration. A `cost.rs`
                // that benchmarked extraction under whatever the developer
                // happened to have set would be a benchmark of a preference.
                //
                // This is checked here and not by adding two more filenames to
                // the list above, because the property that earns the exemption
                // is "not in the shipped binary" — and a filename is a
                // restatement of that which goes stale the moment a third such
                // module is written.
                let file_is_test_only = parsed.attrs.iter().any(|attr| {
                    matches!(attr.style, syn::AttrStyle::Inner(_))
                        && attr.path().is_ident("cfg")
                        && attr.to_token_stream_string().contains("test")
                });
                if file_is_test_only {
                    continue;
                }
                let mut finder = Finder { hits: Vec::new() };
                finder.visit_file(&parsed);
                if !finder.hits.is_empty() {
                    out.push((name, finder.hits));
                }
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../pdfcer-gui-base/src");
        assert!(root.is_dir(), "cannot find src at {}", root.display());
        assert!(base.is_dir(), "cannot find src at {}", base.display());
        let mut violations = Vec::new();
        walk(&root, &mut violations);
        walk(&base, &mut violations);

        // Calibrate the exemption before trusting the emptiness below.
        //
        // `violations.is_empty()` is a green light whether the scan is working
        // or has been silently switched off, and `declared_test_only` is
        // exactly the kind of helper that can switch it off: a version that
        // returned `true` unconditionally would exempt **every** file declared
        // out of line — which is nearly all of them — and this assertion would
        // pass forever while measuring nothing. So the helper is falsified in
        // both directions against two files that are checked in and whose
        // status is not in question.
        //
        // These two paths are deliberately NOT parameters or constants. If
        // either file is renamed, `is_file()` fails loudly here rather than the
        // calibration quietly becoming vacuous — the failure mode this project
        // has recorded more often than any other.
        let test_only = root.join("canvas/forms/boxes/tests.rs");
        let shipped = root.join("app/settings.rs");
        assert!(
            test_only.is_file() && shipped.is_file(),
            "the calibration files for `declared_test_only` have moved; re-point them at a \
             file declared `#[cfg(test)] mod x;` and one declared `pub mod x;`, do not \
             delete this check"
        );
        assert!(
            declared_test_only(&test_only),
            "`canvas/forms/boxes/tests.rs` is declared `#[cfg(test)] mod tests;` by its \
             parent and must be recognised as test-only — otherwise every R2 split of a \
             module away from its tests reports violations of a rule the code does not \
             break"
        );
        assert!(
            !declared_test_only(&shipped),
            "`app/settings.rs` is declared `pub mod settings;` and ships — an exemption that \
             swallows it swallows the whole scan, and the assertion below would pass \
             forever"
        );

        assert!(
            violations.is_empty(),
            "these call sites build their own option struct and therefore discard every \
             setting the operator chose — route them through `SettingsExt` instead:\n{}",
            violations
                .iter()
                .map(|(file, hits)| format!("  {file}: {}", hits.join(", ")))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// **ONLY THE CANVAS WORKER MAY SET `stroke_display`** — the mechanism
    /// behind O137's one non-negotiable constraint.
    ///
    /// > **Canvas only.** Print, print preview and every export — PDF, DXF,
    /// > PNG, JPEG, SVG, EMF, form data, text — render the document's REAL
    /// > widths. *The one thing worse than not having this feature is having it
    /// > follow him into a file he sends a client.*
    ///
    /// That was decided in writing on 2026-09-05, **before the engine field
    /// existed**, and the engine held the same line from its side: its own
    /// backlog row records that there is deliberately no CLI flag, because *"a
    /// hairline export would be an unfaithful file, the one outcome the request
    /// forbids"*.
    ///
    /// # Why this is a check and not a paragraph
    ///
    /// Because the paragraph already exists, at four sites, and this project
    /// has spent several corrections proving that a rule written next to the
    /// code it governs is not a mechanism. The concrete danger has a name and
    /// it is one line long: `app::actions::export`'s image export deliberately
    /// reads the *document's* annotation stance and layer overrides, arguing —
    /// correctly — that an export should be **"a picture of what you can
    /// see"**. Extending that reasoning by one field, in good faith, next year,
    /// silently ships the operator a client deliverable whose line weights are
    /// gone. It would be correct in isolation and would survive review, which
    /// is the exact shape [`no_call_site_builds_its_own_options`] was written
    /// for.
    ///
    /// ⇒ So the export paths are left saying **nothing** about the field — the
    /// funnel's `RenderOptions::default()` is already `StrokeDisplay::Actual` —
    /// and the **assignment** is what is policed.
    ///
    /// # What it looks for, and why an assignment rather than a name
    ///
    /// Any `….stroke_display = …`. Reads are fine and are everywhere:
    /// `ViewState::stroke_display()` is a method, `RenderRequest`'s field
    /// initialiser is a `FieldValue` and not an assignment, and `RenderKey`
    /// stores one. What can send a hairline somewhere it must not go is exactly
    /// one syntactic act — writing it into a `RenderOptions` — and there is
    /// precisely one legitimate instance of that act in the crate.
    ///
    /// `RenderOptions` is `#[non_exhaustive]`, so a struct literal cannot be
    /// written outside `pdfcer-render`, and there is no `with_stroke_display`
    /// builder to smuggle it through — checked against
    /// `pdfcer-render/src/font/mod.rs`, which publishes none. Assignment is the
    /// whole surface.
    ///
    /// # The one exemption
    ///
    /// `pdfcer_gui_base::renderworker::render_on_worker`, which rasterizes the
    /// interactive canvas and is called by nothing else. `#[cfg(test)]` modules
    /// are skipped for the reason its neighbour gives.
    ///
    /// ⚠ **If this test fails, the fix is almost never to add a file to the
    /// exemption list.** Ask first whether the new site can reach a printer, a
    /// file on disk, or the clipboard. If it can, the answer is no.
    #[test]
    fn only_the_canvas_worker_sets_stroke_display() {
        use syn::visit::Visit;

        struct Finder {
            hits: usize,
        }

        impl<'ast> Visit<'ast> for Finder {
            fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
                let is_test_mod = node.attrs.iter().any(|a| {
                    a.path().is_ident("cfg")
                        && matches!(&a.meta, syn::Meta::List(l)
                            if l.tokens.to_string().contains("test"))
                });
                if !is_test_mod {
                    syn::visit::visit_item_mod(self, node);
                }
            }

            fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
                if let syn::Expr::Field(field) = &*node.left
                    && let syn::Member::Named(name) = &field.member
                    && name == "stroke_display"
                {
                    self.hits += 1;
                }
                syn::visit::visit_expr_assign(self, node);
            }
        }

        /// Assignments in `path`, or `0` for a file gated out of release
        /// builds.
        ///
        /// **The `#![cfg(test)]` exemption is recognised from the AST, not
        /// from a filename** — the same rule [`no_call_site_builds_its_own_options`]
        /// writes for itself, and for the same stated reason: the property that
        /// earns the exemption is *"not in the shipped binary"*, and a filename
        /// is a restatement of that which goes stale the moment a second such
        /// module is written.
        ///
        /// It went stale within the hour. This check was written, run green,
        /// and then `render::hairline` — the `#![cfg(test)]` file that measures
        /// whether the mode actually thins a drawing — was added, and the check
        /// **immediately reported it as a violation**. It has to set
        /// `stroke_display` twice: that is the whole of what it measures, and it
        /// reaches no printer, no file and no clipboard because it does not
        /// exist in a release build.
        ///
        /// ⇒ That is the check working, on its first encounter with a legitimate
        /// second site, and the exemption is written the way the neighbour's
        /// already argued rather than by adding a path.
        fn scan(path: &Path) -> usize {
            let Ok(text) = std::fs::read_to_string(path) else {
                return 0;
            };
            let Ok(parsed) = syn::parse_file(&text) else {
                return 0;
            };
            let file_is_test_only = parsed.attrs.iter().any(|attr| {
                matches!(attr.style, syn::AttrStyle::Inner(_))
                    && attr.path().is_ident("cfg")
                    && matches!(&attr.meta, syn::Meta::List(l)
                        if l.tokens.to_string().contains("test"))
            });
            if file_is_test_only {
                return 0;
            }
            let mut finder = Finder { hits: 0 };
            finder.visit_file(&parsed);
            finder.hits
        }

        fn walk(dir: &Path, out: &mut Vec<String>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let name = path.to_string_lossy().replace('\\', "/");
                if name.ends_with("pdfcer-gui-base/src/renderworker.rs")
                    || name.ends_with("pdfcer-gui-base/src/printpreviewkey.rs")
                {
                    continue;
                }
                let hits = scan(&path);
                if hits > 0 {
                    out.push(format!("  {name}: {hits} assignment(s)"));
                }
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../pdfcer-gui-base/src");
        assert!(base.is_dir(), "cannot find src at {}", base.display());
        let mut violations = Vec::new();
        walk(&root, &mut violations);
        walk(&base, &mut violations);
        assert!(
            violations.is_empty(),
            "`stroke_display` is set outside the canvas worker. O137's whole safety argument \
             is that line weights never leave the screen — print, print preview and every \
             export render the document's REAL widths. If the new site can reach a printer, a \
             file or the clipboard, remove the assignment: the funnel's default is already \
             `StrokeDisplay::Actual`.\n{}",
            violations.join("\n")
        );

        // And the other half, so this cannot pass on a build where the
        // legitimate assignment was deleted along with the illegitimate ones —
        // which would leave every export correct and the toggle inert, i.e. a
        // green test over a dead feature. That is the exact failure O137
        // reports about the button this replaces.
        assert_eq!(
            scan(&base.join("renderworker.rs")),
            1,
            "the canvas worker must assign `stroke_display` exactly once — zero means \
             `view.line_weights` reaches no renderer at all"
        );
        // O233: the print dialog's own *Fixed line width*, which the operator
        // asked for by name. Off by default and blind to `view.line_weights`.
        assert_eq!(
            scan(&base.join("printpreviewkey.rs")),
            1,
            "the print dialog's fixed line width must assign `stroke_display` exactly once"
        );
    }

    /// **Every export and every print renders the document's real widths,
    /// while the canvas is showing hairlines** — O137, asserted rather than
    /// promised.
    #[test]
    fn every_export_path_renders_real_widths_with_line_weights_off() {
        use pdfcer_render::font::StrokeDisplay;

        let mut doc = crate::app::state::open_local_fixture("a1-titleblock.pdf");
        doc.view.line_weights = false;

        let request = doc
            .render_request_for(0, 2.0)
            .expect("the fixture has a first page");
        assert_eq!(
            request.stroke_display,
            StrokeDisplay::Hairline,
            "the canvas request did not carry the operator's choice, so this test could not \
             have detected an export carrying it either"
        );
        assert_eq!(
            doc.render_key_for(0, 2.0),
            crate::render::worker::RenderKey::new(
                0,
                2.0,
                doc.annotations_visible(),
                0,
                StrokeDisplay::Hairline,
            ),
            "the key the shell asks for disagrees with the request it sends"
        );

        // The one funnel every export and print path builds its options from.
        let exported = doc.settings.render_options();
        assert_eq!(
            exported.stroke_display,
            StrokeDisplay::Actual,
            "an export would have been rendered with line weights OFF — the one outcome O137 \
             forbids outright"
        );
        assert_eq!(
            exported
                .with_annotation_scope(pdfcer_render::AnnotationScope::DocumentAndMarkups)
                .stroke_display,
            StrokeDisplay::Actual,
            "the print path's builder chain reintroduced the hairline mode"
        );
        assert_eq!(
            doc.settings
                .render_options()
                .with_backdrop(pdfcer_render::PageBackdrop::Transparent)
                .stroke_display,
            StrokeDisplay::Actual,
            "the image-export path's builder chain reintroduced the hairline mode"
        );
    }
}
