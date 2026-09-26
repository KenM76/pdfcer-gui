# `pdfcer-gui/dialogs/open`

**How each dialog is BUILT** — every `DialogsState::open_*` constructor, and
the two guards each of them applies before a window can exist.

# 1. Why this file exists, and what the seam actually is


| family | job | where it lives now |
|---|---|---|
| `open_*` / `deliver_*` | **build** a dialog from defaults, a picked path, or a document survey, and decide whether it may exist at all -- and hand a canvas gesture's answer back into one that is already open | **this file** |
| `ask_*` / `take_*_answer` / `show` | carry a **question** to the operator and its **answer** back to `PdfcerApp`, and drive the per-frame draw-and-drain loop | `dialogs/mod.rs` |

That is not a mechanical halving. The two families have different callers
(dispatch for the first, the frame loop and the save funnel for the second),
different failure modes, and — most usefully — **different invariants**,
which are stated once here rather than repeated at twenty-one sites.

# 2. The two guards every opener applies, and why they live HERE

The shell's dispatch pattern is *"push the chord blind, gate the effect in
dispatch"*. A ribbon control registered `enabled_when("doc.open")` cannot be
pressed without a document and cannot be pressed twice in one frame; **a
keyboard chord bound to the same command id has neither property.** So both
conditions are enforced at the one place a dialog is ever constructed:

- **No document, no dialog.** Otherwise the chord on an empty canvas builds
  a window that [`super::DialogsState::show`] closes again on its very next
  frame — and some of them do real work on the way, such as enumerating the
  spooler over the network for Print.
- **Already open means leave it alone.** These functions build from
  *defaults*. A second press part-way through configuring a job would
  silently discard the range, the scale, the copy count, the annotation
  scope — the operator's own settings, thrown away by the very shortcut
  pressed to look at them.

Enforcing both here fixes the button and the chord **by construction**,
rather than by a condition duplicated at the keymap that can drift from the
one in dispatch. Each function's own doc comment says which of the two it
applies and, where it applies only one, why the other does not arise.

# 3. What this file must NOT grow into

An opener decides **whether** a dialog may exist and gathers **what it needs
to exist**. It does not decide what the dialog does, does not write to the
document, and does not answer its own question. When an opener starts to
need more than a survey of `Status`, that is the signal that the reasoning
belongs in the dialog module itself — the same rule `app/actions/OVERVIEW.md`
sets for action arms, and for the same reason: a constructor that knows the
semantics of the thing it constructs is a second place for those semantics
to live, and two places drift.

## Item notes

### `fn open_print`

**The dispatch target for the `file.print` command.** The command is
registered `enabled_when("doc.open")`, so the ribbon button cannot be
pressed without a document — but a keyboard chord bound to the same id
has neither that guard nor the button's once-per-frame property, and
the shell's own dispatch pattern is *"push the chord blind, gate the
effect in dispatch"*. Both conditions are therefore enforced **here**,
at the one place the dialog is ever built, which fixes the button and
the chord by construction rather than by a condition duplicated at the
keymap:

- **No document, no dialog.** Without this, the chord on an empty
  canvas would enumerate the spooler — a blocking call on a network
  printer — to populate a window [`Self::show`] closes again on its
  very next frame.
- **Already open means leave it alone.** This function *builds* a
  dialog from defaults. A second press part-way through configuring a
  job would silently reset the range, the scale, the copy count and the
  annotation scope — the operator's own settings, discarded by the
  shortcut they pressed to look at them.

# `remembered` — O166

The operator's last-used print settings, read from the preferences file
and handed to the constructor. See
[`crate::app::prefs::PrintPrefs`] for which of the Print window's
controls are in there and, more importantly, which are deliberately
not: the already-open guard above and this parameter answer two halves
of the same complaint, and it is worth seeing them together. The guard
stops a second press *discarding* a job you are half-way through
configuring; this stops the *first* press asking you four questions you
have already answered a hundred times.

⚠ `pub(crate)` where its siblings here are `pub`, and it is the
`remembered` parameter that demands it: `PrintPrefs` carries the print
dialog's own crate-private spooler enums, so a `pub` function naming it
trips `private_interfaces`. Nothing outside this crate calls it — the
only caller is the `file.print` arm of `app::dispatch`.

### `fn open_ocr`

**The dispatch target for the `file.ocr` command**, and it applies the
same two guards [`Self::open_print`] documents, for the same two
reasons: the ribbon control is gated on `doc.pages` and a chord bound to
the same id is not, so both are fixed here at the one place the dialog
is built.

