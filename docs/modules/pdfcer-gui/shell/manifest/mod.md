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
