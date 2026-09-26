# `pagetree` — **does this document's page tree still agree with itself?**


> *"I tested deleting pages from a pdf. when I open the document in Acrobat
> there are blank pages at the end of the document equalling the number of
> pages I deleted."*

## 1. The lesson this module exists to carry: **a GUI that checks its
own work with its own parser cannot see this class of defect at all**

This is the sentence to read before changing anything below, because every
design decision here follows from it.

ISO 32000-1 §7.7.3.2 gives a page tree **two** independent descriptions of
how many pages it has, and requires them to agree:

* `/Kids` — the children, walked recursively to the `/Page` leaves. **This
  is the structure.**
* `/Count` — on every `/Pages` node, *"the number of leaf nodes that are
  descendants of this node"*; on the root, the document's page count.
  **This is the declaration.**

A well-formed file has them equal at every node, so a reader may use either.
Readers therefore split into two camps, and the split is invisible until the
two disagree:

| reader | what it builds the page list from | what it saw after `delete-pages --pages 2,3` on a 36-page nested document |
|---|---|---|
| `pdfcer_core::page_tree::pages` — and therefore this whole shell | `/Kids`, walked | **34 pages. A healthy document.** |
| Acrobat | the root `/Count` | **36 pages, the last two blank** |

⇒ Every unit test this project could have written about page deletion, every
panel, every driven check, and the engine's own reader **all agree the file
is fine**, because they all read the same half of a contract whose other
half is the broken one. Two thousand five hundred passing tests could not
have caught it and did not. **The operator caught it.**

⇒ So this module does the one thing none of those do: it reads **`/Count`
raw off each node's dictionary** and compares it against the leaves actually
hanging below that node. It never asks `page_tree::pages` anything. A
rewrite of this module that starts by calling `pages()` and comparing its
length to something would be **vacuous** — it would compare the walked
structure against itself and pass on every corrupt file in existence.

## 2. And it must be a NESTED tree, or the question is unfalsifiable

The second reason the defect shipped. On a **flat** page tree — one root
whose `/Kids` are all `/Page` leaves — the immediate parent *is* the root,
so a writer that updates only the immediate parent updates the root by
accident and the file is correct. The bug **cannot occur**. Reproduced
against `fixtures/four-pages.pdf` first, and it was clean.

Every synthetic fixture in either corpus had a one-level tree. Real CAD
exports do not: SolidWorks nests, and so does every producer that writes its
page tree in balanced chunks. So the defect was invisible to the corpus and
present on every document the operator actually works with.

⇒ `fixtures/nested-page-tree.pdf` (`tools/gen-nested-page-tree-fixture.py`)
exists for this module and its header states, in as many words, that the
nesting **is the point** and that swapping it for a flat document would make
every assertion here pass against a build whose walk never goes above the
immediate parent.

## 3. What this module is NOT: a repair

It never writes. It has no `&mut` anywhere and takes its graph by shared
reference. The temptation — *"we know the right `/Count`, just fix it"* — is
refused on the boundary argument this project applies everywhere:

`pdfcer-core` **owns** the page tree. Every write of a page-tree `/Count`
is in `pdfcer_core::edit` or `pdfcer_core::pageops::assemble`, and this
shell issues none. A shell that silently patched the same key would
be a **second writer of one structure**, and the two would drift — pdfcer
would then be repairing files against a rule the engine had since changed,
on documents nobody was looking at, with the operator told nothing. The
defect would move from *"Acrobat shows blank pages"* to *"pdfcer edits your
page tree behind your back and is sometimes wrong about it"*, which is
strictly worse because it is no longer visible.

⇒ **Refuse, name what is wrong, say what it cost, and file the defect
upstream.** Filed as
`request_delete_pages_leaves_ancestor_count_stale_on_a_nested_page_tree.md`,
whose closing offer stands: *"If `pdfcer-core` would rather own that check —
a `validate_page_tree()` a writer calls before it commits — we would use it
and delete ours the same day."*

## 4. Why the raw-dictionary read is legitimate here, and where it drifts

