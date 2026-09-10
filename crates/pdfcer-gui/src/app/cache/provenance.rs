//! **One provenance-bearing text extraction per page, shared by everything
//! that edits text.**
//!
//! # What this is, in one sentence
//!
//! `pdfcer_core::text_extract::extract_page_view` with
//! `ExtractOptions::with_provenance(true)`, run **at most once per
//! `(page, edit epoch)`**, so that the six places in this shell that need to
//! know *which show operator a glyph came from* pay for that answer once
//! between them instead of six times each.
//!
//! # ★★★ Why this exists — it is a measured cost, not a tidiness
//!
//! Provenance is the substrate for editing text: it is what turns *"the
//! operator clicked run 41"* into *"byte span 8210..8263 of content stream
//! 12"*, which is the only thing `EditSession`'s text verbs can act on. The
//! ordinary read-only extraction in [`crate::app::cache`] deliberately leaves
//! it **off** — see `ensure_page_text` — because capturing it costs and the
//! find bar does not need it.
//!
//! The cost is not marginal. `crate::panels::properties::refusedchar`'s header
//! records the measurement on the operator's own benchmark CAD sheet:
//!
//! > *`pin::inspect` plus `preview_font_resources` — **392 ms** for the first
//! > alone.*
//!
//! Before this module there were six independent copies of the same four
//! lines, each paying that in full:
//!
//! | site | when it ran |
//! |---|---|
//! | `app::cache::ensure_form_runs` | every click on the canvas, to ask whether the clicked run has an anchor |
//! | `canvas::textedit::pin::inspect` | opening the properties panel on a text run |
//! | `canvas::textedit::pin::resolve` | every restyle |
//! | `canvas::textedit::pin::operators` | every `textstyle` action |
//! | `canvas::textedit::plan` | every committed text edit |
//! | `canvas::textedit::reflow::block_of_run` | every reflow |
//!
//! Two of those run **in the same gesture**: a click on a text run calls
//! `run_has_no_anchor` (extraction one) and then, if the properties panel is
//! open, `pin::inspect` (extraction two) — 784 ms on the benchmark sheet for
//! one click, to compute the same `PageText` twice and throw one of them away.
//!
//! # ★★ Why the cached value is the `PageText` and not the model
//!
//! `EditableTextModel::recognize` borrows the `PageText` it describes, so a
//! cache holding the model would have to hold both together and hand out a
//! self-referential borrow. It also takes `BlockRecognitionOptions`, and
//! `reflow` uses a *different* set from the caret path deliberately — see
//! `reflow::tests`. So the expensive, shared half is cached and the cheap,
//! caller-specific half is not: every caller re-runs `recognize` over the
//! cached text.
//!
//! That is the correct split on cost as well as on correctness. Recognition
//! walks runs that are already decoded and laid out; extraction resolves
//! `/Contents`, inflates, tokenizes, resolves fonts and decodes every string.
//!
//! # ★★★ Why the handle is [`CachedText`] and not `Ref<'_, PageText>`
//!
//! Every other cache in [`crate::app::cache`] hands out a `Ref`, and that
//! module's header sets out the three-part argument for why that is sound.
//! This one does not, for one reason the others do not face: **its callers
//! pass a page index rather than reading the current page.**
//!
//! `pin::inspect(doc, page, run)` names its page. So does `operators`, so does
//! `plan`. Nothing stops one of them from holding a handle to page 3 while a
//! function it calls asks for page 7 — and with a `Ref` outstanding that is a
//! `RefCell` double-borrow **panic**, in release, in front of the operator,
//! for a pattern the type system was happy with.
//!
//! So the cache stores an [`Rc<PageText>`](std::rc::Rc) and hands out a clone
//! of it. The `RefCell` borrow lives for the length of one `Rc::clone` and is
//! gone before the caller sees anything, so two pages in flight is a refcount
//! of two rather than a crash.
//!
//! ⚠ **That trade would give up the other half of the `Ref` argument** — a
//! live `Ref` borrows `&OpenDoc`, which is what makes it impossible for the
//! epoch to move while a cached value is held, which is what makes it
//! impossible to act on stale text. Losing *that* would be worse than a panic:
//! a silent edit against a page as it was two revisions ago.
//!
//! [`CachedText`] is how both are kept. It is an `Rc` for the borrow, plus a
//! `PhantomData<&'a OpenDoc>` for the lifetime, so the compiler still refuses
//! to let the handle outlive the `&OpenDoc` that produced it or coexist with a
//! `&mut OpenDoc`. The staleness property is enforced exactly as it is for
//! every other cache here; only the re-entrancy panic is given up.

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Instant;

use pdfcer_core::text_extract::PageText;

use crate::app::state::OpenDoc;

