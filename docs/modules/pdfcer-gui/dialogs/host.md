# `dialogs::host` — a dialog is an OS WINDOW


> *"Print dialogue box doesn't pop up in its own movable window. It is
> locked within the boundaries of the program's window. Like, I just assume
> you've been trained on a million lines of code and software that pops it
> up in its own window."*

He is right, and the last sentence is the diagnosis. `ui-conventions/dialogs.md`
G1 states the rule and states why immediate-mode toolkits get it wrong:

> **Why immediate-mode toolkits get this wrong.** Their in-viewport "window"
> widget is the path of least resistance — it looks like a dialog and is one
> function call. A real OS window needs a viewport/second-window API that is
> newer and more awkward. The default is wrong and nothing pushes back.

Every retained toolkit makes the OS window the **default** and the inline
panel the special case — `QDialog`, `NSWindow`, `ContentDialog` vs `Window`.
That default ordering is itself the guidance. This module restores it here:
calling [`Host::show`] is now as easy as calling `egui::Window::new`, so the
path of least resistance and the right answer are the same path.

## What an OS window buys, concretely

Not aesthetics. Four things an operator does with a print dialog:

1. **Move it off the document** to read the page underneath while choosing a
   range. An in-viewport window can be dragged to the edge and no further.
2. **Put it on the second monitor**, which is what a two-screen desk is for.
3. **Find it in the taskbar / Alt-Tab** when it has gone behind something.
4. **Resize it past the application window**, which matters for the print
   preview specifically: the preview is the point of the dialog and it is
   the first thing the 520 pt floor squeezes.

## It degrades, and that is deliberate rather than incidental

`Context::show_viewport_immediate` falls back to an **embedded** window —
literally the `egui::Window` this replaces — when the backend has no
multi-viewport renderer, which is the case on **web**. `MODES_AND_PANELS.md`
records the web fork as a live target, so a dialog host that only worked
natively would be a surface that vanishes on one of the two platforms.

The fallback is egui's, not ours: one code path, two renderings, and
[`Frame::class`] says which so a caller that genuinely must know can ask.
Nothing in this module branches on it except the position memory, which has
nothing to remember when the OS is not placing the window.

## G4 — Enter accepts, Escape cancels, and the default is VISIBLY the default

The second half of the operator's item, and the failure mode is the one
everybody has met:

> *"The operator types into the last field, presses Enter out of habit, and
> nothing happens — or worse, something other than what they expected."*

[`Host::buttons`] draws the pair and owns all three obligations, because a
caller that had to remember them would forget one:

- **Enter** activates the affirmative action — but **not while a text field
  has focus and wants the key**, which is why the check asks
  `ctx.text_edit_focused()` first. A multi-line field would otherwise lose
  the ability to type a newline the moment it sat in a dialog.
- **Escape** is equivalent to Cancel *and* to the close button, so all three
  routes out are one outcome — but **not on the press that leaves a text
  field**. egui clears focus before any widget runs, so a guard that asks
  `text_edit_focused()` about Escape is inert rather than weak, and the
  dialog would cancel on the same keystroke that exits the box. The first
  press leaves the field and the second cancels; [`Host::show`] holds the
  mechanism and the argument.
- The affirmative button is **drawn** as the default, from the theme's
  **accent** and the foreground the theme pairs with it
  (`Theme::accent_pair`), so the operator knows what Enter will do before
  pressing it. A default nobody can see is not a default; it is a surprise.
  This sentence said *"the theme's own selection fill"* until 2026-09-03,
  and that was the defect rather than a description of it: `selection_fill`
  is a 27 %-opacity canvas tint, so the default button rendered **paler than
  an ordinary button** and read as disabled. See [`Host::buttons`].

## G6 — it remembers where it was left

Position is held per dialog, keyed on the string [`Host::new`] was given,
and re-applied on the next open through `ViewportBuilder::with_position`.
**Nothing is remembered in the embedded fallback**, because egui places that
window and the OS does not.

It is stored in memory rather than on disk, deliberately. A position that
survived a restart would have to be validated against the *current* monitor
layout — G6 says so in the same breath — and a dialog that opens on a
monitor which is no longer attached is worse than one that opens centred.
Persisting it is a real feature with a real check to write; the session-long
version is the nine-tenths of it that costs nothing and cannot strand
anybody.

