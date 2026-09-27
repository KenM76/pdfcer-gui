# `pdfcer-gui-base/prefs/offpage`

**Whether the canvas grows to show what sits off the sheet — remembered
separately for each ribbon mode.**

This module owns one preference and the whole of its reasoning: the type,
the per-mode default, the file keys, the parser and the writer. The parser
and the writer are two spellings of one vocabulary and must not live in
different files, which is why this group is a module rather than another
family of arms in `prefs::file`. [`printing`](super::printing) is grouped on
the same rule.

# What the preference is about

A CAD export frequently carries marks outside its own `/MediaBox`. When
pdfcer is allowed to show them, the canvas grows a band of pasteboard wide
enough to hold them, and that band is not free: it lengthens every scroll,
and in a continuous layout it opens a visible grey gap between one sheet
and the next exactly where the off-sheet material lives.

So the operator's answer is not *"can pdfcer do this"* — it always can —
but *"is it worth the band right now"*, and that depends entirely on what
they are doing. Reading a drawing, it is not: the page is the subject and
the band is in the way. Reviewing or editing one, it is: a title block
dragged off the sheet is a defect you must be able to see and grab.

# Why the answer is stored per MODE rather than once


> *"by default, read doesn't show off page items, review and edit do show
> off page items. these settings can be changed by the user and their
> preference is remembered for each read review edit modes."*

That is three preferences wearing one name, and the reason it has to be is
that **the modes disagree about what the document is for**. A single global
flag would force the operator to re-answer the question every time they
switched modes, which is the same defect as not remembering it at all —
worse, because it would look like the toggle was forgetting.

The consequence worth stating plainly: **switching mode can change the page
layout**, because leaving Edit for Read removes the band. That is intended.
It is also why the re-seed happens at exactly one place (the mode-change
site in `crate::app::surfaces`), so the layout can never be one mode's
answer while the ribbon shows another's.

# What is NOT stored here

Nothing about the document. Two open drawings can disagree about off-page
display, because the flag lives in [`crate::viewer::ViewState`] alongside
zoom and the overlays; this module holds only the *opening* answer and the
operator's last word per mode. A per-document memory would be a different
feature and would fight this one — see the mode-change note above.

## Item notes

### `fn read_ships_off_and_the_working_modes_ship_on`

Read off, Review and Edit on, asserted against the *ids the manifest
uses* rather than against display names, because those ids are what
reaches [`OffPagePrefs::default_for_mode`] at runtime.

### `fn choosing_the_default_is_still_an_answer`

See [`OffPagePrefs::set`]: *chose the default* and *never answered* are
the same value and different facts, and only the stored one survives a
later build changing its mind.

### `struct OffPagePrefs`

# Why a map of the answers GIVEN rather than three `bool` fields

Three fields would be smaller, and would also be a closed set: a manifest
with a fourth mode could not be remembered, and — more importantly — three
fields cannot distinguish *"the operator chose `false` for Read"* from
*"nobody has said anything about Read yet"*. Those are the same value and
different facts. Keeping only the answers actually given means:

* a fresh profile writes **no** `off_page.*` lines at all, so the file does
  not fill up with keys nobody set;
* [`default_for_mode`] remains the single source of the shipped behaviour,
  free to change in a later build without a stale copy sitting in every
  operator's preferences file contradicting it;
* the round trip is exact — an empty map writes nothing and reads back
  empty, which is what `every_preference_round_trips_through_the_file`
  requires of every member of [`Prefs`](super::Prefs).

[`BTreeMap`] and not [`HashMap`](std::collections::HashMap) for the writer's
sake: the file must be byte-stable across runs or a diff of two preference
files is unreadable noise.

### `fn default_for_mode`

**Read is off; everything else is on**, including a mode this build has
never heard of. The fallback direction is chosen so that an unfamiliar
mode behaves like Edit rather than like Read: showing material that is
there costs a band of pasteboard, whereas hiding it costs the operator
the knowledge that it exists at all. Between a cosmetic cost and a
silent omission, take the cosmetic one — the same posture
`PageDisplay::default_for_mode` takes on its own unknown mode.

