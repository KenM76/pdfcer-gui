# `dialogs::about` — the attribution surface an operator can actually reach

The dispatch target for `file.about`, on **File ▸ pdfcer** beside Settings
and Keyboard shortcuts.

## What it is for, which is not "an About box"

Most About boxes are a version number and a logo. This one exists for a
specific obligation, and the version number is the incidental part.

`pdfcer-gui.exe` redistributes third-party work that `cargo-about` cannot
see — font faces and data tables the engine embeds with `include_bytes!`,
and, when OCR lands, a set of **CC-BY-SA-4.0** neural-network weights the
operator has decided to ship. Attribution-style licences require
the notice to reach the **recipient**, and the recipient of this program is
someone holding a binary, not someone reading a repository. A
`PROVENANCE.md` in the source tree discharges nothing for them.

The full argument, the catalog, and the sources every field was lifted
from are in [`crate::text::about`]. This module is only the drawing.

## Why this dialog is exempt from "a closed document closes the dialogs"

[`super::DialogsState::show`] drops every open dialog the moment the
document goes away, and the reason is sound: a print job configured
against pages that no longer exist is a job against nothing.

**This dialog is about the application, not about a document.** It has to
open with nothing loaded — an operator who has just launched pdfcer and
wants to know what version they are running has no document, and a control
that did nothing in that state would be the placeholder this project
forbids. So `show` has a two-branch shape: document-scoped dialogs are
closed when the document closes, and this one is drawn either way. That
distinction is a property of the module rather than of this file; see
[`super::DialogsState::show`].

## The title bar and this window answer different questions

The window title carries the local build time **to the minute**, from
`PDFCER_BUILD_TIME`. This window carries the **release version**, from the
git tag by way of `build.rs`. They are not two renderings of one value whose
precision has drifted, and **harmonising them breaks whichever one gives
way**.

The title answers *is this the build I just installed*. It shows the minute
because the operator asked for the minute — O101: *"also in the next release
add the local compilation time to the top bar at the end of the date you
added."* A date alone cannot tell two builds apart on a day with several
publishes, and the cost of that is bug reports that resolve to "you were
running an old build". See `text::doctabs::build_day` for the rule and the
zone subtlety.

This window answers *what is this program*, which is a release version, and
not a compile timestamp. See [`version_label`] for why the version comes
from the tag and never from `CARGO_PKG_VERSION`.

## Why it pushes no `Action`

[`super::DialogsState`]'s header gives the rule: the action funnel exists
for changes to *document* state, and a dialog that changes none does not
use it. This one reads three `&'static str` catalogs and a compile-time
version constant. It has nothing to undo, nothing to order against, and
nothing that could alias.

## Layout

One column, scrollable, in the order an operator reads it: what this
program is, what version, under what terms — then the third-party
material, then where the full texts are. The attribution list is a stack
of blocks rather than a table: an attribution is four sentences about one
work, and four sentences do not fit a table cell at any window width worth
having.

## Item notes

### `fn version_label`

Takes the three facts `build.rs` derives — `PDFCER_RELEASE_VERSION`,
`PDFCER_RELEASE_DISTANCE`, `PDFCER_RELEASE_MODIFIED` — and returns the one
line that is true about them. Every word it returns comes out of
[`crate::text::about`]; the only thing decided here is *which*.

# Never `CARGO_PKG_VERSION`

`Cargo.toml` is pinned at `0.1.0` by a recorded decision (O109, O110): this
crate is versioned by the pdfcer workspace it folds **into**, so its manifest
version is not a release version and never becomes one. What an operator
calls a release is a **git tag**. Drawing the manifest here reports a number
faithfully and tells the reader something untrue, in the one window whose job
is to say what they are running.

⚠ The two numbers are not a drift to be closed. **Do not resolve the
difference by bumping the manifest** — that makes two numbers agree that are
not the same number, and contradicts the pin.

# The three cases

| Facts | Line |
|---|---|
| version, distance `0`, clean | `Version 0.5.0` |
| version, any distance or a modified tree | `Version 0.5.0, plus 23 commits — not the released build` |
| no version | `No released version — the build details below identify this program.` |

# Why the third case says a sentence instead of nothing


# Why `distance` is a `&str` and parsed here

It arrives as one, because `cargo:rustc-env` carries only strings. It is
parsed rather than passed on so that the *shape* of the fact — a count —
is honoured in the type the catalog receives, and an unparseable value
falls to `0`, which combined with `modified` still cannot produce a false
`Version 0.5.0`: distance `0` on a clean tree is the only route to the
bare line, and an unparseable distance means `git describe` returned
something this build script did not recognise, in which case
`PDFCER_RELEASE_VERSION` is empty too and the third case has already won.

### `fn build_block`

Every value here arrives through `env!` from `build.rs`, so none of it can
drift from what was compiled: there is no constant to forget to bump.

# Why the components are listed at all

