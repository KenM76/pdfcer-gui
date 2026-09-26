# shell — pdfcer's ribbon, modes, QAT and keymap, as data

This module is pdfcer's half of the contract `SHELL_FRAMEWORK.md` §1
sets up:

> **The shell is data. Tabs, groups, commands, panels, layouts, modes
> and key bindings are a serializable document that the application
> *supplies* and the operator *edits* — not code that has to be
> recompiled to change.**

`egui-shell` owns the *types* and the *rules*: what a tab is, what a
group is, that a command may appear on exactly one tab, that a mode may
only name tabs that exist. It knows nothing about PDF and must never
learn. This module owns the *content*: which tabs pdfcer has, what is on
them, which verbs exist and what each one is called.

| Submodule | Holds |
|---|---|
| [`manifest`] | [`manifest::built_in`] — the whole ribbon as an `egui_shell::Shell` value: eight tabs, thirty-seven groups, three modes, the QAT and the keymap. Plus [`manifest::PLANNED`], the commands deliberately **absent**. |
| [`commands`] | [`commands::register`] — every command the manifest names, with its label, tooltip, icon key, enable predicate and opaque handler token. |
| [`menus`] | [`menus::built_in`] — the four context menus, carried on the same `Shell`, plus [`menus::MenuHost`], the one seam a right-click site uses. |
| [`ron`] | The same manifest as a `.ron` file, with a test that the two agree. |

## The third surface

[`menus`] is the context-menu half of `RIBBON_IA.md` §5.8's three
surfaces, and it is deliberately **not** a second vocabulary: a menu
item is the same `egui_shell::manifest::Item` a ribbon band holds,
resolved through the same registry into the same handler token. So P1 —
one command, one tab — is *not* extended over menus, and must not be:
§5.8 states that a menu carrying a tab's command again *"is not
duplication in the P1 sense — context menus are not tabs"*. The tests
below reflect that split, and [`menus`]' own tests carry the checks
`egui-shell` does not perform.

## The specification this implements

`RIBBON_IA.md` §5, tab by tab and group by group, amended by
`MODES_AND_PANELS.md` Part 1 (the Read/Review/Edit selector, and the
two new View ▸ Window settings). Where this module departs from either
document — and there are a handful of places where they contradict each
other or contradict what exists — the departure is documented at the
site, in the submodule that makes it.

## The rule that shapes this module more than any other

`RIBBON_IA.md` P3, **no placeholders**:

> An unavailable capability renders nothing, not a disabled stub.
> Greying is reserved for *temporarily* unavailable — no document open,
> document encrypted, undo stack empty — and is always explained on
> hover.

`RIBBON_IA.md` §5 marks every command with where it exists today: **G**
in the GUI, **C** in `pdfcer-core`/`pdfcer` only, **N** nowhere. P3
means a **C** or an **N** must be *absent from this manifest*, not
present and disabled — a **C** row is a command whose engine is written
and whose shell is not, which is a cheap win and still not a shipped
command.

Absent is not the same as forgotten, so every one of them is listed in
[`manifest::PLANNED`] with the reason it is not here. That list is
machine-readable, tested against the manifest in both directions, and
is what a later stage reads to find its work.

## Where the strings come from

Nowhere in this module. Every operator-visible string — tab labels, tab
questions, group captions, command labels, command tooltips, mode
labels — is a call into [`crate::text::ribbon`] or
[`crate::text::commands`]. `tools/gates/check-ui-strings.sh` scans this
tree recursively and fails the build on a literal that carries
whitespace, which is the mechanical half of the rule; the reason for
the rule is in [`crate::text`]'s own header.

## Where the behaviour comes from

Also nowhere in this module. A registered command carries an
`egui_shell::HandlerToken`, an opaque `u64` the shell stores and hands
back. This module assigns those numbers and says nothing about what
they do; the application dispatches on them at one choke point, which
is where a confirmation gate, an undo entry or a trace belongs.

## Item notes

### `fn the_built_in_manifest_is_valid`

Everything `egui_shell::Shell::validate` checks, which includes the
rule this whole information architecture is built on: **a command
appears on at most one tab**, counting the contextual Format tab.

