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

# No pointer, so this runs beside somebody working

The document arrives on argv, Edit mode and the Properties panel arrive
through `PDFCER_DIAG_INVOKE`, and the field selection arrives through
`PDFCER_DIAG_SELECT_FIELD`. That last seam exists because
`doc.selected_field` — the Properties pane's only input — had no writer but
a click, which made every properties surface in the shell unreachable by R1
on any day the operator was at his machine.

# Why row 2's own rectangle is not asserted

Because it cannot be brought into view by asking for a taller window.
`PDFCER_DIAG_VIEWPORT`'s height is clamped to the monitor with no error and
no report, so 2000, 2200 and 2600 all produced the same window and the same
clip rect on this machine — measured, not assumed. The third row sits about
fifteen logical points below the panel's scroll viewport at every one of
them, and a check asserting its rect would fail identically on a correct
build and on a genuinely misplaced control.

What is asserted instead is that the **section** is declared and that at
least one row's arrows are, which separates *"the rows draw and the operator
scrolls to the third"* from *"this section shipped unreachable"* — the
defect class `D:/dev/rag/egui/` records the shell shipping before, with
every gate green.
