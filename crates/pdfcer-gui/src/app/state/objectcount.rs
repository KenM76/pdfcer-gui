//! # `app::state::objectcount` — **what a document tells the diagnostic
//! channel about the page in front of it**
//!
//! One function, [`OpenDoc::trace_object_count`], and a long argument for why
//! the number it emits is the number it emits. Split out of
//! [`super`](crate::app::state) on 2026-09-11 under **R2**, when a sixty-line
//! doc block on `OpenDoc::render_refused` took that file to 1,531 lines
//! against the 1,500 ceiling.
//!
//! ## Why this is the seam, and not an arbitrary cut to get under a number
//!
//! `app::state` is, almost entirely, **the shape of an open document**: one
//! enum, one large struct, and the handful of questions you may ask it. This
//! function is the only member that is not about the document at all — it is
//! about **the harness**. Its subject is `PROJECT_PLAN.md` §4.3 requirement 3
//! and `DEFECTS.md` D1: the count that lets `ui-verify` assert a deletion
//! actually removed something, rather than assert that a verb was called.
//!
//! That makes it the third R2 split of the same file and the third one taken
//! along a subject boundary rather than a line number — `fixtures` (how a test
//! opens a document) and `heldpreview` (for how long a picture is still true)
//! came out the same way. A file that has been split three times along real
//! seams is a file whose seams were there all along.
//!
//! ## What stays behind
//!
//! Everything. This is an inherent `impl OpenDoc` block in a child module, so
//! the method is reached as `doc.trace_object_count()` exactly as before and
//! no call site changes. An R2 split moves where code **lives**; it does not
//! change what the crate offers.

use super::OpenDoc;

