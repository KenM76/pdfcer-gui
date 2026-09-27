//! **One provenance-bearing text extraction per page, shared by everything
//! that edits text.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/opendoc/cache/provenance.md`.

use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Instant;

use pdfcer_core::text_extract::PageText;

use crate::opendoc::OpenDoc;

/// **A page's provenance-bearing extracted text, borrowed from the document.**
pub struct CachedText<'a> {
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

pub use crate::doccache::ProvenanceTextCache;

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
    /// # The extraction options are the operator's, MODIFIED
    ///
    /// `settings.extract_options().with_provenance(true)` — the funnel's
    /// output with one field turned on, **never** `ExtractOptions::default()`.
    /// `crate::settings`' header §2 states the rule and this is the site
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
    /// ⇒ That is also why this cache and `crate::opendoc::cache`'s `page_text` are
    /// two caches and not one. They hold the *same segmentation* of the same
    /// page under different provenance settings, and merging them would mean
    /// either paying for provenance on every find (the cost this module
    /// exists to stop paying six times) or editing without it (impossible).
    #[must_use]
    pub fn provenance_page_text(&self, page: usize) -> Option<CachedText<'_>> {
        self.ensure_provenance_text(page);
        let held = self.provenance_text.text.borrow();
        // The borrow is released by the `?`-free clone and the function
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
        // Recorded here rather than after the extraction — see
        // [`ProvenanceTextCache::built_for`] for what that does and does not
        // buy, measured rather than assumed. What matters is that it is
        // recorded on the failure arm too, which it is, because the store at
        // the bottom of this function is unconditional.
        self.provenance_text.built_for.set(Some(key));
        let started = Instant::now();
        let built = self.pages.get(page).and_then(|page_ref| {
            use crate::settings::SettingsExt;
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
    use crate::opendoc::{FOUR_PAGES, open_fixture};

    /// **Two readers of one page get the SAME extraction, not two.**
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
        // And going back is a rebuild too, because the cache holds one page.
        // Asserted so that growing it into a map is a deliberate change with a
        // test to update, rather than something that quietly happens.
        let back = doc.provenance_page_text(0).expect("page 0 again");
        assert!(
            !std::ptr::eq(ptr0, std::rc::Rc::as_ptr(&back.text)),
            "the cache holds one page; returning to page 0 re-extracts it"
        );
        // Dropped only now, after every comparison, for the reason the doc
        // comment above gives at length. Moving this line up is the defect.
        drop(page0);
    }

    /// **An edit invalidates it.**
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
