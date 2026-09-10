//! **The print job itself** — everything from the moment the operator presses
//! **Print** until the receipt is on screen.
//!
//! Split out of [`super`] on 2026-09-10 under **rule R2** (no source file over
//! 1,500 lines), which that file crossed when O166's persistence landed. The
//! seam is the one the dialog already had rather than a line drawn to hit a
//! number: [`super`] owns the *window* — its state, what it draws, what it
//! recomputes every frame from the operator's current answers — and this file
//! owns the *job*, which is a transaction with a single entry point, no widgets
//! and no per-frame behaviour at all.
//!
//! # What is here
//!
//! | Item | Role |
//! |---|---|
//! | [`PrintDialog::commit_notes`] | The receipt: which outcome closes the window, and which sentences travel with it. Pure logic, deliberately, so it can be asserted without spooling a real job. |
//! | [`PrintDialog::commit`] | The spool. The **only** path in this crate that reaches `StartDoc`, and it is reached only from a control an operator deliberately clicked. |
//! | [`PrintDialog::trace_plan`] | The `print-plan` diagnostic line, emitted every frame, which is what `ui-verify` reads to assert the relationships a screenshot cannot show. |
//! | [`render_options`] | The single builder shared by the preview and the spool, so that what is previewed is what is printed. |
//!
//! # What is deliberately NOT here
//!
//! [`super::remembered`] — the projection of this dialog into the preferences
//! file. `commit` calls it, but *which* controls describe the operator rather
//! than the document is a separate judgement and lives beside its own argument.

use crate::app::state::OpenDoc;
use crate::dialogs::print::spooler::{self, Job, PageBitmap, SettingsSource, SpoolReport};
use crate::text::print as t;

use super::{PrintDialog, US_LETTER_PORTRAIT_PT, autopaper, verdicts};