impl OpenDoc {
    /// Report how many objects the current page holds, on the `PDFCER_DIAG`
    /// channel — on document open, on a page change, and after any edit.
    ///
    /// # Why a count and not a deletion event
    ///
    /// `PROJECT_PLAN.md` §4.3 requirement 3, stated in one sentence:
    ///
    /// > Strictly better evidence than a `delete-objects` event: it measures
    /// > the property the check is about rather than the verb that was meant
    /// > to change it.
    ///
    /// The regression test for D1 asks "did pressing Delete remove the
    /// object?". Against a binary with only a deletion event, the available
    /// evidence is the **absence** of that event, and absence is weak: it
    /// cannot distinguish "the deletion path never ran" from "the deletion
    /// path ran and did nothing" from "the event is traced somewhere the code
    /// did not reach". `ui-verify`'s `delete_key` module admits that evidence
    /// exactly once, under three stated conditions, and says that removing
    /// any one of them turns the check into a SKIP.
    ///
    /// A count needs none of that. Read it before, read it after, compare.
    /// It is also robust to a deletion implemented by some future verb that
    /// nobody remembered to add a trace to — the count measures the page, not
    /// the code path.
    ///
    /// # The line
    ///
    /// ```text
    /// pdfcer-diag objects n=412 page=0 paths=380 text=30 images=2 forms=0
    /// ```
    ///
    /// `n=` is the total — the field name `ui-verify`'s vocabulary already
    /// uses for a count — and it is `PageObjects::objects.len()`, the same
    /// index space `EditSession::delete_objects` takes, so "n dropped by one"
    /// and "one object was deleted" are the same statement rather than two
    /// that have to be reconciled. The per-kind breakdown comes free from the
    /// decomposition's own diagnostics and is worth having: a count that
    /// changed by one is more convincing when you can see *which kind* left.
    ///
    /// # Failure is a different event, deliberately
    ///
    /// A page whose content streams will not decode traces
    /// `objects-unavailable page=… reason=…`, **never** `objects` with a
    /// missing or zero `n`. The distinction matters to a consumer: `objects`
    /// is a claim that the count was measured, and a check comparing before
    /// and after must be able to trust that. An `objects` line with no `n`
    /// would read as "the binary stopped reporting", and one with `n=0` would
    /// read as "the page is empty" — a false statement about the document.
    ///
    /// # Cost, and the gate that makes it affordable
    ///
    /// `decompose_page` resolves every `/Contents` stream, inflates it,
    /// concatenates, tokenizes and walks the whole token stream, resolving
    /// fonts as it goes. There is no cache inside `pdfcer-core`; the old GUI
    /// keeps exactly one decomposition per page for precisely this reason.
    /// So this is gated twice: it does nothing at all unless tracing is on,
    /// and even then it runs only when `(page, edit_epoch)` has moved. On a
    /// CAD sheet under `PDFCER_DIAG`, that is one decomposition per page
    /// visited, not one per frame.
    ///
    /// The gate is a stored key rather than [`crate::diag::trace_changed`]
    /// on purpose: `trace_changed` de-duplicates the *rendered line*, which
    /// would still require computing the count in order to render it. The
    /// expensive part is the count itself, so the gate has to sit in front of
    /// it.
    ///
    /// # ★ It counts the SHARED decomposition, not one of its own
    ///
    /// Until S4 this ran its own `decompose_page`, because the only cache was
    /// on the panels and this is not a panel — a second decomposition of the
    /// same page, i.e. the *"two decompositions quietly diverge"* pattern
    /// decision 011 warns about, sitting in the code whose entire job is to
    /// report a trustworthy number about that page. It now reads
    /// [`Self::page_objects`], so `n=` is by construction the count of the
    /// objects the Objects panel lists and the canvas hit-tests. The cost
    /// gate is unchanged: nothing is built with tracing off, and with it on
    /// the page decomposes once per `(page, epoch)` — what the private
    /// decomposition already cost, minus the duplicate.
    pub(crate) fn trace_object_count(&mut self) {
        if !crate::diag::enabled() {
            return;
        }
        let key = (self.view.page_index, self.edit_epoch);
        if self.objects_traced_for == Some(key) {
            return;
        }
        // Recorded BEFORE the work, so a decomposition that fails is not
        // retried on every subsequent frame — the failure is deterministic
        // (same bytes, same code), exactly as `settle_and_rasterize` argues
        // for the render error it holds.
        self.objects_traced_for = Some(key);

        if self.current_page().is_none() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "objects-unavailable page={} pages={} reason=no-such-page",
                    self.view.page_index,
                    self.pages.len()
                )
            });
            return;
        }

        // The shared decomposition — see this method's docs. It is built from
        // `session.view()` and not `document.view()`: the edit session's view
        // is the *edited* state (base plus staged changes), which is the
        // state the operator is looking at and the state a delete is meant to
        // have changed. Counting the base revision would report a number that
        // never moves however many objects are removed.
        if let Some(provider) = self.page_objects() {
            let model = provider.page_objects();
            let d = &model.diagnostics;
            // Read out before the closure so the `Ref` is not held across it.
            let (n, paths, text, images, forms) =
                (model.objects.len(), d.paths, d.text, d.images, d.forms);
            // ★★ **`leaves=` — how many objects are painted from INSIDE those
            // forms**, added 2026-08-27 with form-XObject descent.
            //
            // `n=` counts `PageObjects::objects`, which is the page's own
            // content stream and nothing else. On a wrapped drawing that is a
            // small fraction of what is on screen and of what a click can now
            // select: the industry print-conformance suite's composite page 1
            // reports `n=28 forms=4` and carries **242** leaves;
            // `ncored-benchmark-cad-drawing` page 1 reports `n=129758 forms=1`
            // and carries **10,256**.
            //
            // Without this field, `objects n=` is a half-truth on exactly the
            // documents the operator complained about — and it is the line a
            // driven check reads to answer *"did the page decompose, and how
            // much is there?"*. A harness reading `n=28` on a page with 270
            // selectable things is reading a number that no longer means what
            // its name suggests.
            //
            // ★ `depth_overflow=` and `cycles=` come with it, and they are the
            // half that stops `leaves=` becoming its own half-truth: a non-zero
            // count means the walk did NOT reach everything, so `leaves` is a
            // floor rather than a total. The engine counts them rather than
            // truncating silently for that reason, and a consumer that ignored
            // them would present an incomplete list as complete.
            let (leaves, depth_overflow, cycles) =
                (model.leaves.len(), d.form_depth_overflows, d.form_cycles);
            let page_index = self.view.page_index;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "objects n={n} page={page_index} paths={paths} text={text} images={images} forms={forms} leaves={leaves} depth_overflow={depth_overflow} cycles={cycles}"
                )
            });
            return;
        }

        // A distinct event, deliberately: `objects` is a claim that the count
        // was measured, and a check comparing before against after must be
        // able to trust that. An `objects` line with no `n` would read as
        // "the binary stopped reporting"; one with `n=0` would read as "the
        // page is empty", which is a false statement about the document.
        let detail = self
            .page_objects_failure()
            .map_or_else(|| "unknown".to_owned(), |err| err.clone());
        let page_index = self.view.page_index;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("objects-unavailable page={page_index} reason=decompose-failed detail={detail}")
        });
    }
}
