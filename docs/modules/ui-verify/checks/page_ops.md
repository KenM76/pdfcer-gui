# `ui-verify/checks/page_ops`

`page_ops_round_trip` — **the check for the tab that did nothing**.

# What was wrong, and why nothing noticed

Every one of `pages.rotate_left`, `pages.rotate_right`, `pages.delete`,
`pages.extract`, `pages.move_up` and `pages.move_down` was registered, drawn
on the ribbon's **Pages** tab, offered by the page tile's context menu, and —
for four of them — bound to a chord (`[`, `]`, `Alt+Up`, `Alt+Down`). None of
them had a dispatch arm. Every press traced `command-unimplemented` and did
nothing at all, and `FEATURES.md` said the Pages panel shipped *"a context
menu of the six page verbs"*.

The suite was green throughout, and it had to be. `pdfcer-core` tests
`delete_pages`, `reorder_pages` and `rotate_pages` exhaustively;
`shell::commands` tests that all six are registered;
`panels::pages::select` tests the multi-select that feeds them. What no test
in the workspace can observe is **the join** — that pressing the control
reaches an arm, that the arm reaches the engine, that the engine's result
reaches the page vector the canvas draws from, and that it survives onto
disk.

# What this check adds that `save_copy_round_trip` does not

That check proves an **annotation** reaches a file. This one proves a
**structural** change does, and the two are different claims because the
failure modes are different: an annotation is a new object nothing else
refers to, while a page delete renumbers every index in the application. The
two assertions no other check in the suite makes are:

* **`renumbered=` is right for each verb.** `crate::app::actions::pages`'
  resync decides — from the page-object identities alone — whether an edit
  moved what an index *means*. A rotation must report `renumbered=0` and a
  reorder and a delete must report `1`. A build that got this backwards
  would either throw away the operator's canvas selection on every rotate,
  or keep it pointing at another sheet's objects after a delete with
  `format.delete` one keystroke away. Both are silent.
* **The page count in a SECOND PROCESS.** The in-process count is written by
  the code under test about itself; a build that updated the panel and never
  touched the session would report it perfectly.

# The phases

| Phase | Does | Expected |
|---|---|---|
| A | Review, Pages tab | `open ok pages=N` read as the baseline |
| B | **Rotate right** | `rotate-pages … epoch=` and `pages-resync … renumbered=0` |
| C | **Move down** | `reorder-pages … epoch=` and `pages-resync … renumbered=1` |
| D | **Delete** | `delete-pages … epoch=` and `pages-resync was=N now=N-1 renumbered=1` |
| E | File tab, **Save a copy** | `save-copy … appended=>0`, a file at the named path |
| F | re-read the **source** | byte-identical to before the run |
| G | compare the copy's prefix | the source's bytes are the copy's prefix, verbatim |
| H | grep the copy's **bytes** | `/Rotate 90` appears, and the source has no `/Rotate` at all |
| I | **second process** on the copy | `open ok pages=N-1` |

## Why the three verbs are driven in that order

So that each one's result survives the next, which is what lets one save
prove all three:

1. **Rotate right** turns page 1 to 90°.
2. **Move down** sends page 1 to position 2. Position 1 is now the sheet
   that was page 2. `view.page_index` is unchanged — a reorder removes
   nothing, so nothing is clamped — so the *current page* is now a different
   sheet, which is precisely the renumbering this check is about.
3. **Delete** removes the current page, which is that other sheet. **The
   rotated one survives**, and it survives *because* the reorder moved it —
   so a build whose reorder did nothing would delete the rotated sheet and
   fail phase H, which is a second, independent way for phase C to be
   caught.

None of the three is given an operand by this check. With nothing picked in
the Pages panel they act on the **current page**, which is
`crate::panels::pages::ops::operands`' documented rule and is the path an
operator who has never opened the panel takes.

# Three falsifying phases, and the build each one catches

## D catches: *the panel changed and the session did not*

