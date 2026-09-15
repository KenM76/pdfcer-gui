# `egui-shell` — the reusable application shell

What the shell crate is, what it may and may not know, and how an application
supplies it with a ribbon. Read this before adding anything to
`crates/egui-shell`, and before changing the shape of
`crates/pdfcer-gui/src/shell`.

---

## 1. The shell is data

Tabs, groups, commands, panels, layouts, modes, rails, menus and key bindings
are a **serializable document** that the application *supplies* and the operator
*edits* — not code that has to be recompiled to change.

That one decision is what makes the shell both reusable and customizable. A
ribbon defined in Rust `match` arms can be neither; the same serializer that lets
an operator save a customized ribbon lets a different application ship a
completely different one.

The document type is `egui_shell::manifest::Shell`: inspectable, diffable,
versioned, and testable with no window open. Every test in the crate is headless
for that reason.

## 2. The purity rule (R7)

> **`egui-shell` never learns what a PDF is.**

It may depend on `egui`, `serde`, `ron`, `thiserror` and small leaf utilities.
It may not depend on `pdfcer-core`, `pdfcer-render`, `pdfcer-print` or
`pdfcer-gui` — not by manifest, not by import, not through a re-export.

| Half | Mechanism |
|---|---|
| Positive | `crates/egui-shell/Cargo.toml` — the crate cannot reach for what it has not declared. |
| Negative | `tools/gates/check-shell-purity.sh` — the manifest declares no `pdfcer-*` dependency, and no `.rs` file outside a comment names `pdfcer_core`, `pdfcer_render` or `pdfcer_print`. Comments are exempt, because a rule you cannot describe in a doc comment will not be described at all. |

The gate exits `2`, not `0`, when the crate is absent or holds no `.rs` files: a
clean manifest with no source scanned is check 1 passing and check 2 never
running, and those two states must not print the same line.

The rule has a diagnostic form more useful than the prohibition: **if
`egui-shell` needs to know about pages, the abstraction is wrong.** When the
shell needs something the application knows, invert it — the shell declares a
trait or a manifest type and the application supplies the value. The correct fix
is never an import; it is a seam.

Deliberately *not* checked: the word "pdf" in prose, an icon file named for a
format, a doc comment naming the application. Purity is about the dependency
edge, not about vocabulary. A gate that fired on the word would be switched off
within a week.

**Licence:** MIT, matching the rest of the workspace, so the crate can be
published separately. It is extracted to its own repository at or before fold-in
and consumed by path or git dependency — deliberately not folded in as a private
module, because the whole point is that the next application can take it.

## 3. What lives where

| Module | Owns |
|---|---|
| `manifest` | The serializable definition — tabs, groups, items, modes, QAT, trailing controls, rail, keymap, menus — with validation and the three-layer merge. |
| `commands` | The registry: id → label, tooltip, icon key, enable predicate, opaque handler token. The thing a manifest may only *reference*. |
| `ribbon` | Renders a `Shell`: QAT, tab strip, contextual tabs, the N-position mode selector, captioned group bands, overflow, `ui_rect` reporting, accessible names. Reports intent; executes nothing. |
| `dock` | The panel host: columns per side, vertical stacks, tabbed groups, draggable splitters, an overflow menu whose space is reserved before any tab is measured. A panel is an opaque string id. |
| `layout` | Serialization and persistence of a `DockLayout`, fail-soft per item, plus named workspaces and scoped reset. |
| `menu` | Context menus keyed by an application-supplied context id: the same `manifest::Item`s a ribbon band holds, through the same registry, with keymap-derived chord hints. A menu with nothing to offer never opens. |
| `peek` | The three-state auto-hide machine shared by the ribbon band and the dock rail. |
| `tabstrip` | The document tab strip — a tab names an *operand*, where a dock tab names a *panel*. |
| `theme` | Token palette, presets, and the rendered-pair contrast gate. |
| `verify` | The opt-in `key=value` diagnostic channel `tools/ui-verify` reads. |

There is no `modes` module. A mode is manifest data (`manifest::Mode`) plus a
dock layout (`layout::Workspace`); `ribbon::mode_selector` draws the selector and
`ribbon::tabs` honours the active mode's tab list.

**What stays in the application:** panel *bodies*, command *implementations*,
domain state, and the manifest content itself. `egui-shell` renders a tab called
"Measure" and routes a command called `measure.linear`; it has no idea what
either means. Here that content lives in `crates/pdfcer-gui/src/shell/`:
`manifest` builds the `Shell` value, `commands` registers every verb, `menus`
carries the context menus, and `ron/built_in.ron` is the same manifest on disk.

