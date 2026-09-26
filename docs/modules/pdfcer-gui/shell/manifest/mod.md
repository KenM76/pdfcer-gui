# shell::manifest — pdfcer's ribbon, as an `egui_shell::Shell` value

[`built_in`] returns the complete pdfcer shell: eight tabs (seven
ordinary plus the contextual Format tab), thirty-seven groups, three
modes, the quick-access toolbar and the keymap. It is the **built-in
layer** of `SHELL_FRAMEWORK.md` §4's three-layer merge:

1. **Built-in** — this function. Compiled into the binary, always
   valid, and always available as the reset target.
2. **Application override** — an optional file shipped beside the exe.
3. **Operator customization** — `userdata/shell.ron`.

Layers 2 and 3 override this one **per item**, never wholesale. That is
why this layer has to be complete and has to validate: it is the thing
every other layer is a patch against, and it is what an operator gets
back when they reset.

One file per tab. The tab modules are where the *reasoning* lives —
why a command sits where it does, what moved, what was left out and
why — and they are worth reading before changing anything here.

# The no-placeholders rule, and the two registers that keep it honest

`RIBBON_IA.md` P3: *an unavailable capability renders nothing, not a
disabled stub.* Greying is reserved for **temporarily** unavailable —
no document open, undo stack empty — and is always explained on hover.

`RIBBON_IA.md` §5 marks every command it specifies with where it exists
today:

| Mark | Meaning | In this manifest |
|---|---|---|
| **G** | exists in the GUI now | emitted |
| **C** | exists in `pdfcer-core`/`pdfcer`, no GUI surface | **absent**, in [`PLANNED`] |
| **N** | exists nowhere | **absent**, in [`PLANNED`] |

A **C** row is the cheapest kind of missing command — the hard half is
written and tested — and it is still absent, because P3 is about what
the operator can reach and an engine with no caller is not reachable.

Absent is not forgotten. Two registers make the difference visible:

- **[`PLANNED`]** — every specified command this manifest does *not*
  emit, with the reason. Tested in both directions: nothing in it is
  referenced by the manifest, and nothing in it is registered. That is
  the list a later stage reads to find its work.
- **[`DIRECTED`]** — the small set of commands emitted *despite* not
  carrying a **G** mark, each with the instruction that put it there.
  Without this list those seven entries would look like the manifest
  quietly ignoring P3.

# Command ids

Dotted lowercase, and the prefix is **the tab that owns the command**:
`view.zoom_fit_page`, `pages.rotate_left`, `markup.highlight`. That
makes P1 — one command, one tab — legible in the id itself, and it
makes a violation obvious on sight rather than only at validation.

Two deliberate exceptions:

- `edit.undo` and `edit.redo` sit on **no tab**. They live on the QAT
  alone, which `RIBBON_IA.md` §7 keeps unchanged. The `edit.` prefix
  says where they would go if they ever got one.
- `mode.read`, `mode.review` and `mode.edit` are not tab commands at
  all: they are the three positions of the selector at the far right of
  the tab row, reachable from the keymap.

# What this module deliberately does not decide

**Icons, labels, tooltips and enable predicates** — those are the
registry's half of the split, in [`super::commands`]. A manifest
contains command *ids* and nothing else about them, which is what stops
a customized ribbon from inventing a command and what makes an unknown
id a disclosed skip rather than a crash.

**Behaviour.** Nothing here runs.

## Item notes

### `fn group`

A two-line convenience over `Group::new(..).with_items(..)`, because
this manifest writes thirty-seven of them and the builder chain is the
noisiest thing on the page when every group is one expression.

### `fn group_two_rows`

`OPERATOR_REQUESTS.md` O97 — *"our display buttons should be on two rows to
save space."* For a cluster of icon-only peers that is a **choice** rather
than a list: four square buttons in a row is a strip, and the same four as a
2 × 2 block is half the width and reads as one control. See
[`egui_shell::manifest::Group::prefer_rows`] for what the hint does and does
not promise.

