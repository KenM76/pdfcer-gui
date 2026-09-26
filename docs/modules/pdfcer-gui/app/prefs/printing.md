# `app::prefs::printing` — what the print dialog remembers between jobs

Operator request **O166**, 2026-09-10: *"the printer dialogue box needs to
remember our last settings."*

## What it was doing before, measured rather than assumed

`PrintDialog::open` built **every** field from a literal, every time:
`ScaleMode::Fit`, `copies: 1`, `max_dpi: 300`, `DeviceSettings::default()`
(portrait, one-sided, no tray-by-size, no paper request), `PageSubset::All`,
`reverse: false`, `uncollated: false`, and the printer set to whichever
device Windows calls the default. Nothing survived closing the dialog — not
even within one sitting, because the dialog value is dropped when it closes.

So an operator who prints every drawing landscape, two-sided, on the
plotter, at 600 dpi re-answered all four questions on every single print.

## ★★★ The distinction that decides what is remembered

Some of what is in that dialog is **about the job** and some is **about how
this operator prints**. Only the second kind may be remembered, and the test
is one question: *would this value still be right for a different document?*

| Remembered | Not remembered, and why |
|---|---|
| Which printer, by name | The page range — it names pages of *this* document |
| Orientation, two-sided, tray-by-size | The typed custom range, same reason |
| Paper policy (see below) | Which preview sheet was on screen |
| Scale mode and the custom percentage | Preview zoom, pan, width, popped-out — the arrangement of a window, not a setting |
| Which annotations print | Which tab was open — already argued in `active_tab`'s own doc |
| Resolution ceiling | The driver's `DEVMODE` — one driver's private format, undefined on another device |
| Copies, collate, odd/even, reverse | |

### ⚠ Copies is the one that could bite, and it is remembered anyway

Remembering `copies = 25` and then printing a 200-page drawing set without
noticing is a worse outcome than retyping `25`. It is remembered regardless,
for two reasons. The operator asked for *the settings* to be remembered, and
a silent carve-out is exactly the kind of unstated exception this project
files operator requests to prevent. And the count is already stated on the
commit button's own label, which reads the live number — so a job of 25
cannot be committed without the number being on screen at the moment of
commitment.

### ★ The paper policy is remembered; a specific SHEET is not

[`PaperChoice::Form`] holds a `dmPaperSize` integer, and those are only
standard up to a point: the low ids are Win32 constants, but everything a
vendor defines lives above `DMPAPER_USER` and means whatever that one driver
says. `PrintDialog::refresh_device` already drops a form id on a change of
printer for exactly that reason, and a preferences file outlives a printer
far more thoroughly than one sitting does — the operator replaces the
plotter and `Form(257)` silently requests a different sheet.

So the two **policies** persist ([`PaperChoice::DeviceDefault`] and
[`PaperChoice::AutoFromPages`], neither of which names an id) and a specific
form does not: it is written as `device` and comes back as "from the
printer's own settings". An operator who wants one specific sheet every time
has [`PaperChoice::AutoFromPages`], which asks for the right one on whatever
device is attached — a better answer to that want than a frozen id.

## Where it is stored, and why not `settings.txt`

`preferences.txt`, beside the shell's other preferences: flat `key = value`,
hand-editable, per-key recovery, and already covered by the update
instruction *"replace the program files, keep your `userdata` folder"*.

Not `settings.txt`. Every entry in that file cites a clause the PDF standard
leaves to the implementation; how many copies this operator usually prints is
not one of them.

## Why the values are the dialog's own types and not a mirrored set

Because a mirrored enum is a second source of truth that drifts. This module
stores [`crate::dialogs::print::spooler`]'s own [`Orientation`], [`Duplex`],
[`ScaleMode`], [`PageSubset`] and [`PaperChoice`], and
`pdfcer_render::AnnotationScope` — the exact values the dialog holds — so a
variant added to any of them is a compile error here rather than a silent
round-trip to the default. That is why `dialogs::print::spooler` is
`pub(crate)` rather than private, and why [`PrintPrefs`] and its field on
[`Prefs`](super::Prefs) are `pub(crate)`: the visibility follows the type it
carries.

## The token functions, and the property that binds them

Every enum below has a `*_key` (value → token) and a `*_from_key` (token →
value) function, and each pair is asserted to round-trip over **every
variant** in this module's tests. That is the property the file format
actually needs: a writer that emits a token its own parser rejects turns the
operator's settings into a `BadValue` note on the next launch, which reads
as pdfcer forgetting them — the very complaint this module answers.

`ScaleMode::Custom` is the one variant carrying a payload, and its payload is
**not** in its token. The dialog already keeps the percentage in a separate
field (`custom_percent`) and re-derives the payload from it at the point of
use, so persisting the payload would store the same number twice and give it
two chances to disagree.