## THE HOST OWNS NO STATE, AND THAT IS WHAT MADE THE OTHER THIRTEEN
## DIALOGS ONE LINE EACH

The first version of this module kept the remembered position in a `Host`
**struct field**, which meant every dialog that wanted an OS window had to
grow a `host: Option<Host>` field, a `new_host()` constructor, and a
`take()`/put-back dance around the borrow — because `show` needed `&mut` on
the host while the closure it was handed needed `&mut` on the dialog.

That is fine for one dialog and it is a tax on thirteen. Worse, it made
`ShortcutsDialog` — a **unit struct**, deliberately stateless — impossible
to convert without giving it state it does not otherwise need.

So the position lives in `egui::Memory` keyed on the dialog's own id string,
`show` takes `&self`, and a `Host` is a **description** built fresh each
frame rather than an object with a lifetime. Three consequences, all wanted:

1. A conversion is one expression: `Host::new(…).show(ctx, |ui| …)`.
2. The memory **survives close-and-reopen**, which is what G6 actually asks
   for. The struct version lost the position the moment the dialog closed,
   and called that correct because it had no way to be otherwise.
3. There is no second copy of the position to go stale.

It is `insert_temp`, so it is session-scoped and never written to disk —
see the paragraph above for why that is the feature and not a shortcut.

## What this does NOT fix, said so it is a decision


This section read *"filed as a gap rather than papered over"* for a day. The
gap said: `eframe 0.35`'s `ViewportBuilder` has no owner or parent option —
`grep with_` over `egui/src/viewport.rs` returns thirty builders and none of
them is one — and `egui-winit` never passes down the parent relationship
egui itself tracks in `viewport_parents`. So a dialog could fall behind the
main window, which is *the* classic Windows bug.

**What closed it was a second symptom, not a second look.** The dialog
also **lost the keyboard a third of a second after opening**, measured with
both windows reporting their own focus, with the application asking for none
of it. The operator's version: *drag out a note box, type without clicking
the field first, and the words go nowhere.* Asking for focus again does not
work and that was tried — Windows refuses the foreground to a process that
does not already hold it.

**Ownership is not a request.** An owned window is by definition above its
owner and keeps activation as a property of the relationship. It is what
every native dialog on the machine already uses, which is why none of them
has either problem. `crate::dialogs::host` now sets it through
[`native_window::own_window`], finding the child by title among **this
process's** windows — the crate that call lives in exists so that this
crate's `#![forbid(unsafe_code)]` survives.

`with_always_on_top` remains refused, and the reason is worth keeping now
that it is not the only option: this project's own RAG records an
always-on-top window swallowing the driven harness's clicks with
`SetForegroundWindow` still reporting success. Trading a rare confusion for
a class of undiagnosable harness failure was always a bad trade. Ownership
gets the z-order guarantee without the input trap.
- **G5, focus trapping and tab order.** An OS window gets keyboard focus of
  its own, which is most of what G5 asks for and is strictly better than the
  in-viewport version had. Ordered tab traversal and a focus trap are not
  asserted by anything and are still a gap.

## The diagnostic channel had to learn about viewports, and why

`crate::diag::ui_rect` publishes a named region's rectangle so a driven
check can aim at a control without guessing. Those rectangles are **relative
to the viewport that drew them**, and until this module existed there was
only one viewport, so the harness could add the main window's client origin
and be right.

A dialog in its own window breaks that silently — the coordinates stay
plausible and land somewhere else entirely, which is the exact shape of
defect `D:/dev/rag/egui/` records twice already. So [`Host::show`] publishes
`viewport-inner`, the child's own client rectangle in **desktop**
coordinates, and `ui_rect` tags every region it publishes with the viewport
that drew it. The harness then has both halves and can convert; it is also
the only way a check can *tell* that a dialog opened in its own window,
which is what makes G1 assertable rather than a matter of looking.

## Item notes

### `mod placement`