A named constructor rather than a `.with_prefer_rows(2)` on the call, so that
a tab module reads as a list of groups and the one group that is shaped
differently says so in its first word.

### `fn large_items_already_lead_their_group`

This is the rule that replaced [`super::large`]'s old *"only for a
group whose single item it is"* restriction on 2026-09-04, when
`mockups/pdfcer-shell.html` became this band's specification and a
great many controls became Large.

# Why the rule needs a test rather than a sentence

`egui_shell::ribbon::sizing`'s layout rule is that **Large items lead
their group** — they are drawn first, in a horizontal run at the
group's left, and everything else wraps into the rows beside them. That
is not a preference; a Large control spans the rows and therefore
cannot live *inside* the row wrapping.

The consequence is that writing `large("x")` in the middle of a group
**silently hoists `x` to the front of it**. Nothing fails, nothing
warns, and the only evidence is that the band's controls are in a
different order from the one `RIBBON_IA.md` argued for — an order the
operator reaches for by position, in groups like Cut / Copy / Paste,
where reordering is the whole cost.

So: a `Large` item is legal exactly where hoisting is a no-op, i.e.
where the Large items are already a **prefix** of their group's item
list. Separators and custom items count as non-Large for this purpose,
which is the strict reading — a `Recent ⌄` gallery hoisted past is just
as reordered as a command.

Written as a scan for the first non-Large item followed by a Large
one, rather than as a whitelist of blessed groups. A whitelist is a
second copy of the manifest and goes stale; this cannot, because it is
derived from the manifest it checks.

### `fn the_manifest_draws_large_controls_at_all`

A manifest with no Large item at all satisfies
[`large_items_already_lead_their_group`] perfectly, and would go on
satisfying it after somebody deleted every `large(…)` call in the tree.
The count is a floor rather than an exact number — the exact number is
a manifest decision that will move, and pinning it here would make an
IA change fail a test about hoisting.

### `fn the_ribbon_has_the_documented_shape`

Not a change-detector for its own sake: these numbers are quoted in
prose, in five module headers, as the description of the layout
`RIBBON_IA.md` §5 specifies. A count that drifts silently makes
every one of them wrong, and the failure message says which way it
moved.

**Failing here means editing prose, not just the literal.** The
group count went 31 → 32 with this test passing on the new number
and five headers still saying "thirty-one", because pinning a value
does not pin the sentences that repeat it. The sites are:

- this module's header, and [`group`]'s;
- [`crate::shell`]'s submodule table;
- [`crate::shell::ron`]'s header (groups **and** key bindings);
- [`crate::text::ribbon`]'s header.


⇒ The instruction above — **failing here means editing prose** — is
necessary and is not sufficient, because it only fires when the count
moves. Four sites drifted while the count stood still. The only thing
that catches that is re-measuring the sentence rather than trusting it,
which is why they are enumerated by path below.


The keymap is counted here for the same reason: `ron`'s header
argues that the format can express *the real ribbon* and then lists
its parts, so a binding added without that list moving turns the
argument into a claim about a smaller shell than the one shipped.

### `fn the_keymap_offers_the_chords_a_document_application_must`

Added 2026-08-20, on the operator: *"still no ctrl+c, ctrl+v, ctrl+x or
ctrl+p shortcuts that were requested ages ago."* Three of the four were
bound. `Ctrl+P` was not, and had not been since the manifest was
written.

# Why the whole list, and not a line for the one that was missing

Because the defect was never about Print. It was that **nothing
anywhere asked the question**, and a test naming `Ctrl+P` would leave
the question unasked for the next one. The count assertion above cannot
help: it says how MANY bindings there are, and a keymap with the wrong
thirty-two passes it exactly as well as the right thirty-two.

These are the chords a person arriving from any other PDF or office
application will press without looking. Every one of them is muscle
memory, which means its absence is not experienced as a missing feature
- it is experienced as the application ignoring the keyboard.

