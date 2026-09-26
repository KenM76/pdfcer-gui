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
