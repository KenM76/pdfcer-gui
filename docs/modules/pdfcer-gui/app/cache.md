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
