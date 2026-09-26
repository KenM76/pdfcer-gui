# `text::about` — the attribution surface, in the operator's own window

Every word the About dialog shows, plus the **structured attribution
catalog** it draws from. Consumed by [`crate::dialogs::about`], and by the
test that pins this catalog against the shipped
`THIRD_PARTY_LICENSES.md`.

## Why an attribution surface exists at all, when `LICENSE` already ships

Permissively-licensed *code* does not need one. MIT and Apache-2.0 notices
are satisfied by the `LICENSE` file in the package and by the licence
metadata in `Cargo.toml`, which a reader of the source tree can check:
nobody is handed a file whose licence obliges pdfcer to *tell them*
something they could not already look up.

Redistributed **assets** are the case that breaks it. The `ocrs` OCR model
weights — the operator's ruling was *"yes ship that model in the mit repo
with proper credit"* — are **CC-BY-SA-4.0**, and that licence's **BY**
clause requires attribution to reach the **recipient of the work**, not
merely a reader of the repository it was built from. A `PROVENANCE.md` in a
source tree discharges nothing for someone who was handed
`pdfcer-gui.exe` in a folder.

The same obligation already runs on three works this program redistributes
today: the bundled substitute faces and the two Adobe data tables below.
They arrive through the engine (`pdfcer-core` and `pdfcer-render` are path
dependencies and Rust links them statically), so they are inside
`pdfcer-gui.exe` whether or not anyone here thought about them — which is
why an attribution catalog cannot be deferred until an OCR build.

## The two surfaces, and why neither replaces the other

| Surface | What it carries | Who reaches it |
|---|---|---|
| `THIRD_PARTY_LICENSES.md`, in the package | every licence **text**, in full — hundreds of kilobytes of it | someone who opens the folder |
| this dialog | the **attribution**: who made it, what it is, under what terms, and whether pdfcer changed it | someone who runs the program |

Neither is a substitute for the other and the split is not arbitrary. A
dialog cannot reasonably render that much licence text, and a `.md` file
in a folder is invisible to an operator who launched pdfcer from a shortcut
and is one `del` away from an operator who tidied up. So the dialog names
the works and the terms and points at the file, and the file carries the
texts. `tools/gates/check-shipped-assets.py` requires **both**: a
redistributed asset directory must be cited in `about.hbs` *and* in this
module, because a change that remembers one and forgets the other is the
commonest way an obligation half-lands.

## What this module is NOT

It is not a second copy of `THIRD_PARTY_LICENSES.md`, and it must not grow
into one. It carries no licence text — [`Attribution`] has no field for
any — precisely so that the two surfaces cannot disagree about what a
licence says. Where a licence's own terms require a **link** rather than a
reproduction (Creative Commons licences do; BSD-style ones do not),
[`Attribution::licence_url`] carries it.

## Conventions

The catalog convention of [`crate::text`] applies unchanged: sentence
case, no trailing period on a label, full sentences with punctuation for
prose. One addition specific to this module — **every field of every
[`Attribution`] is lifted from a source that was read, never reconstructed
from what a licence of that family usually says.** The sources are named
per entry. A wrong attribution is worse than an absent one.
