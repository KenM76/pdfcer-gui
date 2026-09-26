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

## ★ Why a panel and not a dialog, when apply is a dialog

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

## ★ The census is read from the SESSION GRAPH, never the base document

`redaction_marks(&doc.session.graph())`, and the distinction is not
academic: it is the shape of a real defect in the shell this one replaces,
where a status-bar census read `session.document()` and therefore reported
**zero marks** for every mark the operator had just made and not yet saved.
A redaction census that silently omits this session's marks is the worst
possible reading of the worst possible counter.

`crate::redact::prepare_redaction_apply` reads the same walk for the same
reason, so the number this panel shows and the number the apply acts on
cannot disagree.

## ★ Layout: state, then action, then detail

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

## ★ Marking by drag: panel-only, and the reasoning

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