That last clause is the one worth having a test for. `RIBBON_IA.md`
§5 is a document written by a person, and it lists `Thin lines`
under both View ▸ Render and View ▸ Display, and `Comments` under
both View ▸ Panels and Markup ▸ Comments. Both would be violations,
both were resolved deliberately (see [`manifest`]'s header), and
this is the check that would have caught them if they had not been.

### `fn the_built_in_manifest_survives_a_build_without_an_optional_capability`

`SHELL_FRAMEWORK.md` §5b, the operator's directive:
*"Everything should be capable of being modular… if not needed by
someone they could just remove them and they would not show up as
options in the GUI."*

# What this simulates, and why simulating it is the right test

It builds the real registry, then **withholds** every command that the
manifest marks conditional — which is exactly the state a
`--no-default-features` build is in — and asserts three things:

1. The merge drops those items, and drops **only** those items.
2. Each drop reports [`SkipReason::CapabilityAbsent`], naming the
   capability. Not `UnknownCommand`: one says *"this build does not
   include that"* and the other says *"someone made a mistake"*, and a
   log that confuses them turns modularity and a typo into one event.
3. **The merged shell still validates.** This is the assertion that
   matters most and the one whose absence would have been catastrophic:
   `validate_against` is strict, an unregistered command fails it, a
   failed validation leaves `PdfcerApp::shell` as `None`, and
   `Capabilities::for_mode` returns **FULL** when the shell is absent.
   So without the merge, a lite build would have lost its whole ribbon
   **and granted every authoring capability to every mode, including
   Read.** A previous session turned eight mode-gating tests red from
   the other direction; this is the same trap facing the other way.

⚠ A cross-compilation would be a better test and is not available from
inside one: `cfg!` describes the build that is running. What this does
instead is exercise **the mechanism** — the merge, the skip reason and
the validation — against a registry in the state the other build
produces, which is the part that could be wrong. The other half (that
the command really is absent when the feature is off) is a one-line
`cfg!` assertion below and is checked for real by building with
`--no-default-features --features jpx,ocrs`.

### `fn the_signing_command_is_registered_exactly_when_the_feature_is_on`

The negative control for the test above, and it is not a formality: that
one withholds the command itself, so it would pass identically against a
build in which `file.sign` was never registered at all. This is what
makes the pair say *"the feature controls it"* rather than *"it is
absent"*.

### `fn every_command_the_manifest_names_is_registered`

Walks every reference site — tab groups, the quick-access toolbar
and the keymap — not just the groups. A key bound to a command that
does not exist is a chord that does nothing, and it is the
reference easiest to leave behind when a command is renamed,
because unlike a button it is invisible until pressed.

**It validates the MERGED shell, not the raw manifest**, because the
manifest carries conditional items.

`validate_against` is strict by design and rejects any reference to an
unregistered command. That is right for a mandatory item and wrong for a
conditional one, whose absence in a build without its capability is the
intended configuration rather than a bug. The merge is what tells the
two apart, and `PdfcerApp::new` runs it before validating for exactly
this reason.

⚠ So this test asserts two things rather than one: that everything
resolves, and that **the only things the merge had to drop were
capability-absent items.** An `UnknownCommand` skip fails
here rather than being swallowed by the merge's fail-soft posture —
which is the hole a naive "merge then validate" would have opened.

### `fn no_registered_command_is_orphaned`

A command in the registry that no tab, no QAT slot and no key
binding mentions is unreachable. It is not a crash and it is not
caught by anything in `egui-shell` — the framework's validation
runs manifest → registry, and this is the other direction.

It is the failure mode a *rename* produces. Change a command's id
in the manifest, forget the registry, and
`every_command_the_manifest_names_is_registered` fires. Change it
in the registry and forget the manifest, and nothing fires at all:
the old id is simply never referenced again, the button disappears,
and the suite stays green.

# A context menu does not count as reachability

`command_references()` walks tab groups, the QAT and the keymap —
and deliberately **not** [`menus`]. That is not an omission in
`egui-shell`; it is the right answer to *this* question. A command
reachable only by right-clicking one particular surface is a command
nobody can find: a context menu is discovered by an operator who
already suspects something is there, which is exactly the state a
command with no other home cannot put them in.

So a menu-only command must fail this test, and
`menus::tests::every_menu_command_is_also_reachable_from_the_ribbon`
states the same rule from the menu side, where the failure message
can name the menu.

# …and a CUSTOM ITEM does count, because it is a ribbon control

`command_references()` walks the places a command *id* can appear, and
an `egui_shell::manifest::Item::Custom` carries none — the shell
reserves the space and the application draws whatever it likes. A
command whose ribbon control is such an item is therefore reachable
(it is a control in a band on a tab, as discoverable as any button)
and invisible to the function this test is built on.

Those are enumerated in [`manifest::CUSTOM_BACKED`], with the item
that draws each and the reason a button could not have. Consulting the
register keeps the rename check the test exists for — a command
reachable by *nothing* still fails — while not forcing a redundant
second button onto the tab to satisfy a check about ids.

### `fn each_mode_is_a_subset_of_the_next`

The premise of the whole mode feature, and the thing nothing else
enforces. `MODES_AND_PANELS.md` Part 1:

> The three positions are **ordered by capability** — each is a
> superset of the one before. A slider says that; three toggle
> buttons do not… The ordering is the information.

If Review ever gained a tab that Edit lacks, the control would
still render as a slider and would be lying about what it does.
`egui_shell::Shell::validate` cannot catch this: it checks that a
mode names tabs that exist, which is a different question, and it
must not assume modes are ordered at all — a different application
may ship three unrelated workspaces.

Checked as a chain over the modes **in manifest order**, not by
naming the three ids, so that a fourth position inserted between
two existing ones is checked too.

### `fn every_mode_names_only_ordinary_tabs_that_exist`

`egui_shell::Shell::validate` already enforces this, and it is
asserted again here against the concrete tab set because the
failure it prevents is specific and silent: a mode naming the
contextual `format` tab would be asking for a tab whose presence is
decided by the selection, not by configuration, and the mode would
appear to work until nothing was selected.

### `fn no_group_is_empty`

P4 makes group captions mandatory; the corollary is that a group
must have something to caption. An empty band is exactly the
"unfinished program" reading `RIBBON_IA.md` §3 describes, and it is
the shape the no-placeholders rule produces if a group's entire
contents turn out to be **N** — Pages ▸ Stamp, Edit ▸ Arrange,
Measure ▸ Quantity and three more, all of which are consequently
absent as *groups* rather than present and empty.

### `fn planned_commands_are_genuinely_absent`

The two lists are complementary by construction and would drift
silently without this. The dangerous direction is the second one: a
command that is registered *and* listed as planned is a command
someone half-built, and the registry is the half that does not show
up in a screenshot.

### `fn every_planned_entry_is_unique_and_explains_itself`

The reason is the entry's whole value. `("pages.crop", "")` records
that somebody once thought about cropping and nothing else; the
list exists so a later stage can tell a **C** row — engine written,
shell missing, a day's work — from an **N** row that is a month.

### `fn the_qat_is_the_four_documented_commands`

Pinned because the QAT is the surface a returning operator's hands
know without looking. Adding to it is a decision; drifting into it
is not.

⚠ **The second slot is `file.save`, not `file.save_copy`.**
`RIBBON_IA.md` §6's list is *open, save, undo, redo* — the four every
application in this class puts there — and a quick-access Save that
opens a file dialog is not the control those hands are reaching for.

### `fn undo_and_redo_are_reachable_without_a_tab`

Mirroring undo/redo onto *every* tab is the obvious way to make them
always reachable, and it does the opposite: the ribbon renders only the
active band, so undo disappears with whichever tab is not in front. The
rule is therefore **one command, one tab**, enforced by `egui-shell`.
That rule's own failure mode is the opposite one — a command reachable
from **no** surface at all — which is what this test guards.

Undo and redo sit on no tab in `RIBBON_IA.md`'s layout. That is
deliberate and it is only safe because the QAT is always visible.

### `fn the_ribbon_overflow_chevron_has_a_glyph`

The third of this family — `crate::app::status` and
`crate::find::bar` each guard their own glyphs, and the reason there
are three rather than one is that none of them can see the others'
strings. This one guards a string neither can see and **neither
crate could have guarded**.

# What shipped, and why nothing caught it

`egui_shell::ribbon::plan::overflow_label` built `"⌄ N more"` from
U+2304, which egui's bundled stack cannot draw — so the affordance
rendered as `□ 1 more` in **every build this project has ever
produced**, on both the ribbon band and the dock tab bar. Found by
an agent reading its own screenshots, not by a test.

It is the same defect that produced `chevron-down.svg` and the same
one `every_glyph_the_status_bar_draws_has_a_glyph` was written after
— that catalog was drafted with `◀ ▶ ▸ ▾`, all four missing. Three
separate sightings of one hazard.

**The structural reason it survived all three:** `cargo test -p
egui-shell` compiles without egui's `default_fonts`, so a
`has_glyph` assertion *inside* the crate that owns the string would
answer about a font set no real build has, and would pass for the
whole life of the defect. The crate cannot check its own string.
This is the test it delegates.

# The coupling, stated rather than hidden

`overflow_label` is `pub(crate)` there, so this spells the character
itself. That is a two-sided pin, the same shape as the
`ribbon.item.*` name contract: `egui-shell`'s own
`the_overflow_label_uses_the_pinned_chevron` asserts the label
*contains* this codepoint, and this asserts the codepoint is
*drawable*. Changing the character fails one; changing it to
something undrawable fails the other.
