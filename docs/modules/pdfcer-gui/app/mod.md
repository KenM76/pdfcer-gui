# app — the one owner of state, and the shape of a frame

[`PdfcerApp`] is the single owner of everything the shell knows. There is
no global, no `thread_local`, no second source of truth: if it is state,
it is reachable from here, and if it is reachable from here it has one
owner. That is what makes the action funnel in [`actions`] enforceable —
a widget cannot mutate a document it has no path to.

## The order of a frame, and why the order is load-bearing

[`PdfcerApp::update`] runs four steps, in this order, every frame:

1. **Collect keyboard actions** ([`keyboard::collect`]) and dispatch the
   frame's **manifest chords** ([`keyboard::commands`] →
   [`PdfcerApp::dispatch_command`]) — before any widget is built, so the
   map sees the frame's raw key presses rather than whatever survived a
   widget consuming them. The split between the two is the subject of
   [`keyboard`]'s section: chords the manifest keymap binds arrive as
   command ids and go through the same dispatcher a ribbon click does,
   and chords the viewer owns outright arrive as actions.
2. **Compose the panels** — draw, and let each surface push more
   actions, then the Find overlay over the top of them. Nothing mutates.
3. **Apply the actions** ([`PdfcerApp::apply_actions`]) — after the frame
   is drawn, in one place, in the order raised.
4. **Settle and rasterize** ([`PdfcerApp::settle_and_rasterize`]) — decide
   whether the cached texture still matches the (now updated) view
   state, and start a render if not.

Step 4 must come after step 3 or every zoom would be rasterized one
frame late — visible as a page that always lags the operator's last
action by a frame. Step 3 must come after step 2 because that is the
actions-not-mutations invariant itself.

## Panel composition order is load-bearing (layout *and* focus order)

egui resolves panels in the order they are added, and that order
determines both the rectangles they get and the order the Tab key visits
their widgets. **S0 adds exactly one panel** — the `CentralPanel` — so
there is nothing to order yet. The rule is written down here anyway,
where the composition lives, because it is the thing the old shell got
bitten by and it constrains every panel S2 and S3 add:

> A full-width bar (toolbar, status bar) must be added **before** any
> side panel, or it starts at the side panel's edge instead of spanning
> the window. A status bar that does not span the window is not a status
> bar.

That constraint conflicts with the Tab order a reviewer would prefer
(toolbar → canvas → footer), and in the old shell the layout property
won, deliberately. The same trade will be made at S2, and it should be
made with this paragraph in front of whoever makes it.

## What this build does not have, stated so it is not mistaken for an
oversight

The list S0 opened with — *"no ribbon, no QAT, no dock, no status bar, no
find bar, no dialogs, no Open command"* — is down to **the save**. The find
bar is [`crate::find`], reached by Ctrl+F and by the status bar's Find
toggle. **Open, Close and Recent are wired**: the picker and its
diagnostics seam are [`files`], the list is [`recent`], and both arrive
through [`actions::Action::Open`] like everything else. What is left of
that sentence is scheduled in `PROJECT_PLAN.md` §4, and the
"no placeholders" invariant still applies to all of it: unavailable
renders **nothing** rather than a greyed-out control that explains itself
badly.

## Item notes

### `const REGION_CENTRAL_PANEL`

The outermost region this crate owns today. When the ribbon, the docks and
the status bar land, each declares its own — see [`crate::diag::ui_rect`]
for the seam and for the naming rule.

### `const REGION_STATUS_MESSAGE`

One name for all four non-open arms. A check asking "is the shell's
explanatory text legible?" is asking about the region, not about which of
the four sentences happens to be in it — and the four are laid out
identically, so they are genuinely the same region.

### `const DOCK_SLOT`

De-duplicated on the rendered line, so a dock that is not changing costs
one line rather than one per frame — the lesson `canvas-pointer` taught
when a stationary pointer emitted fifty identical lines in nine seconds.

### `const DOCK_DROP_SLOT`

Its own slot rather than a second line under [`DOCK_SLOT`], because two
call sites sharing a slot each suppress the other's lines: the dock's own
summary changes on a layout edit and the offer changes on every pointer
move, so sharing would erase whichever moved second.

Reports `none` rather than falling silent when no drag is in flight. A
change-only slot that simply stops emitting cannot be told apart from a
program that stopped running the code, and *the offer was withdrawn* is
exactly what a check asserting a stand-down needs to read.

### `const DOCK_TEAR_SLOT`

Separate from [`DOCK_DROP_SLOT`] for that slot's reason, and because the
two are mutually exclusive by construction — the compass needs a
compartment under the pointer and the tear needs there to be none — so a
frame reporting both named is a defect the two lines make visible.

### `fn opening_ribbon_mode`

# Why this is a function and not two lines inside `PdfcerApp::new`

Because it was two lines inside `PdfcerApp::new` and one of them was
missing for ten days, and the shape of that absence is worth a name.

`new` builds the `RibbonState` **before** it calls [`modes::start`], and
seeds it with `modes().first()` — Read — because an unset mode makes the
shell show every tab regardless of the selector. `start` then restores the
operator's remembered mode into [`modes::Modes`], the dock and the layout.
The ribbon, already built, kept Read. `PdfcerApp::docks` reconciles the two
once a frame by comparing `ribbon.mode()` against `modes.active()` and
treats the **ribbon** as authoritative — so the restored mode lived exactly
one frame and was then thrown away.


> Someone who spent an afternoon in Edit came back the next morning to a
> program that had silently forgotten.

It bites markup hardest: markup is authored in **Review**, and the program
reopened in Read every single time.

# The start-up trace said it was working

`modes::assemble` emits `mode-restore stored=Some("review")
using=Some("review")` — true of what *that stage* adopted — and forty lines
further down the same trace carries `mode-changed from=Some("review")
to=read`. ⇒ **A stage's trace records what the stage decided, not what the
frame settled on.** The two are different claims wherever a later stage may
overrule an earlier one, and a harness grepping only the first would report
a working feature.

It was found, and the fix proven, by launching the **release binary** off
screen against a `layout.ron` holding `mode: Some("review")` and reading
which ribbon tabs published a rect:

| | tabs that drew |
|---|---|
| before | `file`, `view` |
| after | `file`, `view`, `pages`, `markup`, `measure` |

# What the test below can prove, and what only the launch can

This function's test drives the **decision** and is falsifiable: hand it a
restored `"review"` beside a current `"read"` and it must answer `"review"`,
which the ten-day-old code — passing `first` unconditionally — could not.

It cannot prove that `PdfcerApp::new` still *calls* it. An earlier attempt
asserted `app.ribbon.mode() == app.modes.active()` after `PdfcerApp::new()`
and **stayed green with the fix deleted**, because `new` reads the real
layout file and on a machine with no stored mode both sides are Read for a
trivial reason. A check that cannot fail is not evidence, so that test was
removed rather than left to imply coverage it did not have. The wiring's
oracle is the off-screen launch above.

### `fn a_restored_mode_beats_the_seeded_first_mode`

Falsified by replacing the body with `current.or(restored)`, which is
the ten-day-old behaviour: this goes red and the two below stay green,
which is why all three exist rather than only this one.
