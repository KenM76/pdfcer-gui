//! # shell::commands — every verb pdfcer can perform
//!
//! [`register`] populates an `egui_shell::CommandRegistry` with the
//! **hundred and twenty-one** commands this build has, which fall into three
//! groups:
//!
//! | group | count | how the operator reaches it |
//! |---|---|---|
//! | on a tab, the QAT or the keymap | 116 | a control [`super::manifest::built_in`] names by id |
//! | drawn by a **custom item** | 4 | `file.recent` and the three Format ▸ Font controls — see [`super::manifest::CUSTOM_BACKED`] |
//! | drawn on the **status bar** | 1 | `edit.find` — `RIBBON_IA.md` §6 |
//!
//! ★★★ **And it drifted a fifth and a sixth time, both found on 2026-08-27,
//! and the sixth is the one worth reading.** This table said *hundred and one*
//! / 99 / 1 / 1 while
//! [`tests::registration_succeeds_and_registers_every_command`] asserted
//! **115** and passed — nineteen commands out, which is not a rounding error,
//! it is a table describing a different program. The pinned number moved
//! nineteen times and the sentence above it moved never, because a test cannot
//! read prose.
//!
//! The sixth is the same defect one level down: the *custom-item* row said 1
//! for as long as `file.recent` was the only such command, and the Format ▸
//! Font group added three more. That row is the one an engineer would trust
//! when asking *"can a command live off the manifest?"*, and it would have
//! answered "yes, once, exceptionally" about a build where the answer is "yes,
//! four times, and there is a register listing them."
//!
//! ⇒ **Do not trust a count in this file without re-reading the assertion it
//! is supposed to mirror.** That instruction has now been earned six times and
//! the running tally is left in place — see the two paragraphs below — because
//! the tally is the argument.
//!
//! (This header said *eighty-one* and *79* until 2026-08-14, while
//! [`tests::registration_succeeds_and_registers_every_command`] asserted 88
//! and passed. Prose drifting from a number a test pins is a defect this
//! project has now had three times; the test is the fact, and the table has
//! been corrected to it rather than the other way round.)
//!
//! ★ **And it happened a fourth time, in the four hours before `file.new` was
//! written.** This table read *ninety-nine* and *97* while the assertion below
//! held **100**: `file.ocr` had been registered and the two sentences above it
//! were not moved. It is recorded rather than quietly repaired for the reason
//! the paragraph above it was: the count is the one thing here that has drifted
//! every single time, and the running tally is the argument for why nobody
//! should trust a number in this file without re-reading the assertion. Both
//! were corrected together when `file.new` took the total to 101.
//!
//! The last two are the interesting ones, because both are reachable by an
//! operator and neither is a button on a tab. They are kept honest by
//! different mechanisms and the difference is worth knowing: a `Custom` item
//! carries no command id, so `Shell::command_references()` cannot see
//! `file.recent` at all and [`super::manifest::CUSTOM_BACKED`] is the
//! register that says why that is allowed; whereas the status bar is simply
//! not part of the manifest, and `edit.find` needs no exemption because the
//! keymap's `Ctrl+F` binding **is** a reference site — so the orphan check
//! still guards it against a rename.
//!
//! A command carries five things:
//!
//! | Field | Comes from | Why here rather than the manifest |
//! |---|---|---|
//! | `id` | this file | the manifest may only *reference* it |
//! | `label`, `tooltip` | [`crate::text::commands`] | copy is a design surface with one owner |
//! | `icon` | this file, as a key | the icon *set* is the application's; the shell only needs to know which one |
//! | `enable` | this file, as a condition name | *"predicates are safety, not decoration"* |
//! | `handler` | this file, as an opaque `u64` | the shell never interprets it |
//!
//! `SHELL_FRAMEWORK.md` §5 turns that split into the customization
//! contract. An operator may reorder tabs, rename them, hide them, move a
//! command between groups, create tabs, rebind keys, and define new modes.
//! An operator may **not** invent a command, change what a command does,
//! or bypass an enable predicate. Every one of those prohibitions is a
//! consequence of this half being code and the other half being data.
//!
//! # Handler tokens are opaque, and this file does not implement behaviour
//!
//! An `egui_shell::HandlerToken` is a `u64` the shell stores and hands back
//! when the command is invoked. The application dispatches on it at **one
//! choke point**, which is where a confirmation gate, an undo entry or a
//! trace belongs; a registry of closures would scatter that across as many
//! sites as there are commands, and would force the shell to name pdfcer's
//! state type, which would end its reusability.
//!
//! The numbers are assigned here in blocks of one hundred, one block per
//! tab, and they are **stable**: a token is never reused for a different
//! command, because a persisted or traced token that silently changed
//! meaning between builds is a defect with no symptom at the site that
//! caused it. Gaps in the numbering are fine and expected — a command
//! removed leaves its number unused.
//!
//! # Enable conditions
//!
//! `Enable::When("doc.open")` names a condition the application publishes
//! once per frame in an `egui_shell::commands::ConditionSet`. Data rather
//! than a closure, because a name is serializable, testable headlessly,
//! and cannot capture state that makes a command's availability depend on
//! *when* it was registered.
//!
//! Seven conditions are used, and the whole vocabulary is listed here
//! because every one of them is a promise the application has to keep:
//!
//! | Condition | True when | Used by |
//! |---|---|---|
//! | *(none)* | always | commands with no precondition: Open, Settings, the batch tools, the window and render settings |
//! | `docs.multiple` | **more than one document is open** | Next / Previous document |
//! | `panels.floating` | **some panel is in a window of its own** | Dock all panels |
//! | `doc.open` | a document is open | document-level commands — close, save a copy, properties, print |
//! | `doc.pages` | …and it has at least one page | everything that acts on a page |
//! | `undo.available` / `redo.available` | the corresponding stack is non-empty | Undo, Redo |
//! | `selection.any` | something is selected | the contextual Format tab and its Delete |
//! | `selection.bounds` | …and it still resolves to a box on the page shown | Zoom to selection |
//!
//! `docs.multiple` is deliberately **not** a refinement of `doc.open`, and it
//! is published outside that arm. A tab whose file failed to open is still a
//! tab (`crate::app::documents` §2), so an operator can be sitting on a
//! damaged file — `doc.open` false — with three good documents behind it, and
//! that is the moment they most need a way back to one of them. Nesting the
//! condition would grey the only route out of a failed open.
//!
//! `selection.bounds` is separate from `selection.any` for the same shape
//! of reason, one level down. A selection here is an **identity** — page,
//! object, subpath, node — and an identity can outlive the box it once
//! described: it may name an object on a page that is not shown, or one an
//! edit has renumbered. Zoom to selection is the command where that gap is
//! visible, because framing nothing is not a no-op; it is a jump to the
//! origin that looks exactly like a bug.
//!
//! `doc.pages` is separate from `doc.open` because **a PDF with `/Count 0`
//! is a legal document**. pdfcer opens it, shows "This document has no
//! pages", and must not offer to rotate one. Collapsing the two would make
//! that file arm tools that cannot run — the exact class of failure the
//! removal of the `Editing on` master toggle was meant to end.
//!
//! Greying is what a false predicate produces, and P3 permits it only for
//! *temporarily* unavailable, *always explained on hover*. Every command
//! here has a tooltip; [`crate::text::commands`] has no way to express a
//! command without one.
//!
//! # Where the list itself lives
//!
//! [`catalog::all`] — its own file since 2026-08-14, when wiring
//! `file.save_copy` took this one past the 1,500-line gate (**R2**). The seam
//! is the one this header already draws: everything above is the **contract**
//! (what a command is, what a token means, what conditions the application
//! promises to publish), and everything in [`catalog`] is the **list** (which
//! commands exist, with which glyph and which predicate, and why each of those
//! was chosen). The icon-coverage argument moved with the registrations it
//! justifies, because an argument belongs beside the thing it argues for.
//!
//! The list is still **one flat function in one file** — see [`catalog`]'s
//! header for why splitting *that* would have been the cheaper edit and the
//! wrong one.

