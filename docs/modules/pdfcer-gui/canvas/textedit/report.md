# `pdfcer-gui/canvas/textedit/report`

## Item notes

### `struct PageLevelForms`

# Why this type exists, and why the remedy is conditional

This module's header said, correctly and for eight days, that the operator's
half of the shared-content report *"is already handled and is deliberately
not re-worded"*: `pdfcer-core` writes the `"SHARED CONTENT: …"` sentence into
`EditReport::disclosures` and the `edit_text` arm carries it to the status
row verbatim.


# The nesting case, which is the whole reason this is a TYPE and not a
# boolean

The remedy is **not always available**, and offering it where it does not
work would be worse than staying silent — it would send an operator through
undo, a command, and a re-edit to arrive at the identical fan-out.

`format.unshare_form` always hands the engine the **outermost** enclosing
form, because that is the only operand `unshare_form` accepts (a nested one
is refused by name). Now consider a page that invokes form `A`, where `A`
invokes form `B`, and the edited text lives in `B`:

| | before | after unsharing `A` |
|---|---|---|
| this page | `A` → `B` | `A'` → **`B`** |
| the other 35 | `A` → `B` | `A` → **`B`** |

`A` is privatised and `B` — the stream holding the edited glyphs — is
**still shared by both copies**. The command succeeds, discloses that it
succeeded, and the operator's next edit fans out exactly as before. There is
no refusal to catch, because nothing was refused.

⇒ So the remedy is offered **only when the edit went into a form the page
invokes directly**, which is the case where unsharing that form is the
stream holding the text. That is what this set answers, and it is derived
from `FormLeaf::containment[0]` — *"the chain of enclosing form XObjects,
outermost first"* — which is the same element
`panels::objects::provider::ObjectModelProvider::containing_form_object`
hands the verb. One derivation: the sentence cannot promise a form the
command would not act on.

# Why it is gathered BEFORE the edit rather than after

`super::super::super::app::actions::apply::vector_edit` takes `&mut OpenDoc`
and the decomposition is reached through a `Ref` into a `RefCell` cache, so
the borrow must be released before the edit begins. Gathering a `BTreeSet`
of `u32` first costs one walk of a list the frame already built, and the
page's own `/XObject` names cannot change during the edit that is about to
happen — a text edit rewrites a content stream, not a resource dictionary.

### `fn of`

**Empty when the page has not decomposed**, and that is the safe
answer rather than a degraded one: an empty set offers no remedy, and
silence is the correct output when this shell cannot confirm that the
remedy would work. A fallback that guessed *"probably page-level"* would
be advice given without evidence, on the one surface where wrong advice
costs an operator two commands and a re-edit.

### `fn remedy_for`

Two conditions, both necessary:

1. **`form_invocations > 1`** — the engine's own `InvocationSet::is_shared`
   predicate, spelled here as the comparison it is (`sites.len() > 1`),
   so the remedy appears on exactly the edits the `SHARED CONTENT`
   sentence appears on and never beside a report that did not warn. A
   remedy for a problem the operator was not told they had is noise.
2. **the edited form is page-level** — see the type's docs for the
   nesting case this excludes and why excluding it is the point.

### `fn trace_target`

# Shared content, and why this is worth a named function

`Pass 119.0` made form-XObject text editable, and a form XObject may
legally be painted **from several pages and several times on one page** —
ISO 32000-1 §8.10.1 states that as the *purpose* of the feature, and names
a CAD system's standard component as the illustration, which is this
operator's title block exactly.

**No clause in either edition binds a form to a page.** That is a confirmed
permanent negative result in pdfcer's spec corpus (`FX-N1`), argued three
independent ways. So editing text inside a shared form changes **every place
it appears**, and there is nothing pdfcer can do about that: there is exactly
one stream holding those glyphs. The engine's words when it shipped this:

> *"A shell that ignores `form_invocations` is a shell that changes six
> drawing sheets while showing one."*

# The operator's half is already handled, and is deliberately not re-worded

`pdfcer-core` puts a `"SHARED CONTENT: …"` sentence into
`EditReport::disclosures`, worded for direct display, and the `edit_text`
apply arm has always carried that list to the status row. Re-wording it here
would be a second account of one fact, free to drift from the engine's.

It is **absent** on the ordinary single-paint case, by the engine's
design and this project's own rule: a warning that fires every time is one
nobody reads, and this one is meant to be startling. That is also why there
is no badge, tint or flag drawn into the page — R8b rule 4 as narrowed by
pdfcer's decision 059. The disclosure is off-canvas or it is nowhere.

# What this adds is the machine-readable half

A driven check cannot assert on prose, and these three numbers are exactly
what a wrong build gets wrong:

| field | what a wrong build reports |
|---|---|
| `form=` | `none` when the edit was meant for a form — the target collapsed to the page stream |
| `invocations=` | `1` for a shared form, i.e. the fan-out was not asked for |
| `pages=` | a count that disagrees with `invocations` on a form painted twice on one page |

The first is the regression that matters most on this operator's documents,
because `EditTarget::Auto` offers a pinned span to the page's own stream
first — and on the benchmark sheet that stream holds 3,007 single-character
show operators, so a stray match there is a dense field of near-misses
rather than a theoretical collision. See [`plan`], which names the target
from the same provenance record it takes the pin from.

# What is NOT built, said so it is a decision

There is no **pre-commit** warning: an operator whose caret lands in a
shared form is not told before they type. The engine publishes
`text_edit::forms::invocation_map`, which answers the fan-out for every form
in one document walk, so it is buildable — but one walk per click on text is
not affordable uncached, and a cache keyed on the document rather than the
page is a piece of work rather than a line. Recorded in
`OPERATOR_REQUESTS.md` rather than left implied.

### `struct LineReading`

The run count is not decoration. `left` is read by *index*, and an edit is
free to change how many runs a page decomposes into — a replacement that
empties a run, or joins two. When the count moves, `runs[n]` before and
`runs[n]` after are not necessarily the same line, and a displacement
computed across them is an attribution rather than a measurement. Carrying
the count is what lets a check see that and say so, instead of reporting a
confident number about the wrong line.

### `fn read_line`

`None` when tracing is off or the page's text is unavailable; a `Some` whose
`left` is `None` when the page read but that run has no bounding box.
[`trace_left_edge`] spells every one of those as *nothing was measured*
rather than substituting a zero, because a zero is a position and would read
as a line that had moved to the left margin.

# Why the provenance cache and not the cheaper one

`crate::app::cache`'s `page_text` is an order of magnitude cheaper and is
the wrong instrument here for two reasons. It answers for the *current*
page, so it would silently measure the wrong sheet the day a commit runs
against a page the operator is not looking at; and it returns a `Ref` into
the cache, which cannot be held while the funnel takes `&mut OpenDoc`.
`provenance_page_text` is page-indexed and hands back an `Rc`, so the
reading is complete and the borrow gone before the edit starts.

Both caches hold the *same* segmentation — `capture_provenance` populates a
field and changes no run boundaries — so `runs[run]` names the same run
either way, and `run` may keep coming straight from the caret.

# Cost

The reading before the edit is a cache hit: the caret already paid for this
page at this epoch when it placed itself. The reading after is a new epoch
and rebuilds — one extraction per commit, on a path that already saves and
re-rasterises. Bought only when someone is reading.