A command here that this build does not register is a failure of THIS
test rather than a silently dropped binding, which is the second half of
the same argument: `no_registered_command_is_orphaned` catches a binding
pointing nowhere, and this catches a chord that is simply not there.

### `fn every_command_id_names_its_owning_tab`

The convention that makes P1 legible in the id itself. Two
documented exceptions, and they are named here rather than
hand-waved so that a third one has to be added deliberately.

### `fn every_directed_entry_is_emitted`

[`DIRECTED`] is a claim about this manifest. If an entry were listed
there and then not emitted, the list would be documenting a
deviation that had been quietly reverted — which is worse than
either state on its own, because the note would still be there
explaining a decision nobody could see.

### `fn the_window_group_holds_the_commands_it_still_has`

Order is checked rather than mere presence because the group reads
as a progression — what the window shows, then what may float in
it, then how to put it all back — and the reset belongs last for
the same reason a reset button always does.

### `fn the_markup_style_band_is_a_custom_item_not_a_command`

Asserted because the alternative — modelling a colour picker as a
`Command` — is the easy mistake, and it is the one that would push
a `ColourSwatch` variant into `egui-shell` the first time the
renderer needed to tell the two apart.

### `fn no_command_appears_twice_on_the_tabs`

`Shell::validate` enforces one-command-one-*tab* and separately
forbids the QAT listing one id twice. This is the remaining case:
a command that appears once on a tab, once on the QAT and twice in
the keymap is legal and intended (redo), so what is checked is the
narrower thing — no id appears twice within the tab set.

### `fn apply_paste_chords`

Deterministic and side-effect free: called from tests, from the RON
round-trip check, and once at start-up. It allocates a few kilobytes of
`String` and does nothing else.

# Order is presentation

Tabs appear in the order they are added, groups in the order they are
listed, and items in the order they are written. `RIBBON_IA.md` §4's
table is the tab order — File, View, Pages, Edit, Markup, Measure,
Tools — and it is not arbitrary: it runs from what you do to the file,
through what you look at, to what you change, to what you add, ending
at the things that run across other files.

# The menus are part of this value, not a second document

`SHELL_FRAMEWORK.md` §1 says the shell is *one* serializable document,
and `egui_shell::Shell` carries `menus` beside `tabs` for exactly that
reason: one file to ship, one file to merge, one file for the operator
to edit, and — the payoff that decided it — **one keymap**, so a menu
row's chord hint is derived from the same bindings the ribbon uses
rather than written down twice. See [`super::menus`] for what is in
**Point the two form-field paste chords at the operator's chosen order.**

`OPERATOR_REQUESTS.md` **O58**. Ken, 2026-08-29: *"let's make it an option to
have it swap to match Acrobat or work the way we have it now."*

# What it does, and what it deliberately does not

It rewrites **two entries** of the shell's keymap so `edit.paste` and
`edit.paste_duplicate` sit on the chords
[`crate::app::prefs::PasteChords`] names. It changes **nothing else** — not
what either command does, not its label, not its tooltip, not whether it is
on the ribbon.

That is the whole design. Swapping what the *commands* do would make the
labels lie: a button reading **Paste as duplicate** would paste a new field.
Swapping the *keys* leaves every surface honest by construction, because the
ribbon, the context menu, the shortcuts dialog and the keyboard dispatcher
all read this one keymap.

# Why it removes before it inserts

A [`Keymap`] is a map from chord to command, so writing the new pair without
clearing the old one leaves whichever chord is now unused still pointing at
its old command — and under `AcrobatOrder` that is `Ctrl+V` mapped twice.
`BTreeMap::insert` would resolve it silently and the loser would be decided
by nothing the operator can see.

⇒ Both chords are cleared first, then both are written. The result depends
only on the preference, never on what was there before, which is what makes
this safe to call repeatedly — and it IS called repeatedly: once at start-up
and again every time the setting changes.