pub mod catalog;
pub mod mapping;

/// ★ **The sixth obligation, and the only one the five above cannot express:
/// a registered command must be REACHABLE by some arm of `app::dispatch`.**
///
/// `HANDOFF.md` §5's five obligations are all about the *registration* being
/// consistent — a count, a group count, a `PLANNED` removal, a RON
/// regeneration, a `KNOWN` condition name. Every one of them was satisfied by
/// `file.save_copy` on the day it was drawn on the quick-access toolbar, bound
/// to `Ctrl+S`, printed "(Ctrl+S)" in its own tooltip, and **did nothing**,
/// because no dispatch arm existed. [`reach`] is the assertion that closes
/// that: every id in this registry is routed by a literal arm, claimed by a
/// guard arm, or listed in [`reach::SCAFFOLDED`] with a written reason.
///
/// `#[cfg(test)]` because the reader parses `app/dispatch.rs` with `syn`, a
/// **dev**-dependency — see this crate's `Cargo.toml` for why a real parser and
/// not a grep, and [`reach`]'s own header for the two mechanisms that lost.
/// Nothing here is compiled into `pdfcer-gui.exe`.
#[cfg(test)]
mod reach;

/// ★ Re-exported flat, so every caller still writes
/// `shell::commands::measure_command` and nothing outside `shell/` learns that
/// the module was split.
///
/// A `pub use` rather than moving the callers, deliberately: the split is an
/// **R2** consequence, not a change to what the shell offers, and a file-size
/// rule that rewrote fifteen call sites in `app/` would be a rule that makes
/// unrelated diffs. See `mapping`'s own header for what the seam is.
pub use mapping::{
    chrome_command, chrome_for_command, form_for_command, markup_command, markup_for_command,
    measure_command, measure_for_command, page_display_command, page_display_for_command,
    text_mark_command, text_mark_for_command,
};

use egui_shell::CommandRegistry;

/// **Open a document from the recent list.**
///
/// A constant rather than a literal because this id is used in four places
/// that must agree and two of them are not obvious: the registration below,
/// the `CUSTOM_BACKED` entry that records why it is on no tab, the registry
/// lookup in [`crate::app::PdfcerApp::ribbon_band`] that turns the operator's
/// menu choice back into this command's token, and the dispatch arm. A typo
/// in any of them produces silence — a menu that draws and reports nothing —
/// rather than an error.
///
/// The other command ids stay literals at their (single) use sites, which is
/// this file's existing convention; this one earns a name by being spelled in
/// two modules.
pub const FILE_RECENT: &str = "file.recent"; // ui-text-exempt: a command id, never displayed

