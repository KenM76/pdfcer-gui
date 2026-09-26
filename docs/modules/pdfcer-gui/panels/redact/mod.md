# `panels::redact` — marking content for removal, and reviewing what is
marked

The body of `edit.redact`, and the **reversible** half of the redaction
feature. Its irreversible twin is [`crate::dialogs::redact`], and the split
between the two surfaces is the operator-facing distinction
`crate::text::commands::edit_redact` already ships in four words:

> **Marking is reversible; applying is not.**

Everything in this panel can be taken back — by the Remove button on a row,
by `Ctrl+Z`, or by simply not applying. Nothing here removes a byte from
anything.

## Why a panel and not a dialog, when apply is a dialog

[`crate::dialogs`]' header draws the line: *"a dialog is a single
transaction with a start and an end… a panel is somewhere an operator dips
in and out of while working."*

Marking is unmistakably the second. An operator searches for a name, pages
through the document checking what the search caught, marks a title block by
hand, takes one mark off again, and only then applies. That is a working
surface, not a transaction — and the two commands split along exactly that
seam, which is why `edit.redact` opens a panel and `edit.redact_apply` opens
a dialog.

It holds no document state of its own: the marks are `/Redact` annotations
**in the document**, so the list below is rebuilt from
`pdfcer_core::redact::redaction_marks` every frame and there is nothing to
keep in step. What [`RedactUi`] holds is the operator's half-typed search —
their state, not the document's.

## The census is read from the SESSION GRAPH, never the base document

`redaction_marks(&doc.session.graph())`, and the distinction is not
academic: it is the shape of a real defect in the shell this one replaces,
where a status-bar census read `session.document()` and therefore reported
**zero marks** for every mark the operator had just made and not yet saved.
A redaction census that silently omits this session's marks is the worst
possible reading of the worst possible counter.

`crate::redact::prepare_redaction_apply` reads the same walk for the same
reason, so the number this panel shows and the number the apply acts on
cannot disagree.

## Layout: state, then action, then detail

Deliberately **not** the usual detail-then-action order. The count and the
*Review & apply* control come **first**, above the marking controls and
above the mark list.

The order is measured, not assumed. Put the apply control anywhere below
the marking controls — a heading, a button, a field, a second button, a
two-position switch and a four-line hint — and on a 1100×800 window with
one mark made, `tools/ui-verify`'s redaction check finds it declared at
`y = 801.7` inside a panel whose body ends at `y = 770.0`: **off the bottom
of its own pane**. Unit tests stay green throughout, because a unit test
cannot see where a control landed.

So the rule is stronger than *"above the list"*: the census and the apply
control are **the first things in the panel**, and everything that can grow
— the hint text, the mark rows — is below them. A surface whose primary verb
can be pushed out of view by its own explanatory copy is one an operator
will conclude is broken.

## What this panel does NOT have, and why each is deliberate

| absent | why |
|---|---|
| **Canvas drag-to-mark** | Marking is panel-only. See this module's *"Marking by drag"* section below — it is a canvas-tool build rather than a panel addition. |
| **A confirmation on Remove** | Removing a mark is reversible twice over (the mark can be re-made, and `Ctrl+Z` restores it) and nothing has been removed from the document. A confirmation on a reversible action is how operators learn to dismiss confirmations — including the one that matters, three controls away. |
| **A confirmation on Mark whole page** | Same argument, and its tooltip says so in words. |
| **An Apply button that applies** | The control opens a **report**. The click that opens it must not feel like the click that commits, which is why its label ends in an ellipsis and the commit lives behind two checkboxes in another surface. |

## Marking by drag: panel-only, and the reasoning

Marks are made from this panel. There is no canvas drag-to-mark gesture,
and that is a decision rather than an omission: a canvas gesture here is
more than a modest addition, for three reasons.

1. **The shipped tooltip does not promise it.**
   `crate::text::commands::edit_redact` enumerates what marking offers — *"a
   whole page, every occurrence of some text, or everything matching a
   pattern"* — and none of the three is a drag. All three are built.
2. **It is a canvas-tool build, not a panel one.** It would need a
   `CanvasTool` variant, an `app::modes::capability` entry so Read cannot
   reach it, a rung on `canvas::keys`' Escape ladder, an overlay preview in
   `canvas::overlay`, and an `Action` carrying page-space quads. A tool
   substrate is always bigger than the one-line feature it serves, and this
   one would be arming the one irreversible verb in the program.
3. **The proof is what matters, and the proof is the panel.** A correct,
   verified, panel-driven redaction is what `FEATURES.md`'s row depends on.
   A half-built canvas gesture on top of it adds a way to make marks, not a
   way to trust them.

What it would take, so the next hand does not re-derive it: the whole-page
marking path below already builds a `RedactSpec` from a `Rect` and pushes it
through the one action arm. A canvas gesture is that same arm with a
different rectangle — `canvas::markup::band` is the drag machine and
`canvas::mapping` is the screen-to-page conversion — plus the five pieces of
tool substrate in item 2.

## Item notes

### `const REGION_PANEL`

Matched **literally** by `tools/ui-verify/src/checks/redaction.rs`, so
renaming one of these silently un-aims the check that drives it. The same
contract `crate::dialogs::ocr`'s region names carry.

### `const REGION_APPLY`

Declared **only while it is enabled**, which is itself an assertion a
harness wants: its absence from the trace is evidence that nothing is
marked, rather than that a click missed.

