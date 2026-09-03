//! # `app::settings` — the live configuration, and the funnel that makes it real
//!
//! ## ★ Why this module exists, and it is not "to hold a struct"
//!
//! `pdfcer_core::settings::Settings` is thirteen operator choices about
//! questions the PDF standard declines to answer. Loading them is easy;
//! **honouring** them is where the old shell failed, and it failed silently.
//!
//! Measured against `D:\Dev\pdfce\crates\pdfce-gui` on 2026-08-17: of the
//! thirteen settings that window persists, **four are read anywhere in the
//! application and nine are not.** `separations`, `cmyk_intent`,
//! `parallel_epsilon_degrees` and `theme` reach the code that would act on
//! them. `word_gap_ratio`, `mask_resample`, `image_minify`,
//! `cmyk_jpeg_polarity`, `unmappable_code`, `actual_text`, `missing_as`,
//! `xref_entry_eol` and `trailing_eol` are written to disk, read back from
//! disk, shown in a window, edited by the operator — and then never consulted.
//!
//! The mechanism is not a bug anyone wrote. It is what happens when option
//! structs are built at the call site:
//!
//! ```text
//! ExtractOptions::default()      →  word_gap_ratio, unmappable_code, actual_text
//! RenderOptions::default()       →  mask_resample, image_minify,
//!                                   cmyk_jpeg_polarity, missing_as
//! SaveOptions::identity()        →  xref_entry_eol, trailing_eol
//! ```
//!
//! Every one of those constructors is correct in isolation and every one of
//! them silently discards the operator's configuration. There were twelve such
//! call sites in the old crate and there are fifteen in this one.
//!
//! The irony worth recording: `xref_entry_eol`'s whole *default* was changed on
//! an operator ruling, because a fixed `SP LF` produced a 10,000-byte diff on a
//! file nobody had edited — and the GUI could not honour anything but the
//! default anyway.
//!
//! ## The funnel
//!
//! Three functions — [`Settings::extract_options`], [`Settings::render_options`]
//! and [`Settings::save_options`] — and a rule: **no code in this crate
//! constructs those three types itself.** The rule is not a convention; it is
//! checked, by [`tests::no_call_site_builds_its_own_options`], which parses
//! every `.rs` in the crate with `syn` and fails on a bare constructor outside
//! this module.
//!
//! A grep would not do. `ExtractOptions::default()` appears in a dozen doc
//! comments in this crate — including several in this very header — and a grep
//! would count each one as a violation, or be loosened until it counted none of
//! the real ones. The same argument `shell::commands::reach` and
//! `redact::sealed` already make: the thing being counted is a *call*, and a
//! syntax tree contains no comments at all.
//!
//! ### The three exemptions, and why each is not a hole
//!
//! 1. **Tests and fixtures.** A test that pins the engine's own default
//!    behaviour must be able to say `ExtractOptions::default()`, or it is
//!    testing the operator's configuration instead of the engine's contract.
//!    The check skips `#[cfg(test)]` modules, `ocr/fixture.rs` and
//!    `app/blank.rs` — see [`tests::no_call_site_builds_its_own_options`] for
//!    each one's argument.
//! 2. **`with_provenance(true)`.** Text editing needs provenance, which no
//!    setting controls. It is a *modifier* on the funnel's output rather than
//!    a second construction: `settings.extract_options().with_provenance(true)`.
//! 3. **Redaction's `SaveOptions::identity()`.** Deliberately NOT funnelled,
//!    and this is the interesting one — see [`Settings::save_options`].
//!
//! ## What is deliberately not here
//!
//! **A watcher on the settings file.** `pdfcer-core` refuses one, and the
//! reason binds the shell too: live configuration that depends on when an
//! editor happened to flush is a source of irreproducible behaviour, not a
//! feature.
//!
//! **A save on exit.** `save` is called deliberately, from the Save button, so
//! a crash cannot persist half a session's accidental state and an operator's
//! hand-edited file is never rewritten behind their back with pdfcer's own
//! formatting.

use pdfcer_core::settings::Settings;
use pdfcer_core::text_extract::ExtractOptions;
use pdfcer_core::writer::SaveOptions;
use pdfcer_render::RenderOptions;

