# `app::cache` — the three derived values a document is worth keeping, and why they live on it

## What is in here

Three caches, and the [`OpenDoc`] methods that read them:

| cache | what it holds | keyed on | read by |
|---|---|---|---|
| [`PageObjectCache`] | the current page's decomposition | `(page index, edit epoch)` | the Objects panel, the Properties panel, the canvas hit test, the `objects n=` trace |
| [`FontCache`] | the document's font inventory | `edit epoch` | the Fonts panel, the Properties panel |
| [`PageTextCache`] | the current page's **extracted text** | `(page index, edit epoch)` | canvas text selection, `file.copy_page_text` |

## Why this is a module of its own — the seam, stated

`state.rs` answers *"what is open, and what is the operator looking at?"*
— [`crate::app::state::Status`]'s three-way failure distinction,
[`OpenDoc`]'s view fields, the raster bookkeeping that keeps the page
texture honest. The caches answer a different question: *"what expensive
thing derived from the document do several surfaces need, and how do we
compute it once?"* They share one argument (the cost of `pdfcer-core`
recomputation), one hazard (staleness against `edit_epoch`), and one
structural device (a `Cell` key beside a `RefCell` payload, for the borrow
reason below). None of that is shared with anything left behind.

The seam is the one the caches themselves imply: *a cache is bounded by the
lifetime of what it describes* is a statement about caches as a class
rather than about any one of them, and it is why they hang off [`OpenDoc`]
rather than off `crate::panels::PanelsState`.

## Why interior mutability, and why that is not a smell here

A panel body is handed `&OpenDoc`, never `&mut` — that is the
actions-not-mutations invariant, and it is not negotiable
(`PROJECT_PLAN.md` §3). A lazily-built cache behind a shared reference is
precisely what [`RefCell`] is for: the cache is *derived*, so filling it
changes nothing an observer could see, and the alternative — building it
eagerly on every page change whether or not any surface asked — would cost
a decomposition per page step with the Objects panel closed.

It applies to **caches only**, which is the other reason they are worth
collecting in one file: the exemption has a visible boundary. State that
decides what appears on the page (the layer override, the annotation flag,
the selection) stays behind `&mut self` over in `state.rs` and reaches it
through an [`crate::app::actions::Action`], or *"what can change what is
drawn?"* stops having a complete answer.

## Why neither cache can panic on a double borrow

The `RefCell` hazard is a `borrow_mut` taken while a `Ref` is still alive.
It is unreachable here by a borrow-checker argument rather than by care,
and the argument is the same for both caches:

1. The validity key is a [`Cell`], **outside** the `RefCell`, so the
   already-built path reads it and takes only a shared `borrow()`.
2. `borrow_mut` is reached only when the key has *moved*.
3. A live `Ref<'a, …>` borrows `&'a OpenDoc`, so while one exists nothing
   can take `&mut OpenDoc` — and `view.page_index` and `edit_epoch` change
   only through `&mut self`. The key therefore **cannot** move while a
   `Ref` is outstanding.

Keeping the key in a `Cell` rather than inside the `RefCell` is what makes
step 1 true, and is the entire reason each cache is two fields rather than
one. [`tests::a_second_reader_shares_the_decomposition_rather_than_rebuilding_it`]
holds two `Ref`s at once, so the property is exercised and not merely
argued.

## Item notes

### `fn ensure_page_objects`

The key is recorded **before** the work, so a page whose content will
not decode is not re-decomposed on every frame: the failure is
deterministic, and retrying it sixty times a second would peg a core
producing the same error.

### `fn ensure_page_text`

The key is recorded **before** the work, for the reason
[`Self::ensure_page_objects`] records its own: the failure is
deterministic, and retrying it every frame would peg a core producing
the same error.

### `fn a_documents_decomposition_cannot_outlive_the_document`

Asserted rather than argued. A cache hanging off the *application*
outlives the document it describes, so it would have to say **which**
document — and the only token available is an `Arc` address, which is
not an identity (see `OpenDoc::page_objects`).

This replaces one document with another **in the same binding**, the
sequence that would exercise an address reuse. There is nothing to get
wrong: the second document's cache is a field of the second document.
The remaining key is `(page, epoch)`, and it is asserted here so that
putting an address, a pointer or a `Weak` back into it is a test
failure rather than a review finding.

### `fn the_decomposition_is_rebuilt_when_the_page_or_the_revision_moves`

Both halves of the key, one at a time. Serving page 0's objects while
the operator is on page 1 would make every index in the Objects panel
address the wrong object.

It asserts that the key **CHANGED**, not what it changed to. The second
half is not `edit_epoch` but the engine's content digest (see
[`super::OpenDoc::page_objects_revision`]), and a test pinning the
literal would have to be rewritten for a change it exists to be
indifferent to. What matters is that a rebuild happened, which is what
a changed key means and all it means.

### `fn a_missing_page_yields_no_decomposition_and_no_invented_reason`

The attempt is recorded either way, so it is not retried sixty times a
second. But "there is no such page" must not be reported as a decode
failure: the trace channel distinguishes `reason=no-such-page` from
`reason=decompose-failed`, and a consumer is entitled to that.

### `fn a_pages_text_is_extracted_once_and_shared`

The property the whole feature's affordability rests on: a text drag
asks for this on every frame of the gesture, and an unconditional
whole-document extraction costs a measurable fraction of a second —
`crate::find`'s header states that cost and its `find … ms=` trace line
measures it. Holding the first `Ref` across the second call is the
assertion rather than an accident of how the test is written: it is also
the case that would panic if the validity key lived inside the `RefCell`
instead of beside it.

### `fn the_page_text_is_rebuilt_when_the_page_or_the_revision_moves`

Both halves of the `(page, epoch)` key, one at a time — the same two
failures [`the_decomposition_is_rebuilt_when_the_page_or_the_revision_moves`]
guards, and sharper here: a stale `PageText` does not merely list the
wrong objects, it makes every `TextPosition` in a live selection name a
run that has moved, so the highlight would be drawn over the wrong
glyphs and the copy would carry them.

### `fn a_missing_page_yields_no_text_and_no_invented_reason`

The attempt is still recorded, or it is retried every frame — the same
rule [`a_missing_page_yields_no_decomposition_and_no_invented_reason`]
states for the decomposition.

### `fn the_font_inventory_is_kept_across_pages_and_dropped_by_an_edit`

It decodes every embedded font program, so rebuilding it per page is a
large cost for a value that cannot have changed — and an edit *can*
add or remove a font, so keeping it across one reports a font list the
document no longer has.