### `const APPLY_COMMAND`

Raised as [`Action::Command`] rather than opening the dialog here, on
`crate::app::mod`'s stated rule for the Find bar's OCR offer: a surface
outside the ribbon that means an existing *command* routes through the one
dispatch choke point, so the command's guards live in one place. A panel
that called `DialogsState::open_redact` itself would be a second
implementation of `edit.redact_apply`, and the two would drift the first
time the command grew a precondition.

### `fn marking_controls`

Split out because [`body`] would otherwise be one long function whose two
halves — *make marks* and *review marks* — change for entirely different
reasons, which is the same seam `app/actions.rs` is split along.

### `fn mark_rows`

Two controls and no third. The row itself navigates — a plain button rather
than a `selectable_label`, because a row is a **navigation command** and not
a selection, and a highlighted row would imply a selected-mark concept this
panel deliberately does not have.

### `fn a_whole_page_mark_covers_the_displayed_page`

The crop box rather than the media box, argued at
[`whole_page_spec`]. The failure this catches is silent in both
directions: a media-box mark covers content the operator was never
shown, and a hand-shrunk rectangle would leave a margin of live text
under a mark labelled "whole page".

### `fn a_mark_authored_into_the_session_is_visible_to_the_census`

The end-to-end shape in the smallest form a headless test can hold, and
the reason it is worth having: the census reads
`session.graph()` while the mark is authored into the session overlay,
and a build that read `session.document()` anywhere in that chain would
report zero for every mark the operator had just made.

### `fn the_search_query_does_not_survive_a_new_document`

A search term left over from a previous file is one an operator could
run against a document it was never meant for — and this feature answers
a search by authoring marks over whatever it hits.

### `struct RedactUi`

Not document state and not derived from anything — a half-typed search
query and which way the mode switch is set — but it has to outlive a frame.
It lives on [`PanelsState`] for the reason that struct's header gives about
the Pages panel's: `&mut PanelsState` is already threaded to every body, so
a panel's own state needs no interior mutability and the forgetting is free.

**The query is deliberately cleared with the document** (through
`PanelsState::forget_document`, which resets this struct whole). A search
term left over from a previous file is one an operator could run against a
document it was never meant for, and on this feature that authors marks over
whatever it happens to hit.

### `fn body`

The standard body signature: `&OpenDoc` is **shared**, so the
actions-not-mutations invariant is a compile-time fact here rather than a
convention, and every verb below is an [`Action`] pushed for the apply phase.

### `fn whole_page_spec`

Pure, and separate from the action arm for the reason every geometry rule in
this crate is: it is the part that could be wrong in a way an operator would
notice, and a `&mut EditSession` is not available to a test that only wants
to ask what rectangle was chosen.

# The crop box, not the media box

`Page::crop_box` is what a reader **displays** (ISO 32000-1 Table 30:
content is clipped to it at display time), and it defaults to the media box
when the document does not state one — so this is the larger of the two
answers in every case where they differ *for what the operator can see*, and
identical otherwise.

Marking the media box instead would cover area the operator has never been
shown, which sounds harmless and is not: the whole-page control's tooltip
promises to mark *"this entire page"*, and a mark extending past what the
page displays is a claim about content nobody reviewed. Where the two
genuinely differ — a trimmed drawing sheet, an imposed signature — content
outside the crop box is content the operator did not know was there, and
telling them it is covered by a mark they made deliberately would be the
same false-confidence failure the whole feature exists to prevent. That is a
**sanitise** verb, and `crate::shell::manifest`'s own note keeps the two
apart: *"strip metadata, scripts and hidden content. Distinct from
redaction."*

# Why an engine field is not evidence of a consumer

`fill`, `overlay_text` and `quadding` are the operator's choice, made in
[`appearance`] and carried to every marking route by the engine's `_styled`
verbs (`EditSession::mark_redactions_by_search_styled` and its pattern and
page-level siblings). Routing all three routes through one appearance is
the invariant; the rule behind it is the part worth keeping:

**An engine field that exists, is documented, and is written into the PDF
is not evidence that anything reads it.** A field can reach the file with
nothing rendering it, and a field honoured on the whole-page path can be
hard-coded `None` on the search path — which makes a swatch work on
whole-page marks and vanish silently on searched ones. The only check that
separates *supported* from *accepted and discarded* is following the value
to its consumer.

The same rule one layer up, about disclosure: rustdoc is not a disclosure
surface. A deferral described in a doc comment is a claim about a backlog,
never evidence that the operator will be told.

# `fill: None` means TRANSPARENT, which is the dangerous half

`RedactAppearance::fill`'s `None` is transparent, per Table 192 — not a
black box. So a shell that passes `None` removes the content and draws
**nothing over it**: not a security failure, but an operator seeing no
evidence that anything happened, on the operation they cannot undo.

[`appearance::Appearance::default`] therefore passes an **explicit**
`Color::Gray(0.0)`, and its own test asserts that against the engine's type
rather than against the shell's enum.

Inventing a default overlay caption would still put words on the operator's
page that they did not write, which was the original reason for
`overlay_text: None` and stands unchanged — the field is empty until they
type in it.

### `fn mark_ids`

One walk, exposed so `crate::app::actions` can report *how many* marks a
search created without the panel and the action arm deriving the census two
different ways.
