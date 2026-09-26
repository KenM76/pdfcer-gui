# `app::gating` — the mode gate, on the application side

Two methods on [`PdfcerApp`], and they are the seam between *what a mode
is* and *what the canvas is allowed to do*.

[`crate::app::modes::capability`] answers the question in the abstract —
given a manifest and a mode id, which capabilities does that mode carry —
and is pure. This file is where the running application asks it, and where
the answer is **acted on**: once per frame to hand the canvas its
[`Capabilities`], and once per *mode change* to retire the things that
would otherwise survive into a mode that forbids them.

## Where this sits among its siblings

`mod.rs` composes a frame, `dispatch.rs` answers *what does this verb do*,
`conditions.rs` answers *what is true right now*, and this file answers
*what is this mode allowed to do, and what has to be put down on the way
in*.

## The two halves, and why neither is sufficient alone

The gate is genuinely two mechanisms, and the reason is that a mode change
is a moment while a capability is a state:

| | mechanism | covers |
|---|---|---|
| **refuse** | [`crate::canvas::gesture::press_kind`] returns `None` | anything the operator tries to *start* while in the mode |
| **retire** | [`PdfcerApp::on_mode_capabilities_changed`] | anything that was *already there* when the mode was entered |

Refusal alone leaves an armed pen drawing a crosshair over a page it
cannot draw on, and eight resize handles on a selection nothing will move.
Retirement alone leaves every gesture available for as long as the
operator stays put. Both, and the canvas tells the truth in both tenses.

## Item notes

### `fn entering_read_clears_a_selection_made_in_edit`

The defect this closes is not "Delete works in Read" — it is the
*outline and eight resize handles* left on the page, which are visible
controls the operator can aim at and which would do nothing. See
[`PdfcerApp::on_mode_capabilities_changed`] for why a selection is not
"work" under `MODES_AND_PANELS.md` rule 1.

### `fn a_panel_control_closes_a_panel_that_is_on_screen`

The control is a toggle rather than a show. A show-only control for a
panel already on screen renders *pressed* and does nothing when pressed
— a visible control that is silently inert.

### `fn a_panel_behind_a_sibling_tab_is_raised_rather_than_closed`

The middle state, and the one that would be easy to get wrong. Getting
it wrong is worse than not building the toggle at all: the operator
pressing the control for a panel they cannot see means *"show me
that"*, and closing it would unmount the very thing they asked for.
