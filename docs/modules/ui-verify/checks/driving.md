# `ui-verify/checks/driving`

`checks::driving` — the moves that **every check which drives the ribbon**
has to make, in one place.

# Why this module exists

[`crate::checks::markup_rectangle`] was the first check to click a ribbon
control, and it had to invent five small things to do it: read the last
rect the application declared for a name, list the names it *did* declare
for a SKIP reason, re-parse the same captured stderr under the **shell's**
line prefix, measure a control's fill out of a capture, and compare two
fills. All five are properties of *driving an `egui-shell` ribbon*, not of
markup.

The second and third such checks — [`crate::checks::measure_linear`] and
[`crate::checks::read_mode`] — needed the same five, and a third copy of a
function is where copies start to disagree. So they live here, with the
reasoning that shaped them carried across rather than summarised.

`markup_rectangle` deliberately keeps its own copies. Rewriting a check
that is already known to detect its defect, in the same change that adds
two new ones, would mean the three checks stopped being independent
evidence of each other at exactly the moment the harness grew. This module
is a *widening*, not a refactor; when someone next has cause to touch
`markup_rectangle` for its own sake, folding it onto these is a one-line
change per helper.

# The two diagnostic channels, and why a check reads both

One captured stderr file, two vocabularies:

| Channel | Switch | Prefix | Says |
|---|---|---|---|
| the shell | [`SHELL_DIAG_ENV`] | [`SHELL_TRACE_PREFIX`] | a segment/tab/control took a click |
| the application | the profile's `diag_env` | the profile's `trace_prefix` | what the application did about it |

The split is not an accident of this build. `egui_shell::verify`'s header
explains that one environment variable name lets a harness arm tracing on
*any* `egui-shell` application without first discovering its name, and the
prefix is the application's so two crates' lines never blur together. The
consequence for a check is the thing that makes a failure attributable: a
present `ribbon-command-invoked` with an absent application-side effect
names the application's dispatch and nothing else, and an absent
`ribbon-command-invoked` means no click was ever delivered — which is a
SKIP, because a check that could not deliver a click has learned nothing.

## Item notes

### `const MAX_BAND_SCROLLS`

A bound rather than a `while true`, because a harness that hangs reports
nothing at all. It is deliberately far above any real tab — the widest tab
in `RIBBON_IA.md` has nine groups — so hitting it is a defect in the
application (an arrow that never retires) rather than a tab this helper
cannot search, and the two must not be confused.

### `fn at_this_stop`

S3 gave the band a middle rung: when it runs short of width a whole group
folds into a single captioned button, its items reachable through that
button's popup. They are on the ribbon, they are one click away — and they
**publish no rect**, exactly like a scrolled-away group's, which is why this
search exists at all.

`export_dxf_writes_the_pages_geometry` went red on it and the failure read
as a lost command: *"the File tab declares no
`ribbon.item.file.export_dxf`, on the band or in the overflow."* The command
was not lost. The harness was asking about two of the three places it could
be, and the window the harness opens is 1,100 pt wide — precisely the width
at which the Export group collapses.

It is checked at **every scroll stop**, not once, because collapsing
happens before scrolling: a group that scrolls into view can arrive already
collapsed, and looking for its items on the band would find nothing.

# Errors

If the trace cannot be read, or a click cannot be delivered.

### `fn rewind_band`