/// **A page's provenance-bearing extracted text, borrowed from the document.**
///
/// Dereferences to [`PageText`]. Cheap to clone in the sense that matters —
/// producing one is a refcount bump, not an extraction — but it is *not*
/// `Clone`, deliberately: a handle that could be cloned out of the function
/// that made it would invite exactly the escape the lifetime is here to
/// prevent.
///
/// See this module's header, §"Why the handle is `CachedText`", for the whole
/// argument. In short: the `Rc` is so two pages can be in flight without a
/// `RefCell` panic, and the `PhantomData` is so the compiler still forbids
/// holding one across a mutation of the document.
pub(crate) struct CachedText<'a> {
    /// The shared extraction. The `RefCell` borrow that produced this is
    /// already released by the time a caller can observe the handle.
    text: Rc<PageText>,
    /// Ties the handle to the `&OpenDoc` that produced it. Zero-sized; its
    /// only job is to make `&mut doc` and a live handle mutually exclusive,
    /// which is what makes acting on stale text a compile error rather than a
    /// silent wrong edit.
    _doc: PhantomData<&'a OpenDoc>,
}

impl std::ops::Deref for CachedText<'_> {
    type Target = PageText;

    fn deref(&self) -> &PageText {
        &self.text
    }
}

/// **The provenance-bearing text of one page**, keyed by `(page, edit epoch)`.
///
/// One page, not all of them: the shell edits the page in front of the
/// operator, and holding every page's decoded text for a 200-sheet drawing set
/// would be a memory profile nobody asked for. Paging away drops it; paging
/// back rebuilds it, at the same cost as the first visit.
///
/// The two-field shape — key in a [`Cell`] outside, value in a [`RefCell`]
/// inside — is the same one every cache in [`crate::app::cache`] uses, and its
/// header carries the argument for why. The short version: the already-built
/// path must not need `borrow_mut`.
#[derive(Default)]
pub(crate) struct ProvenanceTextCache {
    /// The `(page index, edit epoch)` the text below describes, or `None`
    /// before the first build.
    ///
    /// ★★★ **What actually makes a failed extraction cheap, and it is NOT the
    /// order this line is written in.**
    ///
    /// The sibling caches in [`super`] carry a comment saying the key is set
    /// *before* the work "so a failed extraction is not retried every frame",
    /// and this module was written with the same sentence until it was
    /// **falsified on 2026-09-09**: moving the `set` below the extraction and
    /// re-running the tests changed nothing, in any of them.
    ///
    /// It could not. The value store at the end of
    /// [`OpenDoc::ensure_provenance_text`] is *unconditional* — a failed
    /// extraction stores `None`, and the key is recorded either way — so both
    /// orders record the attempt and both make the next frame a `Cell` read.
    /// What stops the retry is that **the key is recorded at all, on the
    /// failure arm as well as the success arm**. That is the property to
    /// preserve: an early `return` on failure, before the key is set, would
    /// reintroduce the third-of-a-second-per-frame cost the sibling comment
    /// warns about — and would do it while leaving that comment looking
    /// satisfied.
    ///
    /// The order is kept anyway, for a smaller reason that is real: it makes a
    /// re-entrant ask for the same page terminate (answering `None`) rather
    /// than recurse.
    built_for: Cell<Option<(usize, u64)>>,
    /// The extraction, or `None` when it failed or the page does not exist.
    ///
    /// `None` here means **"could not be measured"**, never "this page has no
    /// text" — a page with no text extracts successfully to an empty
    /// `PageText`. Every caller reads it as *"there is nothing to edit"*,
    /// which is the correct reading of both, but a caller that ever needs to
    /// tell them apart must not infer one from this field.
    text: RefCell<Option<Rc<PageText>>>,
}