The already-open guard is the stronger of the two here. A second press
while a recognition is running would abandon a live worker thread and
start another beside it, and a second press *after* one finished would
discard recognised bytes the operator has not saved yet — several
seconds of work and an unwritten document, thrown away by the shortcut
they pressed to look at it.

### `fn open_redact`

**The dispatch target for the `edit.redact_apply` command**, and it
applies the same two guards [`Self::open_print`] documents — the ribbon
control is gated on `doc.pages` and a chord bound to the same id is not.

Both guards are load-bearing here in a way they are not elsewhere,
because [`redact::RedactDialog::open`] **runs the whole removal**.

- **No document, no dialog.** Without this, an invocation over an empty
  shell would build a window that [`Self::show`] closes again on its very
  next frame — a control that visibly flickers rather than one that
  declines.
- **Already open means leave it alone**, and this is the strong one. A
  second press would re-run a full rewrite of the document *and* discard
  the operator's two acknowledgements — throwing away the reading they
  have just done on the one report in this program that has to be read.
  Worse, it would silently replace a report computed against the marks as
  they were with one computed against the marks as they are now, which is
  the difference between the numbers on screen and the bytes that would
  be written.

### `fn open_offpage`

The dispatch target for the command, and it is the one window in this
file that opens **unconditionally** over any open document. Its
neighbours all compute something first and decline when the answer is
empty; this one must not, because *"nothing is drawn outside any page
boundary in this document"* is the answer the operator pressed the
control to get — see `crate::dialogs::offpage`'s header.

The no-document guard is still real: there is nothing to scan, and a
window over no document would be closed again by [`Self::show`]'s own
guard on its very next frame, which is a control that flickers rather
than one that declines.

The already-open guard is the strong one here, for a reason none of
its neighbours have: this window is **mid-walk**. A second press would
throw away a scan that may be twenty sheets in — each of which cost up
to half a second — and restart it from page one, while looking to the
operator exactly like a window that had reset itself for no reason.

### `fn open_protect`

Both guards [`Self::open_print`] documents are real, and the already-open
one is the strong one: rebuilding would silently discard four password
boxes and a permission list on a second press of a double-clicked button.
It also declines a press of the *other* control while the window is up,
which is right — switching the task under a half-filled form would change
which job is about to be done without changing what is in the boxes.

### `fn open_sign`

The already-open guard is the strongest one on this file, and it is
not about losing typed text. The window holds a **loaded private key**
once the operator has opened their certificate; rebuilding it on a
second press would discard that and make them type the passphrase again
— and, worse, an operator who pressed twice would be looking at a form
with an empty passphrase box and no identity, which reads exactly like
the certificate having been rejected.

The no-document guard lives in [`super::sign::open_for`], which returns
`None` rather than building a window over nothing.

### `fn sign_outcome`

Called by [`crate::app::actions::sign::apply`]. Silently ignored when
the window has been closed in the meantime, which is a real sequence: a
signature can take a second on a large document, and an operator who
closed the window is not waiting for an answer. The file was still
written and `sign-written` is still on the trace, so nothing is lost —
only the sentence about it.

### `fn open_scale`

The already-open guard is the same one every dialog here has, and it
matters more than usual: a second press must not discard a ratio the
operator has half typed, and re-opening would also re-capture the active
group — so a group change made while the dialog was up would silently
redirect the calibration.

**It destructures the document rather than testing for one** — O192.
This read `if !matches!(status, Status::Open(_))`, throwing away the
`OpenDoc` it had just proved it had, which is the mechanical reason the
Set-scale window was the only surface in the application that could not
see the number it was about to change. The guard is unchanged in
meaning; it now keeps what it checked.

### `fn open_scale_calibrated`

The calibration path's fallback entry point: a two-point pick completed
with no Set-scale window open to hand the answer to.

# It REPLACES an open dialog, where [`Self::open_scale`] refuses to

That guard exists so a second press of the ribbon control does not
discard what the operator has half typed. The situations are opposite
here: the operator asked to measure on the drawing, and they have now
finished. A guard that refused would leave them looking at a stale
window with no measurement in it — the one outcome the whole gesture
exists to avoid.

# ⚠ This is the FALLBACK, and [`Self::deliver_scale_length`] is the road


It is kept rather than removed even though this application can no
longer reach it, because removing it would make the two-point gesture's
result depend on a window's continued existence: a pick that completed
with nowhere to land would measure the page and throw the number away.
A gesture that can silently produce nothing is worse than a redundant
constructor.

### `fn deliver_scale_length`

Answers `true` when a window was there to take it, which is the caller's
signal that the fallback above is not needed.