**The dock model** is four levels — side ▸ column ▸ stack ▸ tab — and the
persisted schema *is* that model, so there is nothing to translate on load. It is
built on `egui` directly; `egui_tiles` is not a dependency of this crate, and
`MODES_AND_PANELS.md` carries the reasoning and what that gives up. Two
invariants this crate owns rather than inherits: `dock::tabs` reserves the
overflow affordance before the first tab is measured, so it cannot itself be
squeezed out, and `plan::drag_boundary` writes to exactly two slice entries, so
one splitter cannot move every column. `plan::MIN_COLUMN_WIDTH` is a constant,
and a test asserts no minimum consults a label.

## 4. The manifest

### Regions

| Field | Holds |
|---|---|
| `schema` | The version the document was written against. `0` means unstated and is read as current. |
| `modes` | Named workspaces, each naming the tabs it contains. Read/Review/Edit is a *configuration*; nothing in the crate knows those names. |
| `tabs` | The ordinary tabs, in display order. |
| `contextual_tabs` | Tabs present only while their `visible_when` condition holds. Separate from `tabs` because they are not mode members. |
| `qat` | The quick-access toolbar: command ids, in order. |
| `trailing` | The controls past the mode selector at the right of the tab-strip row. |
| `rail` | The vertical strip down a dock side's outer edge: panel tabs, navigate selectors, selection tools. Data rather than a callback, because a rail only the application knows about breaks "the shell is data" quietly. |
| `keymap` | Key chord → command id. |
| `menus` | Context menus, keyed by the context id the right-click site supplies. |

`Shell::SCHEMA` is the version this build writes and understands. Bump it when a
change would make an *older* build misread a newer file — not for an added
optional field, which an older build already ignores safely.

### One type is both document and patch

`Shell` describes a complete ribbon and is *also* the type of each merge layer.
That is why nearly every field is `Option`: the `Option` distinguishes *"set this
to empty"* from *"do not mention this"*, and that distinction is the whole
difference between a per-item override and a wholesale replacement. A layer
saying `Shell(tabs: [ Tab(id: "tools") ])` means "move the Tools tab to the
front", not "delete every other tab".

The cost is that a `Shell` can be incomplete, and the answer is
`Shell::validate`: **a complete manifest is one that validates.** A layer is not
expected to; the merged result is required to. One type, two roles and a checked
boundary between them, rather than a second `ShellPatch` type that would have to
be kept in step field by field forever.

A tab with no `groups` key is a layer's *reference* to a tab. A deliberately
empty tab is written `groups: []`. Hiding a tab (`hidden: true`) keeps its
definition; deleting it from a customization file gets the built-in one back at
the next merge.

### Item vocabulary

| Item | Meaning |
|---|---|
| `Command { id, size, visible_when, capability }` | A registered command, presented here. The manifest carries nothing *about* the command — no label, no icon, no tooltip — only how it appears at this position. |
| `Separator` | A vertical rule. Presentation only. |
| `Custom { kind, payload, visible_when }` | Something the application draws itself — a colour swatch, a zoom slider, a scale picker. The shell reserves the space and hands `kind` and `payload` back; it draws nothing and interprets neither. |

`Custom` is the extension point that keeps the vocabulary from growing a variant
per widget an application happens to want, which is the road by which a reusable
shell acquires a `ColourSwatch` variant and stops being reusable.