impl PrintDialog {
    /// **What a finished commit owes the operator, and whether the window is
    /// done** — as a pure function, so it can be tested without a printer.
    ///
    /// Returns `Some(notes)` when the job went to the spooler: the sentences to
    /// put on the application's disclosure row, in reading order. The caller
    /// records them and closes the dialog. Returns `None` on failure, which
    /// means *"say nothing here and leave the window open"* — the footer draws
    /// the driver's own words and the operator picks another printer.
    ///
    /// # ★★★ Why this is extracted rather than left inline
    ///
    /// Because the behaviour it decides is the operator's 2026-09-03 report —
    /// *"it doesn't close after I hit the print button [...] there was a dozen
    /// jobs there"* — and the only way to drive the inline version is to
    /// actually print. Spooling a real job to his printer to prove a window
    /// closes is not a test, it is the defect.
    ///
    /// So the decision is separated from the act. `ui-verify` cannot reach it
    /// (no headless route ends in a real spool), and this project's rule is
    /// that a unit test is the floor rather than the ceiling — so what is
    /// asserted here is deliberately the part that is **pure logic**: which
    /// outcome closes, and which sentences travel. The act of printing is
    /// `Self::commit`'s, and is covered by `print_dialog_reaches_the_spooler`.
    ///
    /// ★ Stated plainly because it is a real gap: *"the window closes after a
    /// successful print"* is asserted as a decision, not as an observed
    /// window disappearing. Closing that gap needs a driven check that prints
    /// to a file device — `Microsoft Print to PDF` is on this machine — and it
    /// is worth building; it is not built.
    pub(super) fn commit_notes(outcome: Result<&SpoolReport, &String>) -> Option<Vec<String>> {
        let report = outcome.ok()?;
        let mut notes = vec![t::sent(report.pages)];
        // ★ The only one of the four `SettingsSource` values that is disclosed,
        // and the operator could not learn it any other way: the job printed,
        // and everything the driver held that pdfcer does not model was
        // silently absent from it. See `SettingsSource::Synthesised`.
        if report.settings_source == SettingsSource::Synthesised {
            notes.push(t::settings_synthesised().to_owned());
        }
        // ★ Both sentences in ONE call. `record_notes`' own doc comment records
        // why: the slot holds a single disclosure, so a second `record_note`
        // REPLACES the first rather than joining it, and which one survived
        // would be decided by statement order.
        Some(notes)
    }
    /// Render every planned sheet and hand them to the spooler.
    ///
    /// # ★ The one place in the GUI that starts a print job
    ///
    /// Reached only from the commit button, via [`Self::commit_requested`].
    /// Nothing here runs as a side effect of opening, previewing, saving or
    /// rendering — which is the shell's half of `pdfcer-print`'s own contract
    /// that *"`spool` is the only function that reaches `StartDoc`, and it is
    /// reached only from a control an operator deliberately clicked."*
    ///
    /// # Why the whole job is rasterised inline
    ///
    /// It blocks the UI thread for as long as the job takes. That is the
    /// honest behaviour for now and it is not an oversight: a print that
    /// proceeds in the background needs a cancel affordance, a progress
    /// surface and an answer to "what happens if the document is edited
    /// mid-job", and shipping the render off-thread without those three would
    /// replace a visible wait with an invisible race. The single-slot render
    /// worker next door is for *display*, where a cancelled render costs
    /// nothing; a cancelled print costs paper.
    pub(super) fn commit(
        &self,
        printer: &str,
        doc: &OpenDoc,
        job: &Job,
        page_sizes: &[(f64, f64)],
    ) -> Result<SpoolReport, String> {
        // The SAME builder the preview calls. See `render_options` for the
        // choices it encodes and why a second copy of them here would defeat
        // the preview's purpose.
        let options = render_options(self.scope, &doc.settings);
        let view = doc.session.view();

        let mut bitmaps = Vec::with_capacity(job.plans.len());
        for plan in &job.plans {
            let (Some(page), Some(&size)) = (doc.pages.get(plan.index), page_sizes.get(plan.index))
            else {
                // A plan naming a page the document no longer has. Skipped
                // rather than refused, matching `plan_job`'s own posture: *"a
                // job that refuses wholesale because one index is stale is
                // worse than one that prints what it can and reports the
                // count."*
                continue;
            };
            let rendered = pdfcer_render::render_page_with_view(
                &view,
                page,
                plan.render_scale as f32,
                &options,
            )
            .map_err(|e| e.to_string())?;
            bitmaps.push(PageBitmap {
                width: rendered.pixmap.width(),
                height: rendered.pixmap.height(),
                // Premultiplied RGBA8, handed over unchanged — the engine's
                // stated contract. Any conversion here would be a second
                // colour convention.
                rgba: rendered.pixmap.data().to_vec(),
                placement: plan.placement,
                page_pt: size,
            });
        }

        // ★ The orientation page is the FIRST PLANNED page, taken from the
        // bitmaps rather than from the document. The sequence may be reversed
        // or range-filtered, which is exactly when `pages[0]` would be the
        // wrong page — and the driver picks its paper from whichever one it is
        // handed.
        let first_page_pt = bitmaps
            .first()
            .map_or(US_LETTER_PORTRAIT_PT, |bitmap| bitmap.page_pt);
        spooler::spool(
            printer,
            &bitmaps,
            // ★ The RESOLVED device, never `self.device`. `AutoFromPages` is
            // not a value `pdfcer-print` can act on, and handing it over would
            // silently print on whatever the machine was standing on while the
            // disclosure line above claimed a matched sheet.
            self.effective_device(),
            self.config.as_ref(),
            first_page_pt,
        )
        .map_err(|error| error.to_string())
    }

