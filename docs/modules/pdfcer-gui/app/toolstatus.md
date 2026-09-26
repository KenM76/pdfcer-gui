# `app::toolstatus` — the one line that replaced the Tool panel

`OPERATOR_REQUESTS.md` **O123**, verbatim:

> *"I never understood why there is a tool dock when everything can be in
> object and properties. … The Tool panel becomes a one-line tool status
> (name, one sentence, 'Put this tool down'); its buttons duplicate the
> ribbon and go."*

## ★★★ What moved where, and why nothing was deleted

The standing objection to collapsing the Tool panel is that it deletes the
armed block's live controls and orphans a disclosure slot. That objection
is right about the cost and wrong about the remedy, and the operator's
first sentence is why: the controls were never the tool panel's to hold.
They are properties of what is selected or about to be drawn.

| what the Tool panel held | where it is now |
|---|---|
| the armed tool's **name** and **stage** | here, on one line |
| **Put this tool down** | here, at the end of that line |
| the pointer sentence (Block A) | here, as the sentence for the resting tool |
| every stage's **second** sentence | here, in the strip's hover — see [`sentence`] |
| the **tool list** (Block B) | **gone**, on the operator's instruction — every row was a route to a ribbon command |
| the text pen's **font, size and colour** | `crate::panels::properties::tool` |
| the circular measure's **pick list** | `crate::panels::properties::tool` |
| the three **scale switches** | `crate::panels::properties::tool` |
| the **disclosure block** (Block C) | `crate::panels::properties::disclose` |

★ The tool list is the only genuine subtraction, and it is the one he asked
for by name. It answered a **discoverability** defect — `panels::tool`
exists because *"The feature works. He could not find it."* — and removing
it is his call to make and not this module's. What survives of that argument is the sentence on
this strip: it is permanent chrome, it names what is armed at frame one with
no clicks, and it cannot be closed, which is more than the panel could say.

## ★★ Why a dock banner and not a status-bar item

The strip has to be **beside the document**, permanently, and it has to have
somewhere to put a button. The status bar is under R128 — its row must not
grow — and it already carries an *elided* copy of the same disclosure slot
this change re-homes ([`crate::app::status::disclosure`]). A second, wider
claimant on that row is the exact feedback loop R128 exists to forbid.
[`egui_shell::dock::banner`] gives the right dock a reserved strip whose
height is a constant the dock takes off the top before it resolves the
columns, so nothing here can drive a width or a height.

## The resting state draws no button, and that is R9 rather than an omission

`CanvasTool::Select` **is** the resting state; putting a tool down returns
to it. A *Put this tool down* button beside `Select` would be a control
whose press changes nothing, and R9 forbids a dead control. So the button
appears when something is armed and is absent otherwise.

## ★★★ And the strip draws its sentence even when it cannot name the tool

`OPERATOR_REQUESTS.md` **O66**. A `CanvasTool::Place` is armed from inside a
dialog that then hides itself, so there is no ribbon control to name and
[`command_for`] answers `None`, so there is no identity row to draw in
exactly that case. The *stage* line still draws, and its own comment says
why:
*"This one is the ONLY place the gesture and the way out are stated …
Deleting this line would strand an operator who has forgotten what they
armed."*

⇒ So a missing name suppresses the **name**, never the sentence. Written as
its own early-return rather than folded into the format string, because the
tempting shape — one `format!` with an empty name — silently ships a line
beginning with an em dash.
