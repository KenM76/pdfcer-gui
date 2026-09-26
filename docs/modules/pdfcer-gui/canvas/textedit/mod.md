# `canvas::textedit` — **editing the page's own words**, and placing new ones

`DEFECTS.md` **D4** is the defect that began this project, reported as:

> *"text editing is weird and doesn't just edit the existing box and move the
> text correctly as you type plus flow to the next line doesn't work."*

This module is the shell half of the answer. It arms a tool, puts a caret in
a run, collects keystrokes into a draft, and commits the draft as **one**
`EditSession` command. What it fixes that the old shell did not is in
[`disposition`] — the two cases D4b names as *wrong on commit* — and what it
deliberately does not fix is listed under **What is out of scope** below,
in words, because a silently-disabled typing loop is how the old shell told
an operator it could not do something.

---

## §1. The admission argument `canvas::tool`'s header asked for

That header names one exclusion and one only:

> **Text *editing*** — Phase 5, the defect that began this project — remains
> outside, and for exactly the original reason: it is a caret in a
> re-laid-out box, it would drag a whole subsystem's state through this type
> […] **Whoever brings the second should have to make this argument again,
> in this file.**

It is made there, at [`CanvasTool::TextEdit`](crate::canvas::tool::CanvasTool::TextEdit),
and the short form is this: the objection was *"it would drag a whole
subsystem's state through this type"*, and the state does not go through the
type. What crosses the boundary is one [`TextEditKind`] — the same single
value `Markup` and `Measure` each carry — and the draft, the caret and the
anchor live in `egui::Memory` exactly where a half-finished measure pick
lives, for the reason `canvas::measure`'s header gives: *"a half-finished
pick is not part of the document and a document saved mid-gesture must not
carry one."* A half-typed word is the same category.

## §2. Two kinds, one tool — and why it is not two tools

`edit.text` edits a run that is already on the page; `edit.add_text` places
new page content. They are the same *gesture* — click somewhere, type, press
Enter — differing only in what the click resolves to and which engine verb
the commit calls. So they are one [`CanvasTool`] variant carrying a
[`TextEditKind`], which is `MarkupKind`'s argument restated: the operator is
doing exactly one of the two, and a type that could say both would need
discipline to keep honest.

## §3. It clicks AND it drags

> *"I should be able to make it multi line."*

**A PDF has no paragraph.** Each visual line is its own show operator at its
own absolute position, so something has to decide where the second line
starts: a width to wrap against and a leading to step by. That is
`AddTextRequest::with_box`, and it needs a rectangle — so the gesture is a
drag, and the `DragKind` is not a placeholder but the whole feature.

| gesture | anchor | what commits |
|---|---|---|
| click on existing text | [`Anchor::Run`] | `edit_text` |
| click on bare page | [`Anchor::Origin`] | `add_text`, one line at a point |
| **drag a rectangle** | [`Anchor::Box`] | `add_text` boxed, a wrapped paragraph |

★★ The drag belongs to **this** tool and not to `CanvasTool::Text`. On the
sweep tool's rung the box would take the text sweep away in Edit, which
`text_tool_selects_and_marks_in_edit` depends on to make a selection the
markup verbs can act on. **Two features claiming one drag is a choice
somebody has to make, and taking a shipped gesture away to make room is the
wrong way to make it.** Add-text drags; the text tool goes on sweeping.

## §4. It does not disturb the text-selection gate

`canvas::textsel::gate`'s §3 warns that exclusivity between the text sweep
and the content marquee is now *by precedence*, not by construction. This
variant does not touch that: `takes_the_press` asks `tool.is_text()`, which
is `matches!(tool, CanvasTool::Text)` and is therefore **false** for
`TextEdit(_)` by construction rather than by an added condition. A press with
this tool armed is claimed by this module's own rung in `press_kind`, which
sits *above* the text-selection question for the same reason the measure rung
does — an armed tool takes the press — so the two can never both claim one
press. `gate.rs`'s tests carry a case asserting exactly that.

## §5. Capability: `edit_content`, and nothing wider

Unlike text *selection*, this authors. Every entry point is gated on
`Capabilities::edit_content`, which is Edit alone in the shipped manifest —
the dispatch arms decline by name and trace, and
[`crate::canvas::tool::retire_forbidden`] disarms the tool on the way into a
mode that cannot author, so a draft cannot survive into Read.

---

## §6. What is out of scope, said in words rather than by a dead key

`DEFECTS.md` D4a's **cross-run editing** is not built: it needs a multi-run
edit request in `pdfcer-core` that does not exist — `EditRequest` pins to one
show operator, and *"a `TJ` array is one operator"*.

★ What a caret landing where two runs meet does **not** do is go quiet. It
opens on the piece that was clicked and **discloses** the consequence on the
status bar — `crate::text::textedit::shares_the_line_note` — because a
control that takes keystrokes it will not honour is this module's defining
defect class, and so is a typing loop that is silently disabled.

D4c's **reflow gates** are out of scope and untouched.

---

## §7. The two costs an operator pays, and where they are disclosed

Rule 4 — *disclosure lives off-canvas* — so neither of these is drawn on the
page. Both reach the status bar through the disclosure list `vector_edit`
records, and one of them is a disclosure this shell adds because the engine
does not:

* **`Reflow`** — the engine already discloses that the line may now overrun
  its margin.
* **`Pin`** — the engine discloses nothing, because from its side pinning is
  what was asked for. But a pinned tail does not make room, so a longer
  replacement grows *into* it. [`plan`] adds
  `crate::text::textedit::pinned_tail_disclosure` for exactly this, which is
  why the pin is never silent.
