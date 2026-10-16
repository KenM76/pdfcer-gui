# `ui-verify/checks/option_arrows`

`the_option_arrows_are_greyed_only_at_the_ends_of_the_list` — the Move-up
and Move-down buttons beside a drop-down's choices are dead at the ends of
the list and live everywhere else.

# The defect this is written against

An `/Opt` editor can draw all three rows' arrows live and then filter the
press — `if clicked() && n > 0`. The top row's Move-up button then looks
pressable, reports a click, and has its result discarded. The operator
presses it and the list does not move.

# Why a driven run, and why the trace had to gain a field first

**A correctly disabled control and a live one whose result is thrown away
are the same observation.** A harness that presses the button and then reads
the list sees *"nothing moved"* in both cases, so pressing it proves nothing
at all — which is why this check presses nothing.

What separates them is `egui::Response::enabled()`, whose value is decided
by whether the widget was allocated inside a disabled `Ui`. A widget merely
painted grey, or given a weaker `Sense`, reports itself **enabled**. So
`enabled=false` is evidence about the mechanism rather than about the
appearance, and this check exists because `pdfcer_gui::diag::ui_control` now
publishes it.

# What a blanket disable cannot produce

Six assertions across three rows, not two:

| row | up | down |
|---|---|---|
| 0 | **disabled** | enabled |
| 1 | enabled | enabled |
| 2 | enabled | **disabled** |

Asserting only the two disabled ones would pass on a build that disabled
every arrow — a shipped section where nothing reorders, which is worse than
the defect it replaced. The four `enabled` rows are the half that names what
the wrong mechanism cannot do.

Three rows is also the fewest that can distinguish *"the ends"* from *"the
first and the last are special-cased by index"*: with two rows every arrow
is at an end.

# Only the scripted wheel, so this runs beside somebody working

The document arrives on argv, Edit mode and the Properties panel arrive
through `PDFCER_DIAG_INVOKE`, and the field selection arrives through
`PDFCER_DIAG_SELECT_FIELD`. That last seam exists because
`doc.selected_field` — the Properties pane's only input — had no writer but
a click, which made every properties surface in the shell unreachable by R1
on any day the operator was at his machine. The one gesture the check makes
is a wheel through the scripted pointer, which takes neither the OS cursor
nor focus.

# Every row must come on screen under the wheel

The arrow STATES are read from the `ui-control` lines, which every row
publishes whether or not it is scrolled into view. Reachability is a separate
claim and is asserted separately: the panel is wheeled over the section until
each of the three rows has declared a visible rectangle (`diag::ui_rect_visible`
publishes only a region at least 60% shown).

A taller window cannot replace the wheel. `PDFCER_DIAG_VIEWPORT`'s height is
clamped to the monitor with no error, so the third row sits below the
panel's fold at every height this machine allows; only scrolling, which is
what the operator does, brings it up. A row that never comes on screen is the
"shipped unreachable with every gate green" class `D:/dev/rag/egui/` records.