`summary()` already says the engine is built in. That answers *"does this
talk to pdfcer or contain it"* and not the question an operator asks when a
bug appears and disappears between two copies of the program: **which
pdfcer**. Two builds an hour apart can carry different engines, and until
this block existed the only way to find out was to read `BUILD-INFO.txt`
beside the executable — which the operator does not have when someone sends
them a screenshot of the window.

# Why an absent component is named

See [`crate::text::about::component_absent`]. Short version: the
no-placeholders rule governs controls, and this is a report.

### `fn component`

The two cases share a function so they cannot be worded inconsistently, and
so a component moving from absent to present needs no edit here — the
lockfile decides.

### `fn attribution`

# Why every field is drawn, including "no changes were made"

The four lines are not decoration. Identification of the creator, a notice
of the licence, a link where the licence family asks for one, and an
indication of whether the material was modified are the *separate*
obligations an attribution-style licence imposes, and leaving one out
leaves that obligation undischarged while the surface looks complete. A
"no changes" statement is a real answer to the fourth, not the absence of
one, which is why [`crate::text::about::Attribution::changes`] is never
empty and a test asserts it.

The licence name is drawn in the plain text role and the supporting lines
in the muted one. Not `.strong()`: `DEFECTS.md` D11 records that role as
unusable in this theme, and the whole point of a named palette is that a
surface added later does not have to rediscover that.

### `fn a_new_dialog_does_not_immediately_close`

Guards the two-step: if `close_requested` ever defaulted to `true` the
window would open and vanish on the same frame, which looks exactly
like a command that does nothing.

### `fn one_frame_draws_and_leaves_the_dialog_open`

Drawn through `egui::Context::run` rather than by inspecting the code,
because the failure this guards against is a *drawing* failure: a
`ScrollArea` whose `max_height` goes negative, an `available_height`
read before there is any, a panic inside a closure the compiler is
perfectly happy with. A passing test is not evidence that a surface
works; this is the weaker claim that it at least composes, and the real
check is `ui-verify` driving the window.

The window rect is deliberately small. A dialog that only survives on
a large screen is a dialog that crashes on a laptop.

### `fn a_window_too_small_for_its_own_header_still_draws`

The specific thing this catches. [`AboutDialog::body`] sizes its
scroll area as `available_height() - 44.0`, and a window shorter than
its own header makes that **negative**. A negative `max_height` is not
a compile error and is not a panic either — it is a scroll area that
silently draws nothing, which would empty the attribution list on a
small screen and on no other. That is the shape of defect this project
keeps finding by running the program rather than by testing it, so the
degenerate size is asserted here instead of waited for.

What it does NOT prove is that the words are legible or in the right
order. Only `ui-verify` driving the real window can say that.

### `fn a_build_past_the_tag_does_not_pass_for_the_release`

The quiet half of the wrong-version failure. A bare `Version 0.5.0` on
a build twenty-three commits past the tag tells an operator comparing
their build against the released one that the two match.

### `fn the_unavailable_case_invents_no_number`

A tarball with no `.git`, a machine with no `git`, a clone with no tags:
`build.rs` emits empty strings, and the *only* number anywhere in reach
at that point is `CARGO_PKG_VERSION`, which is pinned and is not a
release version. Reaching for it looks like a tidy fallback and puts a
false release number back in the headline.

So the property asserted is not "it does not say 0.1.0", which a
different wrong number would satisfy. It is that the sentence contains
**no digit at all**.

### `fn the_build_script_emitted_a_usable_release_field`

The unit tests above are about the decision; this one is about the
build input existing at all — a `cargo:rustc-env` line with a typo in
its name fails no compile and shows up only as a permanently empty
value, which the decision tests would happily pass on.

Deliberately tolerant of empty: this test must pass in a checkout with
no tags, which is the state it is asserting the program survives. What
it refuses is a *malformed* value — a leading `v` that was never
stripped, a `git describe` suffix that leaked through, a distance that
is not a number.

### `fn the_dialog_has_something_to_attribute`

Stated here as well as in `text::about` because the two tests answer
different questions. That one asks whether the catalog is well formed;
this one asks whether this dialog has anything to say — and a dialog
registered on the ribbon that renders a heading and nothing under it
is a placeholder — the thing this project forbids — arriving through
data rather than through code.

### `struct AboutDialog`

It holds **no configuration at all**, which is unusual for a dialog and is
the honest shape here: everything it shows is a constant, so there is
nothing for the operator to change and nothing for closing it to forget.
It is still a struct rather than a bare `bool` so that it sits in
[`super::DialogsState`] under the same idiom as every other dialog; a
second, simpler mechanism for one surface is how two ways to do one thing
get started.

### `fn open`

Takes nothing, because it shows nothing that varies. Kept as a
constructor rather than letting callers write `AboutDialog::default()`
so that the day it *does* need something — a build identity, a
packaged-build marker — the call sites do not have to change.