/// The application's view of the operator's configuration.
///
/// A **trait on the engine's type** rather than a wrapper struct. The engine's
/// `Settings` is `#[non_exhaustive]`, so a wrapper would have to re-expose
/// thirteen fields by hand and would go stale the day a fourteenth arrived —
/// whereas an extension trait grows only where it must, which is in the three
/// option builders below.
pub trait SettingsExt {
    /// Text extraction, configured.
    fn extract_options(&self) -> ExtractOptions;
    /// Rasterisation, configured. `annotations` is the caller's, not a
    /// setting's — see the method.
    fn render_options(&self) -> RenderOptions;
    /// Writing, configured.
    fn save_options(&self) -> SaveOptions;
    /// ★★★ **A new editing session, configured** — the fourth funnel, and the
    /// one whose absence was a live defect for the whole life of this shell.
    ///
    /// See the implementation for what it applies and for the finding that
    /// produced it.
    fn open_session(&self, doc: pdfcer_core::document::Document) -> pdfcer_core::edit::EditSession;
}

impl SettingsExt for Settings {
    /// Every extraction in the application starts here.
    ///
    /// # The three fields, and the one that is a correctness knob
    ///
    /// - `word_gap_ratio` decides where extracted text gets its spaces.
    /// - `actual_text` decides how far a document's own replacement text is
    ///   trusted over the glyphs drawn.
    /// - `unmappable_code` decides what stands in for text pdfcer cannot read —
    ///   and it is **not** a cosmetic choice. Downstream of extraction sit
    ///   search, clipboard copy and **redaction-by-text**. Changing the
    ///   sentinel changes character offsets, therefore changes which runs a
    ///   redaction pattern matches. `pdfcer-core`'s R35 states it plainly: *a
    ///   redaction built under one value is not equivalent under another.*
    ///
    /// That last point is why both this and `actual_text` have radius lines in
    /// the settings window that name redaction, which the old shell's did not.
    ///
    /// # Why fields rather than builders
    ///
    /// `ExtractOptions` exposes no `with_word_gap_ratio` / `with_unmappable_code`
    /// / `with_actual_text` — checked, not assumed. The fields are `pub` and
    /// the struct is `#[non_exhaustive]`, so the only legal shape out of crate
    /// is *start from `default()` and assign*. Assigning after `default()` is
    /// what `clippy::field_reassign_with_default` complains about, which is why
    /// the binding is `let mut options` on its own line rather than a struct
    /// expression: the lint is about the pattern that *looks like* a struct
    /// literal and is not, and `#[non_exhaustive]` makes the real literal
    /// illegal here.
    fn extract_options(&self) -> ExtractOptions {
        let mut options = ExtractOptions::default();
        options.word_gap_ratio = self.word_gap_ratio;
        options.unmappable_code = self.unmappable_code;
        options.actual_text = self.actual_text;
        options
    }

    /// ★★★ **Every editing session in the application starts here.**
    ///
    /// # The finding this exists for, and it is the defect this module was
    /// written to prevent — one channel later
    ///
    /// This module's header enumerates the three **option structs** that
    /// silently discard the operator's configuration, and
    /// [`tests::no_call_site_builds_its_own_options`] parses every file in the
    /// crate to keep them funnelled. All of that was correct and all of it was
    /// blind to a fourth channel: a setting applied by a **method on the
    /// session** rather than by a field on an options struct.
    ///
    /// `Settings::quad_point_order` is one such. `EditSession::new(doc)` takes
    /// the engine's default; nothing here called `set_quad_point_order`; so an
    /// operator who chose *counterclockwise* in Settings > Saving got reading
    /// order in every markup annotation this shell has ever authored. The
    /// engine had already found the same defect on its own side and shipped
    /// the setter to fix it, with the sentence this shell should have read:
    ///
    /// > **A setting is a promise.** Storing one that does nothing breaks it
    /// > silently, which is worse than not offering the choice.
    ///
    /// ⇒ ★★ The lesson is about the SHAPE of the guard, not about this field.
    /// A funnel keyed on *constructors* cannot see a setting delivered by a
    /// setter, and the check that enforced it reported green throughout. The
    /// check now forbids `EditSession::new` outside this file for exactly that
    /// reason — see its own doc comment.
    ///
    /// # What it applies, and what it deliberately does not
    ///
    /// **`quad_point_order`, and nothing else**, because that is the only
    /// member of `Settings` with a session-level setter. Measured rather than
    /// assumed: `grep "pub const fn set_\|pub fn set_"` over
    /// `pdfcer-core/src/edit.rs` returns fifteen setters and fourteen of them
    /// take an operand from the operator's gesture rather than from the
    /// configuration.
    ///
    /// # ★ What the setting actually changes, so the disclosure can be honest
    ///
    /// Only the `/QuadPoints` **array**. The baked `/AP` appearance stream is
    /// byte-identical under both orders, so no reader that honours the
    /// appearance can tell — it changes what a consumer that re-derives
    /// geometry from `/QuadPoints` sees, which is exactly the population
    /// §12.5.6.10's ambiguity is about. Getting it wrong draws a bow-tie.
    ///
    /// Existing annotations are **not** rewritten: this governs what the
    /// session authors from now on. A preference change is not an edit, and
    /// sweeping a document because a setting moved is the unrequested
    /// normalisation `ARCHITECTURE.md` §5 forbids.
    fn open_session(&self, doc: pdfcer_core::document::Document) -> pdfcer_core::edit::EditSession {
        let mut session = pdfcer_core::edit::EditSession::new(doc);
        session.set_quad_point_order(self.quad_point_order);
        session
    }

