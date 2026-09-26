# `app::panels` — putting a panel on screen, and taking it off again

Two methods, and the distinction between them is the whole subject of this
file: **not every command that shows a panel is a toggle.**

| | [`PdfcerApp::toggle_panel`] | [`PdfcerApp::show_panel`] |
|---|---|---|
| the control asks | *"is this panel open?"* | *"tell me about this thing"* |
| pressing it when open | **closes** it | shows it again, idempotently |
| callers | `view.panel_*`, `file.fonts` | `file.properties`, `markup.comments` |

## Where this sits among its siblings

`mod.rs` composes a frame, `dispatch.rs` answers *what does this verb do*,
`conditions.rs` answers *what is true right now*, `gating.rs` answers *what
is this mode allowed to do*, and this file answers *where does a panel go,
and what does pressing its control mean*.

## The distinction is load-bearing

Making the toggle a property of `show_panel` would turn **every** panel
command into a toggle, including `file.properties`. That one is offered by
the **Objects row context menu** to describe the row just clicked, so
right-clicking a second row and choosing Properties would *close* the
description instead of re-pointing it at the new row;
`app::tests::the_properties_command_puts_the_panel_on_screen_in_every_mode`
holds that property.

The trap in the phrasing *"make panel commands toggles"* is that it assumes
every command reaching a panel is a panel **control**, and two of them are
not: a command whose question is *"is this panel open?"* toggles, and a
command whose question is *"tell me about this thing"* shows.
