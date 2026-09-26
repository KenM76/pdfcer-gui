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

### `fn capabilities`

One expression, in one place, so that every consumer — the canvas
gestures, the Delete key, the context menus, the tool arming — reads
the same answer rather than re-deriving it. Cheap enough to call per
frame per consumer (a linear scan of at most a handful of modes over a
handful of tab ids) and `Copy`, so nothing has to cache it and no
cache can go stale.

The ribbon rather than `self.modes` is asked for the active mode,
deliberately: the ribbon is where the operator's click lands, and
`self.modes` catches up with it later in the same frame (see the
mode-change arm in [`Self::dock_area`]). Reading the laggard would put
the canvas one frame behind the selector on the frame the mode
changes — which is precisely the frame a stray click is most likely,
because the pointer is already down over the chrome.

See `app::modes::capability` for the derivation and for why an
unrecognised mode gets everything.

### `fn on_mode_capabilities_changed`

Called once, from the mode-change arm in [`Self::dock_area`], *after*
`Modes::on_mode_changed` has rearranged the panels — so the two halves
of "what this mode is" land in one frame and in a fixed order.

Three things, and each is a thing that would otherwise **survive** into
a mode that forbids it. That is the shape of the whole problem: the
gesture gate in `canvas::gesture::press_kind` stops anything *new* from
starting, and cannot by itself retire what was already there.

1. **An armed tool** — see [`crate::canvas::tool::retire_forbidden`],
   which carries the argument. Its visible symptom is the cursor.
2. **The selection.** A selection made in Edit outlives the switch,
   because it lives on the document and `MODES_AND_PANELS.md` rule 1
   forbids a mode change from destroying work. A selection is **not
   work** — nothing about the document changes, the undo stack is
   untouched, and it is re-made with one click on returning. Leaving it
   would put eight resize handles and an outline on a page in Read:
   controls the operator can see, aim at, and drag with no effect, which
   is precisely the *"visible control, silently inert"* failure the mode
   system exists to avoid. Clearing is what lets the gesture gate refuse
   a grip **that is not there** rather than one the operator is looking
   at.
3. **A gesture in flight.** Rule 1 again, and this time it is the rule's
   own wording: *"If a mode change would hide a pending, uncommitted
   gesture … that gesture is committed or cancelled first."* Cancelled,
   not committed — the operator asked for a mode, not for the half-drawn
   rectangle under their pointer, and `GestureOutcome::Cancelled`'s
   contract is that nothing is written.

Nothing here fires when capability did not change: `retire_forbidden`
reports `false` for a permitted tool, a `clear` on an empty selection is
a no-op, and Read → Review leaves an armed Rectangle armed.