The left arrow is drawn **only** while the band is scrolled off its start
(`egui-shell`'s `ribbon::band`: `if scrolled > 0`), so its absence is the
termination condition rather than a count this helper would have to keep in
step with the application's.

On a band that is already at position zero this costs one trace read and
no clicks, which is why `declared_or_in_overflow` can call it
unconditionally.

# Errors

If the trace cannot be read, or a click cannot be delivered.

### `fn collapsed_groups`

A collapsed group publishes `ribbon.group.<tab>.<id>.collapsed` — a
deliberately distinct name from the expanded `ribbon.group.<tab>.<id>`, so
that a check can tell *"on the band, collapsed"* from *"on the band"* and
from *"gone"*. This is the consumer that distinction was created for.

### `fn a_regions_last_declaration_is_the_one_that_is_used`

The same property [`crate::checks::markup_rectangle`] pins for its own
copy. Pinned twice on purpose: the two copies exist to be independent,
and an independent copy with no test of its own is not independent
evidence, it is an untested duplicate.

### `fn the_application_and_shell_streams_do_not_contaminate_each_other`

If a future prefix change made one a prefix of the other, this test is
what says so — and the symptom otherwise would be a check that reads a
`ribbon-command-invoked` that is not there, or misses one that is.

### `fn the_threshold_separates_pressed_from_unpressed_under_both_palettes`

The second assertion is the one that matters: `AA_LARGE` is 3.0 and
these fills are 1.5:1 and 1.3:1 apart, so a check written against the
harness's usual legibility oracle would report "no difference" about a
control that is visibly blue.

### `const SHELL_DIAG_ENV`

See the module header. `pdfcer-gui` does not call
`egui_shell::verify::set_prefix`, so the shell's lines arrive under the
crate's default prefix, [`SHELL_TRACE_PREFIX`].

### `const MODE_EVENT`

Emitted on **every** click of a segment, including a click on the segment
that is already selected (`ribbon::mode_selector` sets `chosen` from the
click and filters for "did this change anything" only in its *return
value*, after the line is written). That is what makes it usable as the
input-channel proof for a mode a check merely wants to be *in*, rather than
only for a mode it is switching *to*.

### `const UNIMPLEMENTED_EVENT`

Read only to *improve a failure message*: its presence alongside a missing
application-side effect is the signature of a dispatch that received the
command and had no arm for it, which is a different fix from a dispatch
that never received it at all.

### `fn declared`

**Last wins.** A region is re-declared whenever it moves, and an early
frame can carry a rect from before the layout settled — the find bar's
one-frame misplacement was exactly that, and taking the first occurrence
would aim a check's clicks at it.

### `const VIEWPORT_OUTER_EVENT`

Use this, never [`VIEWPORT_INNER_EVENT`], to check a window against a
position the application *asked for*: `egui::ViewportBuilder::with_position`
speaks outer points. The two rectangles differ by the host's border and
title bar, so the inner one answers a placement question off by the chrome —
an error that looks like a real defect and varies by platform.

### `fn frame_for`

Every `ui-rect` rectangle is relative to **the viewport that drew it**.
There was one viewport until `crate::checks` was written and until this
harness's whole coordinate model was built, so
`session.frame()?.declared_center(rect)` — add the application window's
client origin — was right everywhere.

`dialogs::host` made a dialog a real OS window. Its regions publish
rectangles that look **exactly like** the ones this harness has always
converted and name a place several hundred points away, because the origin
they are relative to is the dialog's client area rather than the
application's.

That is a coordinate-space defect with plausible numbers, which is the
single most expensive shape of bug in this project's record — `D:/dev/rag/egui/`
carries three instances, every one presenting as *"the click lands somewhere
else"*. The application therefore tags each region with the viewport that
drew it and publishes that viewport's own origin, and this function joins
the two.

# What it returns

The `WindowFrame` to convert with. For an untagged region — the application
window, which is every region that existed before this — that is
`session.frame()`, unchanged, so no existing call site changes behaviour.

# Why an absent `viewport-inner` is an ERROR and not a fallback

Because falling back to the main window's frame would produce a **click at
a plausible wrong place**, which is precisely the failure this exists to
prevent, arriving through the code written to prevent it. A tagged region
with no published origin means the application drew a dialog and did not say
where — a defect worth reporting, not one worth guessing around.

### `fn declared_in`

[`declared`]'s twin, for a caller that is going to CLICK the region rather
than measure it. The rectangle alone is not enough to aim with once dialogs
have their own windows — see [`frame_for`].

Shares `declared`'s retirement rule, and shares it by calling it: a region
that has been retired is not declared, whichever viewport drew it.

### `fn stable_rect`

Reads the region, settles, reads it again, and repeats until two consecutive
reads agree — or until it gives up and returns the last one it saw.

# Why this exists, and it is a defect report

`ui-rect` is a **change log**: the application emits a line when a rect
moves, so [`declared`] answers *where that control was as of the last frame
the application drew*. That is exactly right for a settled window and
exactly wrong for one in motion, and the difference is invisible — a stale
coordinate is a number, not an error.


> **A harness that reads a coordinate and then acts on it owns the interval
> between the two.** The only honest way to close that interval is to watch
> the coordinate until it stops.

# Why it gives up rather than failing

A rect that never settles is a real state — an animation, a spinner, a
progress bar — and this helper cannot know whether the caller is aiming at
one. Returning the last observation lets the caller's own assertion produce
the verdict, in its own words, with its own diagnosis. A `Result` here would
make every call site handle a failure mode most of them cannot describe.

### `fn declared_since`

# Why [`declared`] is the wrong question for a gesture-only overlay

`declared` asks *"is this on screen now?"*, and it is right to: a region
retired after its last declaration is a fossil, and reading one produced a
confident, detailed, entirely wrong layout-defect report once already.

But an overlay that exists **only while the pointer is down** — a drop
caret, a rubber band, a snap indicator — is *guaranteed* to be retired by
the time a check can look at it. The harness cannot read the trace mid-drag:
`Driver::drag` presses, moves and releases before it returns. So `declared`
answers `None` for a caret that drew perfectly, and the check reports the
feature missing.

That is not hypothetical either. It is exactly what happened on
2026-08-19: `pages_drag_shows_where_it_lands` failed with *"NO
`panel-pages-drop-caret` region was ever published"* while the trace
carried `ui-rect name=panel-pages-drop-caret rect=[[258.0 239.1] - [262.0
331.9]]` four lines above the release. The indicator worked. The check was
reading a change log as a snapshot in the other direction — asking for
presence *now* about a thing whose whole nature is to be gone now.

# What this asks instead, and why the anchor is required rather than optional

*"Was it published during THIS gesture?"* The `after` line number is the
gesture's own start event, so a caret left over from an earlier drag in the
same run cannot satisfy it. Without that anchor this would be
`last-rect-ignoring-retirement`, which is the fossil-reading bug wearing a
helpful name — and it would pass on a build where the caret drew once at
startup and never again.

`TraceLine::lineno` is the position in the file, so the two streams are
directly comparable — the same property [`declared`] relies on.

### `const UI_RECT_GONE_EVENT`

Matched literally, like every other event name in this crate, so renaming
it in `crate::diag` without changing it here silently returns [`declared`]
to reading fossils.

### `fn clipped_away`

Returns the application's own measurement - the region's rectangle, the
clip it was tested against, the fraction that survived and the floor it
failed - or `None` when no such line stands.

# What this is for, and it is a fix for a failure MESSAGE


Measured that day: `restyling_selected_text_reaches_the_document` reported
the Properties panel as saying nothing about a 12-character selection, and
named `app::panels::show_panel`, `panels::properties::text::section` and
`TextStyleDraft::sync` as candidates. All three were correct code. The
Text section had drawn every one of its controls; its CMYK refusal
sentence sat in a `ui.horizontal`, which does not wrap, so the section's
box came out 851.7 pt wide inside a 354 pt dock and 42 % of it survived.
The operator saw a truncated sentence; the check saw an absence; the two
were the same defect and neither said so.

⇒ A check that reports an absent region should ask this first. The
candidate list is for the case where the answer is `None`.

# The retirement rule, and it is [`declared`]'s

The clipped line is a change log entry like every other line on this
channel: it is written when the verdict CHANGES and stands until it
changes back. A `ui-rect` for the same name at a later line number means
the region became visible again, so the clipped line is a fossil and this
answers `None`. Reading `.last()` alone would report a region as clipped
on the strength of a frame it has long since left - the exact mistake that
once made the UI-scale check report eighteen ribbon controls as mislaid.

### `fn live_names`

# Why this exists beside [`declared_names`], which counts fossils

[`declared_names`]'s own documentation says *"used only for SKIP reasons"*,
and it means it: it collects every name that has **ever** appeared, because
an error message listing what the application *did* declare is more useful
the more it lists. Retirement is irrelevant to that job.

It is exactly wrong for a **count**. The `ui-rect` channel is a change log,
so a row that was deleted leaves its last declaration standing for ever, and
counting names therefore counts rows that are gone.

That is not hypothetical. On 2026-08-19 the Manage-groups check reported
*"the round trip did not close: 1 row before, 2 after the delete"* over a
trace containing `dimension-group-delete id=1`, `delete-dimension-group
epoch=3` **and** `ui-rect-gone name=dimension-groups.draw_into.1`. The
delete had worked, at every level, and the check said it had not — a
confident, specific, entirely wrong defect report about a feature that was
correct, produced by a helper being used outside the job its own doc comment
names.

So: **[`declared_names`] to say what was seen, this to say what is there.**
If a check compares two numbers, it wants this one.

### `fn declared_names`

Used only for SKIP reasons. A reason that says "I did not find X" and does
not say what it *did* find sends its reader to guess; this crate has a
standing rule about that ([`crate::checks`] rule 5).

### `const OVERFLOW`

One file, two vocabularies — see the module header. `Session::trace` parses
with the profile's prefix; everything `egui-shell` writes carries its own
and lands in [`Trace::other`] on that parse. Re-parsing is cheap next to a
click and keeps both streams honest: a line is attributed to whichever
crate actually wrote it.

# Errors

If the captured stderr cannot be read at all.
The control that holds the groups a narrow ribbon could not fit.

### `const SCROLL_LEFT`

Drawn only while the band is scrolled off its start, so its presence is the
predicate *"this band is not at position zero"* — which is how `rewind_band`
knows when to stop.

### `fn declared_or_in_overflow`

The fix for a whole class of false SKIPs, and it is worth understanding
why they were false rather than treating this as a convenience.

The harness drives a **1100 pt** window. At that width the ribbon correctly
re-wraps, collapses and finally scrolls its rightmost groups — that is the
responsive behaviour working, not failing. A check that looked only at the
tab surface then reported *"no `ribbon.item.file.print` region on the File
tab"*, which is true and reads as *"the command is missing"*, which is
false. It cost `print_dialog_reaches_the_spooler` a standing FAIL that was
written up as a harness gap and left, and it would have cost `about` the
same.

# The overflow is a SCROLL, not a menu — and this helper did not know




⇒ The published *name* being a stability contract is right, and it is
exactly what made this survive: nothing renamed, nothing failed to compile,
and the one caller that would have noticed reported the application as
broken instead.

# What it does now

1. Look where the band is standing. Return immediately if the item is
   there — the overwhelmingly common case, and it costs one trace read.
2. Otherwise **rewind** the band to its first group, so the search covers
   the whole row rather than the part of it to the right of wherever a
   previous call left it. This is what makes the helper *idempotent*:
   without it, a check that asks for an item in the last group and then for
   one in the first would be told the second does not exist.
3. Walk left to right one stop at a time. At each stop, look on the band,
   then open each **collapsed** group in turn and look inside it.
4. Give up only when the right arrow has retired — the band is showing its
   last group — and rewind before answering, so a `None` leaves the ribbon
   where it was found.

# Why this returns the rect rather than clicking

Because *"where is it"* and *"press it"* are different decisions, and some
callers want to measure a control rather than invoke it. On success the band
is left **scrolled to where the item is visible** and a collapsed group's
popup is left **open**, which is what a caller that is about to click wants;
a caller that is not can dismiss the popup with Escape.

# Errors

If the trace cannot be read, or a click cannot be delivered.

### `struct BandSearch`

# Why the search is instrumented at all

Because otherwise the fix to the one-click bug is **unfalsifiable from
outside**. Every existing caller asks a yes/no question — *is the command
reachable* — and on a correct build the answer is `Some` whether the helper
scrolled once or five times. A check written against that answer alone would
pass just as happily on the broken build for any command that happens to sit
one stop past the fold, which is most of them.

`scrolls` is the number the assertion wants: a run that found the item after
**two or more** stops is a run the single-click implementation could not have
completed. See `crate::checks::band_scroll`.

### `fn search_the_band`

The whole search lives here and the plain form is a one-line wrapper, so
there is exactly one implementation and the instrumented answer describes
the code every caller runs. Two implementations would be two behaviours the
day one of them was edited.

# Errors

If the trace cannot be read, or a click cannot be delivered.

### `const ENABLEMENT_EVENT`

**Until 2026-09-14 this harness could not measure greying at all**, and
the gap had a shape: `ui_rect` publishes a rectangle for every control,
enabled or not, deliberately, because the consumer's question is *where is
this control* and a greyed control is still drawn somewhere. So a check
could prove the five Font controls were on the band and could not prove that
any one of them could be pressed. `font_group` said so in its own header and
then wrote the word *"greyed"* into a note it had not measured.

That is the exact sentence `OPERATOR_REQUESTS.md` O198 claim 3 makes —
*"get the font selector and editing tools like [bold] and italic working.
That entire area is always greyed out in the menu"* — reported against a
build in which every published condition passes and every region is present.
Nothing this harness could read distinguished that report from a healthy
frame, so the reply to it could only be an opinion.

### `struct Enablement`

**Two predicates, because one renderer has two.** `enabled` is the
registered command's own `enabled_when` evaluated against the published
conditions — the thing every ordinary control on the band is greyed by.
`live` is present only when the renderer applies a SECOND test of its own
before calling `add_enabled_ui`, which `pdfcer-gui`'s font band does: it
re-reads the document to resolve the selection's actual face, and greys on
whether that read-back produced anything.

So `enabled: true, live: Some(false)` is a control greyed while every
condition says it should not be. That state has a name here,
[`Enablement::disagrees`], because a check that merely asserts *"pressable"*
reports it identically to an honest refusal — and the two want opposite
fixes. See [`Enablement::pressable`] for the one a check usually wants.

### `fn pressable`

A renderer with no second test is `live: None`, which must read as
*"nothing further to satisfy"* rather than as *"not live"*: the two
ordinary command controls in the Font group have no read-back and a
harness treating their missing field as `false` would report the whole
group dead on a frame in which half of it works.

### `fn disagrees`

The interesting failure, and the one a bare `pressable()` assertion
hides. An `enabled=0` control is a surface honestly reporting that its
precondition is unmet, which R9 requires it to explain on hover. An
`enabled=1 live=0` control is a surface saying one thing in its
conditions and another in its pixels, and the operator can only see the
pixels.

### `fn enablement`

**Both prefixes, merged by physical line number.** The two crates write
to the same stderr under different markers ([`SHELL_TRACE_PREFIX`] and the
application's own), and [`Trace::parse`] filters on one. A check that asked
only the shell would see two of the Font group's five controls and miss the
three custom ones; a check that asked only the application would see the
other three. Either answer is worse than none, because a partial group reads
exactly like a measured group.

Merged on [`crate::trace::TraceLine::lineno`], which is the line's
position in the shared file, so "last" means last **in the run** rather than
last within whichever prefix happened to be read second. The event is
emitted on change, so the last line is the current state — with the change
log's standing weakness that a control which stopped being drawn leaves its
final answer standing. A check that cares must also assert the region.

### `fn fill_of`

`None` when the region resolved to no pixels, which means the application
declared it outside its own client area. That is a finding rather than a
measurement and the caller reports it as one.

### `const MIN_PRESSED_DELTA`

The derivation is [`crate::checks::markup_rectangle`]'s `MIN_PRESSED_DELTA`
and is not restated here, because restating it would create two accounts of
one measurement that can drift apart. In summary, and only as a pointer
into that argument:

* `egui-shell`'s `quiet` preset is what the built binary paints with:
  `app::frame` calls `theme.apply(&ctx)` every frame, from the operator's
  own settings. It separates unpressed from pressed by **209** now that
  `visuals.selection` carries the accent, and by **39** before that;
* `egui`'s stock light palette — which the binary does *not* install —
  separates unpressed `#E5E5E5` from pressed `#90D1FF` by **85**;
* two identically filled controls in a lossless BGRA capture differ by
  **0**, not by a small number.

Twelve sits above zero and a factor of three below **39**, the smallest of
the three real differences, so the verdict is the same whichever palette is
in force.

A channel difference rather than a contrast ratio, because both pairs are
near-equal in luminance (about 1.5:1 and 1.3:1) and would therefore be
called *identical* by [`crate::pixels::AA_LARGE`]. Contrast answers "can
this be read"; the question here is "is this a different colour".

### `fn click_mode_segment`

The move both new checks make repeatedly, with the counting that makes it
honest folded in.

# Why the count rather than "is there a line for this mode?"

Because a run switches modes more than once, and a check that asked
"did the shell ever report `mode=read`?" would be satisfied by a click it
made a minute ago. The event is emitted on every segment click — including
a click on the already-selected segment — so the number of them is the only
thing that distinguishes *this* click from the previous one.

# Why a failure here is a SKIP and not a FAIL

Same reason [`crate::checks::find_bar`]'s chord control exists: a check
that could not deliver a click has learned nothing about the application,
and naming a feature as the culprit when nothing was ever clicked at it is
worse than no check at all. The two readings — pointer injection is not
reaching this window, or the shell diagnostic switch did not reach the
process — are both stated, and this function declines to choose between
them.

# Errors

* the application declared no rect for the segment, so there is nothing to
  aim at (the reason lists the segments it *did* declare);
* the segment was declared at no usable size;
* the pointer could not be driven;
* the shell traced no new [`MODE_EVENT`] for this mode after the click.

### `fn arm_select_from_ribbon`

# Why this exists: a keystroke is not a harness primitive with a raised
panel on screen



| attempt | result |
|---|---|
| `V` | **never arrived** — no invocation traced at all |
| one Escape | arrived sometimes |
| five Escapes, polling for the region | attempt 1, or not in five |

⇒ **A chord is routed through whatever holds keyboard focus**, and that
check raises a dock panel by construction. The failure's shape is the
dangerous part: `V` produced *no line anywhere*, so the check reported the
Tool panel as drawing the wrong block when nothing had ever reached the
application. A harness primitive that can fail silently will eventually be
believed.

A click does not depend on focus, it is this suite's most exercised
primitive, and it has an oracle — the shell writes
`ribbon-command-invoked id=view.tool_select`.

# Why `view.tool_select` specifically, and not its neighbours

`app::dispatch`'s arm calls `canvas::tool::arm::select`, a plain write into
tool memory. Its two neighbours on the same band — `view.tool_hand` and
`view.tool_text` — are **toggles** (`toggle_hand`, and its text twin), so a
second press of either flips back. This is the one control on that row that
cannot be wrong about its own state, which is what makes the step
deterministic rather than merely more reliable.

# Why it returns `bool` rather than failing

Because what an unavailable route *means* is the caller's to say, and the
two callers disagree. A check whose subject sits inside a raised panel must
SKIP — falling back to the chord there would restore exactly the flake this
removes, silently. A check that merely needs the pen down, and that has
been passing on `V` for weeks, should press `V` and say so. Neither verdict
belongs here; both messages do belong in their own check.

`false` means *the pointer route was not available or did not land* — the
View tab is missing, the control is on none of the band, a collapsed group
or the overflow, or the click produced no invoke. Each of those writes a
note before returning, so a caller's own message never has to guess which.

# Errors

If the trace cannot be read or the pointer cannot be driven. Note that a
*missing* control is not an error — it is `Ok(false)`.

### `const PRESS_TRIES`

Four rather than two, and the number comes from a measurement rather than
from taste: `scale_switch`'s header records a bare `V` **arriving zero times
in six runs** with a dock panel raised. A non-arrival on this machine is not
a rare coincidence to be papered over with one retry — it is a routine
outcome of a window-manager transition landing between the raise and the
key — so a loop that gives up at two would report "not delivered" on runs
where a third press would have landed, and the suite would learn nothing on
those runs either.

It is bounded, and small, because a press is not free of consequence: see
this function's contract about repeatability below.

### `fn press_until_traced`

Returns `true` as soon as any line named in `evidence` appears that was not
there before the first press, and `false` after [`PRESS_TRIES`] presses with
no new line.

# THE RULE THIS ENCODES

> **Nothing measured after a press is evidence about the program until the
> press is shown to have arrived.**

`Driver::press` answers `Ok(())` when the **keystroke was sent**. It refuses
with no target window and it raises the target first — both real guards, and
neither of them is the statement *"the application processed that key"*.
Between the two lie a foreground transition, an egui frame boundary, and, on
this machine, a measured failure rate that is not small: `scale_switch`'s
header records a bare `V` arriving **zero times in six** runs with a dock
panel raised.

A check that presses once, measures nothing, and reports a defect has
reported a defect about a program it never spoke to. That is strictly worse
than reporting nothing, because it is a confident accusation naming a
specific line — and this suite has now produced one of those (`annot_delete_gate`
phase D on 2026-08-29 said *"the keystroke did not reach `canvas::keys` at
all"* about a keystroke whose effect was four lines further up the same
trace). ⇒ A press that cannot be shown to have landed is a **SKIP**.

# The caller owes two things, and both are contracts rather than advice

1. **`evidence` must name every line the key could produce**, including the
   ones that mean the program is wrong. A list containing only the
   good outcome turns a broken build into "the key did not arrive", which is
   the same false negative in a new place. `annot_delete_gate` lists three:
   the decline it wants, the funnel line that means the gate was walked past,
   and the funnel's refusal line that means it was walked past and the engine
   caught it.
2. **The key must be safe to press more than once.** Every press after the
   one that lands is suppressed — the loop returns immediately — but presses
   before it are real and may repeat. Delete on a document that refuses every
   delete is safe by construction; Delete on a document that performs them is
   not, and such a caller must press once and take the SKIP.

# Relation to `read_mode_chrome::press_until_invoked`

That is this function's **click** half — same rule, same shape, and it
additionally re-reads the control's rect between attempts because a click
needs a target and a ribbon relays itself out when the window resizes. A key
has no rect, so there is nothing to re-read and the two cannot share a body
without one of them carrying a parameter it ignores. They are kept as a pair
deliberately, and this module's own rule applies to folding them: *"a third
copy is the point at which folding becomes worth doing on its own rather
than in the change that happens to need it."*