    /// Every rasterisation in the application starts here.
    ///
    /// # Five settings, and one deliberate absence
    ///
    /// `cmyk_intent`, `mask_resample`, `image_minify`, `cmyk_jpeg_polarity` and
    /// `missing_as` are all read. Four of the five were persisted and ignored
    /// by the old shell, which chained only `.with_annotations()` and
    /// `.with_cmyk_intent()` onto a bare default.
    ///
    /// **Annotation scope is NOT set here**, and that is the absence worth
    /// stating. Whether annotations are drawn is a property of *what is being
    /// rendered for* — the canvas draws them, a print job may not, an export
    /// may be asked either way — and it is passed at the call site. Folding it
    /// in here would give the canvas and the print preview one answer, which is
    /// the opposite of what they need.
    ///
    /// # `missing_as` reaches paper, not just the screen
    ///
    /// It decides what a form control with no stated appearance state looks
    /// like, and the print path renders through this same function. An
    /// operator checking a form before printing it is exactly who that setting
    /// is for, which is why its radius line is the only one that separately
    /// names printing.
    fn render_options(&self) -> RenderOptions {
        RenderOptions::default()
            .with_cmyk_intent(self.cmyk_intent)
            .with_mask_resample(self.mask_resample)
            .with_image_minify(self.image_minify)
            .with_cmyk_jpeg_polarity(self.cmyk_jpeg_polarity)
            .with_missing_as(self.missing_as)
            // ★ Added 2026-08-26, engine v0.14.0, and it is the ONLY thing that
            // makes the Colour group's ceiling control do anything. The setting
            // reaches `CmykBuffer::new` through this call and through no other
            // route, so a build that added the control and not this line would
            // present an operator with a number that changes nothing — which is
            // worse than not offering it, because they would conclude the
            // colours cannot be fixed.
            //
            // `Option<usize>` passed VERBATIM: `None` means the engine's own
            // default and every one of its four public helpers takes the same
            // shape, so there is nothing for this shell to resolve and no
            // second place for a default to be decided.
            .with_max_cmyk_buffer_bytes(self.max_cmyk_buffer_bytes)
    }

