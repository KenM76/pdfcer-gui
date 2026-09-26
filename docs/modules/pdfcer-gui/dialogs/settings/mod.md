# `dialogs::settings` — where a spec ambiguity becomes a choice

## What this surface is for


> Where standards are ambiguous those should become settings that the user
> can choose direction one, with the initial installed default as the best
> guess of what is usually followed.

`pdfcer_core::settings` is the store that makes such a choice survive a
restart. This module is the only place an operator can *make* one without
opening a text editor — and the no-affordance-without-capability rule cuts
both ways here: **a setting the program honours but offers nowhere is just
as much a gap as a control that does nothing.**

The engine honours its settings whether or not anything offers them, which
is how a program reaches the state the operator reported: features added
with *"no surface for changing or editing the settings for them."*

## Why a window and not a dock panel

The dock's own test is **"selection state is watched, workflows are
entered."** Watched things — the page rail, the object tree, the armed
tool's options — earn a permanent compartment because they are consulted
continuously *while doing something else*. Settings are the opposite:
consulted in bursts, deliberately, when there is something to change.
Properties is out of the dock for precisely this reason; putting Settings
*in* would be the same mistake with a different noun.

So it is a window, opened from the File tab's pdfcer group, in the shape an
operator arriving from Office or PDF-XChange already expects of *File →
Options*.

## Groups, and why a heading is navigation rather than tidiness

**An operator opens this window with a *symptom*** — "my black lines look
grey", "copied text has no spaces", "my dimension came out as an angle" —
and the group headings are how a symptom finds its setting. A flat list
makes the reader scan every row.

Which means a setting filed under the wrong heading is not untidy, it is
**unreachable**. `parallel_epsilon_degrees` governs whether two lines are
dimensioned as a distance or an angle, so it belongs in [`measuring`] and
not under *Copying and extracting text* — which is where being a slider
like the word-gap one would put it.

**The group list and its order live in [`show`]'s body, which says so and
is the contract.** They are deliberately not restated here: a second copy
of an order that changes whenever a setting is added is a copy that goes
quietly wrong, and a wrong map is worse than no map.

### Colour is the one group that starts expanded

It holds the setting most likely to have brought someone here — and the
only one whose default knowingly differs from other PDF viewers, which is
the "my black lines look grey" symptom. Every other group starts collapsed.

## The three obligations, enforced by a function signature

A settings screen that listed keys and radio buttons would satisfy nobody
here. Three things must be visible that a conventional one omits:

1. **What the default rests on.** Most of these defaults are *reasoned
   inference* — a guess — and a guess must say it is a guess. Exactly one
   is well-sourced and says that too.
2. **That a choice was made at all.** These are settings *because the
   standard declines to have an opinion*. An operator who does not know
   that reads a difference between pdfcer and Acrobat as a pdfcer bug.
3. **Which way costs what.** A setting whose blast radius is the SAVED
   BYTES is a different kind of decision from one that only changes the
   preview.

Obligations 2 and 3 are not left to discipline: [`widgets::header`] takes
`title`, `silence` and `radius` as **required arguments**, so a setting
cannot be added without answering all three. Obligation 1 lives in the
option notes and is pinned by tests in [`crate::text::settings`].

## Cancel is real

The window edits a **working copy**. Nothing reaches the live configuration
or the disk until *Save*, and *Cancel* discards the lot. This is not
ceremony: several of these settings change **saved bytes**, so a radio click
that took effect immediately would be an edit the operator never intended
and cannot see.

**Theme is the single exception**, and it is deliberate — see [`Draft`].

## Item notes

### `const SOURCES`

HAND-WRITTEN, AND THEREFORE AUDITED — see
[`every_source_in_this_directory_is_listed`].

A file missing from this list is not a gap in coverage, it is a **false
report of a gap**: the completeness test below searches only what is
listed here, so a control written in an unlisted file reads as a setting
with no control at all. A check that cannot see a file reports its
contents as absent, which is indistinguishable from the defect it exists
to find — and sends the next session to write a control that already
exists.

