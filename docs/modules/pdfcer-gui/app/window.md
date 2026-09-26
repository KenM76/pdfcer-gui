# `app::window` — the two verbs in View ▸ Window that change the *shape of
the application* rather than anything about the document

`view.read_mode` (`Ctrl+H`) and `view.fullscreen` (`F11`). Five surfaces
register, draw, group and bind them, and without this module every one of
those surfaces promises a behaviour that does not exist. `RIBBON_IA.md` §3
names the state that produces:

> Read mode and full screen have **no ribbon control at all** — they are
> keyboard-only (Ctrl+H, F11) on a tab literally named View. This is the
> single most confusing thing in the current ribbon.

They have controls; this file is the behaviour behind them. It exists as a
module rather than as two `match` limbs because
`app::dispatch`'s header states the rule the limbs have to keep — *"the arms
route; they do not compute"* — and every decision below (what read mode
hides, what it deliberately keeps, how the two states are stored, which one
is allowed a shadow copy and which is not) is a rule rather than a route.

# 1. `view.read_mode` is NOT `mode.read`, and the difference decides
whether this file should exist at all

The question to settle before any of the rest, because the honest answer to
it could have been *delete the command*. This shell has a **Read mode in
the mode selector**
(`mode.read`, `Ctrl+1`), and a command called `view.read_mode` sitting one
tab over is exactly the shape of a duplicate that should be removed rather
than wired twice.

It is not a duplicate. The two answer different questions, and they compose:

| | `mode.read` | `view.read_mode` |
|---|---|---|
| What it is | a **named workspace plus a capability set** — `app::modes` | a **view stance**: chrome on or off |
| What it changes | which ribbon tabs exist, which panels are mounted, whether the canvas will author anything (`app::modes::capability`) | whether the ribbon and the docks are *drawn at all* |
| What it does not change | it still draws a ribbon (Read is shown `file` and `view`), still mounts panels, still has a status bar | nothing about capability: an operator in Edit ▸ read-mode can still author with a chord, and the canvas gate is untouched |
| Persistence | the mode and its arrangement are remembered per mode, on disk | per session, and per *window* rather than per document |
| Where it lives | the mode selector | View ▸ Window, beside Full screen |

The reference applications agree, which is standing instruction 4's test.
**Acrobat** has both: a *Read Mode* (`Ctrl+H`) that hides the toolbars and
panes and gives the window to the page, and a separate notion of what the
reader is allowed to do to the file. **Inkscape** has the identical pair —
`F11` full screen and a "wide screen"/focus toggle that hides dialogs —
neither of which is a permission. **SolidWorks**' full screen (`F11`) hides
the CommandManager and the FeatureManager and changes nothing about whether
the model can be edited. Three of three separate *chrome* from *capability*,
and the chord this command carries — `Ctrl+H` — is Acrobat's chord for
precisely the chrome half.

So the two are kept, they are orthogonal, and pressing one does not touch
the other: an operator can be in Review with the chrome hidden, and that is
a meaningful state rather than a contradiction.

# 2. What read mode hides, and the one thing it deliberately does not

[`crate::text::commands::view_read_mode`] is the promise the operator is
shown, and it is the specification this keeps:

> *Hide the ribbon and the panels and give the whole window to the page
> (Ctrl+H).*

Two surfaces, named. So [`draws_chrome`] governs exactly two composition
steps in `PdfcerApp::ui` — the ribbon band and the docks — and **the status
bar stays**. That is a decision rather than an omission, and it has two
independent reasons:

1. **The words say ribbon and panels.** A status bar that vanished would be
   a third thing hidden by a control that named two, which is the sort of
   quiet over-delivery that makes an operator distrust the next label.
2. **It is the only remaining route to page navigation and zoom.**
   `RIBBON_IA.md` §6 puts `page ◀ n/N ▶`, the editable page box and
   `zoom −/%/+` on that bar *because* they are the controls a reader touches
   constantly. Hiding them would leave a reading mode in which the reader
   cannot turn the page except by keyboard — the opposite of what the mode
   is for. Acrobat's own Read Mode keeps exactly this cluster, as a floating
   strip; pdfcer already has it as a fixed bar, so nothing has to float and
   `view.app_initiative`'s default of **Never** is not strained.

## Getting back out

**The chord, said out loud on two surfaces for as long as the mode is on.**
[`publish_exit_chord`] resolves `view.read_mode` against the live keymap
once a frame; `text::doctabs::window_title` puts it at the FRONT of the
window title, and [`crate::app::status::readmode`] puts it on the status bar
— the one piece of chrome §2 above deliberately keeps.

### Why the chord is stated permanently, and not left to a tooltip

The operator:

> *"I didn't see a way to get back out of read mode. if there is a shortcut
> for this it should have a note what the key combo is in the top bar that
> holds the window controls."*

The tempting argument is that Acrobat puts the chord on the control's
tooltip and the tooltip is shown before the press. **Both halves are true
and the conclusion does not follow.** It assumes the operator **arrived
here by pressing that control** and therefore hovered it. `Ctrl+H` is a
*bound chord*: it can be pressed from memory, or hit by accident reaching
for `Ctrl+G`, having never pointed at anything. And even the operator who
does use the control has not necessarily rested the pointer on it long
enough for a tooltip to appear — a click is not a hover.

