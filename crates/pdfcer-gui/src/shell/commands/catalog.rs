//! # `shell::commands::catalog` — the list itself, and the argument for every
//! entry on it
//!
//! One function, [`all`], holding every command this build has in manifest
//! order, and one helper, [`command`], that builds each from a catalog entry.
//! Nothing else.
//!
//! ## Why this is its own file
//!
//! A file has to have a single subject, and the registry has two.
//!
//! The parent's subject is **the registry contract** — what a command is, why
//! its five fields are split between code and manifest, what a handler token
//! means, what vocabulary of enable conditions the application promises to
//! publish, and what must be true of the finished registry. This file's
//! subject is **the catalog**: which commands exist, in what order, with which
//! glyph and which predicate, and *why each of those was chosen*. The two
//! change for entirely different reasons — a new condition name is a parent
//! change, a new command is a change here — and they are read at different
//! times.
//!
//! It is the same seam [`super::mapping`] (the id ↔ operand bindings) sits on.
//! The test for whether a split is along a seam is whether the *reasoning* came
//! with it: every paragraph here is an argument about a registration.
//!
//! ## One registry, assembled from one band per tab
//!
//! [`all`] concatenates a `band()` per tab module in manifest order, so each
//! tab's registrations and the arguments behind them read together while the
//! **registry stays flat**: one namespace, one order, one list for
//! `super::tests::every_handler_token_is_unique` to sweep.
//!
//! ⚠ **The per-tab arrangement does not hide a handler-token collision**, and
//! [`all`]'s own doc says why — two tests take that property, so it is never an
//! argument for or against where the registrations live.
//!
//! # Icons
//!
//! A `String` key resolved by the application's icon set, not a texture:
//! icon rendering is a licensing and rasterization decision that belongs
//! to pdfcer, and the shell only needs to know that a control has an icon
//! and which one. The keys are the ones the salvaged icon set already
//! uses (`open`, `rotate-ccw`, `shape-rect`, …).
//!
//! Two conventions worth stating, because both look like mistakes:
//!
//! - **Keys are shared.** `copy` is on both text-copy commands, `redact`
//!   on both redaction commands, `measure` on both dimension tools,
//!   `export` on both export verbs, `delete` on both delete verbs, `list`
//!   on both "manage a list of things" dialogs. A family of related
//!   commands sharing a glyph is how a ribbon reads as grouped; uniqueness
//!   is a property of ids, not of icons.
//! - **`None` is a real answer.** A command with no key renders as text.
//!   Every icon is a drawing somebody has to make, and inventing a key for
//!   an icon that does not exist would produce a missing-glyph box at run
//!   time — a placeholder, arriving through the back door.
//!
//! ## Coverage — the numbers live in the TEST, not in this sentence
//!
//! **No count of named-versus-refused commands is written here.** The split is
//! pinned by `super::tests::the_icon_coverage_split_adds_up_to_the_registry`,
//! which prints the current pair and fails when it moves; a copy of it in prose
//! is a claim nothing checks, and `registration_succeeds_and_registers_every_command`
//! stays green through any drift in it. The general rule, which this project
//! applies in three places: **when prose and a measurement disagree, delete the
//! prose's copy of the measurement rather than correcting it.**
//!
//! What that test asserts is the one property worth holding — *named + refused
//! must equal the registry* — rather than either number on its own.
//!
//! ## Which commands refuse a glyph, and why
//!
//! A refusal is stated in full at its own registration below and summarised in
//! `crate::icons::assets` §5 deviation #8. In one line each:
//!
//! | Command(s) | Why no glyph |
//! |---|---|
//! | `view.zoom_actual` | the icon ui-spec §3.2 argues against it by name |
//! | the five `view.render_*` | their labels are the parameter's whole content; no conventional glyph exists for any of them |
//! | `view.app_initiative` | any honest drawing pictures what its default forbids |
//! | `mode.read`/`review`/`edit` | the mode selector renders text segments and has no icon path |
//! | `edit.select_all` | no comparable program draws one, and a marquee would say "rubber band" |
//!
//! ⚠ **Two kinds of refusal, and only one of them can be discharged by art.**
//!
//! - **Supply.** `icons/assets/PROVENANCE.md` declares that directory the
//!   operator's **own art**, which is what exempts it from
//!   `check-shipped-assets`. A machine-drawn SVG dropped into it makes the note
//!   false, and **a false provenance note is a worse defect than a control that
//!   draws its own words** — so the alternative to reusing a neighbour's glyph
//!   is never "draw one", it is "ask him for one". A command refused on this
//!   ground gains a glyph the day the art arrives.
//! - **Argument.** `view.zoom_actual` and `edit.select_all` are refused on
//!   grounds no amount of art discharges, and they must stay in the table.
//!
//! ⇒ Where a refusal names a reuse that would mislead — `document` is
//! Properties, `upload` is import, `search` is Find, the `measure` ruler
//! belongs to Linear — that constraint **survives** any later art: the new
//! drawing has to stay distinguishable from exactly the neighbours the refusal
//! named.
//!
//! ⚠ **Shared art is deliberate; identical art for controls that differ is
//! not.** The four controls in the Text markup band differ only in the mark
//! they draw, so `text-underline`, `text-strikeout` and `text-squiggly` are
//! separate drawings rather than four reuses of `shape-highlight` — which would
//! leave four identical buttons carrying four different words.
//!
//! **A band control's icon does not replace its label.**
//! `egui_shell::ribbon::band::command_button` is called with
//! `shows_label: true` from the band, always; only the QAT goes icon-only, and
//! only `file.open`, `file.save`, `edit.undo` and `edit.redo` are on it. So the
//! choice a refusal makes is never "a glyph *or* a findable word" — the word is
//! there either way, and what a missing glyph costs is recognition at a glance,
//! not reachability. Reasoning as though the label were at stake is what keeps
//! a control bare longer than any decision would.