/// **Register every command the built-in manifest names.**
///
/// # Panics
///
/// If two commands claim one id. That is a programming error in
/// [`catalog::all`] and not a condition any input can produce, so it fails
/// loudly at
/// start-up rather than being swallowed: the registry refuses a duplicate
/// precisely so that behaviour cannot come to depend on the order of
/// start-up code, and catching the error here to ignore it would give back
/// exactly the defect the refusal prevents.
pub fn register(reg: &mut CommandRegistry) {
    reg.register_all(catalog::all())
        // ui-text-exempt: a panic message, read by whoever is looking at
        // the stack trace. Never rendered to an operator — the process
        // does not reach a window if this fires.
        .expect("two shell commands claim the same id");
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::commands::ConditionSet;
    use std::collections::BTreeSet;

    fn registry() -> CommandRegistry {
        let mut reg = CommandRegistry::new();
        register(&mut reg);
        reg
    }

    /// Registration succeeds and produces the documented count.
    ///
    /// The number is quoted in this module's header and in
    /// `super::manifest`'s, and a silent drift makes both wrong.
    #[test]
    fn registration_succeeds_and_registers_every_command() {
        // ★ 121 → 122 on 2026-08-28: `file.save_compacted`
        // (`OPERATOR_REQUESTS.md` O48), the save that rewrites the whole file
        // so the space a deletion freed is actually reclaimed.
        // ★ 122 → 123 on 2026-08-28: `edit.reflow_block` (O54b), the
        // paragraph re-wrap `pdfcer-core` has carried since Pass 91.
        // ★ 123 → 124 on 2026-08-28: `edit.attachments`, the Attachments panel
        // — `attach_file`, `detach_file`, `list_attachments` and
        // `extract_attachment` had existed in `pdfcer-core` with no command, no
        // menu item and no panel, which is a capability that does not exist as
        // far as the operator is concerned.
        // ★ 124 → 125 on 2026-08-28: `format.unshare_form`, the "option"
        // half of `pdfcer-core`'s decision 076. `EditSession::unshare_form` had
        // existed in the engine with no command, no menu item and no route of
        // any kind — so the operator had the edit-in-place default and no
        // choice at all, which is the state `R206` exists to prevent.
        // ★ 125 → 126 on 2026-08-29: `edit.paste_duplicate`, the second
        // sense of a form-field paste (`OPERATOR_REQUESTS.md` O58). It is a
        // registered COMMAND rather than a modifier read inside `edit.paste`
        // because a command is the unit this shell can bind, place on a
        // ribbon, put in a menu and withhold by mode — a modifier read inside
        // a handler is reachable from the keyboard and from nowhere else.
        // ★★ 126 → 129 on 2026-08-29: `pages.copy`, `pages.cut` and
        // `pages.paste` — `OPERATOR_REQUESTS.md` O59 item 2, consuming the
        // engine's page clipboard.
        //
        // ★★★ Three COMMANDS and no chord, which is the whole design decision.
        // `Ctrl+C` belongs to the canvas: every `pages.*` verb takes its operand
        // from one rule — picked sheets, else the current page — and that rule
        // ALWAYS resolves, so a chord rung consulting it would answer yes on
        // every document and take the clipboard from the canvas permanently.
        // See `app::dispatch::pageclip`.
        // ★★ 129 → 130 on 2026-08-30: `edit.redact_selection` — the THIRD
        // redaction-marking route, and the first that does not go through text.
        // Ken: *"am I able to select objects on the canvas and redact them that
        // way yet? … it just told me it couldn't."* It could not: the search box
        // reaches text pdfcer can read as text, and on a CAD drawing a
        // title-block value is often vector strokes and a stamp is often an
        // image. Neither is findable by typing.
        // ★★★ 130 → 129 on 2026-08-31: `edit.objects` DELETED — O69. Ken:
        // *"We shouldn't even need an Edit Objects button."* It was a route to
        // `view.tool_select` under a tooltip promising node dragging, so
        // pressing it after arming the Points tool put him back on the arrow.
        // The count moves WITH the list, in the same commit, which is this
        // file's standing rule and the reason four earlier drifts are recorded
        // above it.
        // ★★★ 129 → 126 on 2026-08-31: `tools.split_files`, `pages.split`
        // and `view.sidebar` UNREGISTERED — `OPERATOR_REQUESTS.md` O68. Ken
        // named the first; the sweep found the other two. All three were
        // drawn, enabled and had no dispatch arm at all, so a press traced
        // `command-unimplemented` and did nothing he could see.
        //
        // ★ Note which way each went. `tools.merge_files` was the fourth of
        // that set and is IMPLEMENTED, because its blocker named a missing
        // host and the engine verb was complete. These three are removed
        // because R9 says a capability that is not built renders nothing —
        // greying is for the temporarily unavailable, and "the dialog was
        // never written" is not temporary. The two splits come back together
        // when the boundary chooser exists.
        // ★ 126 → 127 on 2026-08-31: `view.smart_select` REGISTERED —
        // `OPERATOR_REQUESTS.md` O70. Ken: *"we should have a checkbox in
        // navigate for a Smart-Selector option."* The count moves WITH the
        // list, in the same commit, which is this file's standing rule.
        // ★★ 127 → 128 on 2026-09-01: `edit.select_all`. Ken: *"we should be
        // able to select things off the side of the page, especially since I
        // sometimes drop objects there, and when I do I can't get them back."*
        // The canvas senses input over the page rect only, so an object dragged
        // past the edge is unclickable, unbandable and unpainted — this is the
        // way back to it.
        // ★★ 128 → 129 on 2026-09-02: `file.save_as`. Ken: *"we need a Save As
        // option so that we are then making edits in the save as file instead
        // of the original just like other programs have it."* Save a copy
        // already wrote the bytes; what it could not do was MOVE the document,
        // so the next Ctrl+S went back to the file he was leaving.
        // ★★ 129 → 130 on 2026-09-04: `file.export_image` — O120. Ken, to the
        // engine side on 2026-09-03: *"can you add the ability to export
        // page(es) to png, jpg, svg."* `RIBBON_IA.md` §5.1 had carried the row
        // since the ribbon was specified, and nothing was required to read it.
        // ★ 130 → 131 on 2026-09-04: `file.open_in_acrobat` — O122. The
        // operator: *"beside our read-review-edit buttons at the top there
        // should be an open in acrobat button."* The first command this shell
        // has whose job is to STOP being the program holding the document.
        // ★★ 131 → 130 on 2026-09-04: `view.panel_tool` RETIRED — O123. The
        // panel it toggled no longer exists; its status is permanent dock
        // chrome, its live controls are in Properties, and a toggle for a
        // surface that is always drawn would be a control with nothing to
        // toggle (R9). ★ The first DECREMENT this counter has taken, and it is
        // recorded the same way every increment is — with the operator's
        // reason, in the same commit as the list — because a count that only
        // ever goes up is a count nobody has had to think about.
        // ★★★ 130 → 134 on 2026-09-04: the four panel-layout verbs —
        // `view.panel_float`, `view.panel_dock`, `view.panel_close` and
        // `view.dock_all_panels`. Panels now tear out into real OS windows
        // (`egui_shell::dock::float` / `::floatwin`), and R8 is why these are
        // commands rather than three buttons the dock draws for itself:
        // registering one is the only way this shell may learn a capability
        // exists, and it is what puts them in the keymap, in the menu
        // document, and under the operator's own customization.
        //
        // ★ Three of the four are `manifest::TAB_SCOPED` — their operand is
        // the panel the operator right-clicked, which no ribbon control can
        // supply. The fourth has no operand at all and is on the ribbon in
        // View ▸ Window, which is where the capability is discovered.
        // ★★★ 134 → 136 on 2026-09-04: `file.encrypt` and `file.permissions` —
        // `OPERATOR_REQUESTS.md` O119, approved as *"yes add encryption and
        // permissions"*. Two commands and one window; the pair sits in a new
        // File ▸ Security band immediately after Export, which is where the
        // approved mockup draws it and where the operator's own framing of the
        // question puts it — *protect a drawing before you send it out.*
        //
        // ★ …and 136 → 137 the same afternoon: `file.export_text`, from a
        // CONCURRENT track in the same working tree. Recorded as its own
        // movement rather than folded into the line above, because this
        // counter's whole value is that each step names who moved it — and a
        // reader who finds one increment covering two features has no way to
        // learn that two sessions were writing at once, which is the fact that
        // explains the token collision recorded in `catalog::file`.
        //
        // ★★★ …and 137 → 138 on 2026-09-04: `edit.copy_as_vector`
        // (`OPERATOR_REQUESTS.md` **O120**) — *"copy and paste vector graphics
        // into word or inkscape"*. The clipboard's copy-OUT, and the first
        // command in this registry whose whole point is what happens in
        // SOMEBODY ELSE'S program: it places four public formats — SVG, EMF,
        // PNG, DIB — in a measured order, in one transaction, so a paste into
        // Word arrives as editable geometry rather than as a picture of it.
        //
        // ★ A registered COMMAND rather than a modifier on `edit.copy`, for
        // `edit.paste_duplicate`'s reason two dozen lines up and one more
        // besides: the operator did not know pdfcer could do this, which is why
        // he asked, so a keyboard-only route would have answered the request
        // with something he still could not find.
        assert_eq!(registry().len(), 138);
    }

    /// ★ **The icon-coverage split adds up to the registry.**
    ///
    /// This module's header quotes a split — *"N of M named, K refused"* — and
    /// the three dated notes below it are a record of that pair drifting. It
    /// drifted a fourth time and in a way the earlier notes make embarrassing:
    /// the header read *82 of 93 named, 12 refused*, and 82 + 12 = **94**, while
    /// the registry held 93. The truth was 81 named.
    ///
    /// Nothing caught it, and the reason is stated in the note it sits under: the
    /// registry's *size* is pinned by
    /// [`Self::registration_succeeds_and_registers_every_command`] and its
    /// *split* was pinned by nothing. So this pins the split — as an identity
    /// rather than as two more literals, which is deliberate. Two literals would
    /// be two more numbers to drift; an identity is a property, and the only way
    /// to make it false is to make one of its terms genuinely wrong.
    ///
    /// The refused list lives in this module's header table rather than in code,
    /// because each entry is an **argument** and arguments belong at the
    /// registration they justify. What this asserts is that the arithmetic
    /// closes: every command either names a glyph or is one of the refusals, and
    /// the two counts partition the registry with nothing left over.
    #[test]
    fn the_icon_coverage_split_adds_up_to_the_registry() {
        let reg = registry();
        let total = reg.len();
        let named = reg.iter().filter(|c| c.icon.is_some()).count();
        let refused = total - named;
        assert_eq!(
            named + refused,
            total,
            "the split must partition the registry"
        );
        // ★ The literals, and they are now the ONLY copy of these numbers.
        //
        // This block used to end "update that sentence together", pointing at
        // `catalog`'s `## Coverage` heading. It drifted anyway — a fifth time,
        // to *86 of 101 named, 15 refused* against a registry of 94 — because a
        // test cannot enforce a comment. The heading no longer carries numbers
        // and this is where they live: a literal that fails loudly beats a
        // sentence that is wrong quietly.
        //
        // Failing here means the registry changed. Read the diff, decide
        // whether the new command should have a glyph, and move the number
        // that is genuinely wrong.
        // ★ 104 → 105 on 2026-08-28: `format.unshare_form` NAMES a glyph,
        // breaking the run of four refusals above it, and the reason it is
        // entitled to one is that it is not new art. It reuses
        // `pick-form-xobject` — the pick filter's form class — under the
        // header's shared-key convention, exactly as `format.select_form` does
        // one registration earlier. Two controls whose entire subject is a form
        // XObject, drawn the same, is the convention working rather than an
        // economy; `icons/assets/PROVENANCE.md` is untouched, because nothing
        // was drawn.
        // ★ 105 → 106 on 2026-08-29: `edit.paste_duplicate` reuses the
        // `paste` glyph under the header's shared-key convention. A second
        // paste icon would be a distinction the operator has to learn for no
        // gain; Word and Acrobat tell their paste variants apart by label and
        // by chord, not by art, and `icons/assets/PROVENANCE.md` is untouched
        // because nothing was drawn.
        // ★ 106 → 109 on 2026-08-29: the three page-clipboard commands reuse
        // the `cut`, `copy` and `paste` glyphs under the header's shared-key
        // convention. A second set of scissors for pages would be a
        // distinction the operator has to learn for no gain — the tab they are
        // on already says what the command acts on, which is exactly what a
        // shared icon plus a distinct label is for.
        // ★ 109 → 110 on 2026-08-30: `edit.redact_selection` reuses the
        // `redact` glyph, as `edit.redact_apply` already does. Three controls
        // about one operation, told apart by their labels.
        // ★ 110 → 109 on 2026-08-31: `edit.objects` deleted (O69), and the
        // `edit-objects` glyph it named is now unused. The asset stays in the
        // directory — `icons/assets/PROVENANCE.md` makes that the operator's
        // own work and deleting his drawing because a button went away is not
        // ours to do — but no command claims it.
        // ★ 109 → 106 on 2026-08-31: the three unregistered by O68 each
        // named a glyph (`split` twice, `sidebar` once). The assets stay in
        // the directory — `icons/assets/PROVENANCE.md` makes them the
        // operator's own work — but no command claims them.
        // ★ 106 → 107 on 2026-08-31: `view.smart_select` names the
        // `show-points` glyph (O70). Deliberately a REUSE rather than a new
        // asset: the two controls are about the same subject — how deep a
        // click reaches — and `icons/assets/PROVENANCE.md` makes that
        // directory the operator's own drawing, so inventing art for a control
        // he asked for this afternoon would be putting a machine's hand in it.
        // ★★★ 107 → 119 on 2026-09-04, in ONE move and deliberately so.
        //
        // Twelve commands gained a key from the icon batch adopted that day
        // (`GLYPH_ADOPTION.md`): nine whose refusal was written out at their
        // registration — `file.new`, `file.new_from_template`, `file.ocr`,
        // `file.save_as`, `file.save_copy`, `file.save_compacted`,
        // `file.recent`, `edit.attachments`, `edit.reflow_block` — plus
        // `measure.finish`, `markup.finish` and `view.close_other_documents`.
        //
        // ★ Eleven MORE controls changed picture in the same pass and are
        // invisible here, which is worth stating so a reader does not conclude
        // the pass was small. They were already counted as `named` because they
        // named a BORROWED glyph: four form-field tools sharing `form-field`,
        // four measure tools sharing `measure` (a five-way share — Linear keeps
        // it as its documented owner), two redaction verbs sharing `redact`,
        // two copy-text commands sharing `copy`, two font commands sharing
        // `fonts`, `pages.merge_into` on `combine`, `measure.manage_groups` on
        // `list`, `tools.render_diagnostics` on `tools`, and previous/next
        // DOCUMENT on the previous/next PAGE chevrons.
        //
        // ★★ **This counter cannot see a borrow, and that is the interesting
        // part.** Eight controls rendered as two pictures for weeks with this
        // assertion green, because `icon.is_some()` is true of a control
        // wearing someone else's art. The gap is closed elsewhere and on
        // purpose: `icons::tests::no_two_icons_render_as_the_same_picture`
        // compares the 16 px RASTERS, which is the only question that can tell
        // "has a glyph" from "has a glyph of its own".
        //
        // ★ Moved as ONE edit rather than twelve. Three concurrent sessions
        // wired three bands of the ribbon on the same afternoon and each
        // correctly refused to touch this literal, because a shared counter
        // bumped three times in a race records the last writer's arithmetic and
        // none of the reasoning. The bands reported their deltas and the
        // coordinating session settled it once.
        // ★ 119 → 120 on 2026-09-04: `file.export_image` (O120) NAMES a
        // glyph, and is entitled to one for the reason `format.unshare_form`
        // above is — it is not new art. It reuses `export`, the download
        // glyph its two band neighbours already wear, under the header's
        // shared-key convention: "out of this document, into a file" is
        // equally and completely true of all three, and what differs is the
        // FORMAT, which is a word only a label can say. `icons/assets/
        // PROVENANCE.md` is untouched, because nothing was drawn.
        // ★ 120 → 119 on 2026-09-04: `view.panel_tool` is retired (O123) and it
        // named `pointer`. The glyph itself is untouched — `view.tool_select`
        // still wears it — so this is a command leaving, not art leaving.
        // ★ 119 → 120 on 2026-09-04: `file.open_in_acrobat` (O122) NAMES a
        // glyph. It was registered a few hours earlier with the refusal argued
        // at its registration — see the `refused` note below — and
        // `open-in-acrobat.svg` then landed on the icon track, drawn for this
        // command before this command existed to name it. Purpose-drawn art,
        // so `icons/assets/PROVENANCE.md` is untouched.
        // ★ 120 → 121 on 2026-09-04: `edit.select_all` names a glyph. Not a
        // new capability and not new art arriving on its own — a **correction**.
        // Its refusal was written by a build session, quoted four times, and
        // reported to the operator as settled until he said *"I didn't refuse
        // that."* The registration carries the account.
        // ★ 121 → 124 on 2026-09-04: `view.panel_float`, `view.panel_dock`
        // and `view.dock_all_panels` name `floating-panels`, which was in the
        // set before anything used it — drawn for a capability that had been
        // specified and not built. `view.panel_close` refuses one and is
        // counted below with the other refusals.
        //
        // ★★★ **CORRECTED IN PLACE 2026-09-04.** This entry read: *"there is
        // no close art in this set, and the row is a labelled menu item where
        // a glyph would add nothing a word does not."* Both halves are false,
        // and they were false when written:
        //
        //   · **There is close art.** `close.svg` / `crate::icons::Icon::Close`
        //     has been in the set since it landed and is worn by `file.close`.
        //     A supply claim is the one kind of claim in this file that anyone
        //     can check in ten seconds, which is exactly why an unchecked one
        //     is so expensive: it reads as settled and it is quotable.
        //   · **The row is not a place a glyph could not go.** That half rested
        //     on this build's context menus wiring no icon painter, which was
        //     true until `shell::menus_wiring::attach` wired one on 2026-09-04
        //     — after which 25 menu rows began drawing glyphs their commands
        //     had named all along. A missing wire is not a design decision, and
        //     stating it in the grammar of one is how a refusal outlives its
        //     reason.
        //
        // The sentence that replaces it: **`view.panel_close` should draw
        // `close`, and does not yet only because its registration lives in a
        // file another track owns this session.** The full ruling, and the
        // argument for why sharing the X with `file.close` is the relationship
        // rather than a collision, is at the refusal count below.
        // ★★★ 124 → 126 on 2026-09-04: `format.bold` and `format.italic`. **A
        // CORRECTION of a refusal, not a new capability** — the same shape as
        // `edit.select_all` five entries up, and the second time in three days.
        //
        // Both commands did on 2026-08-26 exactly what they do now. What changed
        // is that the reason they were bare — *"Word draws `B` and `I` as
        // glyphs; this build has no such art"*, in the `refused` note below —
        // was a statement about **supply**, and the operator has a standing
        // ruling on supply that predates it by three weeks. From 2026-08-06,
        // carried in `icons::Icon::Back`'s doc comment: a missing glyph is
        // **AUTHORED**, not worked around, because working around it *"spends
        // the operator's affordance to protect the font stack; an icon costs one
        // asset and keeps both."* On 2026-09-04 he applied it to this pair
        // himself: *"if bold and italics have no art in the set, why weren't
        // they made automatically as I have instructed to be done for anything
        // that a glyph is missing for on multiple occasions?"*
        //
        // ★★ **Why the distinction between a correction and a discharge is the
        // whole point of this entry.** A DISCHARGE (the twelve of 2026-09-04,
        // below) is a refusal that was right when written and stopped applying
        // because the world changed — art arrived. A CORRECTION is a refusal
        // that was never entitled to be made: it contradicted a ruling that was
        // already on the books, and it survived because it was quoted rather
        // than checked. Netting the two into one counter movement would lose
        // exactly the fact that matters, which is that nothing about the ribbon
        // needed to change for this to have been wrong all along.
        //
        // ⇒ The rule this leaves behind, for the next reader deciding whether a
        // bare control may stay bare: **"no art exists" is not a reason, it is a
        // work item.** A refusal survives only if it names a WRONG PICTURE
        // (`view.zoom_actual`, argued against by name in the icon ui-spec §3.2),
        // a MISSING SLOT (a custom widget, the mode selector's text segments, a
        // menu row in a build whose menus wire no icon painter), or a CLAIM the
        // command cannot support (`Icon::Signatures`' seal, `Icon::Fonts`'
        // pencil). The three surviving Format ▸ Font refusals are the second
        // kind and are argued at their registration.
        // ★ 126 → 128 on 2026-09-04: `file.encrypt` and `file.permissions`
        // (O119). Both name a glyph and neither needed one drawn — `encrypt`
        // and `permissions` were adopted in the 2026-09-03 batch and have been
        // in `icons/assets/` waiting for the commands that would use them, which
        // is the adoption rule working exactly as `GLYPH_ADOPTION.md` states it.
        //
        // ★ 128 → 129 the same afternoon: `file.export_text`, from a concurrent
        // track. Named separately for the reason the registry counter above
        // gives.
        //
        // ★ 129 → 130 the same afternoon: `edit.copy_as_vector` (O120). Another
        // adoption already on disk — `copy-as-vector` was drawn in the
        // 2026-09-04 batch for a control the ribbon did not yet reach, and
        // `icons/catalog/mapping`'s note calling it *"art before button"* is one
        // name shorter as of this commit. Art waiting for a command is the
        // adoption rule working; a command waiting for art is the refusal this
        // counter's other half exists to make somebody argue for.
        assert_eq!(named, 130, "commands naming an icon");
        // ★ 12 → 17 on 2026-08-27: the Format ▸ Font group's five commands
        // all refuse a glyph, and they refuse it for one reason argued once at
        // their registration. Word draws `B` and `I` as glyphs; this build has
        // no such art, and `icons/assets/PROVENANCE.md` declares that directory
        // the operator's own work — which is what exempts it from
        // `check-shipped-assets` and what a machine-drawn substitute would make
        // false. Without an icon a `Small` item resolves to `Medium`, so the
        // labels render, and "Bold" is less ambiguous than a home-made glyph
        // would have been.
        //   ⇒ ★★★ **Two of those five were CORRECTED on 2026-09-04** — see the
        //   17 → 15 entry at the end of this block, and the `named` note above
        //   for why "corrected" and "discharged" are different words. The
        //   paragraph is kept unedited because it is the exhibit: it welds a
        //   supply claim (*"this build has no such art"*) to a provenance
        //   constraint (*"a machine-drawn substitute would make that note
        //   false"*) in one sentence, and being unable to tell the two apart is
        //   how a work item spent six weeks looking like a decision.
        // ★ 17 → 18 on 2026-08-28: `file.save_compacted` refuses a glyph, and
        // it refuses one for a reason worth stating rather than inheriting.
        // Its two neighbours in the Save group carry icons, and a third disc
        // beside them would be a picture whose only job is to look like the
        // other two — which is exactly the confusion this command's NAME is
        // built to prevent. `icons/assets/PROVENANCE.md` makes that directory
        // the operator's own work, so the alternative is not "draw one" but
        // "ask him for one", and the label reads better than any of the three.
        // ★ 18 → 19 on 2026-08-28: `edit.reflow_block` refuses a glyph. Its
        // three neighbours in the Edit ▸ Content group carry one, so this is
        // the same judgment as `file.save_compacted` one paragraph up and
        // reached the same way: the operator's own art is the only art this
        // build ships, and *"re-wrap this paragraph"* has no conventional
        // glyph to borrow — Word gives it a menu line, not a picture. A
        // home-made pilcrow-with-arrows would be a symbol nobody has been
        // taught. The label says it and the tooltip qualifies it.
        // ★ 19 → 20 on 2026-08-28: `edit.attachments` refuses a glyph, and it is
        // the third registration in a row to reach the same judgment by the same
        // route. The conventional icon for this is a paperclip;
        // `icons/assets/PROVENANCE.md` makes that directory the operator's own
        // work, so the alternative is not "draw one" but "ask him for one", and
        // a home-made paperclip beside hand-drawn art is the mismatch a
        // borrowed icon set exists to avoid. The label is the word Acrobat uses.
        // ★ 20 → 21 on 2026-09-01: `edit.select_all` refuses a glyph, and the
        // reason is the same one the three before it reached. There is no
        // conventional icon for Select All — Word, Acrobat and Illustrator all
        // present it as words, in a menu or a list, because what it selects is
        // the thing a picture cannot show. A marquee glyph would say "rubber
        // band", which is the gesture this command exists to replace when the
        // rubber band cannot reach.
        // ★ 21 → 22 on 2026-09-02: `file.save_as`, refused on the same
        // reasoning `file.new` and `file.ocr` record. There is no conventional
        // Save-As glyph — Word and Acrobat both present it as words — and every
        // reuse would mislead: the disk of `save` says "save", which is the
        // sibling command this one must not be confused with.
        // ★★★ 22 → 10 on 2026-09-04. Twelve of the refusals argued above were
        // DISCHARGED by the icon batch, and "discharged" is the right word for
        // eleven of them rather than "reversed": each named, correctly, a reuse
        // that would have misled, and each ended with some version of *"the
        // alternative is not draw one but ASK HIM FOR ONE"* — because
        // `icons/assets/PROVENANCE.md` declares that directory the operator's
        // own art and a machine-drawn substitute would make the note false.
        //
        // The asking happened. Every constraint those refusals named SURVIVES:
        // the new art was drawn to stay distinguishable from exactly the
        // neighbours they warned about, and each registration now records which
        // ones. **What ended was the supply problem, not the argument.**
        //
        // ★ The paragraphs above are kept rather than deleted. A refusal that
        // was right for six weeks and then stopped applying is a more useful
        // thing for the next reader to find than a gap where it used to be —
        // and two of them have NOT been discharged and must not be:
        //
        //   · `view.zoom_actual` — argued against BY NAME in the icon ui-spec
        //     §3.2. No supply of art touches that.
        //   · `edit.select_all` — refused because no comparable program draws
        //     one and a marquee glyph would say "rubber band", which is the
        //     gesture the command exists to replace when it cannot reach. The
        //     2026-09-03 sheet did offer a `select-all` marquee. It was
        //     deliberately not adopted, for this paragraph's reason.
        //
        // ★★★ **And that last sentence was wrong, within hours, in the one way
        // this file keeps proving is possible.** It read:
        //
        // > The five Format ▸ Font refusals also stand: the sheet offered no
        // > `B`/`I` art and the argument was never only about supply.
        //
        // The argument was *partly* about supply — *"this build has no such
        // art"* is the first clause of it — and a refusal that is partly a
        // supply statement is partly expired. Two of the five (`format.bold`,
        // `format.italic`) were corrected the same day; the art was drawn
        // rather than sourced from a sheet, which is what the operator's
        // standing ruling asks for and what "the sheet offered no `B`/`I` art"
        // was quietly treating as the end of the matter.
        //
        // The other **three stand and are not expiring**: `format.font`,
        // `format.font_size` and `format.font_colour` are drawn by an
        // `Item::Custom` — a combo box, a drag field and a colour swatch — and
        // none of those widgets has an icon slot. That is a MISSING SLOT, not a
        // missing picture, and no amount of drawing touches it.
        // ★ 11 → 10 on 2026-09-04, within hours of 10 → 11. The intervening
        // entry was `file.open_in_acrobat`, registered with no glyph and with
        // the refusal argued at its registration — two reuses were available
        // and both would have MISLED: `export` says "out of this document,
        // into a file", which this does not do, and `open` says "bring a file
        // in here", which is its opposite.
        //
        // It was discharged the same afternoon by a purpose-drawn
        // `open-in-acrobat.svg` that had landed on the icon track for this
        // command before this command existed to name it. Recorded as two
        // movements rather than netted silently to zero, because the argument
        // is the record: a refusal that names a MISSING SUPPLY rather than a
        // wrong picture is a refusal with an expiry date, and both of this
        // file's discharges have now proved it.
        assert_eq!(
            // ★ 10 → 9, 2026-09-04 — `edit.select_all`, and the reason is a
            // correction rather than a discharge. See the `named` note above.
            //
            // ★ 9 → 10 the same day — `view.panel_close`. It refuses a glyph
            // because it is only ever drawn as a MENU ROW, and a menu row is
            // a line of words: the icon column exists on the ribbon, not in a
            // context menu. Its two siblings name `floating-panels` because
            // they are the same act in two directions and the picture
            // distinguishes them from the text rows around them; a close has
            // nothing to be distinguished from.
            //   ⇒ ★★ **The verdict stands and two of its sentences do not**,
            //   corrected 2026-09-04 by the refusal audit. This entry also
            //   said *"there is no close art in this set"*. There is:
            //   `close.svg`, `icons::Icon::Close`, worn by `file.close` since
            //   the set landed. A false supply claim inside a valid structural
            //   refusal is the worst version of the pattern this file keeps
            //   finding, because the paragraph reads as settled and half of it
            //   is checkable and wrong.
            //   The structural half was then checked and is TRUE, more
            //   completely than it claimed: `shell::menus::MenuHost::attach_with`
            //   builds its `ContextMenu` with `reporting_rects_to` and **no
            //   `with_icon_painter`**, so every context-menu row in this build
            //   draws a label and nothing else. So the sibling sentence is
            //   wrong too — `view.panel_float` and `view.panel_dock` name
            //   `floating-panels`, but nothing paints it on this surface;
            //   their keys are correct data waiting for a surface that reads
            //   them, not a picture the operator sees today.
            //
            //   ⇒ ★★★ **And the structural half expired the same day, 2026-09-04,
            //   by being acted on rather than re-argued.** Every sentence above
            //   is kept, because the shape of the mistake is the record; this
            //   is what is now true instead.
            //
            //   `ContextMenu::with_icon_painter` had existed since the menu
            //   engine landed. Nothing called it. `shell::menus_wiring::attach`
            //   now does — one builder call, `crate::icons::paint_ribbon_icon`,
            //   the ribbon's own painter — and **25 menu rows across all nine
            //   context menus began drawing glyphs they were already carrying**,
            //   with no per-row work, because their commands had named a key all
            //   along. `shell::menus_wiring::tests` holds the count.
            //
            //   So *"the icon column exists on the ribbon, not in a context
            //   menu"* was never a fact about menus. It was a fact about one
            //   line nobody had written, stated in the grammar of a design
            //   decision — which is the exact failure this ledger keeps
            //   catching, arriving this time in the STRUCTURAL half rather than
            //   the supply half. The operator's test (2026-08-06, quoted in
            //   `crate::icons::Icon::Back`) discriminates them: *"there is no
            //   icon SLOT on this surface"* is a valid refusal only if adding
            //   one would be **wrong**, not merely if it would be **work**.
            //   Here it was one line of work.
            //
            //   ⇒ **The verdict on the merits, now that the slot exists:
            //   `view.panel_close` SHOULD draw `close`.** Not because the
            //   column wants filling — a column that is half empty is better
            //   than a column of pictures that mean nothing — but because the
            //   picture is right and is already in the set:
            //
            //     · `close.svg` / `crate::icons::Icon::Close` is an X, which is
            //       what "close the thing this row is about" looks like in
            //       every application anyone has used. It says nothing this
            //       command does not do, which is the whole wrong-picture test.
            //     · Sharing it with `file.close` is not a collision, it is the
            //       relationship: **one verb, two operands** — the same
            //       relationship `view.panel_float` and `view.panel_dock`
            //       already have with the one `floating-panels` glyph they
            //       share, which this file accepted at their registration.
            //     · The inconsistency is otherwise visible in two menus at
            //       once. `document.tab` draws `file.close` **with** the X;
            //       `dock.tab` would draw `view.panel_close` bare, one row
            //       below a Float that has a glyph. The same word, twice, one
            //       of them pictured.
            //
            //   ⇒ ★ The one-line change (`.with_icon("close")` at this
            //   command's registration in `catalog::view`) is NOT made here.
            //   `shell/commands/catalog/` belongs to a concurrent track this
            //   session and a two-agent edit of one registration list is how a
            //   command gets registered twice. The decision is recorded, the
            //   count below stays at 8 until it is applied, and the day it is,
            //   this becomes 8 → 7 and the third discharge in this ledger.
            //
            // ★★★ 10 → 8 on 2026-09-04 — `format.bold` and `format.italic`,
            // **corrected, not discharged**. The `named` note above carries the
            // account and the rule it leaves behind; the two assets carry the
            // art's own reasoning, which is where `icons/assets/PROVENANCE.md`
            // says a per-glyph ruling belongs.
            //
            // ★★ The other eight were AUDITED at the same time, against the
            // operator's ruling rather than against each other, and every one
            // survives on a reason that is not about supply:
            //
            //   · `format.font`, `format.font_size`, `format.font_colour` —
            //     MISSING SLOT. Drawn by an `Item::Custom`; a combo box, a drag
            //     field and a colour swatch have nowhere to put a glyph, and the
            //     swatch's whole face is the value it reports.
            //   · `mode.read`, `mode.review`, `mode.edit` — MISSING SLOT.
            //     `egui_shell::ribbon::mode_selector` draws text segments and
            //     contains no icon path at all; a key here would name art
            //     nothing draws.
            //   · `view.zoom_actual` — WRONG PICTURE, argued against BY NAME in
            //     the icon ui-spec §3.2 and marked `{noicon:1}` in the approved
            //     mockup. No supply of art touches that.
            //   · `view.panel_close` — **NO LONGER REFUSED ON THE MERITS**, and
            //     the only entry in this list whose reason has expired rather
            //     than survived. It was recorded as MISSING SLOT because this
            //     build's context menus wired no icon painter; they wire one as
            //     of 2026-09-04, the slot exists, and the ruling immediately
            //     above is that `close` is the right picture for it. It is
            //     still counted below only because the registration lives in a
            //     file another track owns this session.
            refused,
            8,
            "commands with no icon, each argued at its registration"
        );
        // Each refusal is argued at its own registration and listed in the
        // header's table. Asserting the ids too would be a third copy of that
        // list; asserting the *count* is what stops a glyph being quietly
        // dropped from a control that had one.
    }

    /// **★ No two commands share a handler token.**
    ///
    /// The shell explicitly permits it — two ids may share a token if the
    /// application wants two names for one handler — which is exactly why
    /// this needs asserting on *our* side. pdfcer has no such pair, so a
    /// collision here is a typo in a hand-assigned number, and its symptom
    /// would be one command silently doing another's work. Nothing else in
    /// the system can detect that.
    #[test]
    fn every_handler_token_is_unique() {
        let mut seen: BTreeSet<u64> = BTreeSet::new();
        for command in registry().iter() {
            assert!(
                seen.insert(command.handler.get()),
                "handler token {} is assigned twice; `{}` collides with an earlier command",
                command.handler.get(),
                command.id
            );
        }
    }

    /// Handler tokens sit in their tab's hundred-block.
    ///
    /// The blocks are what make a collision improbable in the first place
    /// and what makes a raw token in a trace readable — `4xx` is an Edit
    /// command without looking anything up. A number in the wrong block is
    /// how the next one gets assigned on top of an existing command.
    #[test]
    fn every_handler_token_is_in_its_tabs_block() {
        let blocks = [
            ("file.", 100),
            ("view.", 200),
            ("pages.", 300),
            ("edit.", 400),
            ("markup.", 500),
            ("measure.", 600),
            ("tools.", 700),
            ("format.", 800),
            ("mode.", 900),
        ];
        for command in registry().iter() {
            let (prefix, base) = blocks
                .iter()
                .find(|(p, _)| command.id.starts_with(p))
                .unwrap_or_else(|| panic!("`{}` has no known prefix", command.id));
            let token = command.handler.get();
            assert!(
                (*base..base + 100).contains(&token),
                "`{}` has token {token}, outside the `{prefix}` block {base}..{}",
                command.id,
                base + 100
            );
        }
    }

    /// Every enable condition is one of the five documented names.
    ///
    /// A predicate naming a condition the application never publishes is a
    /// command that is permanently greyed — and it fails silently, because
    /// an unset condition and a false condition are the same value. The
    /// vocabulary is small on purpose; this is what keeps it small.
    #[test]
    fn every_predicate_names_a_documented_condition() {
        const KNOWN: &[&str] = &[
            // ★ NOT nested inside `doc.open` where it is published, and the
            // header says why: the one state that needs it most is a failed
            // open with other documents behind it.
            "docs.multiple",
            // ★★ **At least one panel is in a window of its own**, published
            // by `app::conditions` from the dock's live layout since
            // 2026-09-04. `view.dock_all_panels` is the only command that
            // waits on it, and it is the RECOVERY command for a float window
            // the operator cannot reach — so this is greying in R9's strict
            // sense: temporarily unavailable, because there is nothing to
            // dock this second, and the tooltip says what it would do.
            //
            // ★ Deliberately NOT `!panels.floating` on anything. Nothing is
            // hidden by a panel being floated; a float is a place a panel is,
            // not a state the application is in.
            "panels.floating",
            "doc.open",
            "doc.pages",
            "undo.available",
            "redo.available",
            "selection.any",
            // ★★ WIDER than `selection.any`, not a refinement: it is also set
            // for a selected form field, which lives in `doc.selected_field`
            // rather than in `SelectionState`. `format.delete` and
            // `format.properties` take this one because both can act on a
            // field; the contextual Format TAB still takes `selection.any`,
            // because a field has no font or stroke for it to offer.
            "selection.actionable",
            // ★★★ **NOT a refinement of either neighbour, and its default is
            // TRUE.** It answers *would the engine refuse a delete?* rather
            // than *is there anything to delete?*, so it is set in almost every
            // state including the empty one, and cleared only for a selected
            // annotation that `annotation_deletion_refusal` or §12.5.3's
            // `Locked` bit forbids. `format.delete` carries it as its
            // `visible_when` on the Format tab and on the canvas object menu,
            // where `selection.actionable` stays its `enabled_when` — two
            // predicates, two questions, and R9 decides which gets greying and
            // which gets absence. See `PdfcerApp::conditions`.
            "selection.delete_permitted",
            // ★★★ **NOT a refinement of the one above, and they disagree in
            // BOTH directions.** A redaction mark can be deleted and cannot be
            // cut — deleting it removes a pending operation, which is a thing
            // an operator may want; cutting it would put it on a clipboard that
            // could arm it somewhere else. A locked annotation can be neither.
            // Default TRUE, cleared only for what the clipboard cannot carry.
            // Asked by `pdfcer-core` by name; see `canvas::cutgate`.
            "selection.cut_permitted",
            // Not a refinement of `selection.any` — see `PdfcerApp::conditions`.
            // A selection can exist and resolve to no box.
            "selection.bounds",
            // ★ This one IS a refinement of `selection.any`, unlike its
            // neighbour above, and it is still its own name because it answers
            // a question `selection.any` cannot: is there a **container** to
            // select? Set when something selected on the current page is drawn
            // from inside a form XObject.
            "selection.in_form",
            // ★ The only condition about a **gesture in progress** rather
            // than about the document, the selection or the view.
            //
            // `measure.finish` ends the radius/diameter gesture, which is the
            // one gesture on the canvas with no natural end, so its control
            // must be live exactly when there is something to end — a Finish
            // that is always enabled is a control that does nothing on almost
            // every press. Published by `PdfcerApp::conditions` from
            // `canvas::measure::finishable`, which is the same derivation the
            // command's own arm asks, so the control cannot be enabled while
            // pressing it would do nothing.
            "measure.finishable",
            // ★ **A live text selection**, and the second condition here about
            // something other than the document or the view.
            //
            // The three Text markup commands act on the selection rather than
            // arming a tool (`canvas::markup::text` §1), so without one they
            // would be controls that do nothing on almost every press. It is
            // **not** a refinement of `selection.any`, which is the *object*
            // selection: the two are mutually exclusive by construction
            // (`canvas::textsel` §3), so a build that confused them would grey
            // these three in exactly the mode where they work.
            //
            // "Live" is part of the name's meaning rather than a detail: a
            // selection resolved against a revision that has since moved is
            // refused by `markup::text::mark`, and the condition asks the same
            // question so the control cannot be enabled while the press would
            // decline.
            "selection.text",
            // ★ **A vertex run ready to be committed** — `measure.finishable`'s
            // twin, and the second condition here about a **gesture in progress**.
            //
            // `markup.finish` ends the PolyLine and Polygon gestures, which are
            // the only markup gestures with no natural end: a band drag and a
            // freehand stroke both end when the button comes up, and a run of
            // clicks does not end itself. So its control must be live exactly
            // when there is a run to end — a Finish that is always enabled is a
            // control that does nothing on almost every press.
            //
            // Published by `PdfcerApp::conditions` from
            // `canvas::markup::vertex::finishable`, which is the same derivation
            // the command's own arm asks, so the control cannot be enabled while
            // pressing it would do nothing. It is **not** a refinement of
            // `measure.finishable`: a measure tool and a markup tool cannot both
            // be armed, so exactly one of the two can ever be true, and a build
            // that collapsed them would light one tab's Finish from the other
            // tab's gesture.
            "markup.finishable",
            // ★★ A condition NOTHING SETS, and that is its whole purpose.
            //
            // The operator's ruling of 2026-08-26: push buttons stay on the
            // ribbon, greyed. R9 permits greying only for a TEMPORARILY
            // unavailable capability explained on hover, and this is exactly
            // that — `add_push_button` authors one fine; what pdfcer cannot do
            // is RUN what a button does, because it executes no PDF actions.
            //
            // Expressing "permanently disabled until a capability arrives" as
            // an unset condition rather than as a `disabled: true` flag means
            // un-greying it is one line in `app::conditions` on the day pdfcer
            // runs an action — and until then the ribbon needs no special case
            // and no `#[cfg]`.
            "forms.push_button_runnable",
            // ★ Published by `app::conditions` for the ribbon's Font group and
            // the Points tool since 2026-08-17, and named here for the first
            // time on 2026-08-31 when `view.smart_select` became the third
            // control to wait on it (`OPERATOR_REQUESTS.md` O70).
            //
            // It was a *shown_when* predicate on the manifest side until now,
            // and manifest predicates are not registry `Enable` values — which
            // is why a condition that has been live for two weeks was not in
            // this list. That is worth noticing rather than quietly adding: the
            // list asserts what a COMMAND may wait on, and this is the first
            // command to wait on it.
            "mode.edit_content",
        ];
        for command in registry().iter() {
            if let egui_shell::commands::Enable::When(name) = &command.enable {
                let bare = name.strip_prefix('!').unwrap_or(name);
                assert!(
                    KNOWN.contains(&bare),
                    "`{}` waits on `{name}`, which is not a published condition",
                    command.id
                );
            }
        }
    }

    /// **With no document open, only the commands that make sense without
    /// one are available.**
    ///
    /// The headless equivalent of launching pdfcer and looking at the
    /// ribbon. It is asserted as an exact set rather than a count, because
    /// the interesting failure is a *specific* command escaping its
    /// predicate — `pages.delete` live with nothing open — and a count
    /// would pass as long as some other command lost one.
    #[test]
    fn with_no_document_only_the_document_free_commands_are_enabled() {
        let nothing = ConditionSet::new();
        let reg = registry();
        let live: BTreeSet<&str> = reg
            .iter()
            .filter(|c| c.is_enabled(&nothing))
            .map(|c| c.id.as_str())
            .collect();

        let expected: BTreeSet<&str> = [
            // About describes pdfcer, so it is offered before anything is
            // open — see its registration.
            "file.about",
            // ★ New has no predicate for the strongest version of `file.open`'s
            // reason: an empty shell is not a state New is *tolerated* in, it is
            // the state New exists for. A `doc.open` gate here would grey the
            // one control that answers "there is nothing here".
            "file.new",
            // The sized New, for exactly `file.new`'s reason. Two commands
            // that both answer "there is nothing here" must both be reachable
            // from that state, and a predicate on one of them would be a
            // difference between siblings with no argument behind it.
            "file.new_from_template",
            "file.open",
            // Available with nothing open, like `file.open`, and for the same
            // reason: it is how you GET a document. Its own control greys
            // itself when the list is empty — see the registration's comment
            // on why that rule lives with the menu rather than in a sixth
            // published condition.
            "file.recent",
            "file.settings",
            "file.shortcuts",
            "mode.edit",
            "mode.read",
            "mode.review",
            "tools.font_folders",
            "tools.merge_files",
            "view.fullscreen",
            // ★★ The three panel-layout verbs need no document, and that is
            // deliberate rather than an omission. A panel arrangement is
            // CHROME: it belongs to the operator, it is persisted beside the
            // settings rather than in the file, and it survives closing every
            // document. An operator who floated the Layers panel and then
            // closed their last document must still be able to dock it back —
            // gating these on `doc.open` would leave a window on screen with
            // no command able to act on it.
            "view.panel_close",
            "view.panel_dock",
            "view.panel_float",
            "view.read_mode",
            "view.reset_layout",
        ]
        .into_iter()
        .collect();

        assert_eq!(live, expected);
    }

    /// A document with no pages is a legal document, and it must not arm
    /// anything that acts on a page.
    ///
    /// `/Count 0` is valid PDF. pdfcer opens such a file and says "This
    /// document has no pages" rather than reporting a failure — so the
    /// condition set it publishes has `doc.open` and not `doc.pages`, and
    /// this asserts the consequence.
    #[test]
    fn an_empty_document_arms_nothing_that_needs_a_page() {
        let empty_doc = ConditionSet::new().with("doc.open");
        let reg = registry();
        for id in [
            "pages.rotate_left",
            "pages.delete",
            "edit.text",
            "markup.rectangle",
            "measure.linear",
            "view.zoom_fit_page",
        ] {
            assert!(
                !reg.get(id).expect("registered").is_enabled(&empty_doc),
                "`{id}` acts on a page and must not be armed by a document with none"
            );
        }
        // …while the document-level commands are live, because there is a
        // document: its properties, its fonts and its metadata all exist.
        for id in ["file.properties", "file.fonts", "file.close"] {
            assert!(reg.get(id).expect("registered").is_enabled(&empty_doc));
        }
    }

    /// Undo and redo are the canonical *temporarily* unavailable pair.
    #[test]
    fn undo_and_redo_follow_their_stacks() {
        let reg = registry();
        let undo = reg.get("edit.undo").expect("registered");
        let redo = reg.get("edit.redo").expect("registered");
        let nothing = ConditionSet::new();
        assert!(!undo.is_enabled(&nothing));
        assert!(!redo.is_enabled(&nothing));
        assert!(undo.is_enabled(&ConditionSet::new().with("undo.available")));
        assert!(redo.is_enabled(&ConditionSet::new().with("redo.available")));
        // And each has a tooltip, which is what P3 requires of anything
        // that can be greyed.
        assert!(undo.tooltip.is_some());
        assert!(redo.tooltip.is_some());
    }

    /// Every registered command has a tooltip.
    ///
    /// The catalog type makes this structurally true, so the test is
    /// guarding the *wiring*: a command built with `Command::new` and
    /// never given `.with_tooltip` would compile.
    #[test]
    fn every_command_has_a_tooltip() {
        for command in registry().iter() {
            assert!(
                command
                    .tooltip
                    .as_ref()
                    .is_some_and(|t| !t.trim().is_empty()),
                "`{}` has no tooltip; greying it would be unexplainable",
                command.id
            );
        }
    }

    /// ★★★ **Every icon key a command names is a key the icon set has.**
    ///
    /// The missing half of [`Self::the_icon_coverage_split_adds_up_to_the_registry`],
    /// added 2026-09-04 during the mockup-parity pass, and the two are
    /// deliberately adjacent because they are one question asked at two
    /// depths:
    ///
    /// | test | question |
    /// |---|---|
    /// | the split | *does this command name a glyph at all?* |
    /// | this one | *and does that name resolve to a picture?* |
    ///
    /// # What a wrong key actually does, which is why this is not cosmetic
    ///
    /// It does **not** crash and it does **not** draw nothing.
    /// `icons::paint_ribbon_icon` falls through to `paint_missing_mark`, which
    /// draws a rounded square with a diagonal slash — a deliberate, visible
    /// mark, argued at length in `icons::paint`'s header as *not* a
    /// placeholder: it says "there is no glyph for this", which is a true
    /// statement about the build rather than an invitation to believe a
    /// control is coming.
    ///
    /// That is the right behaviour at run time and it is exactly why a test
    /// is needed. The failure is **legible on screen and silent everywhere
    /// else**: a typo in a `with_icon("…")` string compiles, registers,
    /// renders, passes the coverage split (the key is `Some`), passes the
    /// kebab-case check (the typo is kebab), and ships as a slashed box in
    /// the middle of the File tab. The only oracle was a screenshot, and
    /// `MODES_AND_PANELS.md` is clear that a defect an oracle found deserves
    /// a test that would have found it too.
    ///
    /// ★ Asserted over the **whole registry** rather than over the ribbon
    /// manifest, and that is the wider claim on purpose: a command's icon is
    /// drawn wherever the command is drawn — the band, the quick-access
    /// toolbar, the overflow menu, a context menu, the collapsed-group popup,
    /// the shortcuts dialog. Scoping this to the ribbon would bless a broken
    /// key on any of the other five surfaces.
    #[test]
    fn every_icon_key_a_command_names_resolves_to_real_art() {
        let reg = registry();
        let mut broken: Vec<(&str, &str)> = Vec::new();
        let mut checked = 0_usize;
        for command in reg.iter() {
            let Some(key) = command.icon.as_deref() else {
                continue;
            };
            checked += 1;
            if crate::icons::Icon::from_key(key).is_none() {
                broken.push((command.id.as_str(), key));
            }
        }
        // The vacuity guard, and it is not decoration: `iter()` returning an
        // empty registry, or `icon` becoming `None` everywhere, would make the
        // loop above pass by never running. The floor is deliberately loose —
        // the exact count is pinned by the coverage split next door, and a
        // second copy of it here would be a second number to drift.
        assert!(
            checked > 100,
            "only {checked} commands named an icon, so this test barely ran. The \
             coverage split next door pins the real number; this is the guard that \
             says the loop had something to look at"
        );
        assert!(
            broken.is_empty(),
            "these commands name an icon key the set does not have, so each one draws \
             a slashed box where the operator expects a picture: {broken:?}"
        );
    }

    /// ★★ **…and the check above can fail**, which is the half a green test
    /// cannot demonstrate about itself.
    ///
    /// `PROJECT_PLAN.md` §4.1 records a gate that printed "clean" while
    /// checking a handful of files, and the standing lesson from it is that
    /// *finding nothing looks exactly like finding no violations*. So the
    /// predicate the test above is built on — `Icon::from_key` returning
    /// `None` for a name that is not in the set — is asserted directly,
    /// against a key shaped exactly like the typo this is guarding against:
    /// plausible, kebab-case, and absent.
    #[test]
    fn a_plausible_but_absent_icon_key_does_not_resolve() {
        assert!(
            crate::icons::Icon::from_key("new-documnet").is_none(),
            "`Icon::from_key` resolved a misspelling of a real key, so \
             `every_icon_key_a_command_names_resolves_to_real_art` would pass over \
             exactly the defect it exists to catch"
        );
        assert!(
            crate::icons::Icon::from_key("new-document").is_some(),
            "…and the correctly-spelled key must resolve, or the assertion above is \
             satisfied by an `Icon::from_key` that resolves nothing at all"
        );
    }

    /// Icon keys are lower-case kebab, matching the salvaged icon set's
    /// naming.
    ///
    /// A key that does not match the set's spelling resolves to nothing at
    /// run time and renders as a missing glyph — a placeholder arriving
    /// through the back door, and one that no headless test would
    /// otherwise see.
    #[test]
    fn icon_keys_are_kebab_case() {
        for command in registry().iter() {
            let Some(icon) = &command.icon else { continue };
            assert!(
                icon.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "`{}` names icon `{icon}`, which is not lower-case kebab",
                command.id
            );
        }
    }
}