`ItemSize` is `Medium` (icon, gap, label, one row — the default), `Small` (icon
only) or `Large` (icon above label, spanning the band's rows). `Small` is
**earned, not asserted**: a control renders icon-only only when it names an icon,
carries a tooltip and has a painter installed. The tooltip is the icon's
accessible name; without one an icon-only button is an unlabelled rectangle to a
screen reader. A `Small` that has not earned it falls back to `Medium` rather
than rendering a mystery.

Two conditions on an item, and confusing them is why they are two fields:

| Field | Asks | Answered |
|---|---|---|
| `visible_when` | is it relevant to this document, this mode, this moment? | every frame; its space is reclaimed **before measurement**, so the group re-flows |
| `capability` | is it in this build at all? | once, at start-up; it cannot change while the program runs |

`Group::collapse` enters a group in the width ladder: `None` — the default, and
the meaning of an absent key — never collapses; `Some(n)` collapses at priority
`n`, lower first, ties broken on manifest order.

### On-disk form

RON, via `Shell::from_ron` and `Shell::to_ron`, rather than JSON: the format has
real enums (`Command(id: "view.single")` beside `Separator`), comments and
trailing commas, and all three matter for a file an operator edits by hand.

```ron
Shell(
    modes: [
        Mode(id: "read",   label: "Read",   tabs: ["file", "view"]),
        Mode(id: "review", label: "Review", tabs: ["file", "view", "pages", "markup", "measure"]),
    ],
    tabs: [
        Tab(id: "file", label: "File", groups: [
            Group(id: "security", caption: "Security", items: [
                Command(id: "file.encrypt", size: Large),
                Command(id: "file.sign", size: Large, capability: "signing"),
            ]),
        ]),
        Tab(id: "view", label: "View", question: "What is on my screen?", groups: [
            Group(id: "page_display", caption: "Page display", items: [
                Command(id: "view.single"), Command(id: "view.continuous"),
            ]),
        ]),
    ],
    contextual_tabs: [
        Tab(id: "format", label: "Format", visible_when: "selection.any", groups: []),
    ],
    qat: ["file.open", "file.save_copy", "edit.undo", "edit.redo"],
    keymap: { "Ctrl+E": "edit.text", "Ctrl+1": "mode.read", "F11": "view.fullscreen" },
)
```

**`IMPLICIT_SOME` is enabled on the `ron::Options` used in both directions.**
Without it every `Option` field — which is nearly every field — has to be written
`tabs: Some([…])`, and the obvious spelling fails to parse with `ExpectedOption`:
a message naming a Rust type the operator has never heard of, with no hint that
the fix is four characters. A round-trip test cannot catch this, because the
writer and the reader agree by construction; the population that breaks is the
one that never goes through the writer. The guard is a test whose input is a
string literal written by a person and produced by no serializer.

`crates/pdfcer-gui/src/shell/ron/built_in.ron` is the whole real ribbon as RON,
generated and checked in, with a test asserting it parses to the same `Shell` the
Rust builds — an equality of *parsed values*, not of text. It is the proof that
"the shell is data" describes a product rather than a data structure. Regenerate
it after changing the manifest:

```sh
cargo test -p pdfcer-gui rewrite_built_in_ron -- --ignored
```

## 5. Commands are referenced, never defined

A manifest contains command **ids**. It contains no labels for them, no icons, no
handlers, and no way to add any. Those live in `commands::CommandRegistry`, in
code: each `Command` carries an id, a label, an optional tooltip and icon key, an
`Enable` predicate evaluated against a `ConditionSet`, and an opaque
`HandlerToken` the application dispatches on at one choke point. That is what
stops a customized ribbon from inventing a command that does not exist, and it is
why an unknown id can be a disclosed skip rather than a crash: there is nothing
for the shell to try to run.

`manifest::CommandCatalog` is the trait that answers *"is this id real?"*, and
`CommandRegistry` implements it. The trait exists so the manifest module does not
depend on the registry: a manifest must be parseable, mergeable and
round-trippable by a tool that has no registry at all — a schema linter, a diff
viewer, a harness inspecting a `.ron` file without linking the application.
`AnyCommand` accepts every id and is a named type at the call site rather than a
default, because using it in production would disable the check that makes an
unknown id a disclosed skip.

## 6. Three layers, merged at load

| Layer | Source |
|---|---|
| `BuiltIn` | Compiled into the binary. Always valid, and the reset target. |
| `AppOverride` | An optional file shipped beside the executable. |
| `Operator` | The operator's own customization. |

`MergeInput::built_in(&shell)` takes the required layer; `.with_app_override(…)`
and `.with_operator(…)` add the others. Later layers override earlier ones **per
item**, not wholesale — the same per-key fail-soft contract `settings.txt` uses.

`merge` never fails. Anything it cannot carry across becomes a `Skip` in
`Merged::report`, carrying the layer, the `Site` and the reason:

| Reason | Raised when |
|---|---|
| `UnknownCommand` | An overlay item referenced a command that is not registered. That one item is lost; the layout is not. |
| `CapabilityAbsent { capability, command }` | A **conditional** item's command is not registered — see §7. |
| `UnknownTab` | A mode named a tab that does not exist after merging. |
| `UnsupportedSchema { found, supported }` | The whole layer declared a schema newer than this build understands, so none of it was applied. A field this build does not know may be the one that makes the rest of the file mean what it says. |

A `Skip` is a structured value, not a message: the shell has no business deciding
how another application words a note to its operator, and an application offering
"remove this stale entry from your file" needs the id, not a sentence containing
it. `Display` is for diagnostics only, never operator-visible copy.

Skips must be disclosed, never swallowed. A silently dropped ribbon item is the
operator seeing a button missing and concluding the application removed it. The
application traces every skip at start-up; in a default build the loop runs zero
times, so a `shell-item-skipped` line in a trace is itself the signal that the
binary was built smaller.

**The built-in layer is not filtered against the catalog**, with the one
exception in §7. It is compiled in, it is the reset target, and an unknown
command in it is a programming error that should surface as a validation failure
in the application's own test suite — not be quietly repaired at start-up, which
would hide the bug on every machine that runs it.

**Current wiring:** the application merges the built-in layer only. The
application-override and operator layers are supported by `MergeInput` and by the
merge algorithm; nothing yet reads them from disk.

## 7. Capabilities that can be removed (R8)

> **A capability's presence is expressed by registering its command, and by
> nothing else. No other code in the GUI may test for a capability.**

If the ribbon renderer hard-codes where a button goes, or a panel does
`if cfg!(feature = "…")`, then a later move from statically linked modules to
loaded modules stops being a swap and becomes a rewrite. The registry does not
care whether a command was registered by a linked module or by one loaded at
start-up — but only if it is the *only* thing that knows. This also satisfies R9
(no placeholders) by construction rather than by discipline: an unavailable
capability renders nothing because there is nothing to render.

### Mandatory and conditional

An item naming a command that is not registered has two entirely different
meanings, and the manifest has to say which:

| Item | Command registered? | Result |
|---|---|---|
| mandatory (`capability` is `None`) | no | hard validation failure — a programming error |
| conditional (`capability` is `Some`) | no | dropped, `SkipReason::CapabilityAbsent` — informational |
| either | yes | rendered |

`CapabilityAbsent` is a *different* reason from `UnknownCommand` precisely so the
two never get confused in a log: one says "this build does not include that", the
other says "someone made a mistake". Without the distinction, modularity and a
typo are the same event, and the only ways to handle them are to block start-up
on a legitimate lite build or to swallow a real bug on every machine that runs
it.

`merge::prune_absent_capabilities` runs over the whole merged shell — tabs,
contextual tabs and the trailing region — **including the built-in layer**. That
is the one deliberate exception to §6's rule, and it narrows rather than weakens
it: only conditional items are dropped. A mandatory built-in item with a missing
command is untouched here and still reaches `validate_against` to fail loudly.
Every skip this pass reports names `BuiltIn`, by construction: an overlay's items
were already filtered as that layer was applied, with their own layer on the
skip.

**Empty groups and empty tabs are left where they are.** A group with no items
does not render — a rendering rule the renderer already honours. Deleting the
group here would also destroy the operator's customization of it, which would
come back blank the day they installed a build that has the capability again. An
empty group is invisible; a deleted one is forgotten.

The capability string is **never matched against anything**. The shell holds no
list of known capability names and has no way to ask whether one is available.
The only question it asks is the one it already asked — *is a command with this
id in the registry?* — and the name is carried so the skip report can say what
was missing. A field the shell interpreted would be a second place that knows.

### Cargo features are the mechanism today

Removing a feature removes the dependency, the code, and — by the rule above —
the ribbon item, at zero new machinery. `pdfcer-core`'s strippable-capability
convention is the established form: an optional dependency named by a feature,
forwarded through every intermediate crate, with CI building
`--no-default-features` to prove the stripped build is real. A capability that
silently disappears from a default build is a regression wearing a feature flag.

True drop-in modules need an ABI boundary, and Rust has no stable ABI: a
`cdylib` cannot expose Rust types across it safely. The options are an
`extern "C"` interface with a hand-written vocabulary of plain-data types, or a
crate that provides one. Either is a real project, and it constrains what a
capability may exchange with the host. What it would **not** have to include is
any GUI work, provided the rule above holds: loading a module at start-up and
calling its `register(&mut CommandRegistry)` is the same act as calling a linked
module's, and the shell cannot tell the difference.

The guard on all of this is
`shell::tests::the_built_in_manifest_survives_a_build_without_an_optional_capability`,
which merges against a registry with the conditional commands withheld and
asserts the shell still validates. Without it, a build compiled without an
optional capability loses its entire ribbon: the built-in manifest fails strict
validation, the shell is `None`, and `Capabilities::for_mode` grants FULL when the
shell is absent — a lite build with no ribbon and every permission.

## 8. Validation

Two calls, and the split is the difference between a rejection and a disclosure.

`Shell::validate` is **strict** and runs on the merged result. What it rejects
are contradictions no fail-soft rule can repair:

| Error | Condition |
|---|---|
| `UnsupportedSchema` | The document declares a schema newer than this build. |
| `DuplicateTabId`, `DuplicateGroupId`, `DuplicateModeId` | An id must resolve to one thing. |
| `MissingTabLabel`, `MissingModeLabel` | A complete manifest must be renderable. |
| `MissingTabGroups` | No `groups` key at all — that is a reference to a tab, not a tab. |
| `MissingGroupCaption` | An uncaptioned band is a row of controls whose relationship the operator has to infer. |
| `CommandOnTwoTabs` | The one-command-one-tab rule. |

`Shell::validate_against(catalog)` runs `validate`, then resolves every command
reference. `command_references` walks, in stable document order: tabs (ordinary
then contextual), the QAT, the trailing region, the rail, and the keymap. A typo
in the trailing region or the rail is a start-up failure exactly as a typo in the
QAT is — it matters most in the rail, which is permanent chrome, where a
mis-typed id costs a hole on screen for the whole session rather than one absent
button on one tab.

**Menus are deliberately outside the one-command-one-tab rule and outside
`command_references`.** The rule exists so a command has one discoverable *home*;
a menu is a shortcut to that home rather than a rival to it. The same reasoning
permits the QAT, the trailing region and the rail to mirror a tab's command.

The uniqueness test runs against the merged manifest, so a customization that
puts one command on two tabs is rejected at load with a message naming the
command — which no compile-time test could do for a user-supplied layout.

## 9. What customization permits

| Change | Verdict | Why |
|---|---|---|
| Reorder tabs, rename them, hide them | allowed | Presentation. |
| Move a command between groups or tabs | allowed | Presentation. |
| Create a custom tab or group | allowed | Presentation. |
| Rebind keys | allowed | Presentation. |
| Define a new mode with its own tab set | allowed | Modes are configuration. |
| Save, load or share a named workspace | allowed | Covers the ribbon **and** the panel layout. |
| Invent a command | refused | Commands come from the registry. |
| Change what a command does | refused | Behaviour is code. |
| Bypass a command's enable predicate | refused | Predicates are safety, not decoration. |

## 10. Layout persistence

`layout::LayoutDocument` holds the active `DockLayout` and the named workspaces,
round-trips through RON, and loads **fail-soft per item**: a malformed or stale
entry costs that entry, never the arrangement.

| Condition | Outcome | Reason |
|---|---|---|
| No file | the default arrangement | `FileMissing` — a first run is not a failure |
| Unreadable file | the default arrangement | `Unreadable` |
| Broken syntax | the default arrangement | `ParseFailed` |
| A newer schema | the default arrangement | `UnsupportedSchema` |
| A panel the application does not register | that tab dropped | `UnknownPanel` |
| A panel mounted twice | the second mount dropped | `DuplicatePanel` |
| A compartment left with no tabs | that compartment dropped | `EmptyContainer` |
| A share or width that is not a number | replaced with a default | `InvalidSize` |
| An active index past the end | the last tab selected | `ActiveOutOfRange` |
| One unparseable workspace | that workspace dropped, the others kept | `WorkspaceName` and per-item reasons |

`UnknownPanel` is how a compiled-out capability's saved panel becomes a disclosed
skip rather than an empty compartment — the §7 rule reaching the layout file.

`ResetScope` is `Left`, `Right` or `All`, offered narrowest first, because a
destructive command's least destructive form should be the one an operator
reaches by accident. `All` still does not mean everything: saved workspaces
survive it.

## 11. Extraction

`egui-shell` is built *as* its first consumer is built, never ahead of it: a
framework designed without a consumer gets the abstractions wrong.

The boundary is only real if something outside this workspace compiles against
it. The test is a throwaway second application — a few hundred lines, a different
domain, three tabs and two panels — written against `egui-shell` alone. If it
needs one line of the application crate, the boundary is wrong and gets fixed
then.

Until that exists, the purity gate is the standing mechanism:

```sh
bash tools/gates/check-shell-purity.sh
cargo test -p egui-shell
```
