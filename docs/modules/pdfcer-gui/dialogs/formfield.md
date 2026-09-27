# `dialogs::formfield` — the details a placed form control needs

**Operator request, 2026-08-26:** *"when I click one I should be able to
click on the canvas to place the position or drag a box for size then a pop
up lets me set the details for the feature."* This is the pop-up.

It opens on `Action::BeginFormField`, which the canvas raises on the click or
release that finishes placing, and it **authors nothing until Accept**. That
is the whole reason a dialog is in this path rather than a properties pane
after the fact: a form field is invisible on a printed page and swallows
every keystroke aimed near it, so a mis-drag that left one behind would be
both hard to notice and annoying to find.

## The tooltip field is not a nicety — it is the feature's blocker

Every one of `pdfcer-core`'s five authoring verbs refuses a spec whose
tooltip is `TooltipChoice::Undecided`, because an interactive control owes a
screen reader a name and the engine will not invent one silently. That
refusal was recorded in this project's backlog as *"core's STRUCTURAL
certification gate"* and parked form authoring for nine days.

There is no gate. The blocker is **this text box**. An empty one becomes
`Declined` — the operator saying *"this control needs no name"*, which is a
decision the engine accepts and is sometimes right — and a filled one becomes
`Text`. What the engine will not accept is nobody having been asked, and now
somebody has.

## Why one dialog for five kinds, and how it stays legible

`pdfcer_gui_base::formdraft::Draft`'s header argues the model side:
the five engine specs share nine fields and differ in one to five, so five
GUI structs would mean writing the shared half five times. The same argument
holds for the surface, with one addition — **the shared half is the half an
operator adjusts.** Name, tooltip, required, read-only and border are asked
identically for all five, and only the kind-specific rows change.

So the layout is: the common rows, a separator, then [`Self::specific`],
which is the only `match` on kind in the file. A reader looking for "what is
different about a check box" has exactly one place to look.

## What is remembered, and where

Nothing, here. The dialog opens with a draft that
`Remembered::next` already prepared, and `Action::CommitFormField` is what
records the accepted one — **at the point it was accepted**, so a draft the
operator cancelled is not remembered. See `app::actions::apply`'s arm.

## Item notes

### `const REGION_BODY`

These names are a **cross-repo stability contract**: `tools/ui-verify`
asserts on them by string, so renaming one silently turns a check into a
skip rather than a failure. Treat them as published API.

### `const TOOLTIP_MAX`

A `/TU` is what a screen reader reads aloud, in one utterance. Past roughly
this length it stops being a label and becomes a paragraph nobody waits
through.

### `const FOCUS_ATTEMPT_FRAMES`

Long enough to outlast the pointer release that opened the window being
resolved; far short of a human reaching for the mouse, so a request cannot
fight the operator's own click on Cancel.

### `const CONTROL_PTS`

This shell's theme presets draw controls at 28 pt (see `FEATURES.md` on the
status bar's height constant, which was written for 24 and was wrong for
exactly this reason), and the inventory prices the row rather than the
control, so the item spacing is in the number.

### `const NOTE_PTS`

Priced at **three** lines rather than the two most of them take at
[`WINDOW_PTS`]'s width. Deliberately generous: the wrapped height of a
sentence is a function of the font, the preset and the width, none of which
this arithmetic may read without becoming a measurement — so the honest
thing is to over-price it and be a few points tall rather than to price it
exactly at one preset and be a line short in another.

### `const INTRO_PTS`

Priced at two lines because `text::formfield::intro`'s longest — the radio
button's *"One of a set of alternatives — picking one clears the others."* —
wraps to two at this width.

### `const FOOTER_PTS`

# It is one constant used twice, and it was a literal `40.0` in one of the
# two places

The body reserves this out of the scroll area's `max_height` because the
buttons are drawn *after* it, and the window's opening height adds it back
because the buttons need somewhere to be. Those two uses must agree — a
reservation smaller than the row pushes Accept off the bottom of a dialog
whose entire purpose is Accept — so they read one number.

46 pt, matching `print::layout::FOOTER_HEIGHT_PTS`, which reserves the same
thing for the same reason.

### `const WINDOW_PTS`

The width is the whole of the horizontal story: the body is a stack of
full-width controls, so nothing here competes for width and the number only
has to be wide enough that the explanatory sentences wrap to two lines
rather than four. 480 rather than the previous 440 buys about ten characters
a line, which is one line off each of the four notes.

### `fn content_height`

# What is counted, and the one thing that is not

Everything inside the scroll area, for the dialog **as it opens**: the two
common rows at the top, the kind's own rows, and the two common flags plus
the border row at the bottom.