# It is a no-op on a shell with no keymap

`Shell::keymap` is an `Option`, and a manifest that failed validation leaves
it `None`. Silently doing nothing is right here: the operator has already
been told the ribbon is unavailable, and a second complaint about key
bindings would be noise about a consequence rather than the cause.

### `const SELECTION_ANY`

One spelling, one source. It is the enable predicate of `format.delete`
and `format.properties`, and the condition
[`super::menus::MenuHost::with_condition`] corrects when a right-click
selects the object under the pointer. Two surfaces reading two spellings
of one condition is a defect whose only symptom is a menu row that is
greyed while the thing it acts on is plainly selected.


It used to read `pub const SELECTION_ANY: &str = format::VISIBLE_WHEN;`,
with a doc comment naming three surfaces that shared one condition. That
was true and it stopped being true when the Format tab grew a **Font**
group: the tab now carries controls for two different kinds of selection --
a page object, addressed by paint-order index, and a swept text range,
addressed by run -- so *"is the Format tab about anything?"* is a strictly
wider question than *"is an object selected?"*. The tab's condition moved
to `selection.formattable`; this one stayed where it was and kept its two
honest readers.

The alias is what made the drift dangerous rather than merely untidy.
Changing [`format::VISIBLE_WHEN`] in place would have silently retargeted
the **canvas context menu**, which has nothing to do with the Format tab
and whose Delete would then have been enabled by a text sweep -- a
destructive control lit by a selection it cannot act on. Spelling it out
here is what makes the two independently editable.

### `const SELECTION_ACTIONABLE`

The Format **tab** still takes `SELECTION_ANY`. A field has no font, no
stroke and no fill for that tab to offer, so widening the tab's own
predicate would draw a band of controls that cannot act on what is
selected.

### `const ACROBAT_AVAILABLE`

# Why this is a `visible_when` and never an `enabled_when`

R9, exactly: *an unavailable capability renders nothing; greying is
reserved for **temporarily** unavailable and is always explained on hover.*
"This machine has no Acrobat" is not temporary and there is no hover
sentence that would help — the remedy is installing a different program,
which is not something a tooltip can walk somebody through and not
something this shell should nag about on every hover for the life of the
installation.

The command's own `enabled_when("doc.open")` is the greying, and it is the
legitimate case: *no document open* is temporary, is the operator's to fix
in one click, and IS explained on hover.

# Published by `PdfcerApp::conditions` from ONE resolved viewer

Not from a fresh registry read per frame — see `crate::acrobat`. The
resolution is cached on the application and recomputed when the setting
changes, so this condition and the path the button will actually launch are
the same fact rather than two reads that could disagree.

### `const DELETE_PERMITTED`

# Why it lives here rather than in [`format`], and the precedent is one
screen up

[`SELECTION_ANY`] carries the account of what an alias cost: while it read
`= format::VISIBLE_WHEN`, changing the Format tab's own condition would have
silently retargeted the **canvas context menu**, whose Delete would then have
been lit by a text sweep. This constant has exactly the two readers that one
has — `manifest::format`'s Selection group and `menus`' `CANVAS_OBJECT` — so
it is spelled out in the one place both can see, and neither owns it.

# It is a `visible_when`, and [`SELECTION_ACTIONABLE`] is still the
`enabled_when`. Two predicates on one control

| predicate | asks | when false |
|---|---|---|
| [`SELECTION_ACTIONABLE`] | *is there anything to delete?* | **greyed** — a selection is one click away, which is exactly the temporary, operator-fixable condition R9 reserves greying for |
| this | *would the engine refuse?* | **absent**, with a sentence in the Properties panel — a certification signature is neither temporary nor arguable |

Collapsing them either way is a defect. Greying on this one would promise an
operator that selecting differently might help; hiding on the other would
make Delete flicker in and out of the ribbon on every click.

# What clears it, and what deliberately does not