### `fn every_source_in_this_directory_is_listed`

Unlike the catalog's own version of this guard, there is **no
exclusion list**: this search is for *"is there a control bound to this
setting anywhere in the window"*, and a control could legitimately be
written in any file here — including `widgets.rs`, if a helper ever
bound a field directly. Every file counts, so every file is listed.

The failure this prevents is the quiet one. A missing file does not
make the completeness test fail loudly; it makes it **report the
missing file's settings as uncontrolled**, sending the next session to
write a control that already exists a few lines away.

### `fn every_setting_the_store_carries_has_a_control_in_this_window`

`NO_SURFACE.md` is a kept inventory — it says what is missing when
somebody looks. This is a build failure, and it fires when somebody
*adds*. They are not substitutes, and without this one the engine can
gain a setting that reaches no operator and nothing says so.

# Where the list of settings comes from, and why it is not the struct

From [`Settings::write_to_string`] — the engine's own settings-file
writer, at **runtime**, against the shipped default. Every setting the
store round-trips appears there as a `key = value` line, because that is
what the file is; a setting missing from it could not be persisted at
all, which is a different and larger defect the engine's own tests own.

Reading the store's **source** instead would need a relative path into
another crate's tree. `pdfcer-core` is a **git dependency** here and its
source is on no path this crate can name — and the runtime reading is
the stronger instrument anyway, not merely the available one:

* it reads the **compiled dependency**, so it is answering about the
  engine this build actually links, not about a file on disk that may be
  from a different revision;
* it cannot be fooled by a field that is `pub` and not persisted, or by
  one persisted under a key that differs from its field name;
* it needs no path into another repository, which is what makes it
  survive the fold-in in either direction.

# The crude half, kept crude deliberately

Coverage is asserted by reading **this directory's own source** and
looking for `working.<key>`. That is a text search and it can be
defeated — by a control that names the field in a comment and never
binds it, say. It is kept anyway, because the alternative is a
hand-maintained list of which settings have controls, and a
hand-maintained list is exactly the thing that goes stale silently. A
crude check that fails when a setting is added beats an exact one that
nobody updates.

**When this fails, the fix is a control, not an edit to this test.** The
failure message says which setting, and the group it belongs in is
decided by the symptom that brings an operator looking for it — see this
module's header.

### `fn the_sweep_is_scoped_to_the_engines_store_on_purpose`

[`crate::app::prefs::Prefs`] lives in a different file, is written by the
shell rather than by the engine, and reaches the window through
`draft.working_prefs` rather than `draft.working`. The sweep is
deliberately scoped to the ENGINE's store, because that is where the
asymmetry it exists to catch comes from: `pdfcer-core` gains settings on
its own schedule and this project finds out by reading a note.

A preference added to `Prefs` is added by this project, in the same
session that would add its control, so the failure mode is not the same
one. If that ever stops being true — if the shell's preferences start
arriving from elsewhere — this is the test to widen.

### `fn changing_a_value_back_makes_it_clean_again`

A radio click and a click back is not an edit, and a Save button that
stayed live afterwards would be telling the operator they have unsaved
changes when they have none — which is the same lie as a Save button
that is always live, arrived at by a different route.

### `fn a_draft_started_from_non_default_settings_is_clean_but_not_all_default`

