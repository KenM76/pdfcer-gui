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

## The distinction that decides what is remembered

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

### The paper policy is remembered; a specific SHEET is not

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

## Item notes

### `fn default`

That is the whole specification of this impl, and it is worth stating as
a rule rather than as a coincidence: a fresh `userdata` folder must open
the print dialog in the state every previous build of pdfcer opened it
in. Anything else would make "delete the preferences file" a way to
change behaviour rather than a way to reset it.

### `fn every_orientation_round_trips`

The list is exhaustive by construction: a variant added to the enum
makes `orientation_key`'s `match` fail to compile, which is the point of
storing the dialog's own type rather than a mirrored one.

### `fn every_scope_round_trips`

Including `ContentOnly`, which the dialog does not offer — see
[`scope_key`] for why a value the UI cannot produce still needs a token.

⚠ "Every scope this build can name" is the strongest claim available:
the type is `#[non_exhaustive]`, so this list is hand-written and a scope
added by a newer engine will not appear in it and will not fail to
compile. That is the standing lesson about hand-written lists inside
completeness tests, stated here rather than left to be rediscovered —
the mitigation is [`scope_key`]'s `_` arm being a *correct* answer rather
than a panic, not this test being complete.

### `fn every_scale_mode_round_trips_to_the_same_mode`

`Custom`'s payload deliberately does not survive, and this test says
so by comparing the token rather than the value. The percentage has its
own key; asserting the payload here would pin a duplication the module
header argues against.

### `fn a_form_id_is_deliberately_not_persisted`

⚠ The property asserted is *deliberate loss*: `Form(257)` must come back
as "from the printer's own settings" and must **not** come back as
`Form(257)`. A file that could round-trip a form id would silently
request a vendor-defined sheet on whatever printer the operator owns
next — see the module header.

### `fn no_token_carries_a_character_the_format_reserves`

`preferences.txt` is `key = value` split on the first `=`, and values are
trimmed. A token containing `=`, `#` or a newline would be unparseable or
would silently truncate the line into a comment.

### `fn every_remembered_field_is_read_back_by_the_print_dialog`

The half of O166 that no compiler and no other test can see, and it is
the half most likely to rot.

`PrintDialog::habits` is a struct literal with no `..Default::default()`,
so **writing** a new preference is compiler-enforced: add a field to
[`PrintPrefs`] and that function stops building. The **reading** side
has no such property. `PrintDialog::open` is a struct literal of the
*dialog's* fields, and a field of `PrintPrefs` that nothing over there
mentions compiles perfectly — the dialog simply opens on its hard-coded
value while the preferences file dutifully records, writes and reloads
a number nobody ever looks at. Every test in this module would still
pass: the round trip works, the tokens are unique, the default matches.
The only symptom is the operator saying *"it still doesn't remember
the copy count"*, months later.

⚠ **The field list is read out of this file's own source, not typed
here.** A hand-written list inside a completeness test is exactly the
gap it was built to find — a new field would be invisible to the check
and the count would still add up. This parses the `struct PrintPrefs`
block, so adding a field adds an assertion whether or not anybody
remembers this test exists.

It is a source-text check and therefore proves only that the name is
*mentioned* in the right place, not that it is used correctly. That is
still the whole difference between a preference that is wired up and
one that is silently inert, and the driven `ui-verify` check for O166
is what proves the value survives a restart.

### `fn the_default_is_what_the_dialog_used_to_hard_code`

The specification of [`PrintPrefs::default`], asserted rather than left
as a comment: deleting `preferences.txt` must be a way to reset pdfcer,
never a way to change what it does.

### `const MIN_PRINT_DPI`

**36, because that is the floor of the dialog's own `DragValue`.** The
numbers here are deliberately the control's numbers rather than a
separately-reasoned pair: a file that refused a value the operator could
produce by dragging the box would silently discard a setting they had just
made, and a file that accepted one the box could not reach would put the
control out of range of its own stored value.

A hand-edited `print_max_dpi = 5` is a typo, and it is clamped to this
rather than rejected — see `parse_key`'s note on why out-of-range and
unreadable are different answers.