Cleared **only** for a selected annotation that `annotation_deletion_refusal`
or §12.5.3 Table 165's `Locked` bit forbids — so its default is *true*, and
it stays true with nothing selected, with a content object selected, and with
a form field selected. `app::conditions` argues at length why that direction
is the safe one: a control drawn where it refuses is the defect being fixed,
and a control withheld where it would have worked is a worse one, because the
operator has no gesture left that reports it.

### `const FONT_FACE`

Three, and each is a control a **button cannot be** -- which is
[`CUSTOM_BACKED`]'s bar and the only reason any of them is drawn this way:
a face chooser has to ask *which* of the page's fonts, a size field has to
accept a number, and a colour needs a swatch that shows the current one.

Same one-spelling-one-source rule as [`RECENT_FILES`], and the same reason
it is a rule: [`COLOUR_SWATCH`]'s own note records the manifest writing a
literal kind that **no renderer ever matched**, so a captioned group drew
an empty band for the whole of v0.1.0 with nothing anywhere reporting the
mismatch. The shell reserves the space, the application declines to draw,
and the symptom is a gap.

### `const RECENT_FILES`

One spelling, one source: the manifest writes it in File ▸ File and
[`crate::app::PdfcerApp::ribbon_band`]'s custom-item renderer matches on
it. A mismatch between those two is invisible — the shell reserves the
item's space, the application declines to draw it, and the band shows a
gap — so the string is a constant rather than a literal in two files.

### `const COLOUR_SWATCH`

Same one-spelling-one-source rule as [`RECENT_FILES`], and this constant
arrived late for a reason worth recording: the manifest wrote the literal
`"colour_swatch"` from S2 and **no renderer ever matched it**, so the Style
group drew a caption over an empty band for the whole of v0.1.0. That is
precisely the invisible failure the constant's existence is meant to
prevent — the shell reserves the item's space, the application declines to
draw it, and nothing anywhere reports a mismatch.

It is deliberately **not** in [`CUSTOM_BACKED`]. That register is for
commands whose only ribbon control is a custom item, and this item backs no
command at all: it edits `PdfcerApp::pen`, raises no `Action`, has no undo,
and returns no handler token. Listing it there would claim a command id
that does not exist.

### `const OCR_BLEND`

`OPERATOR_REQUESTS.md` O226 asks for a slider, in those words, and a slider
is the one control on this tab that a button provably cannot be: its
operand is a position on a continuum, and the whole point of it is that the
operator watches the page change while dragging.

It is deliberately **not** in [`CUSTOM_BACKED`], for [`COLOUR_SWATCH`]'s
reason exactly: it edits `ViewState::ocr_overlay` — what is drawn on
screen, no document, no undo entry — and raises no `Action`. Listing it
there would claim a command id that does not exist.

R8 is satisfied by the item's `shown_when`, not by a registration. The
condition is the *toggle's* own selected-condition, so the slider is drawn
exactly while `view.ocr_layer` is pressed. A build whose registry has no
`view.ocr_layer` cannot press it, cannot turn the mode on, and therefore
never draws this — the capability's absence removes both controls, which is
what R8 is protecting. Spelling the condition as the toggle's own id rather
than as a second, hand-written condition name is what makes that true
rather than merely intended.

### `const MARKUP_STROKE`

`RIBBON_IA.md` §5.8's *Markup annotation* row, drawn by
[`crate::app::markupband`]. Each clears [`CUSTOM_BACKED`]'s bar in the same
shape the Font group's three do: the command needs an **operand a button
cannot ask for** — two colours that must also *show* the current one, a
number the operator drags, a percentage, and two multi-way choices.

They are **registered commands**, unlike [`COLOUR_SWATCH`] one screen up,
and the difference is the same one that separates the Font group from the
pen: `COLOUR_SWATCH` edits `PdfcerApp::pen` — application state, no
document, no undo entry — while these five raise an
`Action::SetMarkupStyle` that rewrites an annotation's appearance stream and
lands in the engine's command log. R8: a capability that edits the document
is a registered command, because registering one is the only way this shell
may learn a capability exists, and because a build compiled without it must
lose the control rather than draw a dead one.