The window is not reopened, **because it was never closed** — the same
sentence [`DialogsState::deliver_placement`] carries, and now the second
round trip in this directory that earns it. It starts drawing again on
the frame the pick tool is disarmed, with everything the operator had
already typed still in it and the measurement added.

⚠ It delivers into the dialog's **own** group, not into the canvas's
active authoring group. Those were the same thing before O193 gave the
window a group picker, and they are not now: an operator who opened the
window, aimed it at the detail group, and then measured a line would
otherwise have calibrated whichever group the canvas happened to be
drawing into. Nothing here has to do anything to get that right, which
is the point of delivering into the window rather than rebuilding one.

### `fn open_text_annot`

Raised by `Action::BeginTextAnnot`, which the canvas pushes on the
gesture that finishes placing.

It REPLACES an open dialog rather than refusing, unlike
[`Self::open_scale`]. The situations are opposite: that guard protects a
half-typed value from a second ribbon press, and here a second placing
gesture is the operator plainly saying they want to annotate somewhere
else. Refusing would leave them looking at a window describing a box
they have moved on from.

### `fn open_form_field`

Raised by `Action::BeginFormField`, which the canvas pushes on the click
or drag-release that finishes placing, and by nothing else.

It REPLACES an open dialog rather than refusing, for the reason
[`Self::open_text_annot`] gives: a second placing gesture is the operator
plainly saying they want a control somewhere else, and refusing would
leave them looking at a window describing a rectangle they have moved on
from. What it costs is the abandoned draft, which authored nothing.

### `fn open_diagnostics`

**The dispatch target for the `tools.render_diagnostics` command**, and
it applies the same two guards [`Self::open_print`] documents, for the
same two reasons: the ribbon control is gated on `doc.open` and a chord
bound to the same id is not.

The no-document guard is the sharper of the two here. Without it a chord
on an empty canvas would build a window that [`Self::show`] closes again
on its very next frame — a control that visibly flickers rather than one
that visibly declines, which is the harder of the two to diagnose.

The already-open guard costs nothing (there is no configuration to
discard) and is kept for About's reason: rebuilding would move the
window back to the centre and the findings list back to the top, which
for an operator half-way down a census reads as the program losing their
place.

Note what it does **not** guard on: whether anything has been
rasterized. `doc.open` is the registered predicate, and a document with
no texture yet is precisely when an operator asks what the renderer did
— so the dialog opens and *says* that nothing has been drawn, rather
than the command silently doing nothing.

⚠ **This comment spent an unknown period attached to the WRONG
FUNCTION** — stacked on top of `open_scale`'s title, so `open_scale`
carried two functions' documentation and this one carried none.
`check-orphan-docs` cannot see a seam of this shape: it matches the
bold-title convention and this title is plain prose. Its header names
the discriminator that would have caught it — a real orphan implies an
undocumented item in the same file.

### `fn open_about`

**The dispatch target for the `file.about` command.** Unlike
[`Self::open_print`] it takes no [`Status`], because it needs none:
About describes the application, and the application is always there.
The command is registered with no `enabled_when` for the same reason.

The already-open guard is kept, and for a slightly different reason
than print's: this dialog holds no configuration to discard, so
rebuilding it would lose nothing — but it would *move* the window back
to the centre and reset its scroll position, which for an operator
half-way down the attribution list reads as the program losing their
place.

### `fn open_insert_pages`

**The dispatch target for `file.new_from_template`.** Like
[`Self::open_about`] it takes no [`Status`] and the command is
registered with no `enabled_when`: New is the command an empty shell
exists to offer, and gating it on a document would grey the one control
that answers *"there is nothing here"*.

The already-open guard is print's rather than About's: this window holds
a size, an orientation and two typed numbers, and a second press of the
ribbon control part-way through would silently reset all four to A4
portrait — the operator's own choices, discarded by the control they
pressed to look at them.
Open the insert dialog for `path`, having counted its pages.

**The dispatch target for `pages.insert_from_file`, after the picker.**

# Why the page count is read here and not in the dialog

Because a file that will not open must be reported **instead of** the
dialog, not after the operator has filled one in. Opening a window that
says "0 pages" and refuses its own commit button would be a surface
asking a question that cannot be answered.

The load is cheap relative to what follows — the same file is opened
again by the insert itself — and a document parsed twice is the honest
trade for a dialog that can state a fact before the operator commits.

### `fn open_import_text`

Unlike [`Self::open_insert_pages`], this does **not** read the file
to say something about it before the window opens, and the difference is
worth stating because the two windows look alike.