A draft opened from non-default settings is **clean** (nothing has been
edited) and **not all-default** (something is not pdfcer's answer).
Collapsing them would grey *Restore defaults* for the operator who
changed something last week and wants it back — which is most of the
people who will ever press it.

### `mod signatures`

The two settings behind `ENGINE_BACKLOG.md`'s trust rows. Its header carries
why it is NOT filed under *Where Acrobat is* — the symptom that brings
somebody here is *"the Signatures panel says it did not check who signed
this"*, and nobody carrying that symptom looks under a group about which
program a button starts — and why the inspect control is absent when there
is no store while the path field stays visible.

### `const REGION_BODY`

**This is what aims `ui-verify`'s `settings_headings_legible`**, the
regression test for `DEFECTS.md` D2 — a collapsible heading rendering
near-white on light grey, at around **1.1:1** against a 3:1 floor. The
check measures the running program rather than a screenshot, so it finds
the window by this name. Renaming this constant un-aims it.

### `const REGION_CANCEL`

The two DO agree — `app::settings_window` documents both as dropping the
draft — but a check that can only reach one of them proves nothing about the
other, and the whole point of the check that wanted this is that the
coupling it guards fails **silently**.

### `const REGION_HEADING_PREFIX`

One per collapsible header, so the contrast check can measure **each**
heading against its own background rather than sampling the window and
hoping. D2's defect was a foreground/background *pairing*, and a pairing
only exists once something is drawn — so the check needs the rectangle the
application actually laid the text into, not a rectangle derived from a
palette.

### `const REGION_THEME_PREFIX`

Exists so a check can **click** one. Proving that a theme picker is on
screen is not the property anybody cares about; the property is that
choosing Dark makes the window dark, which needs a rect to aim at and a
capture afterwards. See `DEFECTS.md` D10.

### `struct Draft`

# Theme is the one setting that breaks the draft contract, on purpose

Every other setting here is draft-until-Save. A theme cannot be judged from
a radio label — you choose it by *seeing* it — so the selection takes effect
on the next frame. The draft still governs what is **saved**; it just no
longer governs what is **shown**.

The mechanism is one line in the application's per-frame `ui()`, before any
widget is built: the theme token is read from the draft when a draft exists
and from the live settings otherwise. Cancel drops the draft, so the look
reverts with it — no separate undo path, and nothing that can get out of
step. The window says so in the theme setting's own radius line rather than
leaving it to be discovered.

### `fn new`

Live rather than a re-read of the file, and the difference matters in
exactly one case: if a previous save failed, the session is honouring a
choice the disk does not have. The window must show what pdfcer is
actually doing, not what it wished it had written.

### `fn is_dirty`

Drives whether *Save* is offered at all. A Save button that is always
live cannot tell the operator whether they have unsaved changes, and
this is a window someone may open just to read.

**Not latched.** Click a radio and click it back, and the draft is clean
again — because it is, and a dirty flag that only ever went one way
would make Save mean "you visited this window".

### `fn is_all_default`

# Why this is not the same question as [`Self::is_dirty`]

A draft opened from non-default settings is **clean but not
all-default**: loading is not editing. Collapsing the two predicates
would disable *Restore defaults* for exactly the operator who most needs
it — the one who changed something in a previous session and wants it
back.

### `enum Outcome`

Returned rather than performed. This module renders and does not own
application state — the split every other dialog and panel in this shell
uses — and the three verbs have consequences (adopting a configuration,
writing a file, invalidating every cached raster) that belong in the
dispatcher where they can be seen together.

### `fn show`

# Geometry, and why every number has a reason

| aspect | value | why |
|---|---|---|
| screen source | `content_rect` | not `viewport_rect`: it subtracts safe-area insets, so centring uses *usable* space |
| width | `620` clamped to `[420, screen − 40]` | wide enough for a full sentence at the body's text size |
| height | `82 %` of screen, clamped `[420, 900]` | a fixed height leaves half the screen unused *while still scrolling* — the worst combination |
| position | `default_pos`, centred, `−20` vertically | see below |

**`default_pos` rather than `anchor`**, so the operator can drag it aside.
A window pinned in the middle of the screen is a window in the way of the
document it is about. And egui's own default position put it top-left, over
the quick-access toolbar and the ribbon tabs — so *opening Settings hid the
control that opened it*. The `−20` keeps the button row on a short screen.

# The scroll area's height is computed, not fixed

`available − 96`, floored at 180. A fixed height clips the last group's
heading in half on a short screen, and a half-drawn heading reads as a
rendering fault rather than as a group. The reserved 96 is the intro, the
store line, two separators and the button row — everything that is not the
list.
