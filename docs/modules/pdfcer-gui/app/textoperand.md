# `app::textoperand` — **which runs a Format command acts on**, from either
gesture


> *"Also get the font selector and editing tools like old [bold] and italic
> working. That entire area is always greyed out in the menu, and the
> properties area is uneditable too. This is true even when I add a new line
> of text."*

## ★★★ The deadlock this module exists to break

Every one of the five Format ▸ Font controls — the face chooser, the size
field, the colour swatch, Bold and Italic — was gated on **one** field:
[`OpenDoc::text_selection`], a range swept with the Text tool. The
Properties panel's font editor was gated on the same field. And that field
can only be written by a press the text-selection gate accepts:

```text
canvas::textsel::gate::takes_the_press(tool, caps)
    = tool.is_text() || (tool == Select && !caps.edit_content)
```

In **Edit** mode `caps.edit_content` is true, so the Select tool's press
yields an *object* selection and never a text range. In **Read** and
**Review** the Select tool does produce a range — but the Font group is only
shown at all when `mode.edit_content` holds, which is Edit.

⇒ **The only mode that shows the Font group is the only mode whose default
gesture cannot produce its operand.** Two correct halves; no way through.
That is the whole of *"that entire area is always greyed out"*, and no
amount of pressing the button would ever have helped.

## What this module does about it

It gives the question *"which runs would a restyle act on?"* **one** answer
with **two** sources, in a fixed order:

| rung | source | cost |
|---|---|---|
| 1 | a **live swept range** — [`OpenDoc::text_selection`] | free |
| 2 | a **single selected text object**, via [`crate::canvas::textedit::pin::object_text`] | one extraction, 392 ms on the benchmark sheet |

Rung 1 first, always, because a sweep is the narrower and more deliberate
statement: an operator who swept three words and then happened to have an
object selected meant the three words.

## ★★ No new selection unit was invented, and that is the safety argument

Rung 2's operand is `first_run..=last_run` derived from the object's
`BT`…`ET` **byte span**, joined to each glyph's provenance operator span in
the same buffer. That is byte-range containment — exact, total, and the same
fact the pinned edit path already stakes every restyle on. It is **not** a
bounding-box overlap, which is the geometric inference
[`crate::panels::properties::text`]'s header refuses by name because it
restyles text the operator did not select, silently, in a file they then
send to somebody.

[`crate::panels::properties::textobject`] has shipped exactly this join
since O89, for the colour swatch alone. This module is that join lifted out
of one panel so the ribbon, the dispatch arm and the panel all read it from
one place — which is the rule [`crate::app::dispatch::format`]'s own note
states: *"which runs does a restyle act on?"* is a **rule**, and a rule
stated twice diverges.

## ★★★ Why the cheap half is a separate function

[`selected_text_object`] answers *"is there an object-shaped operand?"*
without reading a single glyph, and it exists because **a condition is
evaluated every frame and an operand is resolved on a press**. Publishing
`selection.text_runs` from [`resolve`] would put a 392 ms extraction in the
per-frame path on the drawings this program is for, which is
`OPERATOR_REQUESTS.md` O74 at its most expensive point.

★ It reads the decomposition through `OpenDoc::page_objects`, which
**builds on first use** — 469 ms on the operator's benchmark sheet. That is
not a new cost here and cannot be: the function returns early unless exactly
one content object is selected on the current page, and an object selection
can only have been made by a hit test, which had to build the model to
answer. Every reachable call is therefore a cache hit.

## Rule 4

Nothing here marks the canvas and nothing here can — it returns run
ordinals. Every disclosure a restyle causes is raised off-canvas by
[`crate::app::actions::textstyle`], and restyled text renders exactly as the
saved file will render it.
