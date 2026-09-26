# `pdfcer-gui/app/cache/provenance`

**One provenance-bearing text extraction per page, shared by everything
that edits text.**

# What this is, in one sentence

`pdfcer_core::text_extract::extract_page_view` with
`ExtractOptions::with_provenance(true)`, run **at most once per
`(page, edit epoch)`**, so that the six places in this shell that need to
know *which show operator a glyph came from* pay for that answer once
between them instead of six times each.

# Why this exists — it is a measured cost, not a tidiness

Provenance is the substrate for editing text: it is what turns *"the
operator clicked run 41"* into *"byte span 8210..8263 of content stream
12"*, which is the only thing `EditSession`'s text verbs can act on. The
ordinary read-only extraction in [`crate::app::cache`] deliberately leaves
it **off** — see `ensure_page_text` — because capturing it costs and the
find bar does not need it.

The cost is not marginal. `crate::panels::properties::refusedchar`'s header
records the measurement on the operator's own benchmark CAD sheet:

> *`pin::inspect` plus `preview_font_resources` — **392 ms** for the first
> alone.*

Before this module there were six independent copies of the same four
lines, each paying that in full:

| site | when it ran |
|---|---|
| `app::cache::ensure_form_runs` | every click on the canvas, to ask whether the clicked run has an anchor |
| `canvas::textedit::pin::inspect` | opening the properties panel on a text run |
| `canvas::textedit::pin::resolve` | every restyle |
| `canvas::textedit::pin::operators` | every `textstyle` action |
| `canvas::textedit::plan` | every committed text edit |
| `canvas::textedit::reflow::block_of_run` | every reflow |

Two of those run **in the same gesture**: a click on a text run calls
`run_has_no_anchor` (extraction one) and then, if the properties panel is
open, `pin::inspect` (extraction two) — 784 ms on the benchmark sheet for
one click, to compute the same `PageText` twice and throw one of them away.

# Why the cached value is the `PageText` and not the model

`EditableTextModel::recognize` borrows the `PageText` it describes, so a
cache holding the model would have to hold both together and hand out a
self-referential borrow. It also takes `BlockRecognitionOptions`, and
`reflow` uses a *different* set from the caret path deliberately — see
`reflow::tests`. So the expensive, shared half is cached and the cheap,
caller-specific half is not: every caller re-runs `recognize` over the
cached text.

That is the correct split on cost as well as on correctness. Recognition
walks runs that are already decoded and laid out; extraction resolves
`/Contents`, inflates, tokenizes, resolves fonts and decodes every string.

# Why the handle is [`CachedText`] and not `Ref<'_, PageText>`

Every other cache in [`crate::app::cache`] hands out a `Ref`, and that
module's header sets out the three-part argument for why that is sound.
This one does not, for one reason the others do not face: **its callers
pass a page index rather than reading the current page.**

`pin::inspect(doc, page, run)` names its page. So does `operators`, so does
`plan`. Nothing stops one of them from holding a handle to page 3 while a
function it calls asks for page 7 — and with a `Ref` outstanding that is a
`RefCell` double-borrow **panic**, in release, in front of the operator,
for a pattern the type system was happy with.

So the cache stores an [`Rc<PageText>`](std::rc::Rc) and hands out a clone
of it. The `RefCell` borrow lives for the length of one `Rc::clone` and is
gone before the caller sees anything, so two pages in flight is a refcount
of two rather than a crash.

⚠ **That trade would give up the other half of the `Ref` argument** — a
live `Ref` borrows `&OpenDoc`, which is what makes it impossible for the
epoch to move while a cached value is held, which is what makes it
impossible to act on stale text. Losing *that* would be worse than a panic:
a silent edit against a page as it was two revisions ago.

[`CachedText`] is how both are kept. It is an `Rc` for the borrow, plus a
`PhantomData<&'a OpenDoc>` for the lifetime, so the compiler still refuses
to let the handle outlive the `&OpenDoc` that produced it or coexist with a
`&mut OpenDoc`. The staleness property is enforced exactly as it is for
every other cache here; only the re-entrancy panic is given up.

## Item notes

### `fn a_second_reader_shares_the_extraction_rather_than_rebuilding_it`

The property this module exists for, asserted the only way a unit test
can assert it: [`Rc::ptr_eq`](std::rc::Rc::ptr_eq) on the two handles.
A cache that rebuilt would hand back two distinct allocations holding
equal text, and `assert_eq!` on the text would pass on both — which is
exactly the shape of check that let six duplicate extractions sit in
this crate unnoticed.

The two handles are **held at once**, deliberately. The old caches in
[`super::super`] hand out `Ref`s and their equivalent test holds two to
prove a shared borrow is enough; this one hands out an `Rc` precisely so
that overlapping handles are possible, and a test that took them one
after the other would not exercise that.

### `fn a_page_step_rebuilds_the_extraction`

The control for the test above, and not a formality: a cache that
ignored its key would pass that one and serve page 0's runs for page 1
— which on the edit path means a pinned span naming a byte range in the
wrong content stream, i.e. an edit applied to a page the operator is not
looking at.
**The handle is HELD, never dropped, and that is the whole
correctness of this test.**


⇒ **A pointer-identity check across a deallocation is not an identity
check.** It is asked "is this a different allocation?" and the
allocator is free to answer "no" about a genuinely different object.
Both directions are unsound: it can report a rebuild as a cache hit
(the failure actually seen), and it could equally have reported a cache
hit as a rebuild, which would have sent somebody hunting a cache bug
that does not exist.

Holding `page0` for the life of the test fixes that at the root rather
than by tolerance: while a strong `Rc` is alive the address cannot be
reused by anything, so a differing pointer is proof of a different
allocation and an equal pointer is proof of the same one. It costs
nothing — `CachedText` is a handle, and this module hands out an `Rc`
precisely so that overlapping handles are legal.

⚠ Its sibling above, `a_second_reader_shares_the_extraction_…`, was
never exposed to this: it compares two handles that are **both alive**,
which is the sound shape. Do not "fix" that one by symmetry.

### `fn an_edit_invalidates_the_extraction`

The second half of the key. Serving pre-edit text after an edit would
hand the next verb a byte span measured against a content stream that
no longer exists — the silent-wrong-edit failure the module header
calls worse than a panic.

The epoch is moved directly rather than by performing an edit,
because this is a test of the **cache key**, not of any verb. A test
that ran a real edit would fail for a dozen reasons that are not this
one, and would stop compiling every time a verb's signature moved.

### `fn an_unreadable_page_records_the_attempt`

The failure arm. The second clause is what stops a document whose
current page cannot be extracted from paying a third of a second per
frame — the measurement in `app::cache`'s header — to learn the same
thing sixty times a second.


What this does catch is the change that actually breaks the property:
returning early on the failure arm without recording the key.