The build this check was written against. A delete that updated a page count
held in the shell — or that removed a tile from a grid — and never called
`EditSession::delete_pages` produces a correct-looking count everywhere an
in-process assertion could look. It is caught here because `pages-resync`
is emitted from a comparison against **`EditSession::pages()`**, and again in
phase I by a process that never saw this one.

## H catches: *the rotation never reached the file*

A `/Rotate` written into a page dictionary the save did not carry, or a
rotate that only spun the raster, produces a saved copy with the right page
count that opens perfectly. The source is required to contain **no**
`/Rotate` at all (see the SKIP list) so that finding `/Rotate 90` in the copy
is unambiguous evidence of this run's edit rather than of the fixture's own
furniture.

## I catches: *a file was written and the delete is not in it*

The one that gives the check its name, and `save_copy_round_trip`'s phase F
argument applies unchanged: a build that wrote the **base document's** bytes
produces a file that exists, opens, and is byte-identical to the source. It
passes E, F and G — G *trivially*, because the copy simply is the source —
and its page count is the one it started with.

# Two plants were RUN, and both fired

[`crate::checks`]' rule for a new check is that *"it must fail against a
build where the wiring is absent"*, and that every check here "has been run
against such a build and seen to fail". Two builds were planted against
`D:\Dev\pdfcer\fixtures\synthetic\pageops\four-pages.pdf` (4 pages, 2453 B,
no `/Rotate`):

**Plant 1** — the five `pages.*` dispatch arms **deleted** from
`app/dispatch.rs`, which is the shipped v0.1.0 state. **FAIL at B**:
*"`ribbon.item.pages.rotate_right` WAS INVOKED AND THE ENGINE NEVER RAN
… the application traced `command-unimplemented id=pages.rotate_right`"*.

**Plant 2** — the resync's renumbering test in `app/actions/pages.rs`
reduced to `before.len() != now.len()`. **FAIL at C**: *"A REORDER WAS NOT
TREATED AS A RENUMBERING: `pages-resync was=4 now=4 renumbered=0`"*, having
passed B.

The second is the more instructive, and is the reason phase C exists rather
than being folded into phase D. **A page count is not a renumbering test.**
The planted build's rotate phase passed, its delete would have passed, its
save would have passed and the round trip in phase I would have passed —
every page really is in the file, in the right order, because the *engine*
did the reorder correctly. What that build gets wrong is entirely inside the
shell: the canvas selection and every cached strip raster go on pointing at
page indices that have changed meaning, which is invisible in a count and
visible on screen as an outline round the wrong object.

The unedited build passes both. That pair is what makes a green result here
evidence rather than an absence of evidence.

# What this check does NOT cover, stated rather than implied

* **The keyboard.** `[`, `]`, `Alt+Up` and `Alt+Down` are bound in the
  manifest and none of them is pressed *here*: this check drives the ribbon.
  They are not unreachable. Synthetic keystrokes DO reach the target window,
  and [`crate::checks::chords`] presses `[` and `Alt+Down` and asserts the
  `chord-command` line each one resolves to. `]` and `Alt+Up` rest on
  `shell::manifest`'s keymap test and on the single dispatcher every route
  shares, and that narrower gap is on the record rather than papered over by
  a green result.
* **The page tile's context menu.** Reaching it means mounting the Pages
  panel, finding a tile and right-clicking it; the menu that opens is an
  `egui` popup which declares no `ui-rect` regions, so there is nothing to
  aim at. The six verbs it offers are the six driven here through the ribbon,
  and both routes reach `PdfcerApp::dispatch_command` — which is the whole
  point of one choke point.
* **Extract.** It opens its own window, which this OS-input check does not
  drive; `extract_pages_keeps_or_drops_the_labels` drives it with the
  scripted pointer, from the ribbon click to the written file.
* **Which page was rotated.** Phase H proves *a* page in the file carries
  `/Rotate 90`; it does not prove it is the one that was on screen. Reading
  that from the bytes needs a page-tree walk this crate has no parser for.

# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the diagnostic switches did not reach the process;
* the document has **fewer than three pages** — a delete needs a spare and
  the count assertion needs headroom;
* **the document already carries a `/Rotate` entry** — phase H's evidence
  would then be indistinguishable from the fixture's own;
* a mode segment, a tab or a control was never declared, or took no click.

## Item notes

### `const MODE`

`crate::app::modes::capability`'s own table records that the Pages tab is
*"❌ in Read, ✅ in Review"*, and `crate::panels::pages`' header carries the
operator's reason for that: *"Reviewing a set means rotating a sheet to read
it and extracting the pages you were asked about … page operations do not
alter content."* Driving the weaker of the two modes that offer these verbs
is the stronger claim — what works in Review works in Edit.

### `const RESYNC_EVENT`

Emitted by `crate::app::actions::pages::resync` only when the page vector it
held disagrees with the one `EditSession::pages()` now reports, so its
*absence* after a page verb means the edit never reached the session and its
`renumbered=` field is the shell's own answer to *did an index change
meaning?*.

### `const MIN_PAGES`

Three, and each one is spoken for: one is rotated and must survive, one is
deleted, and one is left over so the document is not reduced to a state where
`EditSession::delete_pages`' *"§7.7.3.3 requires at least one page"* refusal
is anywhere near.

### `fn digest`

The same function, for the same reason, as `save_copy`'s and `ocr`'s: the
question is *"did this file change"*, the adversary is a bug rather than a
forger, and the **length is part of the digest** so a truncation cannot hide
behind a hash collision.

### `fn occurrences`

A count rather than a presence, because phase H's evidence is comparative:
the interesting statement is *the copy has one and the source has none*, and
a bare `contains` could not say the second half about a fixture that happened
to carry the string in a comment.

### `fn invokes`

A **count**, never a presence: this check clicks five different controls in
one run, and *"has it ever been invoked?"* would be answered `true` by a
click made ten seconds earlier.

### `fn opened_pages`

From `open ok pages=N`, which `crate::app::lifecycle` builds from
`doc.pages.len()` — the vector this whole check is about — at a moment when
that vector has just come out of `page_tree::pages` on the file. In the
second process that makes it a statement about **the file on disk**, made by
the engine, in a process that never saw the first one.

### `fn click_command`

A SKIP rather than a failure when nothing was reported, on
[`crate::checks::markup_rectangle`]'s rule: a check that could not deliver a
click has learned nothing about the application, and naming a feature as the
culprit when nothing was ever clicked at it is worse than no check at all.

### `fn no_effect`

One function for three phases, so the three read identically and none of
them can quietly lose the `command-unimplemented` half — which is the single
most informative line a reader of this failure can be handed, because it
distinguishes *"there is no arm"* from *"the arm ran and the engine refused"*.

### `fn resync_after`

Split out because all three phases ask the same question and the *absence*
of this line means something specific and worth spelling out once: the page
vector the shell holds and the one the session reports still agree, which
after a page verb means the verb did not change the document.

### `fn the_selectors_match_the_shells_own_spelling`

Pinned for the reason every sibling check pins its own: the two crates
are joined by a **string** and nothing else, so a rename would leave both
sides compiling while every assertion here quietly stopped matching — and
a check that matches nothing passes vacuously.

### `fn the_occurrence_counter_reads_both_directions`

Phase H's whole verdict rests on this function in both directions: a
counter that answered zero for a present needle would report a working
rotate as missing, and one that answered non-zero for an absent needle
would let a fixture's own `/Rotate` masquerade as this run's edit.

### `fn the_digest_notices_a_single_changed_byte_and_a_truncation`

Phase F's verdict rests on it, and phase F is this check's assertion that
a **page delete** did not reach the file the operator opened. A digest
that answered "unchanged" for a modified file would turn that into a
formality that always passes.