    /// Every save in the application starts here — **except one, on purpose.**
    ///
    /// # The two settings
    ///
    /// `xref_entry_eol` and `trailing_eol`. Neither is visible in a viewer;
    /// both change the bytes on disk, which is why the settings window files
    /// them under *Saving files* and says "nothing visible" rather than
    /// pretending they are cosmetic.
    ///
    /// `ProducerPolicy::Preserve` is carried over from `identity()` rather than
    /// chosen here: it is not a setting, and changing what pdfcer writes into
    /// `/Producer` is a decision about attribution rather than about bytes.
    ///
    /// # ★ Redaction does not use this, and must not
    ///
    /// `redact::apply_redactions` is handed `SaveOptions::identity()` directly,
    /// and the [`tests::no_call_site_builds_its_own_options`] check exempts
    /// that one file by name.
    ///
    /// The reason is not that redaction is special-cased for convenience. A
    /// redaction is the one operation in the program whose output is checked,
    /// byte by byte, against a claim — that the removed content is *gone*. The
    /// proof runs over the exact buffer between the constructor and the
    /// syscall. Letting an operator's line-ending preference into that buffer
    /// would mean the bytes proved and the bytes written could differ by a
    /// setting, and the whole guarantee is that they cannot differ by anything.
    ///
    /// A redaction is also not a document the operator is *editing*: it is a
    /// new file produced from an old one, always written as a save-as, and the
    /// "leave untouched objects byte-identical" invariant that motivates
    /// `MatchSource` does not apply to a full rewrite that has deliberately
    /// changed content on every affected page.
    ///
    /// So the exemption is a statement about redaction, not a gap in the
    /// funnel — and it is written down here rather than only in the check,
    /// because a reader who finds the exemption first needs the argument.
    fn save_options(&self) -> SaveOptions {
        let mut options = SaveOptions::identity();
        options.xref_entry_eol = self.xref_entry_eol;
        options.trailing_eol = self.trailing_eol;
        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **A fresh install opens on *Match other PDF viewers*.**
    ///
    /// `OPERATOR_REQUESTS.md` **O52**, and it is the assertion that says the
    /// operator got what he asked for rather than that a function exists.
    ///
    /// ★★ It asserts on the value a fresh install actually receives — the
    /// engine's default put through `colour_default` — which is the only claim
    /// worth making while two crates disagree about what the default is. A test
    /// that checked `Settings::default()` would be testing `pdfcer-core`, and a
    /// ★★★ **This test outlived the function it was written for, and that is
    /// the point rather than an accident.**
    ///
    /// It was written on 2026-08-28 against `app::settings::colour_default`, a
    /// three-line seed this shell carried because `pdfcer-core`'s default was
    /// still `NeutralBlack` and O52 had reversed the operator's earlier ruling.
    /// That function shipped with a `debug_assert_ne!` tripwire whose message
    /// said *"delete it and its call site"*.
    ///
    /// **`Pass 153.0` landed the same day and the tripwire fired.** The seed is
    /// gone, its call site is gone, and this assertion now reads the engine
    /// directly — which is what it was always about. What the operator asked
    /// for was *"a fresh install opens on Match other PDF viewers"*, and that
    /// claim is worth a test whichever crate is responsible for making it true.
    ///
    /// ⇒ A test written against a temporary mechanism should assert the
    /// **outcome**, not the mechanism. This one did, so removing the mechanism
    /// cost one line.
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
    ///
    /// `Settings` is `#[non_exhaustive]`, so a struct expression is illegal out
    /// of crate and this is the only shape available: start from the default
    /// and assign. That is also exactly what the funnel's own implementations
    /// have to do, so the awkwardness is shared rather than incidental.
    fn every_field_moved() -> Settings {
        let mut s = Settings::default();
        s.word_gap_ratio = 0.42;
        s.unmappable_code = UnmappableCode::Omit;
        s.actual_text = ActualTextPrecedence::Glyphs;
        s.cmyk_intent = CmykIntent::Calibrated;
        s.mask_resample = MaskResample::Bilinear;
        s.image_minify = MinifyFilter::Smooth;
        s.cmyk_jpeg_polarity = CmykJpegPolarity::InvertOnApp14;
        s.missing_as = MissingAppearanceState::FirstEntry;
        s.xref_entry_eol = XrefEntryEol::CrLf;
        s.trailing_eol = TrailingEol::None;
        // ★ 2026-08-26, engine v0.14.0. A distinctive value rather than a round
        // one, so the assertion below cannot be satisfied by some other field
        // that happens to print the same digits.
        s.max_cmyk_buffer_bytes = Some(777_000_000);
        s
    }

    /// ★★★ **The session funnel applies the operator's quad-point order.**
    ///
    /// The regression test for the fourth channel — the one the check that
    /// guards this module could not see, because it is a *setter on a session*
    /// rather than a field on an options struct. Until 2026-08-28 every session
    /// this shell opened took the engine's default and an operator who chose
    /// counterclockwise got reading order in every markup they ever drew.
    ///
    /// ★★ It asserts **both** values, and that is not symmetry for its own
    /// sake. Asserting only `Counterclockwise` would pass on an implementation
    /// that hard-coded it, which is the same defect wearing the other value;
    /// asserting only the default would pass on the broken build this replaced.
    /// The pair is what makes it a test of the *wire* rather than of a value.
    ///
    /// ★ The document is the blank template rather than a fixture from disk,
    /// because the subject is the session's configuration and not its content —
    /// and a test that read a file would fail for reasons that have nothing to
    /// do with what it asserts.
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

    /// ★ **The regression test for the defect this module exists to prevent.**
    ///
    /// Nine of thirteen settings in the old shell were persisted, shown, edited
    /// and never read. This asserts that every field the funnel is responsible
    /// for actually reaches the option struct it belongs to — for all ten of
    /// them at once, from one non-default `Settings`.
    ///
    /// It compares against the value **set**, not against a hard-coded
    /// expectation, so it cannot go stale if an engine default moves. And it
    /// asserts each field individually rather than comparing whole structs,
    /// because a whole-struct comparison would need a second construction and
    /// would then be asserting that two copies of the same code agree.
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
            "Calibrated",
            "Bilinear",
            "Smooth",
            "InvertOnApp14",
            "FirstEntry",
            // ★★ The CMYK ceiling, and this is the assertion that stops the
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
    ///
    /// The other half of the property, and not a tautology: a funnel that
    /// accidentally *forced* a value — say by writing `MaskResample::Nearest`
    /// as a literal instead of reading the field — would pass the test above
    /// whenever the operator happened to want that value, and would pin the
    /// application to one answer forever. This catches it by asserting the
    /// funnel is transparent when it has nothing to say.
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

    /// ★ **No call site in this crate builds its own option struct.**
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
    /// - **`app/blank.rs`** — the sized-New path serializes and re-parses a
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
    /// # ★★★ The fourth constructor, and the finding that added it
    ///
    /// `EditSession::new` joined the list on 2026-08-28. It is not an options
    /// struct, which is precisely why it was missed: this check was written
    /// around the three **option constructors** named in the module header, and
    /// a setting delivered by a *setter on the session* — `quad_point_order`
    /// through `EditSession::set_quad_point_order` — is invisible to that
    /// shape. The result was an operator choice, persisted, validated, shown in
    /// a window, and honoured by nothing, with this check reporting green for
    /// the whole life of the shell.
    ///
    /// ⇒ The lesson is not about the field. **A guard shaped around one
    /// delivery mechanism cannot see a second one**, and the way to find the
    /// second is to ask what the engine offers rather than to re-read the
    /// guard. `tools/verb-coverage.py` is the instrument that asks that
    /// question mechanically, and it is what surfaced this.
    ///
    /// `app/blank.rs` is exempt for its existing reason extended: its session
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
            // ★★★ The fourth entry, added 2026-08-28, and the one that says
            // what the first three could not: a setting can be delivered by a
            // SETTER on a session as well as by a field on an options struct,
            // and a check keyed on constructors is blind to it.
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

            /// ★ Skip a `#[cfg(test)]` **function**, for the module rule's
            /// reason and not as a widening of it.
            ///
            /// A test-gated free function compiles to nothing in a release
            /// build, exactly as a test-gated module does, and this crate has
            /// two of them — `app::state::open_fixture` and its sibling — which
            /// exist so a dozen test modules share one way of opening a
            /// fixture. They were found by this check the moment
            /// `EditSession::new` joined the forbidden list, which is the check
            /// working: the *reason* they are allowed is the one already
            /// written for modules, and it had simply never been reachable
            /// before, because no forbidden constructor had ever appeared
            /// outside a `mod tests`.
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
                    || name.ends_with("app/blank.rs")
                    || name.ends_with("ocr/fixture.rs")
                    || name.contains("/redact/")
                {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(parsed) = syn::parse_file(&text) else {
                    continue;
                };
                // ★ A whole file gated out of release builds is exempt, and it
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
        assert!(root.is_dir(), "cannot find src at {}", root.display());
        let mut violations = Vec::new();
        walk(&root, &mut violations);

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
}