impl OpenDoc {
    /// **This page's text, with provenance, built at most once per edit.**
    ///
    /// `None` when the page index does not exist or the extraction failed —
    /// see [`ProvenanceTextCache::text`] for why that is one answer and not
    /// two.
    ///
    /// # What the caller does with it
    ///
    /// Almost always this, and the two halves are deliberately not both
    /// cached — see the module header:
    ///
    /// ```ignore
    /// let text = doc.provenance_page_text(page)?;
    /// let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    /// let pin = pin::of_run(&model, run)?;
    /// ```
    ///
    /// # ★★★ The extraction options are the operator's, MODIFIED
    ///
    /// `settings.extract_options().with_provenance(true)` — the funnel's
    /// output with one field turned on, **never** `ExtractOptions::default()`.
    /// `crate::app::settings`' header §2 states the rule and this is the site
    /// it was written for.
    ///
    /// The reason is that run indices are a shared vocabulary. The canvas
    /// hit-tests run 41, the find bar highlights run 41, and the commit pins
    /// run 41 — and two extractions of one page under two configurations
    /// **segment differently**, so the glyph the operator clicked and the
    /// glyph this code edits end up one step out of step. Turning provenance
    /// on is safe in a way that changing any other option is not:
    /// `capture_provenance` populates a field and changes no segmentation, so
    /// `runs[i]` names the same run with it on or off.
    ///
    /// ⇒ That is also why this cache and `crate::app::cache`'s `page_text` are
    /// two caches and not one. They hold the *same segmentation* of the same
    /// page under different provenance settings, and merging them would mean
    /// either paying for provenance on every find (the cost this module
    /// exists to stop paying six times) or editing without it (impossible).
    #[must_use]
    pub(crate) fn provenance_page_text(&self, page: usize) -> Option<CachedText<'_>> {
        self.ensure_provenance_text(page);
        let held = self.provenance_text.text.borrow();
        // ★ The borrow is released by the `?`-free clone and the function
        // return, before any caller code runs. That is the whole point of the
        // `Rc` — see the module header.
        let text = Rc::clone(held.as_ref()?);
        Some(CachedText {
            text,
            _doc: PhantomData,
        })
    }

    /// Build [`Self::provenance_text`] for `(page, epoch)` if it is not already
    /// built. Idempotent, and a `Cell` read on every call after the first.
    fn ensure_provenance_text(&self, page: usize) {
        let key = (page, self.edit_epoch);
        if self.provenance_text.built_for.get() == Some(key) {
            return;
        }
        // ★ Recorded here rather than after the extraction — see
        // [`ProvenanceTextCache::built_for`] for what that does and does not
        // buy, measured rather than assumed. What matters is that it is
        // recorded on the failure arm too, which it is, because the store at
        // the bottom of this function is unconditional.
        self.provenance_text.built_for.set(Some(key));
        let started = Instant::now();
        let built = self.pages.get(page).and_then(|page_ref| {
            use crate::app::settings::SettingsExt;
            let opts = self.settings.extract_options().with_provenance(true);
            pdfcer_core::text_extract::extract_page_view(
                &self.session.view(),
                page_ref,
                page,
                &opts,
            )
            .ok()
            .map(Rc::new)
        });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // ★★ **The count of these lines IS the measurement.** One line per
            // extraction, so a harness can tell a cache that works from one
            // that does not — and this is the line that proves the six
            // duplicate extractions this module removed are actually gone. A
            // click on a text run with the properties panel open used to emit
            // `form-runs` and then a silent second extraction inside
            // `pin::inspect`; it now emits exactly one `provenance-text`.
            format!(
                "provenance-text page={page} ms={} runs={} ok={}",
                started.elapsed().as_millis(),
                built.as_ref().map_or(0, |t| t.runs.len()),
                u8::from(built.is_some()),
            )
        });
        *self.provenance_text.text.borrow_mut() = built;
    }
}

#[cfg(test)]
mod tests {
    use crate::app::state::{FOUR_PAGES, open_fixture};

    /// **Two readers of one page get the SAME extraction, not two.**
    ///
    /// The property this module exists for, asserted the only way a unit test
    /// can assert it: [`Rc::ptr_eq`](std::rc::Rc::ptr_eq) on the two handles.
    /// A cache that rebuilt would hand back two distinct allocations holding
    /// equal text, and `assert_eq!` on the text would pass on both — which is
    /// exactly the shape of check that let six duplicate extractions sit in
    /// this crate unnoticed.
    ///
    /// ★ The two handles are **held at once**, deliberately. The old caches in
    /// [`super::super`] hand out `Ref`s and their equivalent test holds two to
    /// prove a shared borrow is enough; this one hands out an `Rc` precisely so
    /// that overlapping handles are possible, and a test that took them one
    /// after the other would not exercise that.
    #[test]
    fn a_second_reader_shares_the_extraction_rather_than_rebuilding_it() {
        let doc = open_fixture(FOUR_PAGES);
        let first = doc.provenance_page_text(0).expect("page 0 extracts");
        let second = doc.provenance_page_text(0).expect("page 0 extracts twice");
        assert!(
            std::rc::Rc::ptr_eq(&first.text, &second.text),
            "★ the second read re-extracted the page: the cache is not being hit"
        );
    }

