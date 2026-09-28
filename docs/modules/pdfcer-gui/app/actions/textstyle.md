# `app::actions::textstyle` — changing how existing text looks

> *"We should also have all the font tools available that Word does."*
> — `OPERATOR_REQUESTS.md` **O37**

O37's inventory read the engine's text verbs — `add_text`, `edit_text`,
`delete_text_run` — and concluded, in a table with a column of crosses, that
pdfcer could choose how text looked when it was created and could not change
how existing text looked at all. Every cross in that column was wrong:
`EditSession::format_text` had shipped weeks earlier and had been extended
twice since. **An absence claim about a crate you do not build is a claim
about every route, and one verb is not every route.** This module exists
because somebody asked the engine instead of inferring from its index.

## What can actually be changed

| control | verb | limit |
|---|---|---|
| **size** | `set_size` | none — the `Tf` operand changes and the line is relaid out |
| **colour** | `set_fill` | none. pdfcer stores the SPACE the operator chose (`rg`/`g`/`k`) instead of force-converting to DeviceRGB the way Acrobat does |
| **face** | `set_font` | the target must **already be a font resource on the page**; refused by name otherwise (`FF-C`) |
| **face from a file** | `embedded_font` | the file must subset (`plan_subset`) and cover every character of the runs; one plan per gesture, built from every swept run's characters, so each operator's request carries the same subset |
| **bold / italic** | `set_style` | walks the ladder below; one named refusal, on italic only |

## The style ladder belongs to the engine, and rung 2 is why that matters

[`FormatRequest::set_style`] answers *"make this bold"* — as against
`set_synthetic`'s narrower *"thicken the strokes"* — by walking four rungs
inside the engine and reporting which one bound:

| rung | what it binds | reported as |
|---|---|---|
| 1 | a real face already on the page that claims the style and passes the coverage gate — same family first, then any family | `StyleRung::RealFaceOnPage` |
| 2 | the **standard-14 sibling of the run's own family**, as a new `/Font` resource, nothing embedded | `StyleRung::StandardFourteenSibling` |
| 3 | a donor face the caller supplies (`FormatRequest::style_donor`) | the shell passes none, so this rung never binds from the GUI |
| 4 | synthesis — the stroke or the shear | `StyleRung::Synthetic` |

Rung 2 is the rung that fires constantly and the one a shell-side walk
cannot reach. A CAD title block exported with `Helvetica` and nothing else
carries **no bold font resource at all**, so a gate that asks only *"is a
real bold face on this page?"* answers no — which is true of the page and
false of the world. `Helvetica-Bold` is one of the fourteen faces every
conforming reader is **required** to carry, needs no font file (ISO 32000-1
§9.6.2.2), and costs about sixty bytes to name. Thickening the strokes of
`Helvetica` instead is visible at 1:1 on a plotter and invisible to a test
that asserts the edit epoch moved.

⇒ So the walk is **not** re-implemented here. This module sets one verb,
passes the operator's posture straight through, and words what the engine
reports. A synthetic weight is the regular face thickened by stroking; a
synthetic slant is the upright face sheared. `R90` means neither is ever a
preference — only an explicit, per-use request — and the report says which
fired, in words passed through rather than re-written.

## Bold and Italic are never greyed

The engine's instruction, and this shell obeys it: *"Do not grey out a bold
button. Offer it, and surface the disclosure when synthesis fires."*

Greying would need this shell to predict a refusal that depends on a per-run
glyph-coverage test against a **candidate** face. `run_repertoire` answers a
narrower question — which characters the run's *current* font accepts — and
is cheap enough that `canvas::textedit::repertoire` calls it on every caret
landing. The question greying needs answered has only one instrument,
`preview_font_resources_for`, and the engine's own note on it says what that
costs: it walks every operation on the page, 129,758 objects on the
operator's benchmark sheet. Per frame, per button, that is not a greying
rule — it is a hang.

⚠ Both halves of that matter to whoever re-opens the question. *Nothing can
be measured here* is false; a verb measures exactly that. What keeps the
buttons live is the cost of the one instrument that measures the **right**
thing, not the absence of any instrument.

