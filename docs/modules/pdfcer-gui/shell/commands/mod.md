# shell::commands — every verb pdfcer can perform

[`register`] populates an `egui_shell::CommandRegistry` with every command
this build has. They reach the operator by three routes:

| route | how the operator reaches it |
|---|---|
| on a tab, the QAT or the keymap | a control [`super::manifest::built_in`] names by id |
| drawn by a **custom item** | `file.recent` and the Format ▸ Font controls — see [`super::manifest::CUSTOM_BACKED`] |
| drawn on the **status bar** | `edit.find` — `RIBBON_IA.md` §6 |

⇒ **Do not write a command count into this prose.** The count lives in
[`ledger`]'s assertions, which move with the catalogue; a number in a
sentence does not, and a test cannot read prose, so the two diverge
silently and the sentence is the one a reader believes.

The last two routes are the interesting ones, because both are reachable by
an operator and neither is a button on a tab. They are kept honest by
different mechanisms and the difference is worth knowing: a `Custom` item
carries no command id, so `Shell::command_references()` cannot see
`file.recent` at all and [`super::manifest::CUSTOM_BACKED`] is the
register that says why that is allowed; whereas the status bar is simply
not part of the manifest, and `edit.find` needs no exemption because the
keymap's `Ctrl+F` binding **is** a reference site — so the orphan check
still guards it against a rename.

A command carries five things:

| Field | Comes from | Why here rather than the manifest |
|---|---|---|
| `id` | this file | the manifest may only *reference* it |
| `label`, `tooltip` | [`crate::text::commands`] | copy is a design surface with one owner |
| `icon` | this file, as a key | the icon *set* is the application's; the shell only needs to know which one |
| `enable` | this file, as a condition name | *"predicates are safety, not decoration"* |
| `handler` | this file, as an opaque `u64` | the shell never interprets it |

`SHELL_FRAMEWORK.md` §5 turns that split into the customization
contract. An operator may reorder tabs, rename them, hide them, move a
command between groups, create tabs, rebind keys, and define new modes.
An operator may **not** invent a command, change what a command does,
or bypass an enable predicate. Every one of those prohibitions is a
consequence of this half being code and the other half being data.

# Handler tokens are opaque, and this file does not implement behaviour

An `egui_shell::HandlerToken` is a `u64` the shell stores and hands back
when the command is invoked. The application dispatches on it at **one
choke point**, which is where a confirmation gate, an undo entry or a
trace belongs; a registry of closures would scatter that across as many
sites as there are commands, and would force the shell to name pdfcer's
state type, which would end its reusability.

The numbers are assigned here in blocks of one hundred, one block per
tab, and they are **stable**: a token is never reused for a different
command, because a persisted or traced token that silently changed
meaning between builds is a defect with no symptom at the site that
caused it. Gaps in the numbering are fine and expected — a command
removed leaves its number unused.

# Enable conditions

`Enable::When("doc.open")` names a condition the application publishes
once per frame in an `egui_shell::commands::ConditionSet`. Data rather
than a closure, because a name is serializable, testable headlessly,
and cannot capture state that makes a command's availability depend on
*when* it was registered.

Every condition name is a promise the application has to keep, and the
authoritative list of them is `tests::KNOWN`, which the registration test
checks every `Enable::When` against. The table below explains the ones
whose *shape* is not obvious — it is not the inventory, and must not be
read as one.

| Condition | True when | Used by |
|---|---|---|
| *(none)* | always | commands with no precondition: Open, Settings, the batch tools, the window and render settings |
| `docs.multiple` | **more than one document is open** | Next / Previous document |
| `panels.floating` | **some panel is in a window of its own** | Dock all panels |
| `doc.open` | a document is open | document-level commands — close, save a copy, properties, print |
| `doc.pages` | …and it has at least one page | everything that acts on a page |
| `undo.available` / `redo.available` | the corresponding stack is non-empty | Undo, Redo |
| `selection.any` | something is selected | the contextual Format tab and its Delete |
| `selection.bounds` | …and it still resolves to a box on the page shown | Zoom to selection |

`docs.multiple` is deliberately **not** a refinement of `doc.open`, and it
is published outside that arm. A tab whose file failed to open is still a
tab (`crate::app::documents` §2), so an operator can be sitting on a
damaged file — `doc.open` false — with three good documents behind it, and
that is the moment they most need a way back to one of them. Nesting the
condition would grey the only route out of a failed open.

`selection.bounds` is separate from `selection.any` for the same shape
of reason, one level down. A selection here is an **identity** — page,
object, subpath, node — and an identity can outlive the box it once
described: it may name an object on a page that is not shown, or one an
edit has renumbered. Zoom to selection is the command where that gap is
visible, because framing nothing is not a no-op; it is a jump to the
origin that looks exactly like a bug.

`doc.pages` is separate from `doc.open` because **a PDF with `/Count 0`
is a legal document**. pdfcer opens it, shows "This document has no
pages", and must not offer to rotate one. Collapsing the two would make
that file arm tools that cannot run — the exact class of failure the
removal of the `Editing on` master toggle was meant to end.

Greying is what a false predicate produces, and P3 permits it only for
*temporarily* unavailable, *always explained on hover*. Every command
here has a tooltip; [`crate::text::commands`] has no way to express a
command without one.

# Where the list itself lives

[`catalog::all`]. The seam is the one this header already draws: everything
above is the **contract** (what a command is, what a token means, what
conditions the application promises to publish), and everything in
[`catalog`] is the **list** (which commands exist, with which glyph and
which predicate, and why each of those was chosen). The icon-coverage
argument lives beside the registrations it justifies.

The list is **one flat function in one file** — see [`catalog`]'s header
for why splitting *that* would be the cheaper edit and the wrong one.