### `fn set`

The answer is stored **even when it equals the shipped default**, and
that is not redundancy. An operator who turns off-page display off in
Review has expressed an intent about Review; if a later build changed
its mind about Review's default, the operator's own answer must win
over the new default rather than be indistinguishable from it.

### `fn parse_key`

Returns [`KeyOutcome`] — borrowed from the print group rather than
re-declared, because the caller's dispatch chain needs one vocabulary and a
second three-variant enum meaning the same three things is how the two come
to disagree about what `NotMine` obliges the caller to do.

`value` arrives already trimmed, as `prefs::file` trims both halves before
it dispatches.

⚠ **An empty mode (`off_page. = true`) is `BadValue`, not `NotMine`.** It is
unmistakably one of ours — it carries the prefix — and reporting it as an
unknown key would tell the operator to check the spelling of a key they
spelled correctly, which is the exact confusion [`KeyOutcome`]'s three
variants exist to prevent.

### `fn write_block`

Called once by `Prefs::write_to_string`. The comment is written **always**,
even when no answer has been given, because the file is meant to be opened
in a text editor and a preference nobody can discover is a preference
nobody has. The key lines below it are written only for answers the
operator actually gave — see [`OffPagePrefs`] on why.

### `fn remember`

Called from the single `Action::ToggleViewChrome` arm in
[`crate::app::actions`], for *every* chrome toggle, and it returns
immediately for the [`ViewChrome`] variants that have no memory. That shape
is deliberate, because the obvious alternative — an `if chrome == OffPage` at
the call site — looks tidier and is worse: **the knowledge stays in one
module.** That off-page is the toggle with a remembered answer, that the
answer is keyed by mode, and that a mode-less shell has nothing to key it by
are all facts about *this* preference. A caller that had to know the first of
them would be a second place to update when a second toggle grows a memory.

# What it writes, and why immediately

One `prefs.save()` per click, exactly as `view.smart_select` and the
find-zoom toggle do, and for the reason stated there: **one discrete
operator decision is one write**. Deferring to shutdown would lose the
answer to a crash or a power cut, and the operator's request was that the
preference be *remembered* — a memory that survives only an orderly exit is
not what anybody means by that.

A failed write is swallowed. Preferences are a convenience and a modal in
front of somebody who just flipped a view switch would be a worse defect
than the lost line; the same judgement every other `prefs.save()` call in
this crate makes.

# `mode` is an [`Option`] because the shell's is

[`crate::shell::ribbon`] answers `None` before a manifest has been applied.
There is no sensible key for "no mode", and inventing one (`""`, or
defaulting to `read`) would write an answer against a mode the operator was
never in — which the next real mode would then inherit. Doing nothing is
the honest response: the toggle still works for the session, it simply has
nowhere to be remembered.

### `fn apply_mode`

Called from the one place a ribbon mode change is observed
(`crate::app::surfaces`, beside `on_mode_capabilities_changed`). One site,
stated as a requirement rather than an accident: a second would let the
canvas show one mode's answer while the ribbon shows another's, and that
state is indistinguishable from the toggle being broken.

# The consequence, stated plainly because it is intended

**Leaving Edit for Read can change the page layout.** Read's shipped answer
is off, so the band of pasteboard that held off-sheet material goes, and
with it the gap it opened between one sheet and the next. That is the
operator's request — *"when not showing the stuff that is off page there
shouldn't be a gap between pages where the stuff is"*. It is not a surprise
to be softened; a mode change is a deliberate gesture and this is the thing
it was asked to do.

# Why EVERY document and not just the active one

The answer is a property of the mode, not of the document. A parked tab
that kept the outgoing mode's layout would spring to the incoming mode's
the instant it was activated, with no gesture in between to explain it —
the operator would have watched a document rearrange itself for nothing.
Writing them all now costs one bool per open file and makes activation
inert, which is what the operator already believes it is.

`parked` is taken as a slice rather than the app, so this function cannot
reach anything else and the caller's three field borrows stay disjoint.