## Nothing here works around the engine

A twenty-line shell-side search for a different bold resource would work,
and it would be this project second-guessing pdfcer's font selection — and
the workaround every other consumer would then have to write for
themselves. **A workaround is a boundary defect, and its side effects are
usually invisible from inside the workaround.** The honest move is to report
the gap and let the engine close it once, for everyone.

The same rule decides which field a report is read from. Two `/Font`
resources can share one `/BaseFont`, so anything built from the **name**
reaches only one of the twins — possibly the twin that refuses. `selector`
is defined as the string that reaches the face pdfcer actually checked, and
it is therefore the field to use.

## How the engine's reports are read, and why each field is read that way

* [`StyleLadder::same_family`] is `Option<bool>`, and `None` means *nothing
  was bound*. It must **not** be flattened into `Some(false)`: the engine
  says so at the field, and the difference is *"the same typeface, heavier"*
  against *"a different typeface, which you will see on a plot"*.
  `family_stem` is private and engine invariant `R74` forbids this shell
  re-deriving it, so this flag is the only honest source for that sentence.
* `PassedOver { base_font, reason, refusal }` is structured, and
  `Refusal::character` gives the offending character. Were it a
  `Vec<String>` of `"BaseFont (reason)"`, saying *"pdfcer tried
  `Times-Bold` and it has no `o`"* in this shell's voice would mean a
  locator for the engine's message format living in a GUI, breaking
  silently the first time a reason gained a parenthesis — so the sentence
  would not be written at all, and the operator would be told less because
  of a string format.
* `EditSession::preview_style_ladder` is what the hover hints run, and it
  previews **the ladder**, not the `R90` gate. The gate answers only *"no
  real face on this page claims that style and covers this run"*, which
  stopped being the same question as *"what will pressing Bold do?"* the
  moment rung 2 existed: the standard-14 sibling is by construction **not on
  the page**, so the gate cannot see it. A tooltip built on the gate
  promises thickening on exactly the pages where a real `Helvetica-Bold` is
  about to be bound, and the status line afterwards reports the real face —
  two instruments the operator can consult, disagreeing by construction, on
  the commonest page in the working set. `preview_style_ladder` runs the
  same planner `format_text` runs, walks the page's content once, stages
  nothing, and takes the [`FormatOptions`] the commit will use, so under
  `Refuse` it previews the refusal rather than predicting a synthesis that
  would never happen.

[`FormatRequest::set_style`]: pdfcer_core::text_edit::FormatRequest::set_style
[`StyleLadder::same_family`]: pdfcer_core::text_edit::StyleLadder::same_family

## Why the runs are edited in descending order

The load-bearing decision in the file, and invisible until it is wrong.

`format_text` rewrites one **show operator**. A sweep can cover several, so
a restyle is several calls. Each call rewrites the content stream, so every
pin taken before it is stale afterwards — which is why
[`crate::canvas::textedit::pin::resolve`] is re-run between steps instead of
the pins being batched up front.

Re-resolving fixes the *spans*. It does not fix the **indices**: synthetic
italic brackets its run with two absolute `Tm` operators, and a `Tm` can
split a run, so an edit at index *k* may renumber everything after it.

Descending order makes that harmless by construction. Editing run *k* can
only insert operators at or after *k*'s position in the buffer, so runs
`0..k` keep both their bytes and their ordinals. Working downwards, every run
still to be done is always *before* the one just done, and its index is still
the index it was measured at.

Ascending order would work for four of the five controls and fail for italic,
on multi-run selections only, by restyling the wrong text — the shape of
defect that ships because the case that breaks it is the one nobody tries.

## What is NOT here, and is filed rather than hidden

**One gesture is N undo entries** when the selection covers N runs.
`EditSession` has no grouping verb; the engine solves multi-verb undo by
adding a *combined* verb per case, which is how markup authoring got its
opacity in one entry rather than two. A restyle across a paragraph therefore
takes several `Ctrl+Z` presses to take back.