Reading `/Count` off a dictionary through [`ObjectGraph`] is a seam, and
this project has one standing precedent for it — `canvas::notepopup::model`'s
`open_flag`, which reads `/Open` the same way and admits in its own header
that it is a workaround. The same admission is owed here and is made:


⇒ So there is no non-raw way to ask this question, and the raw way is a
**read of a key ISO 32000-1 §7.7.3.2 defines, through the crate's own public
graph**. Nothing is guessed and nothing is written. The day `pdfcer-core`
models a page-tree node, [`audit`] becomes a loop over that type.

## 5. Why it walks itself instead of using `PageSlot::ancestors`

`page_slots` would have been fewer lines, and
`panels::forms::tab_order::tabs` sets the precedent for reading a raw key
off the nodes it names. It is not used, for one reason that matters and one
that does:

* **It cannot see a node with no leaves under it.** A `/Pages` node appears
  in `PageSlot::ancestors` only when some leaf is beneath it, so a subtree
  whose every page was deleted is invisible to it — and *"every page under
  this node was removed"* is precisely a state a page deletion produces. The
  walk below sees that node, with `reachable: 0`, and can refuse on it.
* It answers the wrong shape: a per-leaf ancestor list has to be inverted
  into a per-node tally anyway, and the inversion is the same recursion.

Worth recording, because it is the engine stating the very contract it
then broke: `PageSlot::ancestors`' own doc comment
reads *"Every ancestor `Pages` node, root first,
excluding the page itself. **A delete must decrement `/Count` on all of
them.**"* The requirement is written into the type. It is `delete_pages`
that does not do it.

## 6. The verdict this module can and cannot return

| state of the file | [`Audit::disagreements`] | the save |
|---|---|---|
| every node's `/Count` equals its leaf tally | empty | proceeds |
| some node declares a number it does not have | one entry per node | **refused** |
| a `/Pages` node carries **no** `/Count` at all | empty — see below | proceeds |
| the bytes cannot be parsed, or there is no `/Pages` root | empty, with [`Audit::walked`] `false` | proceeds |

**An absent `/Count` is deliberately not a disagreement.** §7.7.3.2 requires
the key, so a node without one is malformed — but it is a *different*
defect, it is not one any pdfcer verb produces, and it arrives most often on
a file pdfcer merely opened. A guard that refused it would refuse to save
documents pdfcer did not damage, which is the failure mode
`redact::proof`'s own header warns about at length: a false refusal after
the operator has done the work, with no route to a file. It is counted in
[`Audit::nodes_without_count`] so it is visible in the trace and unmeasured
by nobody.

**An unwalkable document is deliberately not a refusal**, on
`redact::proof::decoded_streams_of`'s standing reason, quoted because it is
the same argument: *"a document that cannot be re-parsed yields an empty
list rather than a panic or an error… a skip narrows the evidence rather
than fabricating it."* [`Audit::walked`] is what makes the narrowing
visible instead of silent — a clean audit and an audit that never ran are
otherwise byte-identical, which is this project's most-repeated failure
shape.

## 7. Guards

A page tree is operator-supplied and may be hostile or merely broken. The
walk is bounded three ways, each mirroring the engine's own walk so that a
document `pages_in` accepts is one this accepts:

* **cycles** — a `visited` set of [`ObjId`]; a node reached twice
  contributes zero and is counted in [`Audit::cycles`].
* **depth** — [`pdfcer_core::page_tree::MAX_TREE_DEPTH`], the engine's own
  constant rather than a second number that could drift from it.
* **recursion** — the walk is recursive, exactly as the engine's is, and the
  depth bound is what makes that safe. `pdfcer-core`'s panic-free policy
  forbids a stack overflow on untrusted input and this honours it by
  borrowing the same ceiling.

## 8. Where it is called from

`app::save::write_copy` — the funnel every save verb goes through
(`file.save_copy`, `file.save_as`, `file.save_in_place`), between the bytes
being built and `std::fs::write`. **Not** on the delete-pages arm.

That placement is the point, and it was vindicated the same day. The defect
is a *writer's* invariant, so all seven page verbs were measured on the
three-level fixture rather than assumed:

