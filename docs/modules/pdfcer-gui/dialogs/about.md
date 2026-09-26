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

## ★ Why this dialog is exempt from "a closed document closes the dialogs"

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

## ★ The title bar and this window answer different questions

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
