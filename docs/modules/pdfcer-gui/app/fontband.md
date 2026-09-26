# `app::fontband` — the three Format ▸ Font controls the ribbon cannot draw
itself

`RIBBON_IA.md` §5.8's *Text run* row, the half of it that is not a button.

## What this module is

The Font group has five controls. Two of them — Bold and Italic — are
ordinary `Item::Command`s and the ribbon draws them, greys them and shows
their tooltips with no help from this crate. The other three are
`Item::Custom`s, because a face chooser has to ask *which* of the page's
fonts, a size field has to accept a number, and a colour needs a swatch
that shows the current one. `egui_shell::manifest::Item::Custom` is the
extension point for exactly that, and it hands the application a `Ui` and
gets out of the way.

This module is what goes in that `Ui`.

## The shell reserves the slot; everything else is ours, including the
greying

This is the sentence to read before changing anything here, because it is
the difference between this file and the ribbon's own control renderer.

`egui_shell::ribbon::control::render_command` does four things for a
command item: it evaluates `enable`, it draws the control greyed when the
predicate is false, it shows the tooltip through `on_hover_text` **or**
`on_disabled_hover_text`, and it publishes the control's rect under
`ribbon.item.<id>`. For a custom item it does **none** of them — it cannot,
because it does not know what is being drawn.

So all four are done here, deliberately in the same shapes and under the
same names, and the reason they are not *approximately* the same is R9: a
greyed control is only legitimate when it explains itself on hover, and
these three are greyed for most of their life. An operator meets them
after clicking a piece of text with the Select tool — the Format tab
appears, the Font group is there, and nothing is swept — and the tooltip is
the one surface in the whole application that can say, at that moment, that
sweeping with the Text tool is what gives them something to act on. That is
O37's *"nothing on screen tells you to press T"*, answered where the
question is asked.

The rect is published under `ribbon.item.<command id>` — the same name
`egui_shell::ribbon::report::band_item` builds for a command control — so a
driven check finds a face chooser the same way it finds a Delete button. A
second naming scheme for "the same kind of thing, drawn by the other half
of the program" is how a harness comes to have two lookup paths, which is
the defect `driving::declared_or_in_overflow` was written to end.

## It reports; it does not dispatch

Every control here parks a [`StyleChange`] and returns the command's
`HandlerToken`. It raises no `Action` and touches no document.

That is `egui-shell`'s contract — *"the shell reports, the application
dispatches"* — and it is also what keeps the five Font commands honest as
one family: `format.bold` and `format.italic` reach
`app::dispatch::format` through a ribbon click, and so do these three, so
the operand derivation (*which page, which runs*) is written **once**, in
the dispatch arm, rather than once there and once here. A chord that never
touched the ribbon then gets the same answer as a click.

`app::recent::menu` is the precedent and the shape is identical: the
picker asks, the command acts.

## Why these read the registry rather than `crate::text::commands`

The label and tooltip could be fetched straight from
`crate::text::commands::format_font()`, which is where the registry got
them, and it would be one source. The registry is asked instead because of
**R8**: a command a build does not register does not exist, and a custom
item that drew a control for one would be the one place in this shell where
a compiled-out capability still had a surface. `registry.get(id)` answering
`None` draws nothing, exactly as `MenuHost::label` answering `None` removes
a row from the Tool panel.

## Rule 4

Nothing here marks the canvas, and nothing here can. Every disclosure a
press causes — a synthetic weight, a real face substituted, a colour space
narrowed — is raised by `app::actions::textstyle` into the status bar. The
restyled text renders exactly as the saved file will render it.
