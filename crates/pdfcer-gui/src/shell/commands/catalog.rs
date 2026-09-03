//! # `shell::commands::catalog` — the list itself, and the argument for every
//! entry on it
//!
//! One function, [`all`], holding every command this build has in manifest
//! order, and one helper, [`command`], that builds each from a catalog entry.
//! Nothing else.
//!
//! ## ★ Why this is its own file
//!
//! `shell/commands/mod.rs` crossed the 1,500-line gate (standing rule **R2**)
//! when `file.save_copy` was wired. The rule's own justification is why the
//! split is *here* rather than at whichever line the count happened to reach:
//! *"the value of the limit is that the file has to have a single subject"*.
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
//! It is the same seam `commands.rs` was already split along once, producing
//! [`super::mapping`] (the id ↔ operand bindings), and the same seam
//! `app/mod.rs` has been split along four times. The test for whether a split
//! was along a seam is whether the *reasoning* came with it, and it did: every
//! paragraph moved here is an argument about a registration.
//!
//! ## ★ The flat list survives the split, and that is the point
//!
//! [`all`]'s own doc comment has always refused a function per tab, because
//! *"a per-tab split would put the handler-token blocks in eight files where a
//! collision between two of them is invisible"*. That argument is untouched:
//! the list is still **one function in one file**, in manifest order, with all
//! nine hundred-blocks visible together and
//! `super::tests::every_handler_token_is_unique` still reading the whole
//! registry. What moved out is the registry's *contract*, not the list.
//!
//! Splitting the list instead would have been the cheaper edit and the wrong
//! one — it would have satisfied a line count by breaking the one property the
//! list's shape exists to protect.
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
//! ## Coverage — ★ the numbers live in the TEST, not in this sentence
//!
//! This heading read *"as of 2026-08-14: 86 of 101 named, 15 refused"* until
//! 2026-08-18, and every one of those three numbers was wrong: the registry
//! held 94, of which 85 were named and 9 refused. `86 + 15 = 101` is
//! internally consistent, which is why nobody looked twice.
//!
//! That is the **fifth** drift of this pair, and the four before it are
//! recorded on `super::tests::the_icon_coverage_split_adds_up_to_the_registry`
//! — which was added, after the fourth, to stop exactly this. It did not,
//! because it pins the split against its own literals and this sentence was
//! never one of them. A test that says *"update the header together"* is a
//! note asking a human to do the thing they just failed to do.
//!
//! So the sentence no longer carries numbers. `run --lib
//! the_icon_coverage_split_adds_up_to_the_registry` prints the current pair
//! and fails when it moves, which is a claim that cannot be stale by
//! construction. This project has now taken the same repair three times — the
//! gate runner's header, `README.md`'s test count, and this — and the shape is
//! always the same: **when prose and a measurement disagree, delete the prose's
//! copy of the measurement rather than correcting it.**
//!
//! What the sentence is still for: before 2026-08-14 the split was 47 named
//! and 41 not, with **no rule behind which was which** — a band drew glyphs and
//! bare words side by side, and the ribbon read as half-finished because it
//! was. Thirty commands gained a key in that pass (25 new glyphs plus two
//! reuses of `chevron-up`/`chevron-down` for page reordering, which is the
//! meaning `crate::icons::Icon::ChevronUp`'s doc comment already gave them).
//!
//! The rest are **recorded refusals**, each stated in full at its own
//! registration below and summarised in `crate::icons::assets` §5 deviation
//! #8. In one line each:
//!
//! | Command(s) | Why no glyph |
//! |---|---|
//! | `view.zoom_actual` | the icon ui-spec §3.2 argues against it by name |
//! | the five `view.render_*` | their labels are the parameter's whole content; no conventional glyph exists for any of them |
//! | `view.app_initiative` | any honest drawing pictures what its default forbids |
//! | `file.recent` | reusing `open` would draw one band control twice |
//! | `mode.read`/`review`/`edit` | the mode selector renders text segments and has no icon path |
//! | `measure.finish` | the set has no check/tick/accept glyph, and the `measure` ruler the three tools share would draw a fourth identical one for a command that places nothing |
//! | `markup.finish` | the same refusal, one tab over: no accept glyph exists, and reusing a shape glyph would draw a fourth near-identical shape in the Shapes band for a command that ends the drawing rather than doing any |
//! | `file.new` | the same refusal as `file.ocr` below, and for the same reason: the icon directory is declared the operator's **own art**, so a new glyph is not a build session's to add. Every reuse was worse than the word — `document` is Properties, `insert-pages` means *pages into this document*, `upload` is import |
//! | `file.ocr` | **the refusal with a different reason from all the others**, and worth reading: there is no recognition glyph and every reuse would mislead (`text-select` is the text *tool*, `search` is Find, `convert` is a format change), but the deciding fact is that the alternative is not available either — `icons/assets/PROVENANCE.md` declares that directory the **operator's own art**, which is what exempts it from `check-shipped-assets`, and adding a machine-drawn SVG would make that provenance note false. A false provenance note is a worse defect than a control that draws its own words |
//!
//! ★ …and moved a third time on 2026-08-14, when the three text-markup kinds
//! were registered **with** three new glyphs (`text-underline`,
//! `text-strikeout`, `text-squiggly`): 79-of-90 became 82-of-93, and the refusal
//! count is unchanged because none of the three refused one. They are new art
//! rather than a reuse of `shape-highlight` for the reason their registration
//! records: the four controls in the Text markup band differ only in the mark
//! they draw, so a shared glyph would leave four identical buttons carrying four
//! different words.
//!
//! ★ The counts above moved twice on 2026-08-14 and the second move is the
//! one to notice: `measure.two_line` was registered **with** a glyph and this
//! line was not updated, so it read 77-of-88 while the registry held 89. A
//! count quoted in prose is not pinned by the test that pins the registry —
//! `registration_succeeds_and_registers_every_command` would have stayed
//! green through any drift here. Both are corrected together.
//!
//! ★ …and a **fourth** move, later the same day, when `view.tool_text` was
//! registered with the new `text-select` glyph: 82-of-93 became 82-of-94 and the
//! refusal count stayed at twelve, because the text tool refused nothing.
//!
//! ★ **Fifth, later still on 2026-08-14**: the three unblocked Phase 6 markup
//! kinds — `markup.polyline`, `markup.polygon` and `markup.ink` — arrived **with**
//! three new glyphs, and `markup.finish` arrived **without** one, refusing it on
//! `measure.finish`'s own argument. 82-of-94 with twelve refusals became
//! **85-of-98 with thirteen**. This is the first of the five moves that was made
//! with the arithmetic under test rather than under advice: the split is now
//! pinned by `super::tests::the_icon_coverage_split_adds_up_to_the_registry`, so the
//! numbers in this section and the numbers in the registry cannot drift apart
//! silently again.
//!
//! ★ **That fourth pass also found the third line above to have been wrong**,
//! and it is worth stating rather than silently repairing, because it is the
//! same defect that line was written to record. It read *"82 of 93 named, 12
//! refused"* — and 82 + 12 is 94, not 93. The registry held 93 and **81** of
//! them named a glyph; the prose had been incremented one step too far in the
//! text-markup pass. Nothing detected it, for exactly the reason that pass
//! wrote down: the test pins the registry's size and nothing pins the split.
//! The arithmetic check that would have caught it — *named + refused must equal
//! the registry* — is the one property worth carrying forward here, and it is
//! now asserted by
//! `super::tests::the_icon_coverage_split_adds_up_to_the_registry` rather than being
//! left to a reader to do in their head.
//!
//! ★ **Sixth, 2026-08-14: `file.about` arrived with the new `info` glyph**,
//! making it **86-of-99 with thirteen** — About refused nothing, a circled `i`
//! being the most conventional glyph any toolbar has. First move made with the
//! arithmetic already under test; it cost one number in one assertion, which is
//! what the five paragraphs above were for. **Three of the four "count in prose"
//! incidents this module records were found by hand; the fourth was found by
//! the first three's own advice, which is why the advice is now a test.**
//!
//! **A band control's icon does not replace its label.**
//! `egui_shell::ribbon::band::command_button` is called with
//! `shows_label: true` from the band, always; only the QAT goes icon-only,
//! and only `file.open`, `file.save_copy`, `edit.undo` and `edit.redo` are
//! on it. Two of the notes retired in this pass had reasoned as though the
//! choice were "a glyph *or* a findable word", and it never was. That
//! misreading is worth keeping written down: it is what kept three Display
//! toggles and the Pages panel bare for longer than any decision did.

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
/// # ★★★ This was one flat list until 2026-08-28, and the reason it stopped
/// being one is worth more than the split
///
/// The paragraph that stood here read:
///
/// > One flat list rather than a function per tab: the registry is a flat
/// > namespace, the ordering here mirrors the ribbon so the two can be read
/// > side by side, and **a per-tab split would put the handler-token blocks in
/// > eight files where a collision between two of them is invisible.**
///
/// The first two clauses are still true and are preserved by the concatenation
/// below: one namespace, ribbon order, readable against §5 side by side.
///
/// **The third clause was already false when it was written.**
/// [`super::tests::every_handler_token_is_unique`] sweeps the whole registry
/// and [`super::tests::every_handler_token_is_in_its_tabs_block`] asserts each
/// token sits inside its own tab's hundred. A collision is not invisible in
/// either arrangement — it is a red test — so the argument that kept 120
/// commands and their prose in one 1,495-line file rested on a property two
/// tests had already taken over.
///
/// ⇒ Recorded rather than quietly reversed. It is the shape this project keeps
/// finding: **a reason that was true when written, is re-read by nobody, and
/// outlives the condition that made it true.** The seventh instance this month.
///
/// # What forced the question
///
/// The Attachments command took this file to **1,495 of R2's 1,500 lines**, so
/// the next command registered would have broken the gate. That is the file-size
/// gate working exactly as its own header says it should — *"when a file
/// approaches the limit, that is the signal to find the seam, not to raise the
/// limit"* — and the seam it found was one an old comment had ruled out.
pub(super) fn all() -> Vec<Command> {
    // ★ One band per tab, concatenated in ribbon order.
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
    out.extend(measure::band());
    out.extend(tools::band());
    out.extend(format::band());
    out.extend(modes::band());
    out
}

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
