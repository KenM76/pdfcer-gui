//! # `settings` — the live configuration, and the funnel that makes it real
//!
//! ## Why this module exists, and it is not "to hold a struct"
//!
//! `pdfcer_core::settings::Settings` is the operator's answers to questions
//! the PDF standard declines to answer. Loading them is easy; **honouring**
//! them is where the old shell failed, and it failed silently.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settings.md`.

use pdfcer_core::settings::Settings;
use pdfcer_core::text_extract::ExtractOptions;
use pdfcer_core::writer::SaveOptions;
use pdfcer_render::RenderOptions;

/// The application's view of the operator's configuration.
pub trait SettingsExt {
    /// Text extraction, configured.
    fn extract_options(&self) -> ExtractOptions;
    /// Rasterisation, configured. `annotations` is the caller's, not a
    /// setting's — see the method.
    fn render_options(&self) -> RenderOptions;
    /// Writing, configured.
    fn save_options(&self) -> SaveOptions;
    /// **A new editing session, configured** — the fourth funnel, and the
    /// one whose absence was a live defect for the whole life of this shell.
    ///
    /// See the implementation for what it applies and for the finding that
    /// produced it.
    fn open_session(&self, doc: pdfcer_core::document::Document) -> pdfcer_core::edit::EditSession;
}

impl SettingsExt for Settings {
    /// Every extraction in the application starts here.
    fn extract_options(&self) -> ExtractOptions {
        let mut options = ExtractOptions::default();
        options.word_gap_ratio = self.word_gap_ratio;
        options.unmappable_code = self.unmappable_code;
        options.actual_text = self.actual_text;
        options
    }

    /// **Every editing session in the application starts here.**
    fn open_session(&self, doc: pdfcer_core::document::Document) -> pdfcer_core::edit::EditSession {
        let mut session = pdfcer_core::edit::EditSession::new(doc);
        session.set_quad_point_order(self.quad_point_order);
        session.set_widget_tab_tail(self.widget_tab_tail);
        session.set_tab_row_tolerance(self.tab_row_tolerance);
        session
    }

    /// Every rasterisation in the application starts here.
    fn render_options(&self) -> RenderOptions {
        RenderOptions::default()
            .with_cmyk_intent(self.cmyk_intent)
            .with_mask_resample(self.mask_resample)
            .with_image_minify(self.image_minify)
            .with_cmyk_jpeg_polarity(self.cmyk_jpeg_polarity)
            .with_missing_as(self.missing_as)
            //
            // The colour-fidelity four. Each answers a rendering ambiguity
            // whose wrong answer is a wrong colour on screen and on paper, and
            // each takes its `Settings` value verbatim.
            .with_page_blend_space_source(self.page_blend_space_source)
            .with_overprint_zero_tint_scope(self.overprint_zero_tint_scope)
            .with_spot_colorant_device_model(self.spot_colorant_device_model)
            .with_mesh_patch_padding(self.mesh_patch_padding)
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
    /// # Redaction does not use this, and must not
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
        options.edited_stream_compression = self.edited_stream_compression;
        options
    }
}
