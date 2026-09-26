# `pdfcer-gui/canvas/clipboard/tests`

## Item notes

### `fn a_cut_that_cannot_delete_does_not_copy_either`

# What this asserts, and why each half is needed

1. **`Err(DeleteRefused)`** — the whole gesture is refused, and it
   carries the reason so the status row can say which of encryption,
   certification or the `/F` Locked bit it was.
2. **No action was raised** — asserting only the `Err` would pass on a
   build that refused *and* pushed the Delete anyway, which is the state
   this fix exists to remove.
3. **Nothing reached the clipboard** — the half that makes it a *cut*
   failure rather than a delete failure. A build that degraded the cut to
   a copy would satisfy 1 and 2 and still hand the operator a duplicate.

`certified-comments.pdf` and `threaded-comments.pdf` differ in exactly
one dictionary — the catalog's `/Perms` — so the pair tells *"withheld
here"* from *"offered there"* while varying one thing. This test drives
the refusing half; the offering half is the driven check's.

### `fn the_offset_is_same_page_only`

Asserted as arithmetic rather than by driving, because the *decision*
is the thing worth pinning: whether the copy is visible when it lands on
top of its original is a property of this one comparison, and a driven
check would prove it for one pair of pages.

### `fn a_sticky_note_reaches_the_clipboard`

This is the operator-facing whole of the change. `Ctrl+C` over a `/Text`
annotation used to answer *"that annotation is not one pdfcer authors …
so there is nothing for it to copy"*, because the clipboard read a
`MarkupSpec` out of the dictionary and `spec_from_dict` has no reader for a
sticky note. A sticky note is the most-copied comment in a review workflow.

# What the assertions are, and why the obvious one is not enough

### `fn a_modelled_markup_keeps_what_a_spec_cannot_say`

It asserted the **carrier**: that a `/Square` came back as
`Clipped::Markup`, because `paste_clip_annotations` planted a clipped
markup with `add_markup` — **not** `add_markup_with` — and therefore
dropped `/CA`, `/T`, `/M` and `/Contents`. Routing everything through the
clip because "the clip is the lossless one" would have compiled, passed a
*"the paste happened"* test, and handed the operator an anonymous, undated,
opaque copy of a signed comment.


⇒ So the assertion moved to the **payload**, and accepts either carrier.
That is the better test and it should have been written this way from the
start: the operator cannot see which route ran, and a route assertion goes
red on an improvement.

`/CA 0.4` rather than `/CA 1` in the fixture is deliberate and is the
difference between this test working and being vacuous — an opacity of 1
is what an absent `/CA` renders as, so a build that dropped the key would
look identical on screen and identical to a sloppier assertion.

⚠ **`/M` is no longer asserted, and that is a real narrowing rather than a
tidy-up.** `MarkupCarry` does not carry it, and it should not: a paste
**authors a fresh mark**, so a new modification date is the correct answer
rather than a dropped one — this shell stamps `/M` itself wherever it
authors (`app::clock::pdf_date_utc`). The spec route did carry the
original's `/M`, which on reflection was the *wrong* behaviour: it dated a
mark created today with the date of the one it was copied from.

### `fn cutting_a_sticky_note_deletes_the_one_it_copied`

Two assertions and the second is the one worth the test. A cut raises the
delete by `ObjId` taken **off the clip**, not by re-reading the selection
after the copy: two walks of `/Annots` with an edit between them can
disagree, and the window between them is exactly where a cut removes the
neighbour of the thing it copied.

### `fn the_annotation_survives_the_clips_own_serialisation`

A build whose serialiser dropped the annotation payload would still park a
clip, still raise a `PasteObjects`, and still trace a paste — the content
half of every one of those is identical. What differs is the bytes, so this
asserts the round trip through `ObjectClip::from_bytes` rather than
trusting the action was raised: the bytes are what the clipboard actually
holds and what a cross-process paste would carry.