⇒ **A tooltip is not a disclosure. It is a disclosure available to
somebody who already knows where to point.** The whole of what this mode
does is remove the thing they would point at, so the one surface the old
argument relied on is the surface the mode deletes. That is not a
discoverability nicety; it is a room whose only door is hidden by the act of
entering it.

The general rule, worth carrying past this file: **a hint that lives on a
control cannot explain how to undo a state that hides that control.** The
statement has to live somewhere the state cannot reach. Two such places
survive read mode, and each covers the other's blind spot:

| surface | survives | blind when |
|---|---|---|
| the **window title** | read mode — the strip is the operating system's and this mode cannot touch it | full screen, where there is no title bar at all; and a maximised window whose title nobody looks at |
| the **status bar** | read mode *and* full screen — §2 keeps it deliberately | never, for these two states |

Normally two surfaces for one fact is a smell. It is not one here, and the
reason is in the table: **read mode composes with full screen**, and in the
combined state the title bar is not drawn, so a title-only hint would be
absent in exactly the state with the least chrome left. Conversely the title
is legible from the taskbar and from Alt-Tab, which the status bar is not.
Neither alone covers the state space. Both derive from one published value
([`exit_chord`]), so they cannot disagree with each other, and that value is
read from the keymap that dispatches, so neither can disagree with the key.

### Full screen is NOT the same trap

`view.fullscreen` hides no chrome of ours: the ribbon stays drawn, its
control stays on View ▸ Window, and `app::conditions` renders it *pressed*.
A second click is right there. So no permanent `F11` hint is added — it
would be the noise this file's own §2 argues against, and it would be wrong
the moment the mode is off.

**The one state where it IS a trap is the combination**, and it is handled:
with read mode on as well, the ribbon is gone, so `view.fullscreen`'s
control is gone with it and `F11` has become undiscoverable too. The status
line names *both* chords in that state and only in that state. It cannot be
the title's job, because full screen is precisely when the title is not
drawn — which is the same argument for the status bar, arriving from the
other end.

**Escape was considered and declined.** `canvas::keys` already ranks four
claimants for Escape (a focused form field, the selection ladder's rungs, an
in-flight gesture, the page box's draft), and `SelectionLevel::ascend`
returning `EscapeOutcome::Nothing` is a documented fall-through seam that
this could have used. It is declined because the fall-through is *quiet*: an
operator who presses Escape to abandon a half-made marquee and instead finds
the whole application's chrome coming back has been surprised by a key that
means "no" everywhere else in this shell. If that changes it should change
as a ruling about Escape's ladder, in `canvas::keys`, and not as a fifth
claimant added here.

# 3. Where each state lives, and why they are stored differently

| | stored in | read by |
|---|---|---|
| read mode | [`egui::Memory`], under [`READ_MODE_ID`] | [`read_mode`] — the frame composition and the `selected:` condition |
| full screen | the **viewport itself** (`ViewportInfo::fullscreen`) | [`fullscreen`] — the `selected:` condition |

The asymmetry is deliberate and is the same argument `app::conditions`'
header makes about the armed canvas tool:

> A shadow copy … would put the truth about which tool is armed in two
> places, and the failure mode is a ribbon that says Hand while the canvas
> selects — a disagreement no test would catch, because each half would be
> self-consistent.

**Full screen has an owner outside this program.** The window manager can
put the window in and out of full screen without pdfcer being asked — a
double-click on the title bar, a window-manager chord, a display change. A
`bool` on `PdfcerApp` would then say one thing while the window said another,
and the ribbon control would render pressed over a windowed application.
`ViewportInfo::fullscreen` is the backend's own report, so there is nothing
to drift.

**Read mode has no owner outside this program**, because nothing but this
command creates it. It needs a home the condition set can reach, and
`app::conditions` takes an `&egui::Context` and no `&mut self`, so
`egui::Memory` is the same route the armed tool and the armed region zoom
already take. It is *not* a shadow of anything.

## Consequences of that choice worth knowing before changing it

* Read mode is **per window and per session**. It is not written to the
  layout store, so relaunching starts with the chrome shown. That is the
  right default — a shell that opened with no ribbon and no explanation of
  how to get one back is a shell that looks broken — and it is why nothing
  here touches [`crate::app::persistence`].
* Read mode is **not** per document. Closing and opening a file leaves it
  where the operator put it, which is what every reader in the class does.

# 4. Why neither verb raises an `Action`

The action funnel is for work that touches a **document** or that must not
happen part-way through laying out a frame. Neither applies:
nothing here changes a byte of the file, so there is nothing for the undo
log to hold and nothing to order against. That is `file.print`'s and
`edit.find`'s reasoning, unchanged — a toggle that only decides what is
drawn next frame is exactly the class the funnel is not for.

It also could not be done through the funnel without weakening it: the apply
phase is deliberately handed no [`egui::Context`], and both of these need
one — the memory write for read mode, and `send_viewport_cmd` for full
screen.
