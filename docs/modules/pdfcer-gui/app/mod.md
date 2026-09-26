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

### `mod filedrag`

The position half of drag-and-drop: `winit` throws the OLE drop point away
and no mouse-move arrives during a drag, so the point is asked of the
operating system. Lets a surface CLAIM a drop that landed on it — the Pages
panel claims a document dropped onto its thumbnails — with an
unconditional fallback to [`dropped`] for everything else.

### `mod markupband`

`pub`, unlike [`fontband`] beside it, and for one mechanical reason:
[`PdfcerApp::markup_change`] parks a `markupband::MarkupEdit` on a public
struct, and a public field of a type nothing outside the module can name is
a private interface the compiler refuses under `-D warnings`. `fontband`
needs no such thing because its operand type lives in `app::actions`, which
is already public.

### `mod pickstore`

The difference is the whole of the module: a splitter drag reports a change
on every frame of the gesture, while a filter can only change on a discrete
click, so one decision already equals one write. Its header also carries the
three on-disk states and why *an empty file* must never be collapsed into
*no file* — that would silently overrule a deliberate choice every restart.

### `mod reachout`

A separate store from `pdfcer_core::settings` on purpose: that one exists
because a **standard declines to have an opinion**, and every entry cites
the clause that is silent. How sharply a page is rasterised cites nothing.
Its header also records which five of the seven commissioned View ▸ Render
settings turned out to have nothing behind them, and why.
**Does this document reach outside itself?** — a submit button, a
launch action, a script that runs on open. Asked once when a document opens,
answered on the status row. Its header carries the engine's own account of
why a scan that under-reports reads as a clean bill of health.

### `mod settings`

Not a struct-holder. Of the thirteen settings the old shell persisted,
**nine were never read by anything** — they were saved, loaded, shown,
edited, and then discarded at every call site that wrote
`ExtractOptions::default()` or `RenderOptions::default()` or
`SaveOptions::identity()`. This module owns the three replacements and a
`syn` check that no other file may bypass them.

### `mod textoperand`

`OPERATOR_REQUESTS.md` O198. The five Format ▸ Font controls and the
Properties font editor were each gated on a swept text range, which the
only mode that shows them cannot produce. This module widens the operand
to include *the single selected text object*, by the byte-span join the
object colour swatch has used since O89 - so an object selection and a
sweep become the same gesture with the same operand. See its header for
the deadlock, and for why the cheap half is a separate function.

### `mod toolstatus`

The strip the right dock reserves above its columns, naming what is armed
and offering to put it down. It is the surviving half of the Tool panel;
that panel's live controls moved to `crate::panels::properties::tool` and
its disclosure block to `crate::panels::properties::disclose`. The module
header tabulates every piece and where it went.

### `struct PdfcerApp`

It derived one until the settings store arrived, and nothing in the
workspace ever called it — checked, not assumed. Removing it is the right
answer rather than a workaround for `StoreLocation` having no `Default`:

A defaulted `PdfcerApp` would have **no shell manifest, no command registry,
no panel registry and no settings store** — a state [`PdfcerApp::new`] can
never produce and every method here assumes away. Deriving a constructor
for an unreachable state is how a test ends up asserting something about a
program that cannot exist, and this crate has a standing preference for
making such states unrepresentable rather than merely unused.

`new()` is the constructor. It is the only one.

### `mod es`

**A mode is a named workspace** (`MODES_AND_PANELS.md` Part 2): Read,
Review and Edit are three defaults the operator then adjusts, and
leaving Edit and coming back restores the arrangement rather than a
default. That is why the mode selector and the dock are not two
independent features — the selector is how you reach a workspace.

### `fn window_handle`

Called once, from `crate::run`, with the `eframe::CreationContext`. See
[`PdfcerApp::window`] for what it is for and why it is captured there.

`None` for every non-Win32 handle and for a platform that reports none.
That is not an error and is not disclosed: the only caller passes it
straight to `pdfcer-print`, whose contract already says a null owner is
legal.

### `fn new`

The shell is assembled **once**, here, and not per frame: merging
and validating a manifest walks every tab, group and item, and doing
that sixty times a second to produce a value that cannot have
changed would be a per-frame cost with no per-frame cause.

Order is load-bearing. Commands are registered **before** the
manifest is merged, because the merge resolves every item against
the registry — that resolution is what makes a capability that is
not compiled in disappear from the ribbon rather than render as a
dead control (`R8`, `SHELL_FRAMEWORK.md` §5b).

### `fn refresh_acrobat`

Called on exactly one event — the operator saving Settings — because
that is the only thing inside pdfcer that can change the answer. See
[`Self::acrobat`] for why this is not asked per frame, and for the one
change it deliberately does not notice.

It reads [`Self::prefs`], so it must run **after** the draft has been
adopted. `crate::app::settings_window::save_settings` calls it there,
and the ordering is the thing to preserve if that function is ever
rearranged: run first and the button would appear one Settings visit
late, which is exactly the "typed a path and nothing happened" failure
the setting exists to fix.

### `fn configure_context`

Only one setting so far, and it is not optional: egui's
`zoom_with_keyboard` makes Ctrl+Plus/Minus/0 rescale the entire user
interface. In a document viewer those chords mean *page* zoom — that is
what they do in every browser, in Acrobat, and in every other PDF
reader — so egui's handler is switched off and [`keyboard`] handles
them. Without this the chords would silently do the wrong thing and any
tooltip advertising them would be telling a lie.

Note that Ctrl+**scroll** is unaffected: egui converts that to a
`zoom_delta` in the input state but does not act on it itself, so the
canvas is free to interpret it.