⚠ Do not confuse `MARKUP_STROKE` with `COLOUR_SWATCH`. One restyles the mark
you have selected; the other chooses the colour of the mark you are about to
draw. They sit on different tabs and mean opposite things about *when*.

### `const CUSTOM_BACKED`

`(command id, custom kind, why)`.

# Why this register has to exist

`egui_shell::Shell::command_references()` walks tab groups, the QAT and
the keymap — the places a command *id* can appear. A `Custom` item carries
no id (that is the whole point of it: the shell reserves space and the
application draws whatever it likes), so a command reachable only through
one is invisible to every reachability check built on that function.

`super::tests::no_registered_command_is_orphaned` is exactly such a check,
and it is a good one: it catches the rename that leaves a command
registered and referenced by nothing, which nothing in `egui-shell` can
see. Without this register it would have to be either weakened — which
gives up the rename check for every other command — or satisfied by
putting a second, redundant button on the tab.


# The bar for an entry

The control must genuinely be one a **button cannot be**. `file.recent`
qualifies because the command needs an operand — *which* of ten documents
— that a button has no way to ask for, and the alternatives are ten
commands or a command that opens whichever file it feels like. A command
that could have been a button and was drawn some other way for taste does
not belong here; it belongs on the tab.

### `fn icon_only`

For a tight cluster of peers where the icon is distinctive and the
operator's eye is already in the right group: the four page displays, the
four pointer tools, the two page rotations, cut/copy/paste, the four text
markups. Word's own icon-only clusters are the same shape — bold, italic,
underline; the alignment buttons — and the reason they work is that
**position in a labelled group teaches the meaning**, not the label on each
control.

Safe to ask for even when it cannot be honoured. `sizing::resolved` falls
back to the labelled form unless the command names an icon, carries a
tooltip **and** a painter is installed, so a command that gains or loses an
icon does not need this list audited.

### `fn large`

It used to read *"used only for a group whose single item it is"*, on the
grounds that `sizing`'s layout rule hoists Large items to the front of
their group, so promoting one item of a multi-item group would silently
re-order what `RIBBON_IA.md` settled — and the ribbon IA is not this
file's to amend.

That reasoning is intact. What changed is that `mockups/pdfcer-shell.html`
is now a specification of this band rather than a sketch of it, the
operator having said *"I want everything to look exactly like that
including sizing"*, and it draws a great many controls large. So the rule
is now the **consequence** rather than the proxy for it:

> **A `large` may be added only where hoisting is a no-op** — i.e. the
> promoted items are already the leading run of their group, or the whole
> group is promoted together. Anywhere else, the promotion is an IA change
> and belongs in `RIBBON_IA.md` first.

`manifest::tests::large_items_already_lead_their_group` asserts it, so the
rule is checked rather than remembered. Two places where the mockup
disagrees with the shipped manifest were therefore **left alone**, and are
recorded here rather than silently skipped:

| mockup | shipped | why not promoted |
|---|---|---|
| Edit ▸ Content draws `Edit text` and `Add text` large, *after* a column of `Select all` / `Reflow paragraph` | all four Medium | promoting the two would hoist them **in front of** the column, which is the reordering this rule forbids. The mock authors its own column order; the band derives one. |
| View ▸ Navigate and Pages ▸ Transform draw their tools large | [`icon_only`] | these are the `RIBBON_SCALING.md` §5.1 icon-only clusters, whose whole argument is that *position in a labelled group teaches the meaning*. Reversing that is a decision about the tool strip, not about a glyph size. |

It is also where it reads best: a lone control in a captioned group looks
stranded at Medium, and Word gives exactly this treatment to its own
one-command groups — Dictate, Editor, Add-ins.
