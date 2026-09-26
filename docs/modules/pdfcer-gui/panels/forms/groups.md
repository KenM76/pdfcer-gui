# `panels::forms::groups` — the Field-groups section, and the shell's only
route to deleting one

A third list in the Forms panel, beside the fill list and the Tab-order
section, answering a question neither of them can: **what names does this
form file its fields under, and what would happen if one of them went?**

## What a field group is, and why the panel had no way to reach one

`AcroForm::groups` is the field-name tree's *interior* — `Personal` and
`Personal.Address` in `Personal.Address.Zip`. §12.7.3 gives such a node
existence only as a link in a `/Parent` chain: **no type, no value, no
widget, no rectangle.** It is drawn nowhere, in any viewer, ever.

Which is exactly why the fill list cannot list it (it has nothing to fill),
the Tab-order section cannot list it (it has no `/Annots` entry), the canvas
cannot select it (it has no box to click), and the Properties pane cannot
describe it (`doc.selected_field` is populated by a click on a widget, and
there is no widget). Until this section, `EditSession::delete_field_group`
was an engine capability with **no surface in the shell at all** — the
finding that produced this work.

## Everything this section does is disclosure, because everything its
verb does is invisible

Rule 4 says *"disclosure lives off-canvas: a status line, a results panel, a
report after the command, a properties field"*, and a panel is the right
home. On this verb it is more than the right home — it is the **only**
evidence. Deleting `Personal` produces:

- the same page, pixel for pixel, at every zoom;
- the same raster, the same print, the same export;
- a fill list some rows shorter, if the operator happens to be looking at
  it and happens to remember how long it was.

⇒ So this section says what would go **before** the press, in numbers and
names, and the funnel says what did go **after** it, from the engine's own
report. Neither is optional and neither substitutes for the other: the first
is a decision, the second is a receipt.

## The two-press protocol, from this side of it

| press | raises | changes | draws |
|---|---|---|---|
| **Delete group…** | `FieldAction::ArmGroupDeletion(Some(name))` | nothing | the disclosure block, next frame |
| **Delete N fields** | `FieldAction::DeleteGroup { group }` | the document, one undo entry | the status bar's receipt |
| **Cancel** | `FieldAction::ArmGroupDeletion(None)` | nothing | the row, plain again |

The preview runs in the funnel and not here, and that is a compile-time fact
rather than a convention: `field_group_deletion_preview` takes `&mut self`,
this body is handed `&OpenDoc`, and the session lives behind an `Arc`. See
[`crate::app::actions::forms::groups`] for the whole argument, including why
this is not a modal dialog.

## R83 — the refusal is asked BEFORE any control is drawn

`EditSession::deletion_refusal` is a pure query and this section asks it
once, at the top, exactly as [`super::body`] asks `fill_refusal` and
`flatten_refusal` for their own controls. On a certified or encrypted
document it renders **the sentence and no controls at all**.

That is R9 rather than greying, and the distinction is the one R9 draws:
greying is for a capability that is *temporarily* unavailable and is always
explained on hover. A certification signature is not temporary and cannot be
argued out of — so a greyed Delete-group button would imply a state the
operator could reach, and would hide the explanation behind a hover they
have no reason to make.

**And it is a sentence, not a silence.** Rendering nothing at all here would
be indistinguishable from a feature nobody built, on a panel that lists the
groups either way.

It asks `deletion_refusal`, not `flatten_refusal` and not `fill_refusal`.
The three are different questions with different answers on documents that
are not exotic — `super`'s body carries the measured account of that — and
core's own doc comment names the hazard precisely: *"a call site that asks
the wrong question is correct only until the answers diverge, at which point
it is wrong silently, in a control that stays enabled while its verb
refuses."*

## Where the section sits, and why

Immediately after the Tab-order section and **above the fill list**. Two
constraints decide it and neither is taste:

1. **Anything below the fill list is unreachable.** That list's own
   `ScrollArea` takes the rest of the pane, and the panel's top level does
   not scroll, so content after it is laid out past the bottom of a
   container with no way to get there. This panel has already shipped that
   defect once, measured in a driven run at y=773 in a body ending at y=770.
2. **It belongs beside the other structural surface.** Tab order lists
   controls the form does not claim and offers to register them; this lists
   names the form files fields under and offers to remove them. Both are
   about the form's *shape*; the fill list is about its *contents*.

## Rule 4: this section draws nothing on the page

Not one pixel. No highlight over the widgets of a group under the pointer,
no badge, no outline. The one-line test — *would a screenshot of the editing
canvas differ from a screenshot of the same document saved and reopened?* —
answers no, and must keep answering no.

Worth naming what rule 4 would *permit*, so nobody reads the absence as a
prohibition: highlighting the widgets beneath the group under the pointer is
the fourth clause's *"a hover highlight … these are the cursor"*, and it
would be a genuinely good affordance for a verb this invisible. It is not
built for the reason [`super::tab_order`] gives for the same wish: the
panel→canvas channel for *which row is hovered* does not exist in this
build, and `crate::canvas` is not this module's to extend.