| verb | ancestor `/Count` after |
|---|---|
| `delete_pages` | ✗ **stale** — only the immediate parent updated |
| `page-copy --cut` | ✗ **stale**, byte-for-byte identical output |
| `insert_pages`, `extract_pages` | ✓ (they rebuild the tree flat) |
| `reorder_pages` | ✓ |
| `paste_pages`, `merge_document` | ✓ — and these demonstrably **do** walk to the root |

⇒ A guard on the delete-pages arm would have passed `page-copy --cut`
straight through, and would have to be remembered again for every verb added
later. `redact::prove_saved_bytes` sits at the same boundary for the
same reason and its argument is the precedent: *"the proof has to be made
here or not at all."*

And **which** sentence a refusal owes is decided here too, by
[`refusal_origin`], rather than at the save. That is not tidiness: the
choice depends on a **second audit** — of the file the document was opened
from, to answer *"was it already like this when he opened it?"* — and this
module is the only place equipped to take one. `pdfcer-gui`'s `text::pagetree`
owns every word.

## 9. What it costs, measured rather than asserted


| document | size | the walk itself | [`audit_saved_bytes`] end to end | `to_incremental_bytes` on the same document |
|---|---:|---:|---:|---:|
| `D:/Dev/pdfTests/SW41177/SW41177.pdf` — 36 pages, nested, his own drawing set | 1,831,090 B | **16.7 µs** | **1.78 ms** | 1.58 ms |
| `D:/Dev/pdfTests/ncored-benchmark-cad-drawing.pdf` — 129,758 objects | 5,724,699 B | **0.6 µs** | **3.51 ms** | 5.41 ms |
| `fixtures/nested-page-tree.pdf` | 5,026 B | 4.2 µs | — | — |

**A first draft of this paragraph claimed the guard was "beneath"
`to_incremental_bytes`, and that was wrong.** It was written from the shape
of the code rather than from a measurement, which is the exact error
`BENCHMARK.md` commemorates — written into the very paragraph citing it. The
measured truth: on the operator's drawing set the guard costs **more** than
the writer that produced the bytes (1.78 ms against 1.58 ms), roughly
doubling a save's CPU; on the large CAD sheet it costs about two thirds of
it. The guard is not free and this paragraph will not pretend it is.

⇒ It is placed on every save anyway, and the reason is that ~2–3.5 ms is
**imperceptible against the act it is part of**: an operator-initiated save
that opens a file dialog and then writes megabytes to disk. Doubling a
millisecond inside a gesture that takes a second is not a cost he can
observe. The number that would change this decision is tens of milliseconds,
and it is an order of magnitude away.

**The walk is not where the time goes; the re-parse is.** The walk barely
moves with file size — the benchmark sheet is three times the bytes and
*faster*, because it has one page — since it is bounded by the page tree
rather than by the document. So a future engine that let the guard read the
written revision without re-parsing the whole file would make this free
rather than cheap.

⇒ So it is **ungated**: run on every save of every document, not only after
a page-count change. That is a decision and it has a reason beyond the
numbers — a gate on *"has the page count changed this session?"* would be a
second source of truth about a fact the bytes already carry, and the
direction it would drift in is a save that quietly wrote a damaged file
because a shell-side flag disagreed with the writer. The same argument
`app::save::write_copy` makes for asking the **session** whether a
redaction is staged rather than keeping a flag.

## Item notes

### `fn walk`

Returns the **structural** answer — what `/Kids` actually holds — never the
declared one. That direction is the whole of §1: a walk that short-circuited
on `/Count` would be reading the field it is here to check.

The disagreement is recorded **after** the children are counted, so
[`Audit::disagreements`] comes out deepest-first with the root last, which
is the order a reader of the trace wants: the first entry is the innermost
node that is wrong.

### `fn count_of`

`as_int` rather than `as_number`: §7.7.3.2 says integer, and a `/Count 3.0`
is a file this guard declines to judge rather than one it refuses — the same
posture §6 takes for an absent count, for the same reason.
