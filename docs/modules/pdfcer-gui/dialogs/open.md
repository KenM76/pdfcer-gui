# `pdfcer-gui/dialogs/open`

**How each dialog is BUILT** — every `DialogsState::open_*` constructor, and
the two guards each of them applies before a window can exist.

# 1. Why this file exists, and what the seam actually is


| family | job | where it lives now |
|---|---|---|
| `open_*` / `deliver_*` | **build** a dialog from defaults, a picked path, or a document survey, and decide whether it may exist at all -- and hand a canvas gesture's answer back into one that is already open | **this file** |
| `ask_*` / `take_*_answer` / `show` | carry a **question** to the operator and its **answer** back to `PdfcerApp`, and drive the per-frame draw-and-drain loop | `dialogs/mod.rs` |

That is not a mechanical halving. The two families have different callers
(dispatch for the first, the frame loop and the save funnel for the second),
different failure modes, and — most usefully — **different invariants**,
which are stated once here rather than repeated at twenty-one sites.

# 2. The two guards every opener applies, and why they live HERE

The shell's dispatch pattern is *"push the chord blind, gate the effect in
dispatch"*. A ribbon control registered `enabled_when("doc.open")` cannot be
pressed without a document and cannot be pressed twice in one frame; **a
keyboard chord bound to the same command id has neither property.** So both
conditions are enforced at the one place a dialog is ever constructed:

- **No document, no dialog.** Otherwise the chord on an empty canvas builds
  a window that [`super::DialogsState::show`] closes again on its very next
  frame — and some of them do real work on the way, such as enumerating the
  spooler over the network for Print.
- **Already open means leave it alone.** These functions build from
  *defaults*. A second press part-way through configuring a job would
  silently discard the range, the scale, the copy count, the annotation
  scope — the operator's own settings, thrown away by the very shortcut
  pressed to look at them.

Enforcing both here fixes the button and the chord **by construction**,
rather than by a condition duplicated at the keymap that can drift from the
one in dispatch. Each function's own doc comment says which of the two it
applies and, where it applies only one, why the other does not arise.

# 3. What this file must NOT grow into

An opener decides **whether** a dialog may exist and gathers **what it needs
to exist**. It does not decide what the dialog does, does not write to the
document, and does not answer its own question. When an opener starts to
need more than a survey of `Status`, that is the signal that the reasoning
belongs in the dialog module itself — the same rule `app/actions/OVERVIEW.md`
sets for action arms, and for the same reason: a constructor that knows the
semantics of the thing it constructs is a second place for those semantics
to live, and two places drift.