Declared here rather than in `dialogs/mod.rs`, so it lives at
`dialogs/host/placement.rs`. That is not a stylistic choice: `dialogs/mod.rs`
stands at 1,496 lines against R2's limit of 1,500 and has no room for a
module declaration and its doc comment.

### `const ENGAGED`

**The real terminator of the focus request**, with [`FOCUS_FRAMES`] as
its backstop rather than the other way round. *"Keep asking for the keyboard
until the operator has used this window"* is the rule that matches intent;
a pass count is only there so a dialog nobody touches stops asking.

Once set it is never cleared for the life of that opening, so a dialog the
operator clicked into and then left never re-seizes the foreground.

### `fn explained`

An empty hover is the two-button caller's way of saying "no tooltip", which
is not the same as an empty tooltip: `on_hover_text("")` still opens a
box, and an empty box under the cursor reads as a surface that failed to
load rather than one that had nothing to say.

### `const BODY_MARGIN_PTS`

# Why a constant, and why it lives on the host

The operator's 2026-09-03 report — *"the print button that is so far off
in the corner it is touching the edge the window"* — was true of all
fourteen dialogs, because the `Ui` egui hands a viewport callback is the
window's root and nothing pads it. The main window never showed it: its
`CentralPanel` brings egui's own inner margin.

One number, owned here, so that fourteen dialogs cannot pick fourteen
values and so that nobody has to remember to pad theirs.

12 pt rather than egui's default 8: this shell's own `Metrics` use
`panel_padding` of 8-12 depending on preset, and a **dialog** is the one
surface where the window edge is a hard boundary rather than a seam onto
the next panel. Windows' own dialogs are roomier at the frame than at
internal gutters for the same reason.

It is deliberately NOT read from `Theme::of(ctx).metrics`, and that
is a real decision. This value is fed into [`Self::fit`], which sizes the
window; a metric that changes with the preset would change the window
size on a theme switch, and `fit` is the function whose doc comment
records an unbounded growth loop. A constant cannot participate in a
feedback loop.

### `fn field_focus_key`

Salted off [`Self::key`] so it cannot collide with the remembered
position even though the two live in different `Context`s. Read
[`Self::show`]'s Escape rung for why the fact has to be carried across a
pass at all.

### `fn each_dialog_gets_its_own_viewport`

A shared `ViewportId` is a shared OS window: the second dialog would
draw into the first one's frame, and which one you saw would depend on
draw order. Cheap to assert, and the failure is invisible until two
dialogs are open at once.

### `fn a_position_is_remembered_per_dialog`

The second half is the one worth a test. Two dialogs sharing a memory
key would drag each other around the desktop, and the key is derived
from the same string as the viewport id, so a mistake there is a
mistake in both places at once and invisible in either.

### `fn a_status_label_after_a_right_to_left_block_overflows_the_row`

The test above models the consequence; this one reproduces the CAUSE,
which is the part a reader will not believe on assertion alone:

> A `Layout::right_to_left` child inside a left-to-right `horizontal`
> is anchored to the RIGHT EDGE of the space it was offered, and its
> `min_rect` reaches that edge **whether or not it needed the room**.
> Anything appended after it is therefore placed past the edge.

Both orderings are laid out here in the same width, and the assertion is
on the resulting row width against the width the row was given. Buttons
first overflows; the status label first does not. That difference is the
entire fix, and this is where it is proved rather than argued.

Run it against the pre-fix ordering — swap the two blocks in
[`super::super::print`]'s `footer` — and the first assertion fails. That
is the falsification, and without it this test would only be describing
the code it sits next to.

Measured, so the size of the thing is on the record: in a 400 pt row
the pre-fix ordering produces **481.9 pt** and the fixed ordering
produces **exactly 400.0**. That 81.9 pt is the step the window grew by
on every single frame the dialog was open after a print — which is what
"little steps to infinity" was.

### `fn set_owner`

# Why a channel through `egui::Memory` and not an argument

Because the alternative is a fourteenth argument on thirteen call sites to
carry one fact that never varies. `eframe` hands the application window's
handle out **exactly once**, to `PdfcerApp` at start-up, and every dialog in
the program wants the same one; threading it by hand would be thirteen
opportunities to pass `None` and one dialog that quietly kept G3's
symptoms.