### `const MAX_PRINT_DPI`

The dialog's own `DragValue` ceiling, for [`MIN_PRINT_DPI`]'s reason.

The bound exists to limit **memory**, not quality: the dialog's disclosure
reports the uncapped megabytes a page would take, and a job at 2400 DPI on
an A0 sheet is measured in gigabytes per page.

### `const MAX_PRINT_COPIES`

Matches the dialog's own `DragValue` range. A driver may refuse fewer; the
dialog reads the device's `max_copies` and clamps against it separately,
because that limit belongs to a device and this one belongs to the file.

### `fn paper_key`

⚠ [`PaperChoice::Form`] writes `device`, not its id. That is not a lossy
accident — it is the rule the module header argues: a `dmPaperSize` above
`DMPAPER_USER` means whatever one driver says, and a preferences file
outlives printers. There is deliberately no token that could parse back into
a `Form`, so no hand-edited file can reintroduce the problem either.

### `fn scale_from_key`

`custom` parses to `ScaleMode::Custom(1.0)` — a placeholder payload, because
the real one is derived from the separately stored percentage the moment the
dialog plans a job. Any other payload here would be a second copy of a number
that already exists, and the two would eventually disagree.

### `fn scope_key`

`AnnotationScope::ContentOnly` has a token even though the dialog does not
offer it. It is a legal value of the type, a hand-edited file may name it,
and the round-trip test below covers every variant this build can name — so
it must have one. Omitting it would make the writer capable of emitting a
token nobody could parse the day the dialog grows a fifth radio.

# ⚠ The `_` arm is the engine's, not a shortcut

`pdfcer_render::AnnotationScope` is `#[non_exhaustive]`, so this crate is
*forbidden* from matching it exhaustively — a newer engine may put a scope
here that this build has never heard of, and the compiler will not point at
this function when it does. The arm returns the token for
[`AnnotationScope::Document`], which is what an unknown scope is written as.

That is a deliberate, disclosed loss rather than a silent one: the operator
cannot select an unknown scope from this dialog in the first place (the four
radios name the four this build knows), so the arm is unreachable from the
UI, and the only way to reach it is a build whose engine moved underneath
its shell — in which case "print the document without review markup" is the
answer that cannot put a comment on paper unasked.

### `enum KeyOutcome`

Three outcomes rather than an `Option<bool>` because the caller has to tell
*three* things apart and a boolean can only carry two: **not one of ours**
must fall through to the next family and eventually to `UnknownKey`, while
**ours but unreadable** must be reported as `BadValue` against this build's
own key. Collapsing those two would report every mistyped print value as an
unknown key, which tells the operator to check their spelling of a key they
spelled correctly.

### `fn parse_key`

# Why the parser for this group lives HERE and not in `prefs::file`

`prefs::file`'s header states the rule this obeys: *"adding a preference is
one edit to one file"*, because the parser and the writer are two spellings
of one vocabulary and a two-file change is how a writer comes to emit a key
its own parser rejects. That rule is about **the pair staying together**,
not about the pair being in `file.rs` specifically.

This group is thirteen keys — more than the rest of the file has between
them — and every one of them is about printing. Inlining them would put a
third of `file.rs` under one subject and hand the *commonest* future edit
(a new print preference) a 900-line file to find its place in. So the pair
moves together, into the file that already owns this group's type, its
defaults and its token vocabulary: adding a print preference is still one
edit to one file, and it is now **this** file.

`file.rs` keeps one arm that delegates here and one call that delegates to
[`write_block`], so the round-trip tests over the whole of
[`Prefs`](super::Prefs) cover this group unchanged.

# The contract

`value` arrives already trimmed, as `file.rs` trims both halves before it
dispatches. Returns [`KeyOutcome`]; see its variants.

### `fn write_block`

Called once by `Prefs::write_to_string`. The comments are as long as they
are because the file is meant to be opened in a text editor, and
`print_paper = match-pages` tells an operator nothing about what else they
could write there.