Disclosed by the count rather than left to be discovered, and filed on the
request channel. **Not** worked around here: a shell-side coalesce would
work and would leave every other consumer with the same defect, which is the
boundary rule above stated from the other side.

## Item notes

### `fn stamp`

Its own function so the mapping from *a control the operator pressed* to
*a field on `FormatRequest`* is one readable table, and so a sixth
control cannot be added without appearing in it.

### `fn request`

Built fresh per run per step; see the module header on why a batch of these
taken up front would be wrong.

# Why there is no `find` string

`FormatRequest::whole_operator(page, span)` is exactly
`new(page, "").pinned(span)`, and the named constructor is used because it
says what it means. The alternative is to pass the run's own text as
`find`, which requires slicing that text into per-operator pieces — **a
second locator living beside the engine's**, and the class of defect where
two locators agree on every fixture and disagree on a ligature. There is no
such slicing here and `pin::Operator` carries no `find`.

An empty `find` with **no** pin is still refused by name, which is the right
way round: a caller that forgot to pin gets a refusal rather than silent
whole-operator behaviour.

# What this deliberately closes off

Restyling **part** of a run. A shorter `find` restyles a shorter span, and
`whole_operator` shuts that door for this call site. `FormatRequest::new`
with a real `find` remains for the day a sweep's byte offsets can be trusted
across an extraction, which `TextSelection::runs` does not yet offer.

### `fn resweep`

# The defect, in the operator's own gesture

Sweep three words. Press **Bold**: it applies. Press **Italic**: nothing
happens, and nothing says why. The wash has also gone from the page.

Bold is an edit, `super::apply::vector_edit` bumps `edit_epoch` on success,
and `TextSelection::live` is an equality test against that number. From the
frame after Bold the selection reports itself stale: it paints nothing and
`TextSelection::runs` hands back an empty list, so Italic's operand is empty
and `restyle` above declines with `NoRun`. Every pair of presses needed a
re-sweep in between, which is not a thing any editor in the class asks for.

# Why it is a re-resolution and not a re-stamp

`canvas::textsel`'s header §7 is emphatic that painting stored geometry
against a moved revision is the one thing rule 4 forbids outright, and it is
right: a restyle that changed a point size moves every glyph after it, so
the old quads would wash the wrong pixels and a subsequent restyle could act
on the wrong runs. So nothing is re-stamped. `textsel::reresolve` re-runs the
whole resolution from the operator's two positions against a fresh
extraction, and refuses unless the characters covered are identical. See its
docs for the argument; this function is only the wiring and the borrow
discipline.

# Called on every exit of `restyle`, including the refusals

A gesture that applied eleven runs and then stopped left the document
edited, so the selection is exactly as stale as a successful one. Wrapping
`restyle` rather than appending to its tail is what makes that true without
four call sites having to remember it — `restyle` has five early returns.

A no-op when nothing was swept, which is the whole clicked-object rung:
that operand comes from `app::textoperand`'s `Cache`, whose stamp includes
the epoch, so it re-resolves itself and needs nothing here.

# The cost, and why it is not a new one

One page extraction. `OpenDoc::page_text` is the shared cache and the edit
has just invalidated it, so this call pays for a rebuild — but the canvas
asks for the same extraction on its very next frame to hit-test the pointer,
so the work happens either way and this only moves it earlier by a frame.

### `fn emit_carried`

A second `vector_edit` would be a second undo entry, so this writes the
disclosure slot directly. That is the one place in this module that reaches
past the funnel, and it is sound because it changes **no document**: the
epoch it stamps is the one the final edit already bumped.

### `fn trace_rung`

Its own function so every `None` arm above costs one readable line, and so
the trace format is written once. `StyleRung` has a `Display` that spells the
rung in words (*"rung 2: the standard-14 sibling"*), which is what a future
session grepping a trace for an unhandled outcome needs to see.

### `fn reflow_refusal`

# The same shape as [`refusal_of`] above, and for the same rule