A push button's *action* rows are deliberately **excluded**, and that is
the one judgement in this function. `dialogs::buttonaction::rows` draws a
different set of controls for each of seven choices — from nothing at all
for *Do nothing* to a radio pair, a label, a three-row box and a note for
*Show or hide fields* — and the choice is made by the operator **after** the
window has been created. Sizing the opening window for the tallest of them
would open every push-button dialog 160 pt taller than the rows it is
showing, to pre-pay for a branch most operators never take. Those rows are
what the scroll area is for, and they are why the bar is now visible.

### `fn window_size`

The height is the inventory plus the chrome outside the scroll area, capped
so the dialog cannot open taller than the screen it has to appear on and
floored at [`MIN_WINDOW_PTS`]. When the cap bites, the body scrolls — with a
visible bar — which is the right answer and the reason the scroll area was
never the defect.

`screen` is the *application* window's content rectangle, used as the
stand-in for the monitor's usable height. It is not the same quantity —
this dialog is an OS window and may legally be taller than its parent — but
it is the only one `eframe 0.35` offers without a work-area query, it is
never larger than the monitor, and erring small here costs a scrollbar
rather than a window with its buttons below the taskbar. The Settings window
makes the same substitution and says so.

### `fn resolve`

A draft's `None` and a saved widget's absent key are the same state said
twice: nothing has been chosen, so the box will be placed the way its kind
is normally placed. That is why *remove* collapses to `None` here rather
than needing a third variant on the draft.

### `fn radio_rows`

The wording differs from the check box's even though the fields are
the same two, and deliberately: for a radio the **name is the group**,
so what tells two members apart is the export value. An operator who
reads "export value" as a technical detail here will place three radios
that are all the same answer.

### `fn common_flags`

# The colours, and why they are here rather than only in the properties

`OPERATOR_REQUESTS.md` **O202**: *"the forms objects have no way to
edit their colour before or after placement."* Placing a row of
identically coloured check boxes and then recolouring each one
afterwards is the workflow that ask is about, and `Remembered` carries
both colours across placements, so the question is asked once.

The reading of the two keys — which states a swatch can draw, what each
undrawable one shows instead — belongs to
[`crate::panels::properties::mkcolour`] and is not restated here. Only
the destination differs: this writes a [`Draft`] field, the properties
pane writes a `WidgetEdit` and an undo entry.

# Rule 4

There is no content to mark. The box does not exist until Accept, and
when it does it is drawn in the colours chosen here with nothing added.

### `fn accept_requested_by_harness`

Read every frame rather than latched, unlike `scripted_invoke`'s counter,
and the difference is real: that one turns an env var into an **event**, so
it must fire once. This is a **standing instruction** — *"in this run, accept
every form-field dialog"* — and a check that places three fields wants all
three accepted. It is idempotent by construction, because accepting closes
the dialog.

Gated on `crate::diag::enabled()` like every other seam, so a stray
environment variable cannot change what the shipped program does for an
operator who is not running a harness.

### `fn every_kind_opens_tall_enough_for_its_own_rows`

The defect in one assertion. The window was a flat `440 x 420` for all
five kinds and every one of them needs more than 420, so the dialog
always opened with content below the fold — behind a two-point floating
scrollbar nobody could see, which is why it was reported as clipping
rather than as scrolling.

Both sides of this comparison come from this file's own constants, so
it does not prove the constants are *right* — no unit test can, because
the true row heights exist only in a laid-out frame under a theme. What
it proves is that the window and the inventory cannot drift apart, which
is the failure that shipped: a size chosen once, by hand, against
content that then grew. The same shape as
`print::layout`'s `the_content_floor_is_the_sum_of_the_column_floors`.

### `fn the_inventory_prices_every_kind`

The failure this guards is the one that shipped: a single hand-chosen
size standing in for five different bodies. A drop-down has a four-row
options box and three flags; a check box has one tick and one value.
If those two ever price the same, the inventory has stopped being an
inventory and is a constant wearing a `match`.

### `fn a_short_screen_caps_the_window_and_the_body_scrolls`

The cap is what keeps the inventory from becoming a licence to open a
window taller than the desktop. When it bites the body scrolls, which is
what the scroll area is for and why the bar was made visible in the same
change.

### `fn the_opening_size_is_never_under_the_floor_it_declares`

A default below the floor is silently clamped by the window manager, so
the dialog would open at a size no constant in this file names — and the
operator could never get back to the one that was intended.

### `fn accepting_raises_one_commit_with_the_draft`

The guard against the shape this dialog exists to avoid: authoring on
placement. Nothing reaches the document until this action does, so a
cancelled dialog must produce none — asserted in the test below.

### `fn comb_is_cleared_when_its_precondition_goes_away`

Drives the real body so the clearing is asserted where it happens, not
restated. Without it, a text field could be authored `comb` with no
`/MaxLen`, which draws a box divided into no cells.
