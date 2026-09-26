# `dialogs::embed` — the confirmation before font programs go into a
document



This header used to say the window has **no** settings, that *"there is no
option to set"*, and that a form here *"would imply choices that do not
exist"*. That was true when it was written and it is not true now, and the
sentence is corrected in place rather than left standing beside its
replacement.

There is exactly one control: **use pdfcer's own copy of a standard-14 face
where none of your fonts answers**. It is off when the window opens, it is
drawn only when it would change something, and the fonts it would stand in
for are named beside it before it is ticked. Everything else in the window
is still a report, and the shape below still governs.

### Why the "no settings by design" argument did not survive




## WHY THE SWITCH IS OFF BY DEFAULT, and it is not a matter of taste

Two reasons, and the second is the one that decided it.

**1. The letters change.** Embedding a stand-in changes what the document
looks like on the screen of whoever it is sent to. That is disclosed per
row either way — see [`crate::text::embed::embed_row`]'s `Bundled` arm — so
on its own it argues for loud disclosure rather than for a default.

**2. It is a LICENCE the operator takes on, not a look they accept.**
pdfcer's fourteen substitute faces are BSD-3-Clause (`THIRD_PARTY_LICENSES.md`,
*"Bundled Foxit substitute faces"*), and embedding one puts it **inside a
file the operator then distributes**, carrying that licence's attribution
condition with it. `pdfcer`'s own CLI says so in the argument's doc comment
and draws the only defensible conclusion: *"That is your decision to make,
so pdfcer does not make it for you."* — `pdfcer-cli`'s
`EmbedFont::use_bundled_fonts` (`--use-bundled-fonts`), off by default.


## What did NOT change

* The rung order. pdfcer's own faces are still consulted **last**, after an
  exact name match and after a standard-14 family equivalence, so a machine
  with fonts configured reaches a real face first whether the box is ticked
  or not.
* The disclosure. Every substituted row still says *"none of your fonts
  matched, so pdfcer used …. It is a stand-in, not the font the document
  asks for."*
* **Nothing is marked on the canvas.** No badge, no tint, no provisional
  styling on substituted text. Rule 4's surviving half is that an inference
  the operator cannot see owes them a report **off** the page — which is
  this window before the act and the disclosure row after it. Both;
  neither on the drawing.

## The preview is `embed_preview`, and it is the SAME computation the
commit runs

`EditSession::embed_preview(&request)` is `&self` and side-effect-free, and
`embed_fonts` calls it internally before mutating — the engine's own words
for the shape are *"the same value is returned by the preview query and by
the committing call, produced by the same function, so a front end cannot
show one thing and do another."*

That is the property `preview_font_resources` had to be fixed to have
twelve hours earlier, and the reason it mattered there applies here: a
preview and a commit that compute the same answer separately eventually
disagree, and the disagreement is silent.

## The plan is computed ONCE, when the window opens

Not per frame. Building it scans every configured font folder — reading and
parsing every font file in each — which measured **3,359 face names** on an
ordinary Windows font directory, and then reads each matched donor's bytes
into memory. A window that redid that sixty times a second would be
unusable, and nothing it depends on can change while it is open: the
document is not editable behind this window, and the folder list is not
either.

## Rule 4

Nothing here marks the canvas, and the disclosures this window shows are the
*pre*-commit half. What the embed actually did lands in the disclosure line
through `app::actions::fonts`.

## Item notes

### `const FOOTER_RESERVE_PTS`

The buttons, plus the checkbox and its two sentences when they are drawn.
A **constant** rather than a measurement, for the reason the print preview's
strip height is a constant and records in full: a scroll area sized from
what is laid out under it is a measurement feeding a size, the caption
re-wraps on a narrow window, and the operator watches the body settle over
several frames for no reason they can see. Reserving for the taller case
costs a little unused height on a window with no checkbox; measuring costs a
feedback loop.

### `struct Planned`

# Why there are two of these and not one recomputed on demand

Because building one costs a folder scan — every font file in every
configured folder, read and parsed, measured at **3,359 face names** on an
ordinary Windows font directory. A checkbox that re-scanned on each click
would put a visible pause on a toggle.

And it is not only speed. Two plans built from **one** scan cannot
disagree about what is on the disk. A plan rebuilt later could resolve a
different donor — a file dropped into a folder while the window was open —
so the operator would tick a box that said one thing and commit another.
The whole of `EmbedDialog`'s existing "the request is the operand, the plan
is its consequence, they travel together" argument applies twice over when
there are two of them.

### `fn active`

One accessor, so the list the operator reads and the request the commit
sends cannot be chosen by two different pieces of code — which is how
a window comes to show one thing and do another, and is the property
`embed_preview` was designed to give this dialog for free.

The `unwrap_or` is not defensive noise: [`Self::use_own_fonts`] can
only be `true` if the checkbox was drawn, and the checkbox is only drawn
when [`Self::with_own_fonts`] is `Some`. The fallback exists so that a
future caller that sets the flag some other way degrades to the **safe**
side — the operator's own fonts — rather than panicking or, worse,
silently committing something else.

### `fn rung`

The two enums exist because the crate boundary is load-bearing —
`pdfcer-core` must not depend on `pdfcer-render`, so neither can name the
other's type and *"a shell converts between them in one line."* This is the
return leg of that conversion, and it is exhaustive rather than
wildcard-defaulted: `FontMatch` is `#[non_exhaustive]`, and a fourth rung
arriving must fail to compile here rather than quietly render as the row for
the most reassuring of the three.

### `fn chosen`

# Why this is a free generic function and not three lines inside
# [`EmbedDialog::active`]

Because the decision it makes is the whole of O47 and there is no other way
to assert it. A `Planned` holds an `EmbedPlan`, which only `pdfcer-core` can
build and only from an open document — so a unit test of `active` would need
a `Session`, a PDF, and a font folder, which is a driven check wearing a
test's clothes. Lifted out and made generic, the *selection* is testable
over anything, and what is tested is the function the running program calls
rather than a paraphrase of it.

⇒ The shape this guards against is one this project keeps meeting:
**a test that the switch is off by default passes on a build that ignores
the switch entirely.** Both positions are asserted below, against the same
function the dialog uses.

`with_own` being `None` wins over `use_own` being `true`, deliberately and
in that order. The checkbox is only drawn when there is something to choose,
so that combination should be unreachable — and if a future caller makes it
reachable, the safe answer is the operator's own fonts. A `panic!` or an
`unwrap` here would turn a wiring mistake into a crash on the Embed window;
silently committing the substitutes would turn it into a licence the
operator never agreed to. Degrading to `own_only` is the only arm that is
wrong in a direction nobody has to live with.

### `fn the_switch_is_off_by_default_and_it_is_the_switch_that_decides`

The two assertions are worthless apart and are written as one test
so that they cannot be separated:


Strings stand in for the two plans. What is under test is the
**selection**, and a selection is the same function whatever it selects
between — see [`chosen`] for why the real type cannot be constructed in
a unit test at all.

### `fn with_nothing_to_offer_the_switch_falls_back_to_the_operators_own_fonts`

Asserted rather than left to the comment, because "unreachable by
construction" is a claim about a construction that somebody will change.
The safe direction is the operator's own fonts; see [`chosen`].

### `fn open`

`None` for a document with no missing fonts is deliberate: opening a
window to say *"there is nothing to do"* is a modal an operator has to
dismiss to learn they did not need it. The disclosure line says it
instead — see [`open_for`]'s caller.