Insert-pages reads its source because *"how many pages does it have?"*
is a question the window must answer and cannot ask the operator — a
dialog offering *"pages 1-N"* with no N is a control with nothing in it.
Nothing this window asks depends on the file's contents: the sheet, the
margin, the face, the size and the position are all decisions about the
*output*, and every fact about the *input* — how many pages it became,
what was split, what could not be written — is knowable only by running
the import, which `place_text` does after planning and refusing with
nothing created.

⇒ So the file is read exactly once, in the apply arm.
`dialogs::import_text`'s header carries the same argument from the
window's side.

### `fn open_page_size`

The dispatch target for `pages.resize`.

# Why the operands are passed in rather than resolved here

`crate::panels::pages::ops::operands` — picked sheets, else the current
one — is stated in exactly one place, and `crate::app::dispatch::pages`
is the caller that states it, for every `pages.*` command. A second
resolution here would be a second place for a rule about *which sheets
an operation is aimed at* to drift, and the direction it drifts in is a
window that measures one set of sheets and changes another.

# It can DECLINE, and declining is not the same as doing nothing

`PageSizeDialog::open` returns `None` when the survey found no sheets.
The idempotence guard above it is the ordinary one — one window at a
time, so a second press does not replace a survey the operator is
reading with an identical one.

### `fn open_shortcuts`

**The dispatch target for `file.shortcuts`.** No document guard, unlike
every other `open_*` here: the window lists key bindings, which exist
whether or not a file is open, and refusing it on an empty canvas would
hide it from exactly the operator most likely to want it.

### `fn open_stamp_collection`

**The dispatch target for the `file.stamp_collection` command.**
[`Self::open_export_dxf`]'s two guards and for its reasons: the window
draws one row per page and seeds its category from the file, so there
is nothing to build without a document.

### `fn open_export_dxf`

**The dispatch target for the `file.export_dxf` command.** The two guards
[`Self::open_print`] documents apply, and the no-document one is real
rather than ceremonial: the window's scale suggestion is computed from
the document's dimension model at construction, so there is nothing to
build without one.


# `remembered` — O196

Threaded through rather than read from a global: the window is seeded
from [`crate::app::prefs::ExportDxfPrefs`], and taking it as an argument
is what lets a unit test construct the window in a known state. Same
shape as [`Self::open_print`], which did this first.

### `fn open_export_image`

**The dispatch target for the `file.export_image` command.** The two
guards [`Self::open_print`] documents apply, and the no-document one is
real rather than ceremonial: the window measures the largest page at
construction to promise a pixel count, and there is nothing to measure
without one.

`remembered` is O196's, and is [`Self::open_export_dxf`]'s argument
applied unchanged.

### `fn open_embed_fonts`

**The dispatch target for `tools.embed_fonts`.** The two guards
[`Self::open_print`] documents apply.

## It returns a sentence, unlike every other `open_*` here

Because this is the one command whose honest answer is often *"there is
nothing to do"* - a document whose fonts are all embedded is the normal
case, not an error - and a window that opened to say so would be a modal
the operator has to dismiss to learn they did not need it. So the
construction is allowed to decline, and the decline becomes a line the
caller records where every other outcome of a command is recorded.

`Some(String)` rather than a `bool`, for the reason
`prefs::fonts::add` returns one: the caller has two different things to
say - *"nothing is missing"* and *"you have no font folders"* - and it
cannot tell them apart from a flag.

### `fn open_compact`

**The dispatch target for `file.save_compacted`.** Returns `Some` only
for a refusal, which is the same shape [`Self::open_embed_fonts`] uses
and for the same reason: the caller records the sentence where every
other outcome of a command is recorded, and a refusal nobody surfaces is
a button that does nothing.

Unlike the two font commands, this **cannot** decline for want of
anything to do. A file with nothing to reclaim still gets the window,
which says so — an operator who asked for a copy is owed one even when it
comes out the same size.

### `fn open_unembed_fonts`

**The dispatch target for `tools.unembed_fonts`.** The same shape as
[`Self::open_embed_fonts`] and for the same reason: a document with
nothing removable is an ordinary document, not an error, and a window
saying so is a modal an operator dismisses to learn they did not need
it.

It takes no folder list. Removal needs no donor - it deletes what the
document already carries - which is the whole asymmetry between the two
commands and the reason only one of them was blocked on a preference.

### `fn open_insert_image`

**The dispatch target for the `edit.insert_image` command**, reached only
after the file has been chosen AND imported — see that arm for why the
import happens first.

The already-open guard matters here the way it matters for OCR: a second
press would discard a placement the operator has typed and replace the
imported bytes with another file's, so the window they pressed the
shortcut to look at would come back describing a different picture.