    /// **A different page is a different extraction.**
    ///
    /// The control for the test above, and not a formality: a cache that
    /// ignored its key would pass that one and serve page 0's runs for page 1
    /// — which on the edit path means a pinned span naming a byte range in the
    /// wrong content stream, i.e. an edit applied to a page the operator is not
    /// looking at.
    /// ★★★ **The handle is HELD, never dropped, and that is the whole
    /// correctness of this test.**
    ///
    /// This test was written with `drop(page0)` before the two comparisons and
    /// it **failed in the full suite while passing when run alone**
    /// (2026-09-09). The reason is not flakiness and not shared state: once
    /// `page0` is dropped the cache has already evicted its own copy — it
    /// holds exactly one page, and page 1 displaced page 0 — so the last `Rc`
    /// is gone and the allocation is **freed**. `ptr0` is then a dangling
    /// address, and the allocator is entitled to hand that same address
    /// straight back for the next `PageText`, which is exactly what it did
    /// once the other tests in this binary had warmed the heap into a
    /// different shape.
    ///
    /// ⇒ **A pointer-identity check across a deallocation is not an identity
    /// check.** It is asked "is this a different allocation?" and the
    /// allocator is free to answer "no" about a genuinely different object.
    /// Both directions are unsound: it can report a rebuild as a cache hit
    /// (the failure actually seen), and it could equally have reported a cache
    /// hit as a rebuild, which would have sent somebody hunting a cache bug
    /// that does not exist.
    ///
    /// Holding `page0` for the life of the test fixes that at the root rather
    /// than by tolerance: while a strong `Rc` is alive the address cannot be
    /// reused by anything, so a differing pointer is proof of a different
    /// allocation and an equal pointer is proof of the same one. It costs
    /// nothing — `CachedText` is a handle, and this module hands out an `Rc`
    /// precisely so that overlapping handles are legal.
    ///
    /// ⚠ Its sibling above, `a_second_reader_shares_the_extraction_…`, was
    /// never exposed to this: it compares two handles that are **both alive**,
    /// which is the sound shape. Do not "fix" that one by symmetry.
    #[test]
    fn a_page_step_rebuilds_the_extraction() {
        let doc = open_fixture(FOUR_PAGES);
        let page0 = doc.provenance_page_text(0).expect("page 0");
        let ptr0 = std::rc::Rc::as_ptr(&page0.text);
        let page1 = doc.provenance_page_text(1).expect("page 1");
        assert!(
            !std::ptr::eq(ptr0, std::rc::Rc::as_ptr(&page1.text)),
            "★ page 1 was served page 0's text"
        );
        // ★ And going back is a rebuild too, because the cache holds one page.
        // Asserted so that growing it into a map is a deliberate change with a
        // test to update, rather than something that quietly happens.
        let back = doc.provenance_page_text(0).expect("page 0 again");
        assert!(
            !std::ptr::eq(ptr0, std::rc::Rc::as_ptr(&back.text)),
            "the cache holds one page; returning to page 0 re-extracts it"
        );
        // ★ Dropped only now, after every comparison, for the reason the doc
        // comment above gives at length. Moving this line up is the defect.
        drop(page0);
    }

    /// **An edit invalidates it.**
    ///
    /// The second half of the key. Serving pre-edit text after an edit would
    /// hand the next verb a byte span measured against a content stream that
    /// no longer exists — the silent-wrong-edit failure the module header
    /// calls worse than a panic.
    ///
    /// ★ The epoch is moved directly rather than by performing an edit,
    /// because this is a test of the **cache key**, not of any verb. A test
    /// that ran a real edit would fail for a dozen reasons that are not this
    /// one, and would stop compiling every time a verb's signature moved.
    #[test]
    fn an_edit_invalidates_the_extraction() {
        let mut doc = open_fixture(FOUR_PAGES);
        let before = doc.provenance_page_text(0).expect("page 0");
        let ptr = std::rc::Rc::as_ptr(&before.text);
        drop(before);
        doc.edit_epoch += 1;
        let after = doc.provenance_page_text(0).expect("page 0 after the edit");
        assert!(
            !std::ptr::eq(ptr, std::rc::Rc::as_ptr(&after.text)),
            "★ the cache served text from before the edit"
        );
    }

    /// **A page that does not exist answers `None`, and the attempt is still
    /// recorded.**
    ///
    /// The failure arm. The second clause is what stops a document whose
    /// current page cannot be extracted from paying a third of a second per
    /// frame — the measurement in `app::cache`'s header — to learn the same
    /// thing sixty times a second.
    ///
    /// ★★ **This test does not assert the ORDER of the `set`, and it was
    /// renamed on 2026-09-09 because its name claimed it did.** Moving the
    /// `set` below the extraction leaves all four tests here green — measured,
    /// not reasoned — because the store is unconditional and so both orders
    /// record the attempt. See [`super::ProvenanceTextCache::built_for`].
    ///
    /// What this does catch is the change that actually breaks the property:
    /// returning early on the failure arm without recording the key.
    #[test]
    fn an_unreadable_page_records_the_attempt() {
        let doc = open_fixture(FOUR_PAGES);
        let missing = doc.pages.len() + 10;
        assert!(doc.provenance_page_text(missing).is_none());
        assert_eq!(
            doc.provenance_text.built_for.get(),
            Some((missing, doc.edit_epoch)),
            "★ the failed extraction was not recorded, so it will be retried \
             every frame"
        );
    }
}