It is safe as a hidden channel for the reason most hidden channels are
not: **it has exactly one writer**, `app::frame`, on the frame path, and the
value is a constant for the life of the process. There is no ordering to get
wrong and no second producer to disagree with.

### `struct Host`

Held by the dialog it belongs to, so its lifetime is the dialog's — which is
what makes the position memory correct without anything having to clear it.
A dialog that is closed and reopened gets a fresh `Host` and therefore opens
where the platform puts it; a dialog that stays open across frames keeps the
position it has been dragged to.

### `const REGION_CANCEL`

This is the button `dialogs.md` G4 makes indistinguishable from Escape and
from the OS close button, so a driven check that presses it is measuring
all three routes at once.

### `const REGION_KEEP`

Absent from the trace on every dialog that passes `None`, which is all of
them but Print. An absent region is not a defect here; it is the R9 answer
-- a route that does not exist renders nothing.

### `fn new`

`id` must be unique and stable per dialog — `"print"`, `"insert-image"`.
It keys the OS window, and it is also what the diagnostic channel
publishes, so a driven check names the same string the code does.

### `fn opening_near`

# Why this exists: A16c


# What the host promises about the position, and what it does not

* It is honoured **only on the pass the window is created**, and only
  when there is nothing remembered — a position the operator dragged the
  dialog to always wins over one the program computed. G6 is unchanged.
* It is **clamped onto the application window**, so a caller may compute
  freely without knowing the desktop's size or which monitor the
  application is on, and cannot open a dialog half off the screen or over
  the ribbon. See `placement::opening`.
* It positions the window's **outer** corner, so the dialog lands about
  one title bar higher than a caller thinking in content coordinates
  imagines. This is a "roughly here" placement; a child window's
  decoration height is not knowable before the window exists.

A builder method rather than a fifth argument to [`Host::new`],
because having an opinion about where you open is the rare case: a
dialog raised from a menu has no reason to, and only one raised by a
gesture on the page does. Every dialog that does not care should not
have to pass `None` to say so. (Deliberately not stating how many do
care — a count written in prose beside the thing it counts is a claim
that decays, and this project has spent six corrections on that shape.)

### `fn show`

`add` is handed a `Ui` inside the window and may do anything an
`egui::Window`'s closure could. What it returns comes back untouched, so
a dialog that computes something while drawing does not need a field to
carry it out.

# Why the close signal comes back rather than through an `&mut bool`

`egui::Window::open(&mut bool)` is the idiom this replaces, and it has a
property worth losing: the flag is written *during* the draw, so a
caller reading it afterwards cannot tell whether the operator closed the
window or the caller's own code did. [`Frame::closed`] is a report about
this frame only, and the caller decides what closing means — which for
a dialog that is mid-transaction is not always "stop".

### `fn scrolled`

> *"Those buttons should always be available, and if there isn't size
> for all the features they get scrolled in their own space."*

He said it while reporting one dialog, and it is written here rather
than there because it is a statement about **every** dialog. It is also
the shape Word and Acrobat use for every options window they have, which
under this project's standing *"use the conventional interaction, never
invent one"* rule makes it the spec and not a preference.

# Why this is the structural fix and [`Host::fit`] is not

`fit` grows a window to its content, and it is a good mechanism, but it
is a **negotiation with the window manager** — it can be refused, it is
bounded by a budget, and it cannot help at all once the operator has
made the window small on purpose or the screen is smaller than the
content. Everything it does is best-effort.

This is not best-effort. The footer is allocated **first**, out of the
window's own rectangle, so the body can only ever have what is left.
There is no size at which the buttons are off-screen, because there is
no path by which the body can take their space. A window too small for
its content becomes a window with a scrollbar, which is a nuisance; the
alternative was a transaction the operator could not finish, which is a
defect.

⇒ The two compose rather than compete: `fit` still opens the dialog big
enough that the scrollbar never appears in the ordinary case, and this
guarantees the outcome when it does.

# Why `state` is threaded rather than captured