use crate::text::commands::CommandText;
use egui_shell::{Command, HandlerToken};

/// One command, with its label and tooltip taken from the catalog.
///
/// The two are always fetched together, from one catalog entry, so a
/// command cannot end up with one command's label and another's tooltip —
/// which is not a hypothetical: the salvage source's two adjacent Content
/// buttons both read `Aa`, and only their tooltips distinguished them.
pub(super) fn command(id: &str, text: CommandText, handler: u64) -> Command {
    Command::new(id, text.label, HandlerToken::new(handler)).with_tooltip(text.tooltip)
}

/// Every command, in manifest order.
///
/// # Why the registry is assembled from one band per tab
///
/// The registry is a flat namespace and this ordering mirrors the ribbon, so
/// the two can be read against `RIBBON_IA.md` §5 side by side — and the
/// concatenation below preserves both: one namespace, ribbon order.
///
/// ⚠ **A per-tab split does NOT hide a handler-token collision**, which is the
/// argument most likely to be raised against it.
/// [`super::tests::every_handler_token_is_unique`] sweeps the whole registry
/// and [`super::tests::every_handler_token_is_in_its_tabs_block`] asserts each
/// token sits inside its own tab's hundred. A collision is a red test in either
/// arrangement, so it is not a reason to keep every command and its prose in
/// one file.
pub(super) fn all() -> Vec<Command> {
    // One band per tab, concatenated in ribbon order.
    //
    // The order is the ribbon's own and it is load-bearing for exactly one
    // reason: `egui_shell` renders a group's items in the order the manifest
    // names them, not in registry order, so this sequence decides nothing about
    // the ribbon — but it decides what a reader of `--all` sees, and a
    // catalogue that listed Format before File would be a second ordering for
    // somebody to reconcile against §5.
    let mut out = Vec::new();
    out.extend(file::band());
    out.extend(view::band());
    out.extend(pages::band());
    out.extend(edit::band());
    out.extend(markup::band());
    // The Markup tab's second band. A group rather than a tab — the ids stay
    // `markup.*`; see that file's header for why the file is named for the group
    // and the ids for the tab.
    out.extend(arrange::band());
    out.extend(measure::band());
    out.extend(tools::band());
    out.extend(format::band());
    out.extend(modes::band());
    out
}

/// the Markup tab's Arrange group — which mark is drawn on top
mod arrange;
/// the Edit tab — changing content that is already there
mod edit;
/// the File tab — opening, saving, exporting, printing, and pdfcer itself
mod file;
/// the Format contextual tab — what changes about the selection
mod format;
/// the Markup tab — what is added for somebody else to read
mod markup;
/// the Measure tab — ce dimensions and the scale they are read at
mod measure;
/// the mode selector — Read, Review, Edit
mod modes;
/// the Pages tab — what happens to the set of sheets
mod pages;
/// the Tools tab — what runs across files, or is configured once
mod tools;
/// the View tab — what is on screen and how the page is laid out
mod view;