## `PDFCER_DIAG` proves what this computed

One `form-groups` census line per frame the section is reached — carrying
the node count and whether the document refused — and one `form-group-row`
line per row, capped. Written whether or not the collapsing header is open,
so the listing is provable from a trace without anyone having to click.

That matters more here than on a visual surface: a screenshot of this
section cannot tell you that a node the file carries is missing from the
list, or that the refusal query was never asked. Both are in the trace.

## Item notes

### `const REGION_HEADER`

A published region name is a cross-repo stability contract: the harness
asserts on it by string, so renaming one turns a check into a skip rather
than a failure.

### `const REGION_ARM`

Suffixed with the grouping node's **object number**, not its index in
`AcroForm::groups` and not its name.

- Not the index: deleting one node renumbers every node after it, so a check
  that pressed "row 1" twice would press two different groups. This is the
  same argument `tab_order::register` makes for keying on tab position
  rather than list position.
- Not the name: a fully-qualified field name is the operator's own words and
  may contain spaces, `=` and anything else `/T` permits (Table 220 makes it
  a text string). Region names are parsed out of a `key=value` trace line,
  so a name would break the parse on exactly the documents whose fields are
  worth naming.

An object number is stable across the session, unique, and safe in a trace.

### `const MAX_TRACED_ROWS`

`pdfcer_core::forms::MAX_FORM_FIELDS` is 500,000 and a pathological form
could carry grouping nodes in proportion, so an uncapped per-row census
would bury every other line in a capture. The summary line is never capped,
so the *count* stays provable even when the enumeration stops — the same
rule [`super::tab_order`] caps its own row census under.

### `fn trace_rows`

This module's header promises the row lines are written *"whether or not the
collapsing header is open, so the listing is provable from a trace without
anyone having to click."* It was not true: the loop that wrote them sat
inside `CollapsingHeader::show`'s body, and egui does not run that closure
while the header is closed — and this section ships **closed**.

So a trace from a run that never opened the header carried the summary and
no rows, and a check reading it would conclude the form has no groups. The
promise was in prose, in a doc comment, checked by nobody.

⇒ Lifted here, above the header, where the claim is true by construction.
The separation is also the more honest one and matches
`crate::panels::comments`: **a trace describes what the panel computed**,
not what it happened to paint. A surface that traced only what it drew would
go quiet exactly when a reader most wants to know what it decided — behind a
closed header, off the bottom of a scroll, inside a collapsed tree.

Capped at [`MAX_TRACED_ROWS`], and the summary line above carries the real
total, so a truncated listing can never be mistaken for a short one.

### `fn rows`

# At most one press per frame, and it is not an accident

The loop stops raising after the first press. Two presses in one frame would
queue two actions against a form parsed **before** either ran, and the
second would be acting on a set the first has already changed. The names
here are stable where indices are not, so the second action would in fact
still name the right node — the discipline is kept anyway, because *"queue
only what was computed against the state you have"* is worth holding
mechanically rather than re-deriving each time a queued verb is added. It
costs the operator nothing: physically, one press per frame is all there is.

# Order is core's, deepest-first, and is deliberately not re-sorted

`AcroForm::groups` is post-order — a child appears before its parent — and
core states it because *"it is the opposite of what DFS order suggests and a
consumer that assumed parents-first would render a breadcrumb backwards."*
It is also the useful order here: the deepest node is the smallest,
least-destructive removal, so the list reads from the safest press to the
most sweeping one.

### `fn disclosure`

# This block is the whole point of the preview existing

The engine's own words for why: *"an operator looking at a collapsed tree
row cannot see how many that is or what they are called. This answers that
question against the live session, before anything changes."* And the reason
it is safe to draw from: the preview runs the **same gates** as the
deletion, because both go through one `group_deletion_preflight` — *"a
preview that succeeds where the act fails is worse than no preview: it
invites the operator to confirm something that cannot happen."*

# The order of what it says

Numbers, then names, then the two controls. The numbers decide *whether*,
the names decide *which*, and a control above either would be a button
offered before its own justification.

# Cancel is a real control and not an implicit click-away

A destructive confirmation the operator can only escape by pressing
something else in the panel is one they can dismiss by accident and cannot
dismiss on purpose. It raises `ArmGroupDeletion(None)`, which changes
nothing and clears the block.

### `fn section`

Called from [`super::body`] with the `/AcroForm` it has already parsed —
not re-derived here, because two parses of one form per frame is a cost with
no benefit and because a second parse could in principle disagree with the
one the rows above came from.

# It renders NOTHING on a flat form, and that is R124 rather than an
oversight

`AcroForm::groups` is empty for a flat form, *"which is every file in the
Pass 7.0 census"* — so on the overwhelming majority of real documents this
section is not drawn, not collapsed-and-empty, not a heading over nothing.
Core's own doc comment on that field states the obligation: *"a consumer
that renders these must therefore render nothing when the list is empty
rather than an empty section."*

`actions` is pushed at most once per frame — see [`rows`] for why one press
per frame is enforced rather than assumed.