Both closures nearly always want `&mut self` of the calling dialog, and
two closures capturing the same `&mut` cannot coexist even though they
run one after the other. Passing the state through as a parameter is the
borrow-checker-shaped way to say *"sequentially, not simultaneously"*.

# A note for `fit`

A `ScrollArea` reports the size it was GIVEN, not the size of what is
inside it, so a dialog converted to this shape stops asking to grow.
That is deliberate and is already documented in [`Host::fit`] — it is
why the print dialog, the first to have a scrollbar, was unaffected by
the fit mechanism. The consequence is that a converted dialog's stated
`default_size` becomes the size it actually opens at, so that number
wants to be generous rather than minimal.

### `fn buttons`

The two-route convenience form of [`Host::footer`], which is where all
of the reasoning lives. Neither button carries a hover sentence and
there is no third route out; a dialog that wants either calls `footer`
directly.

Returns `(accepted, cancelled)`.

### `fn footer`

Each argument is `(label, hover)`. An **empty hover draws no tooltip**,
which is the case [`Host::buttons`] passes for both of its pair.

Returns `(accepted, cancelled, kept)`. `kept` is always `false` when
`keep` is `None`, and no two of the three are ever `true` together:
they are three different rectangles and Enter belongs to exactly one of
them.

# The order is Cancel then Accept, right-aligned

Which is Windows' order and the order every dialog on this operator's
machine uses. It is not a preference: a button's meaning is learned by
position long before it is read, and a dialog that reverses the pair is
a dialog whose Cancel gets clicked by muscle memory aimed at OK.

# Enter is refused while a text field wants it

`ctx.text_edit_focused()` — the same predicate `canvas::textedit`
enforces one copy of, and `tools/gates/check-typing-guard.sh` fails the
build on a second. Without it, a dialog with a multi-line field would
accept on the first newline the operator typed, which is worse than
having no Enter at all: it commits a half-written transaction.

A **single-line** field or a `DragValue` gives up focus on the Enter
that commits it, before this runs, so `text_edit_focused` alone reads
false and the commit would also accept the dialog — typing a poster
scale and pressing Enter would print. So Enter takes the Escape rung:
**the first Enter commits the field, the second accepts**, read from
the flag [`Self::show`] carries across the pass.

# The third button, and why it is LEFTMOST rather than beside the default

Added for **O185**, the print window's *Keep and close*. `dialogs.md`
G4 makes the OS close button, Escape and the cancel button deliberately
indistinguishable — one meaning for every route the window chrome
offers — so a dialog that owns persistent state cannot express *"keep
what I set, but do not act"* through any of them. That is the
`OK / Cancel / Apply` triad, and the third button is the only member of
it G4 does not already govern: it is a labelled control the operator
pressed **on purpose**, which is precisely the case G4 never
contemplated.

It is drawn last in a `right_to_left` layout, so it lands **leftmost**,
furthest from the default. That is the correct position for the
least-used of the three and it keeps the accept/cancel pair in the
place muscle memory expects: adding a route out must not move the two
routes that were already there.

It is deliberately NOT bound to a key. Enter is the default's and
Escape is Cancel's; a third chord would be a gesture with no affordance
naming it, and the one thing worse than an undiscoverable button is an
undiscoverable key that means something different from the button
beside it.


`tools/ui-verify` presses controls by name. This function contained no
`crate::diag::ui_rect` call at all, so **no driven check could press
any dialog's Print, OK or Cancel** — the three most consequential
controls in the application were the ones the harness could not reach.
It was found while designing O185's check, which cannot exist without
them.

The names are **generic** — `dialog.buttons.accept`, `.cancel`,
`.keep` — rather than per-dialog, because the labels are not: this
function is handed *"Print"*, *"Print — 3 sheets will be clipped"*,
*"OK"* or *"Save"* depending on the caller and the state, and a check
that had to know which would be asserting the label rather than
pressing the button.

⚠ **Two dialogs open at once share these three names**, and the last
one drawn wins the frame. Stated rather than guarded: the alternative
is threading a dialog identity through every caller to disambiguate a
case the driven checks never construct, and a name that is sometimes
qualified and sometimes not is worse than one that is never qualified.
A check that needs certainty asserts the dialog's own body region in
the same frame.
