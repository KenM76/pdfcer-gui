# `panels::attachments::listing_tests`

Tests of [`crate::text::panels::attachments::listing_notes`] against the
engine's real attachment fixtures. The copy lives in the base catalog; the
fixture locator is this crate's, so the tests sit beside the panel that shows
the listing.

## Item notes

### `fn a_well_formed_document_needs_no_caveat`

The companion to `pdfcer_gui_base::text::panels::attachments`' `an_undamaged_listing_discloses_nothing`, and the one
that would catch a flag pdfcer sets over-eagerly: a `Default` is a value
nobody produced, and a listing that quietly reported *"pdfcer stopped
reading early"* about every well-formed document would still pass that
test.
