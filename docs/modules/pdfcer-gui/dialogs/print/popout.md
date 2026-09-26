# `pdfcer-gui/dialogs/print/popout`

`dialogs::print::popout` — the print preview in a window of its own.

# What this is, in the operator's words

> *"also the preview should be adjustable size, and even better if it has
> the option to pop out into its own resizeable window - closing the window
> pops it back into place on the print window."* — 2026-09-03,
> `OPERATOR_REQUESTS.md` **O112**.

Ask 1 — the draggable splitter — shipped the same day. This file is ask 2.

# Why it is thirty lines of code and four hundred of reasoning

Because almost all of it already existed and the value of this file is
knowing that. Every dialog in this shell is a real OS window through
[`crate::dialogs::host::Host`], which is keyed on a name string, remembers
where the operator dragged it, and reports its own close as
[`Frame::closed`](crate::dialogs::host::Frame::closed). A popped-out preview
is therefore:

| the ask | what answers it | written here? |
|---|---|---|
| *"its own window"* | a second `Host`, keyed `print-preview` | one call |
| *"resizeable"* | `Host` windows are resizable, with a floor | nothing |
| *"closing the window pops it back"* | `Frame::closed` → `preview_popped = false` | one line |
| it survives being dragged to a second monitor | `Host` remembers position | nothing |
| it is findable when it falls behind | `with_taskbar(true)` in `Host` | nothing |

**The close is already the return path.** That is the sentence worth
carrying out of here: the request reads like a feature with two halves —
pop out, and put back — and the second half is the default behaviour of a
window. Building a "put it back" control would have been building a second,
worse route to something the title bar already does.

# What the print dialog draws while the preview is out: NOTHING

Not a greyed rectangle. Not a *"the preview is in another window"* card. Not
an outline where it used to be. The column collapses and the options take
the room — see `layout::Columns::split`, whose popped arm returns a preview
width of exactly zero and hands the whole content width to the options, and
`layout::tests::popping_the_preview_out_collapses_its_column_and_gives_the_
room_away`, which asserts all three parts of that.

R9 is what forbids the stub, and the reason is not tidiness: a placeholder
occupying the space teaches the operator that this dialog has a dead region
in it, and the next time something *is* broken there they will not report
it.

# The geometry trap this file had to be written around

`Host::fit` grows a dialog whose content is bigger than its window — a
measurement fed into a size, which is R128's shape and has bitten this
project three times, once for this very dialog's footer. The preview's own
`fit` (the sheet-into-canvas ratio) is recomputed from the current rect
**every frame**, on purpose, so that a taller window shows a bigger sheet.
Putting that loop inside a window whose size is itself derived from its
content would close the circle.

It does not close, and the reason is a direction bound rather than a guard:
[`preview::column`] allocates its canvas at *exactly* the width and height
it is handed, and it is handed `ui.available_*` minus a constant. The
content is therefore never larger than the window, `fit_target` only ever
grows, and a function that only grows cannot be driven by content that is
by construction no bigger than what it is measured against. The one case
that can grow the window once is a canvas clamped up to
`CANVAS_MIN_HEIGHT_PTS` on a window shorter than that — bounded by the
clamp, not by a budget, and it settles in one round trip.

`Host`'s three-step growth budget is still behind it, and the trace line
`dialog-fit-runaway title="Print preview"` is what a future mistake here
would announce itself as. Read that line before reading this file.

# What is NOT claimed

**No window was rendered while this was written.** The operator was at the
machine, so `ui-verify` could not be run and no screenshot exists. The
arithmetic of the collapse is unit-tested and falsified; that a popped-out
preview *looks* right, that the window opens at a sensible size on his
monitors, and that the column visibly disappears are asserted by
`ui-verify`'s `the_print_preview_pops_into_its_own_window` — **which has
been written and registered and has not been run.**

## Item notes

### `const DEFAULT_SIZE_PTS`

Taller than it is wide, because a print preview is a sheet and every sheet
an operator of this program prints is either portrait or, rotated, still
better served by height than by width — the fit takes the smaller of the two
ratios either way. Larger than the 340 pt column it came out of, because
making the preview bigger is the entire reason to pop it out; opening at the
same size as the column would answer the request with a window that changed
nothing.

### `const MIN_SIZE_PTS`

A floor and not a preference, for `Host`'s stated reason: a resizable window
with no minimum can be dragged down to a title bar, which is a state with no
way back except closing it. Here that is milder than for a form — closing is
the intended exit — but a preview squeezed under
`CANVAS_MIN_HEIGHT_PTS + STRIP_HEIGHT_PTS` would show a smudge and a
scrollbar, which is not a preview.