The engine's error is a *diagnosis*; the operator needs a *next step*, and
the two are not the same list. `ReflowApplyError` carries many more variants
than there are things an operator can do — save and reopen, remove the
protection, accept that this text cannot be addressed, or stop — so this
maps rather than transcribes. Wording each engine variant separately would
put operator-facing decisions inside an error type written for a library.

# No wildcard, and that is the whole safety of the function

`ReflowApplyError` is `#[non_exhaustive]`. A `match` on it ending in `_`
therefore gains new variants **silently**, and the rule is stated in
`text::textedit::reflow_refusal`'s header in exactly this shape:

> *"any `match` of yours ending in `_` just gained a variant it will not
> distinguish, and the one it will not distinguish is the one you care
> about."*

A wildcard here would route a newly carved-out refusal into
[`ReflowRefusal::Other`] — the generic sentence, no cause, no remedy —
while the engine's whole reason for carving it out was that it had one.

⇒ So everything except [`ReflowApplyError::Encrypted`] routes through
[`ReflowApplyError::decline`]. [`ReflowDecline`] is deliberately **not**
`#[non_exhaustive]` — the engine made the same written promise it made for
`RefusalKind` — so the `match` below is **compiler-proved complete**. A
future engine *refusal* joins an existing arm and keeps its correct
sentence; a future *decline* is a build failure here, which is the one place
it should be.

⚠ Never narrow an operator sentence onto a cause the engine can no longer
produce. `Unsupported(String)` carries many distinct refusals under one
discriminant, only one of which ever had *save and reopen* as its remedy, so
it earns [`ReflowRefusal::EngineDeclined`] — which names no cause and
promises no remedy — rather than a specific sentence that would be wrong for
every other member of the set.

`Encrypted` is matched by variant, above the discriminant and on purpose.
`ReflowDecline::StructureForbids` covers *"encryption, **or** a save that
the edit gates refused"*, and those need different sentences — one says
*remove the protection*, the other cannot say anything so specific. Name the
narrower cause wherever the engine gives a narrower variant, and nowhere
else.

| decline | shell refusal | why |
|---|---|---|
| [`ReflowDecline::RetryAfterSaveAndReopen`] | [`ReflowRefusal::PageAlreadyEdited`] | ⚠ **unreachable at engine `025d703d`** — its only source error has no producer in the engine. The arm is mandatory (this `match` is compiler-proved complete over an exhaustive enum) and the sentence is retained; `check-unreachable-refusals` is what notices if it comes back |
| [`ReflowDecline::NotFound`] | [`ReflowRefusal::CannotTrace`] | *"glyphs cannot be traced back to their show operators"* is that sentence, verbatim |
| [`ReflowDecline::NotReflowable`] | [`ReflowRefusal::EngineDeclined`] | permanent for this document; the operator did nothing wrong and can do nothing |
| [`ReflowDecline::StructureForbids`] | [`ReflowRefusal::Other`] | reachable here only as a gate refusal, which this shell cannot describe more precisely than *"pdfcer could not, and nothing was changed"* |

Note what this shell refuses to do throughout: read
[`std::fmt::Display`]. The sentences are unchanged and are still
implementer-voiced. The engine's own doc says *"Match this, never `Display`
output"*, and that was this shell's position before the type existed.

### `enum StyleChange`

One variant per control, because **one control press is one undo entry**. A
struct carrying five `Option`s would let the panel batch a size and a colour
into a single request — which the engine supports — and would make `Ctrl+Z`
after two separate presses take back a state the operator never saw. The
panel commits on `drag_stopped` / `lost_focus` for the same reason.

### `fn apply`

# The ordering is done here, not asked of the caller

`runs` arrives in whatever order the caller measured it; this sorts,
deduplicates and reverses. A caller that had to remember to pass them
backwards is a caller that will one day forget, and the failure would be
silent and rare — see the module header.

# What a refusal does

**Stops.** A restyle that half-applies and carries on is worse than one that
half-applies and says so: the operator sees some of their text change, has no
way to tell how much, and the undo stack holds an unknown number of entries.