    /// One trace line describing the job the dialog is currently showing.
    ///
    /// ★ `scale=` is on this line beside `orientation=` because they are the
    /// pair that exposes the orientation defect: a radio that changes
    /// `orientation=` and not `scale=` on a landscape page is that regression,
    /// restated. A harness can assert the relationship; a screenshot cannot.
    ///
    /// ★★ `clipped=` and `claim=` are on this line TOGETHER, and the pairing is
    /// the assertion — operator request O113. `clipped=` is the unchanged
    /// geometric count; `claim=` is what the button says, as `<state>:<count>`.
    /// A driven check asserts the *correction* between them, which no capture
    /// can supply: a button reading "Print" and a button reading "Print"
    /// because the cache silently never matched are the same photograph.
    pub(super) fn trace_plan(
        &self,
        printer: Option<&str>,
        job: Option<&Job>,
        claim: verdicts::ClipClaim,
    ) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-plan printer={printer:?} driver={:?} port={:?} sheets={:?} clipped={:?} \
                 claim={}:{} \
                 dpi={:?} capped={:?} uncapped_mb={:?} orientation={:?} duplex={:?} \
                 paper={:?} pick={} auto={} sheet={:?} config={} \
                 scale={:?} tab={:?}",
                self.printers.get(self.selected).map(|p| &p.driver),
                self.printers.get(self.selected).map(|p| &p.port),
                job.map(|j| j.plans.len()),
                job.map(Job::clipped),
                claim.trace_word(),
                claim.count(),
                job.map(|j| j.resolution.dpi),
                job.map(|j| j.resolution.capped),
                job.map(|j| j.resolution.uncapped_page_mb),
                self.device.orientation,
                self.device.duplex,
                // ★ `paper=` and `sheet=` are on this line TOGETHER, and the
                // pairing is the assertion. `paper=` is what was asked for;
                // `sheet=` is the physical sheet the geometry came back with.
                // A build that took the request and planned against the
                // device's default anyway would show `paper=Form(8)` beside an
                // unchanged `sheet=` — the 77 %-scale defect in a second
                // dimension, and invisible in any other evidence.
                // ★★ The RESOLVED paper — what the driver was actually asked
                // for — beside `pick=`, which is what the operator chose, and
                // `auto=`, which is how the choice came out.
                //
                // Three fields because they are three claims: a build that
                // resolved `AutoFromPages` to the wrong sheet and one that
                // never resolved it at all are indistinguishable from `paper=`
                // alone, and a driven check needs to tell them apart.
                //
                // ⚠ `pick=` and `auto=` are deliberately NOT `{:?}`. A machine
                // reads these, and this project has already shipped a driven
                // check that reported the opposite of the truth because it was
                // parsing a `Debug` tuple. Both are stable tokens with their
                // own function and their own test.
                self.effective_device().paper,
                autopaper::pick_token(self.device.paper),
                autopaper::outcome_token(&self.auto_paper),
                job.map(|j| j.device.physical_pt),
                self.config.is_some(),
                job.and_then(|j| j.plans.first()).map(|p| p.placement.scale),
                self.active_tab,
            )
        });
    }
}

/// The render options a print job — and its preview — are drawn with.
///
/// # ★ ONE builder, called from both, and that is the point
///
/// Two independently-written builders eventually disagree about something, and
/// neither side can tell which one they are looking at. For a print preview
/// that failure is the whole feature — a preview exists to say what will come
/// out of the printer, so a preview built from its own options is a preview
/// that can be confidently wrong.
///
/// The choices it encodes, carried across with their reasoning:
///
/// - **`view_magnification` stays `None`** — the PRINT answer under §8.11.4.5,
///   which says a printing application *"shall not apply the changes based on
///   usage application dictionaries"*. Inheriting the canvas's options would
///   apply the zoom-driven optional-content states the operator happens to be
///   looking at.
/// - **The operator's layer overrides are NOT applied**, for the same clause:
///   they are a viewing choice, and §8.11.4.5 puts printing on the document's
///   own default configuration. `RenderOptions::layers` left at `None` is what
///   expresses that — and `None` is *not* an empty set, which would reveal
///   every layer the document turned off.
/// - **The annotation scope IS the operator's**, because it is a statement
///   about the job rather than about the view.
///
/// ## ★ The settings surface landed, and this paragraph is what it changed
///
/// This doc comment used to say:
///
/// > One choice the old shell encoded is missing here and its absence is not an
/// > omission: **the CMYK conversion intent**. `pdfcer-core`'s settings surface
/// > does not exist in this crate yet, so there is no operator choice to carry.
/// > When it lands, it belongs here *and* in [`preview::PreviewKey`] in the
/// > same commit — otherwise the preview keeps showing a page rendered under
/// > the previous intent, which is the exact staleness class that key exists to
/// > close.
///
/// It landed on 2026-08-17 and both halves were done together, as instructed.
/// The options now come from `crate::app::settings::SettingsExt`, which carries
/// **five** settings rather than the one that note anticipated — the CMYK
/// intent, the mask resampling filter, the minification filter, the CMYK JPEG
/// polarity, and what is drawn for an annotation with no stated appearance
/// state. That last one reaches paper as well as the screen, which is why its
/// radius line in the settings window is the only one that separately names
/// printing.
///
/// [`preview::PreviewKey`] gained the same five, for the reason that note gave.
pub(super) fn render_options(
    scope: pdfcer_render::AnnotationScope,
    settings: &pdfcer_core::settings::Settings,
) -> pdfcer_render::RenderOptions {
    use crate::app::settings::SettingsExt;
    settings.render_options().with_annotation_scope(scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A spool report with the given page count and settings source.
    ///
    /// Built by hand rather than by printing, which is the whole reason
    /// [`PrintDialog::commit_notes`] was extracted: proving that the window
    /// closes must not require putting a job on the operator's printer.
    fn report(pages: usize, source: SettingsSource) -> SpoolReport {
        SpoolReport {
            pages,
            printed: true,
            dpi: (300, 300),
            clipped_pages: 0,
            job_id: Some(1),
            settings_source: source,
        }
    }

    /// **A successful print returns sentences, which is what closes the
    /// window** — the operator's 2026-09-03 report.
    ///
    /// > *"it doesn't close after I hit the print button [...] it looks greyed
    /// > out as though it doesn't do anything even when I hit print - but it is
    /// > working, so after many clicks I checked the printer and of course
    /// > there was a dozen jobs there."*
    ///
    /// `Some` is the signal to record and close; `None` is the signal to stay
    /// open. Asserting on the discriminant rather than on the wording, because
    /// the wording belongs to `crate::text::print` and a test that pinned it
    /// here would be a second copy of it.
    #[test]
    fn a_successful_print_asks_the_dialog_to_close() {
        let ok = report(3, SettingsSource::DriverSupplied);
        assert!(
            PrintDialog::commit_notes(Ok(&ok)).is_some(),
            "a job that reached the spooler must produce a receipt, which is what closes the \
             window. Leaving it open is how one press became a dozen queued jobs."
        );
    }

    /// **A FAILED print leaves the window open**, and the asymmetry is
    /// deliberate rather than an oversight.
    ///
    /// On failure the operator's next act is to choose a different printer or a
    /// different range — which is what this window is for — and the driver's
    /// own words in the footer are the only thing telling them which. Closing
    /// would destroy the reason and the settings together.
    #[test]
    fn a_failed_print_leaves_the_dialog_open() {
        let why = "the device is offline".to_owned();
        assert!(
            PrintDialog::commit_notes(Err(&why)).is_none(),
            "a failed job must NOT close the dialog: the footer's message is the only place the \
             reason appears, and the settings that produced it are still on screen."
        );
    }

    /// **The `Synthesised` disclosure travels WITH the receipt, in one call.**
    ///
    /// Two sentences, not two `record_note` calls. `record_notes`' doc comment
    /// records why that matters: the slot holds one disclosure, so a second
    /// call REPLACES the first and which one survived would be decided by
    /// statement order rather than by importance.
    ///
    /// The receipt is first because it is the sentence an operator reads if
    /// they read only one.
    #[test]
    fn a_synthesised_settings_source_adds_a_second_sentence_to_the_same_receipt() {
        let plain = PrintDialog::commit_notes(Ok(&report(2, SettingsSource::DriverSupplied)))
            .expect("a success returns notes");
        let synthesised = PrintDialog::commit_notes(Ok(&report(2, SettingsSource::Synthesised)))
            .expect("a success returns notes");

        assert_eq!(plain.len(), 1, "an ordinary print says one thing");
        assert_eq!(
            synthesised.len(),
            2,
            "a job printed from settings pdfcer synthesised owes the operator that fact, and it \
             is the one `SettingsSource` value they could not learn any other way"
        );
        assert_eq!(
            plain[0], synthesised[0],
            "the receipt must be the same sentence and must come FIRST in both cases"
        );
    }
}
