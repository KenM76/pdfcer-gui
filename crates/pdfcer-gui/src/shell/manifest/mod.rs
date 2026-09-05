//! # shell::manifest — pdfcer's ribbon, as an `egui_shell::Shell` value
//!
//! [`built_in`] returns the complete pdfcer shell: eight tabs (seven
//! ordinary plus the contextual Format tab), thirty-three groups, three
//! modes, the quick-access toolbar and the keymap. It is the **built-in
//! layer** of `SHELL_FRAMEWORK.md` §4's three-layer merge:
//!
//! 1. **Built-in** — this function. Compiled into the binary, always
//!    valid, and always available as the reset target.
//! 2. **Application override** — an optional file shipped beside the exe.
//! 3. **Operator customization** — `userdata/shell.ron`.
//!
//! Layers 2 and 3 override this one **per item**, never wholesale. That is
//! why this layer has to be complete and has to validate: it is the thing
//! every other layer is a patch against, and it is what an operator gets
//! back when they reset.
//!
//! One file per tab. The tab modules are where the *reasoning* lives —
//! why a command sits where it does, what moved, what was left out and
//! why — and they are worth reading before changing anything here.
//!
//! # ★ The no-placeholders rule, and the two registers that keep it honest
//!
//! `RIBBON_IA.md` P3: *an unavailable capability renders nothing, not a
//! disabled stub.* Greying is reserved for **temporarily** unavailable —
//! no document open, undo stack empty — and is always explained on hover.
//!
//! `RIBBON_IA.md` §5 marks every command it specifies with where it exists
//! today:
//!
//! | Mark | Meaning | In this manifest |
//! |---|---|---|
//! | **G** | exists in the GUI now | emitted |
//! | **C** | exists in `pdfcer-core`/`pdfcer`, no GUI surface | **absent**, in [`PLANNED`] |
//! | **N** | exists nowhere | **absent**, in [`PLANNED`] |
//!
//! A **C** row is the cheapest kind of missing command — the hard half is
//! written and tested — and it is still absent, because P3 is about what
//! the operator can reach and an engine with no caller is not reachable.
//!
//! Absent is not forgotten. Two registers make the difference visible:
//!
//! - **[`PLANNED`]** — every specified command this manifest does *not*
//!   emit, with the reason. Tested in both directions: nothing in it is
//!   referenced by the manifest, and nothing in it is registered. That is
//!   the list a later stage reads to find its work.
//! - **[`DIRECTED`]** — the small set of commands emitted *despite* not
//!   carrying a **G** mark, each with the instruction that put it there.
//!   Without this list those seven entries would look like the manifest
//!   quietly ignoring P3.
//!
//! # Command ids
//!
//! Dotted lowercase, and the prefix is **the tab that owns the command**:
//! `view.zoom_fit_page`, `pages.rotate_left`, `markup.highlight`. That
//! makes P1 — one command, one tab — legible in the id itself, and it
//! makes a violation obvious on sight rather than only at validation.
//!
//! Two deliberate exceptions:
//!
//! - `edit.undo` and `edit.redo` sit on **no tab**. They live on the QAT
//!   alone, which `RIBBON_IA.md` §7 keeps unchanged. The `edit.` prefix
//!   says where they would go if they ever got one.
//! - `mode.read`, `mode.review` and `mode.edit` are not tab commands at
//!   all: they are the three positions of the selector at the far right of
//!   the tab row, reachable from the keymap.
//!
//! # What this module deliberately does not decide
//!
//! **Icons, labels, tooltips and enable predicates** — those are the
//! registry's half of the split, in [`super::commands`]. A manifest
//! contains command *ids* and nothing else about them, which is what stops
//! a customized ribbon from inventing a command and what makes an unknown
//! id a disclosed skip rather than a crash.
//!
//! **Behaviour.** Nothing here runs.

mod edit;
mod file;
mod format;
mod ladder;
mod markup;
mod measure;
mod pages;
pub mod rail;
mod tools;
mod view;

use crate::text::ribbon;
use egui_shell::manifest::{Group, Item, ItemSize, Mode, Shell};

/// **The complete pdfcer shell.**
///
/// Deterministic and side-effect free: called from tests, from the RON
/// round-trip check, and once at start-up. It allocates a few kilobytes of
/// `String` and does nothing else.
///
/// # Order is presentation
///
/// Tabs appear in the order they are added, groups in the order they are
/// listed, and items in the order they are written. `RIBBON_IA.md` §4's
/// table is the tab order — File, View, Pages, Edit, Markup, Measure,
/// Tools — and it is not arbitrary: it runs from what you do to the file,
/// through what you look at, to what you change, to what you add, ending
/// at the things that run across other files.
///
/// # The menus are part of this value, not a second document
///
/// `SHELL_FRAMEWORK.md` §1 says the shell is *one* serializable document,
/// and `egui_shell::Shell` carries `menus` beside `tabs` for exactly that
/// reason: one file to ship, one file to merge, one file for the operator
/// to edit, and — the payoff that decided it — **one keymap**, so a menu
/// row's chord hint is derived from the same bindings the ribbon uses
/// rather than written down twice. See [`super::menus`] for what is in
/// **Point the two form-field paste chords at the operator's chosen order.**
///
/// `OPERATOR_REQUESTS.md` **O58**. Ken, 2026-08-29: *"let's make it an option to
/// have it swap to match Acrobat or work the way we have it now."*
///
/// # What it does, and what it deliberately does not
///
/// It rewrites **two entries** of the shell's keymap so `edit.paste` and
/// `edit.paste_duplicate` sit on the chords
/// [`crate::app::prefs::PasteChords`] names. It changes **nothing else** — not
/// what either command does, not its label, not its tooltip, not whether it is
/// on the ribbon.
///
/// ★★ That is the whole design. Swapping what the *commands* do would make the
/// labels lie: a button reading **Paste as duplicate** would paste a new field.
/// Swapping the *keys* leaves every surface honest by construction, because the
/// ribbon, the context menu, the shortcuts dialog and the keyboard dispatcher
/// all read this one keymap.
///
/// # Why it removes before it inserts
///
/// A [`Keymap`] is a map from chord to command, so writing the new pair without
/// clearing the old one leaves whichever chord is now unused still pointing at
/// its old command — and under `AcrobatOrder` that is `Ctrl+V` mapped twice.
/// `BTreeMap::insert` would resolve it silently and the loser would be decided
/// by nothing the operator can see.
///
/// ⇒ Both chords are cleared first, then both are written. The result depends
/// only on the preference, never on what was there before, which is what makes
/// this safe to call repeatedly — and it IS called repeatedly: once at start-up
/// and again every time the setting changes.
///
/// # It is a no-op on a shell with no keymap
///
/// `Shell::keymap` is an `Option`, and a manifest that failed validation leaves
/// it `None`. Silently doing nothing is right here: the operator has already
/// been told the ribbon is unavailable, and a second complaint about key
/// bindings would be noise about a consequence rather than the cause.
pub fn apply_paste_chords(shell: &mut Shell, order: crate::app::prefs::PasteChords) {
    let Some(keymap) = shell.keymap.as_mut() else {
        return;
    };
    // ui-text-exempt: keymap chord spellings, never displayed.
    for chord in ["Ctrl+V", "Ctrl+Shift+V"] {
        keymap.0.remove(chord);
    }
    keymap
        .0
        .insert(order.new_field_chord().to_owned(), "edit.paste".to_owned());
    keymap.0.insert(
        order.duplicate_chord().to_owned(),
        "edit.paste_duplicate".to_owned(),
    );
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "paste-chords order={order:?} new={} duplicate={}",
            order.new_field_chord(),
            order.duplicate_chord()
        )
    });
}

/// them and why.
#[must_use]
pub fn built_in() -> Shell {
    let mut shell = Shell::new()
        // -------------------------------------------------------------------
        // MODES — `MODES_AND_PANELS.md` Part 1.
        //
        // Three positions of one control, ORDERED BY CAPABILITY: each
        // mode's tab set is a superset of the one before it. That ordering
        // is the whole premise —
        //
        //   > The three positions are ordered by capability… A slider says
        //   > that; three toggle buttons do not, and a dropdown hides the
        //   > current position behind a click. The ordering is the
        //   > information.
        //
        // — and `each_mode_is_a_subset_of_the_next` in `super`'s tests is
        // what keeps it true. `egui-shell` cannot check it: a different
        // application may ship three unrelated workspaces, and assuming an
        // ordering there would be the framework legislating about content.
        //
        // Modes are declared before tabs so this list reads as the summary
        // of the ribbon that follows.
        //
        // **Pages is in Review.** Operator decision, 2026-08-13, reversing
        // an earlier draft that excluded it on the reasoning that delete,
        // extract and merge are structural. Reviewing a drawing set means
        // rotating a sheet to read it, extracting the two pages you were
        // asked about, and inserting a marked-up revision — all reviewer
        // work. The stance that matters is *the page content is not yours
        // to alter*, and page operations do not alter content.
        //
        // The contextual Format tab is in NO mode's list, and is present
        // in all three. See `format.rs`.
        // -------------------------------------------------------------------
        .with_mode(Mode::new("read", ribbon::mode_read(), ["file", "view"]))
        .with_mode(Mode::new(
            "review",
            ribbon::mode_review(),
            ["file", "view", "pages", "markup", "measure"],
        ))
        .with_mode(Mode::new(
            "edit",
            ribbon::mode_edit(),
            [
                "file", "view", "pages", "edit", "markup", "measure", "tools",
            ],
        ))
        // -------------------------------------------------------------------
        // TABS
        // -------------------------------------------------------------------
        .with_tab(file::tab())
        .with_tab(view::tab())
        .with_tab(pages::tab())
        .with_tab(edit::tab())
        .with_tab(markup::tab())
        .with_tab(measure::tab())
        .with_tab(tools::tab())
        .with_contextual_tab(format::tab())
        // -------------------------------------------------------------------
        // QUICK-ACCESS TOOLBAR — `RIBBON_IA.md` §6, unchanged from today.
        //
        // Open, Save a copy, Undo, Redo. Two of them mirror the File tab,
        // which amendment P1a permits explicitly: *the QAT and the status
        // bar are shortcut surfaces, not tabs. A command may appear on
        // exactly one tab and additionally on the QAT and/or the status
        // bar.* The other two appear nowhere else, which is why the QAT
        // being always visible is load-bearing rather than convenient.
        // -------------------------------------------------------------------
        .with_qat(["file.open", "file.save", "edit.undo", "edit.redo"])
        // -------------------------------------------------------------------
        // THE TRAILING REGION — `OPERATOR_REQUESTS.md` O122.
        //
        // The operator, 2026-09-04: *"beside our read-review-edit buttons at
        // the top there should be an open in acrobat button."* The far right of
        // the tab-strip row, past the mode selector, which is a region
        // `egui-shell` grew for this and which nothing else uses.
        //
        // ★★★ `shown_when("acrobat.available")` is the whole of R9 for this
        // control, and it is the reason the command is registered
        // unconditionally rather than only on a machine that has an Acrobat.
        // The registry is built once at start-up; the path to Acrobat is a
        // SETTING, and O122's escape hatch requires that typing one makes the
        // button appear without a restart. A condition is re-read every frame
        // and a registration is not.
        //
        // ★ It is on NO tab, which is the first command in this manifest of
        // which that is true, and it is deliberate rather than an oversight.
        // `RIBBON_IA.md` P1 says a command has one discoverable home; this
        // one's home is a fixed position in the chrome that is visible in every
        // mode, which is a stronger form of the same guarantee than a tab
        // provides. Putting it on the File tab as well would give an operator
        // two places to press for one act, one of which appears and disappears
        // — and `Shell::validate`'s uniqueness rule walks tabs only, so nothing
        // would have complained.
        .with_trailing([Item::command("file.open_in_acrobat").shown_when(ACROBAT_AVAILABLE)])
        // -------------------------------------------------------------------
        // THE LEFT RAIL — `OPERATOR_REQUESTS.md` O123 part 7 and O126.
        //
        // The permanent vertical strip down the left dock's outer edge: the
        // five panel tabs, the navigate selectors, the selection controls and
        // rotate. `rail::groups` carries the list and the whole argument,
        // including the two things that differ deliberately from the approved
        // mockup (the lasso, and `edit.select_all`) and the ⚠ that putting
        // rotate here lets Read dirty a document.
        //
        // ★★ Data on the manifest rather than a builder callback, for the
        // reason `SHELL_FRAMEWORK.md` states in one line: *"a rail that only
        // `pdfcer-gui` knows about breaks it quietly."* `Shell::validate` walks
        // it, `merge` filters it, and an operator overlay can reorder it —
        // exactly as for the QAT, the tabs, the trailing region and the keymap.
        .with_rail(rail::groups())
        // -------------------------------------------------------------------
        // KEYMAP
        //
        // Chords are opaque strings here; parsing them into modifiers and a
        // key is the renderer's job, and doing it in the manifest would
        // mean a manifest could not be read by a tool that does not link
        // egui.
        //
        // Every binding below is one the shipped build already honours and
        // already documents in its own keyboard-shortcuts window, EXCEPT
        // the last five, which are new and are marked. Carrying the
        // existing spellings verbatim — including `Ctrl+Y or Ctrl+Shift+Z`
        // as two separate bindings for redo — is what stops the shortcut
        // list and the keymap from being two sources of truth.
        //
        // ★ THIS KEYMAP IS THE ONLY PLACE A CHORD IS BOUND TO A MEANING.
        //
        // `crate::app::keyboard::commands` reads it at run time and hands the
        // command id to `PdfcerApp::dispatch_command` — the same dispatcher a
        // ribbon click reaches — so a chord cannot disagree with the control
        // that shares its command. It used to: this file bound `Ctrl+0` to
        // `view.zoom_actual` and `Ctrl+2` to `mode.review` while
        // `app::keyboard` bound the same two chords to fit page and fit
        // width and got there first, because nothing dispatched this keymap
        // at all. Two operator-visible surfaces named the chords, and both
        // were lying.
        //
        // The list below is therefore load-bearing rather than explanatory,
        // and `app::keyboard::tests::no_chord_has_two_owners` enforces it:
        // every chord `app::keyboard::collect` binds outright is checked
        // against this map, in every spelling, and the test fails naming the
        // chord and both claimants. Add a binding here for one of them and it
        // goes red.
        //
        // NOT bound here, deliberately:
        //
        //   Delete / Backspace   The shortcut window lists these for
        //                        "delete the selected pages", but the same
        //                        keys delete a selected OBJECT on the
        //                        canvas. Which one applies depends on where
        //                        focus is, and a global binding cannot
        //                        express that. It stays canvas-scoped.
        //   PageUp / PageDown / Home / End / Ctrl+Plus / Ctrl+Minus
        //                        Viewer navigation, handled in the app's
        //                        own keyboard layer against the view state.
        //                        They are not ribbon commands and putting
        //                        them here would give them a second owner.
        // ★ Ctrl+F IS bound here now, and the comment it replaces is worth
        // keeping visible because it was right about the control and wrong
        // about the chord:
        //
        //     Ctrl+F   Find lives in the status bar, which this manifest
        //              does not describe.
        //
        // The first clause still holds — `RIBBON_IA.md` §6 puts the Find
        // TOGGLE on the status bar, and `edit.find` is on no tab. The second
        // does not follow from it. A keymap is not a description of controls;
        // it is the ONE place a chord is bound to a meaning, which is the
        // property the two-owner defect was fixed by establishing. Leaving
        // Ctrl+F out of it would have meant binding it in
        // `crate::app::keyboard` instead — a second owner, in the module whose
        // whole header is about why there must not be one.
        //
        // So the rule is: the manifest binds every chord, whatever surface
        // the control lives on; a surface this manifest does not describe is
        // a reason for the command to be on no TAB, not a reason for its chord
        // to be bound somewhere else.
        // -------------------------------------------------------------------
        // ★ Ctrl+N — the universal chord, bound the day its command landed.
        //
        // Acrobat, Inkscape and SolidWorks all bind Ctrl+N to New, as does
        // every other document application; there was nothing to decide here
        // beyond whether it was allowed to be bound at all, and the rule in
        // `crate::app::keyboard::commands`' header says it is: *"a chord here
        // dispatches a command, and a command with no dispatch arm would trace
        // `command-unimplemented` on a keypress that used to do nothing
        // quietly. They land with their commands."* `file.new` has an arm, so
        // the chord lands with it — and `Key::N` joins `DERIVED`'s spelling
        // table in the same edit, because a chord this file binds and that
        // table cannot spell is a chord no keypress delivers. That is the
        // defect `Ctrl+O` sat in for the whole life of the ribbon.
        .with_binding("Ctrl+N", "file.new")
        // ★ Ctrl+Alt+N — Inkscape's own chord for the same split, which is the
        // split this pair copies: Ctrl+N makes a document, Ctrl+Alt+N chooses
        // what kind. `RIBBON_IA.md` §5.1 specifies the row and not a chord, so
        // this is the shell's choice rather than the IA's, and it is a cheap
        // one to overturn — it is one line here and one row in the shortcuts
        // window, which reads the keymap rather than restating it.
        //
        // Ctrl+Alt is AltGr on some European layouts. Accepted here because the
        // chord is a convenience on a command that is also two clicks away in
        // the File band, so a layout that cannot spell it loses nothing.
        .with_binding("Ctrl+Alt+N", "file.new_from_template")
        .with_binding("Ctrl+O", "file.open")
        // ★ **Ctrl+P**, 2026-08-20, on the operator's report: *"still no ctrl+c,
        // ctrl+v, ctrl+x or ctrl+p shortcuts that were requested ages ago"*.
        //
        // The other three were bound and are not the whole of what he means (see
        // `dispatch`'s clipboard arms - they reach markup and refuse page content,
        // which is an engine gap filed on 2026-08-20). **This one was simply
        // absent**, and it is the single most universal chord in any document
        // application. It went unnoticed because Print has a ribbon control, a QAT
        // slot and a menu row, so every surface that lists commands showed it and
        // only the keyboard did not.
        //
        // The lesson is the file-size one in a different suit: a fact that is true
        // in four places and false in the fifth is invisible to anything that
        // checks one place. `keymap_offers_the_chords_a_document_application_must`
        // is the gate, and it asserts the LIST rather than this line.
        .with_binding("Ctrl+P", "file.print")
        // ★★★ **Ctrl+S is SAVE**, 2026-08-20. It was bound to Save-a-copy, so
        // the most reflexive chord in computing opened a file dialog every
        // time. Save-a-copy takes Ctrl+Shift+S, which is where every other
        // program in this class puts Save-as.
        // ★★★ `Ctrl+A`, and it must NOT steal from a text edit.
        //
        // `canvas::textsel::clipboard` already answers `Ctrl+A` with
        // `TextKey::SelectAll` while a text selection or caret owns the
        // keyboard, and that path runs first. This binding is what the chord
        // means on the CANVAS — which is where an operator who has lost an
        // object off the sheet is pressing it.
        .with_binding("Ctrl+A", "edit.select_all")
        .with_binding("Ctrl+S", "file.save")
        .with_binding("Ctrl+Shift+S", "file.save_copy")
        .with_binding("Ctrl+Z", "edit.undo")
        .with_binding("Ctrl+Y", "edit.redo")
        .with_binding("Ctrl+Shift+Z", "edit.redo")
        // ★★ **The four pointer tools on bare letters**, 2026-08-19.
        //
        // `V` `A` `T` `H` is not a preference — it is the layout Illustrator,
        // Photoshop, InDesign, Figma, Affinity and Inkscape (which uses `S`/`N`
        // for the two arrows but `T` and `H` identically) have converged on, and
        // it is what an operator's hands already know. `RIBBON_IA.md` §3's rule
        // for chords is *"take the one the product class has settled on unless
        // it collides"*, and none of these collides: this shell binds no bare
        // letter to anything else.
        //
        // ★ Bare, not `Ctrl+`. A bare letter is safe here precisely because
        // `canvas::keys` gates every keystroke on `text_edit_focused()` — the
        // guard whose ABSENCE was `DEFECTS.md` D1, the old shell's Delete key
        // dying after any canvas click. Typing `v` into a form field or into the
        // canvas caret must never arm a tool, and the one place that could go
        // wrong is the place this project has already been burned and fixed.
        // ★★ The three every program has bound since 1983.
        //
        // `Ctrl+C` is bound here **and** claimed by `canvas::textsel::clipboard`
        // when a text range is swept. That is not a collision to resolve, it is
        // the resolution: text wins, because a swept range is a more specific
        // statement than a selected annotation and the operator made it more
        // recently. Every program in the class answers the same way, and
        // `textsel` runs first by construction — it reads the keys before the
        // command dispatcher sees them.
        .with_binding("Ctrl+X", "edit.cut")
        .with_binding("Ctrl+C", "edit.copy")
        .with_binding("Ctrl+V", "edit.paste")
        // ★ Ctrl+Shift+V, the operator's own choice, 2026-08-29. Unbound
        // before this and unclaimed by egui, so there is no collision to
        // resolve. The convention it follows is the one Word, Excel and every
        // browser use for "paste, but differently": the same key with Shift.
        .with_binding("Ctrl+Shift+V", "edit.paste_duplicate")
        .with_binding("V", "view.tool_select")
        .with_binding("A", "view.tool_node")
        .with_binding("T", "view.tool_text")
        .with_binding("H", "view.tool_hand")
        .with_binding("Ctrl+E", "edit.text")
        .with_binding("Ctrl+Shift+E", "edit.add_text")
        // ★★ **The document chords**, 2026-08-19 with the tab strip.
        //
        // `Ctrl+Tab` / `Ctrl+Shift+Tab` to cycle and `Ctrl+W` to close: the
        // three every tabbed application on this desktop has bound, and
        // therefore the three an operator will try first.
        //
        // ★ `Ctrl+Tab` is safe to bind here and a bare `Tab` would not be.
        // egui's own focus system claims `Tab` — `Memory` matches
        // `Key::Tab if !modifiers.any()` for the next widget and
        // `modifiers.shift_only()` for the previous — so both of these carry a
        // modifier combination egui's focus walker deliberately ignores. A
        // binding on bare Tab would move focus AND fire the command, which is
        // the kind of double meaning that reads as a random control gaining
        // focus every time you switch document.
        .with_binding("Ctrl+Tab", "view.next_document")
        .with_binding("Ctrl+Shift+Tab", "view.previous_document")
        .with_binding("Ctrl+W", "file.close")
        // ★ Was bound to `edit.copy_page_text` until 2026-08-14. The COMMAND
        // moved to File ▸ Export and the chord followed it here, in the same
        // edit, because this keymap is the only place a chord is bound to a
        // meaning: a binding left pointing at the old id would not fail the
        // build — an unknown id is a disclosed skip, not an error — it would
        // simply make `Ctrl+Shift+C` do nothing, which is the silent failure
        // this block's header is entirely about.
        //
        // The move is what makes the chord work in **Read**: the gate in
        // `crate::app::modes::capability::offers_command` lets a chord reach a
        // command the active mode shows, and Read shows File.
        .with_binding("Ctrl+Shift+C", "file.copy_page_text")
        .with_binding("Ctrl+F", "edit.find")
        .with_binding("Ctrl+0", "view.zoom_actual")
        .with_binding("[", "pages.rotate_left")
        .with_binding("]", "pages.rotate_right")
        .with_binding("Alt+Up", "pages.move_up")
        .with_binding("Alt+Down", "pages.move_down")
        // New. `RIBBON_IA.md` §3 records Ctrl+H and F11 as the only way to
        // reach read mode and full screen in the shipped build — they have
        // no ribbon control at all — but no such string appears anywhere in
        // its source, so the operator has no way to discover them. Binding
        // them here alongside the View ▸ Window controls gives them both a
        // visible home and a documented chord.
        .with_binding("Ctrl+H", "view.read_mode")
        .with_binding("F11", "view.fullscreen")
        // New. `MODES_AND_PANELS.md` Part 1 §6 specifies these three, and
        // adds that the selector must also be a real focusable control with
        // arrow-key movement — not a mouse-only affordance.
        .with_binding("Ctrl+1", "mode.read")
        .with_binding("Ctrl+2", "mode.review")
        .with_binding("Ctrl+3", "mode.edit");
    // -------------------------------------------------------------------
    // CONTEXT MENUS — `RIBBON_IA.md` §6, "the other half of making
    // selection meaningful".
    //
    // Assigned rather than chained because `egui_shell::Shell` has no
    // `with_menus` builder, and adding one is `egui-shell`'s change to
    // make, not this crate's. The field is public and the assignment is
    // one line; a builder method would be nicer and is not worth reaching
    // across a crate boundary for.
    //
    // `Shell::validate` does NOT check this half — it walks tabs, the QAT
    // and the keymap — so `super::menus`' own tests carry the checks that
    // matter: every command a menu names is registered, and every menu has
    // something to offer.
    // -------------------------------------------------------------------
    shell.menus = Some(super::menus::built_in());

    // -------------------------------------------------------------------
    // THE COLLAPSE LADDER — S3 of `RIBBON_SCALING.md`.
    //
    // Applied last, over the finished tab list, because a collapse
    // priority is a RANKING OF GROUPS AGAINST EACH OTHER and the only way
    // to review a ranking is to see all of it in one place. See
    // `ladder`'s header for the rule the ranking follows (Word's: the
    // group that never collapses is the one carrying the verb the
    // operator came to the tab for, not the smallest one) and for the two
    // tests that keep the table honest as the tabs move under it.
    // -------------------------------------------------------------------
    ladder::apply(&mut shell);
    shell
}

/// **The condition, published by the application each frame, under which an
/// OBJECT is selected on the page.**
///
/// One spelling, one source. It is the enable predicate of `format.delete`
/// and `format.properties`, and the condition
/// [`super::menus::MenuHost::with_condition`] corrects when a right-click
/// selects the object under the pointer. Two surfaces reading two spellings
/// of one condition is a defect whose only symptom is a menu row that is
/// greyed while the thing it acts on is plainly selected.
///
/// # ★★ It STOPPED being the Format tab's `visible_when` on 2026-08-27,
/// and that is the whole reason it is now a literal
///
/// It used to read `pub const SELECTION_ANY: &str = format::VISIBLE_WHEN;`,
/// with a doc comment naming three surfaces that shared one condition. That
/// was true and it stopped being true when the Format tab grew a **Font**
/// group: the tab now carries controls for two different kinds of selection --
/// a page object, addressed by paint-order index, and a swept text range,
/// addressed by run -- so *"is the Format tab about anything?"* is a strictly
/// wider question than *"is an object selected?"*. The tab's condition moved
/// to `selection.formattable`; this one stayed where it was and kept its two
/// honest readers.
///
/// ★ The alias is what made the drift dangerous rather than merely untidy.
/// Changing [`format::VISIBLE_WHEN`] in place would have silently retargeted
/// the **canvas context menu**, which has nothing to do with the Format tab
/// and whose Delete would then have been enabled by a text sweep -- a
/// destructive control lit by a selection it cannot act on. Spelling it out
/// here is what makes the two independently editable.
pub const SELECTION_ANY: &str = "selection.any"; // ui-text-exempt: a condition name, never displayed

/// **Something Delete and Properties can act on** — wider than
/// [`SELECTION_ANY`], and named separately for the reason `app::conditions`
/// gives at the site that publishes it: a selected **form field** lives in
/// `doc.selected_field`, not in `SelectionState`, so `selection.any` is false
/// while one is selected.
///
/// ★ The Format **tab** still takes `SELECTION_ANY`. A field has no font, no
/// stroke and no fill for that tab to offer, so widening the tab's own
/// predicate would draw a band of controls that cannot act on what is
/// selected.
pub const SELECTION_ACTIONABLE: &str = "selection.actionable"; // ui-text-exempt: a condition name, never displayed

/// **An Acrobat was found on this machine, or the operator has pointed
/// pdfcer at one** — the condition under which `file.open_in_acrobat` is
/// DRAWN AT ALL. `OPERATOR_REQUESTS.md` O122.
///
/// # ★★★ Why this is a `visible_when` and never an `enabled_when`
///
/// R9, exactly: *an unavailable capability renders nothing; greying is
/// reserved for **temporarily** unavailable and is always explained on hover.*
/// "This machine has no Acrobat" is not temporary and there is no hover
/// sentence that would help — the remedy is installing a different program,
/// which is not something a tooltip can walk somebody through and not
/// something this shell should nag about on every hover for the life of the
/// installation.
///
/// The command's own `enabled_when("doc.open")` is the greying, and it is the
/// legitimate case: *no document open* is temporary, is the operator's to fix
/// in one click, and IS explained on hover.
///
/// # ★★ Published by `PdfcerApp::conditions` from ONE resolved viewer
///
/// Not from a fresh registry read per frame — see `crate::acrobat`. The
/// resolution is cached on the application and recomputed when the setting
/// changes, so this condition and the path the button will actually launch are
/// the same fact rather than two reads that could disagree.
pub const ACROBAT_AVAILABLE: &str = "acrobat.available"; // ui-text-exempt: a condition name, never displayed

/// **The engine would not refuse a delete of what is selected** — the
/// condition under which `format.delete` is DRAWN AT ALL.
///
/// # ★★★ Why it lives here rather than in [`format`], and the precedent is one
/// screen up
///
/// [`SELECTION_ANY`] carries the account of what an alias cost: while it read
/// `= format::VISIBLE_WHEN`, changing the Format tab's own condition would have
/// silently retargeted the **canvas context menu**, whose Delete would then have
/// been lit by a text sweep. This constant has exactly the two readers that one
/// has — `manifest::format`'s Selection group and `menus`' `CANVAS_OBJECT` — so
/// it is spelled out in the one place both can see, and neither owns it.
///
/// # ★★★ It is a `visible_when`, and [`SELECTION_ACTIONABLE`] is still the
/// `enabled_when`. Two predicates on one control
///
/// | predicate | asks | when false |
/// |---|---|---|
/// | [`SELECTION_ACTIONABLE`] | *is there anything to delete?* | **greyed** — a selection is one click away, which is exactly the temporary, operator-fixable condition R9 reserves greying for |
/// | this | *would the engine refuse?* | **absent**, with a sentence in the Properties panel — a certification signature is neither temporary nor arguable |
///
/// Collapsing them either way is a defect. Greying on this one would promise an
/// operator that selecting differently might help; hiding on the other would
/// make Delete flicker in and out of the ribbon on every click.
///
/// # What clears it, and what deliberately does not
///
/// Cleared **only** for a selected annotation that `annotation_deletion_refusal`
/// or §12.5.3 Table 165's `Locked` bit forbids — so its default is *true*, and
/// it stays true with nothing selected, with a content object selected, and with
/// a form field selected. `app::conditions` argues at length why that direction
/// is the safe one: a control drawn where it refuses is the defect being fixed,
/// and a control withheld where it would have worked is a worse one, because the
/// operator has no gesture left that reports it.
pub const DELETE_PERMITTED: &str = "selection.delete_permitted"; // ui-text-exempt: a condition name, never displayed

/// **The `Item::Custom` kinds of the Format ▸ Font controls.**
///
/// Three, and each is a control a **button cannot be** -- which is
/// [`CUSTOM_BACKED`]'s bar and the only reason any of them is drawn this way:
/// a face chooser has to ask *which* of the page's fonts, a size field has to
/// accept a number, and a colour needs a swatch that shows the current one.
///
/// Same one-spelling-one-source rule as [`RECENT_FILES`], and the same reason
/// it is a rule: [`COLOUR_SWATCH`]'s own note records the manifest writing a
/// literal kind that **no renderer ever matched**, so a captioned group drew
/// an empty band for the whole of v0.1.0 with nothing anywhere reporting the
/// mismatch. The shell reserves the space, the application declines to draw,
/// and the symptom is a gap.
pub const FONT_FACE: &str = "font_face"; // ui-text-exempt: a custom-item kind, never displayed
/// See [`FONT_FACE`].
pub const FONT_SIZE: &str = "font_size"; // ui-text-exempt: a custom-item kind, never displayed
/// See [`FONT_FACE`].
pub const FONT_COLOUR: &str = "font_colour"; // ui-text-exempt: a custom-item kind, never displayed

/// **The `Item::Custom` kind of the Recent-documents control.**
///
/// One spelling, one source: the manifest writes it in File ▸ File and
/// [`crate::app::PdfcerApp::ribbon_band`]'s custom-item renderer matches on
/// it. A mismatch between those two is invisible — the shell reserves the
/// item's space, the application declines to draw it, and the band shows a
/// gap — so the string is a constant rather than a literal in two files.
pub const RECENT_FILES: &str = "recent_files"; // ui-text-exempt: a custom-item kind, never displayed

/// **The `Item::Custom` kind of the Markup ▸ Style controls.**
///
/// Same one-spelling-one-source rule as [`RECENT_FILES`], and this constant
/// arrived late for a reason worth recording: the manifest wrote the literal
/// `"colour_swatch"` from S2 and **no renderer ever matched it**, so the Style
/// group drew a caption over an empty band for the whole of v0.1.0. That is
/// precisely the invisible failure the constant's existence is meant to
/// prevent — the shell reserves the item's space, the application declines to
/// draw it, and nothing anywhere reports a mismatch.
///
/// ★ It is deliberately **not** in [`CUSTOM_BACKED`]. That register is for
/// commands whose only ribbon control is a custom item, and this item backs no
/// command at all: it edits `PdfcerApp::pen`, raises no `Action`, has no undo,
/// and returns no handler token. Listing it there would claim a command id
/// that does not exist.
pub const COLOUR_SWATCH: &str = "colour_swatch"; // ui-text-exempt: a custom-item kind, never displayed

// ===========================================================================
// CUSTOM_BACKED
// ===========================================================================

/// **Registered commands whose only ribbon control is an [`Item::Custom`],
/// and the item that draws each.**
///
/// `(command id, custom kind, why)`.
///
/// # Why this register has to exist
///
/// `egui_shell::Shell::command_references()` walks tab groups, the QAT and
/// the keymap — the places a command *id* can appear. A `Custom` item carries
/// no id (that is the whole point of it: the shell reserves space and the
/// application draws whatever it likes), so a command reachable only through
/// one is invisible to every reachability check built on that function.
///
/// `super::tests::no_registered_command_is_orphaned` is exactly such a check,
/// and it is a good one: it catches the rename that leaves a command
/// registered and referenced by nothing, which nothing in `egui-shell` can
/// see. Without this register it would have to be either weakened — which
/// gives up the rename check for every other command — or satisfied by
/// putting a second, redundant button on the tab.
///
/// So the exception is **data**, exactly as [`PLANNED`] and [`DIRECTED`] are:
/// enumerable, tested in both directions (the id is registered, and the kind
/// really appears in the manifest), and carrying its reason. A command listed
/// here whose custom item was deleted fails the suite rather than becoming an
/// unreachable command with a note explaining why it used to be fine.
///
/// # The bar for an entry
///
/// The control must genuinely be one a **button cannot be**. `file.recent`
/// qualifies because the command needs an operand — *which* of ten documents
/// — that a button has no way to ask for, and the alternatives are ten
/// commands or a command that opens whichever file it feels like. A command
/// that could have been a button and was drawn some other way for taste does
/// not belong here; it belongs on the tab.
pub const CUSTOM_BACKED: &[(&str, &str, &str)] = &[
    (
        crate::shell::commands::FILE_RECENT,
        RECENT_FILES,
        "The Recent menu in File ▸ File. The command opens a document from the recent list, \
         and WHICH document is a ten-way choice a button cannot express — so the ribbon \
         control is a menu the application draws (`app::recent::menu`), which parks the chosen \
         path and returns this command's token. Same shape as `file.open`, whose operand comes \
         from a file dialog: the picker asks, the command acts.",
    ),
    // -----------------------------------------------------------------------
    // The Format ▸ Font group, 2026-08-27. Three entries, and each clears
    // the bar above for the same reason in a different shape: the command
    // needs an OPERAND that a button cannot ask for.
    //
    // ★ They are drawn this way because the alternative for each is absurd
    // in exactly the way `file.recent`'s is: a button per font on the page, a
    // button per point size, a button per colour.
    //
    // ★★ And they are REGISTERED, rather than being three anonymous custom
    // items like `COLOUR_SWATCH` beside them in Markup ▸ Style. The
    // difference is not presentational: those edit `PdfcerApp::pen` —
    // application state, no document, no undo entry — while these three raise
    // an `Action::TextStyle` that rewrites a content stream and lands in the
    // engine's command log. A capability that edits the document must be a
    // registered command, because registering one is the only way this shell
    // may learn a capability exists (R8), and because a build compiled without
    // it must lose the control rather than draw a dead one.
    // -----------------------------------------------------------------------
    (
        "format.font",
        FONT_FACE,
        "The face chooser in Format ▸ Font. WHICH of the fonts this page carries is a choice \
         with as many answers as the page has fonts, which a button cannot ask; and the list is \
         built from the document's own inventory rather than from a list of typefaces, because \
         `set_font` SELECTS an existing resource and does not create one.",
    ),
    (
        "format.font_size",
        FONT_SIZE,
        "The size field in Format ▸ Font. A point size is a number the operator types or \
         drags, and a button is a control with one value. It commits on release rather than on \
         change, because each commit is a content-stream rewrite and one undo entry.",
    ),
    (
        "format.font_colour",
        FONT_COLOUR,
        "The colour swatch in Format ▸ Font. A colour is a three-dimensional choice and the \
         control must also SHOW the current one, neither of which a button does — and for a \
         run painted in CMYK or a spot colour the swatch is replaced by a sentence, which is a \
         second thing a button has no way to express.",
    ),
];

/// A captioned band of items.
///
/// A two-line convenience over `Group::new(..).with_items(..)`, because
/// this manifest writes thirty-three of them and the builder chain is the
/// noisiest thing on the page when every group is one expression.
fn group(id: &str, caption: &str, items: impl IntoIterator<Item = Item>) -> Group {
    Group::new(id, caption).with_items(items)
}

/// The same, laid out on **two rows** even when one would fit.
///
/// ★ `OPERATOR_REQUESTS.md` O97 — *"our display buttons should be on two rows to
/// save space."* For a cluster of icon-only peers that is a **choice** rather
/// than a list: four square buttons in a row is a strip, and the same four as a
/// 2 × 2 block is half the width and reads as one control. See
/// [`egui_shell::manifest::Group::prefer_rows`] for what the hint does and does
/// not promise.
///
/// A named constructor rather than a `.with_prefer_rows(2)` on the call, so that
/// a tab module reads as a list of groups and the one group that is shaped
/// differently says so in its first word.
fn group_two_rows(id: &str, caption: &str, items: impl IntoIterator<Item = Item>) -> Group {
    group(id, caption, items).with_prefer_rows(2)
}

/// A command reference, by id.
///
/// Named `command` rather than used as `Item::command` so that a tab
/// module's item lists read as a list of commands, which is what they are.
pub(super) fn command(id: &str) -> Item {
    Item::command(id)
}

/// A command drawn **icon-only** — `RIBBON_SCALING.md` §5.1.
///
/// For a tight cluster of peers where the icon is distinctive and the
/// operator's eye is already in the right group: the four page displays, the
/// four pointer tools, the two page rotations, cut/copy/paste, the four text
/// markups. Word's own icon-only clusters are the same shape — bold, italic,
/// underline; the alignment buttons — and the reason they work is that
/// **position in a labelled group teaches the meaning**, not the label on each
/// control.
///
/// ★ Safe to ask for even when it cannot be honoured. `sizing::resolved` falls
/// back to the labelled form unless the command names an icon, carries a
/// tooltip **and** a painter is installed, so a command that gains or loses an
/// icon does not need this list audited.
pub(super) fn icon_only(id: &str) -> Item {
    Item::command(id).sized(ItemSize::Small)
}

/// A command drawn **large** — icon above label, spanning the band's rows.
///
/// ★★★ **The restriction relaxed on 2026-09-04, and the rule that replaced it
/// is narrower than "anywhere".**
///
/// It used to read *"used only for a group whose single item it is"*, on the
/// grounds that `sizing`'s layout rule hoists Large items to the front of
/// their group, so promoting one item of a multi-item group would silently
/// re-order what `RIBBON_IA.md` settled — and the ribbon IA is not this
/// file's to amend.
///
/// That reasoning is intact. What changed is that `mockups/pdfcer-shell.html`
/// is now a specification of this band rather than a sketch of it, the
/// operator having said *"I want everything to look exactly like that
/// including sizing"*, and it draws a great many controls large. So the rule
/// is now the **consequence** rather than the proxy for it:
///
/// > **A `large` may be added only where hoisting is a no-op** — i.e. the
/// > promoted items are already the leading run of their group, or the whole
/// > group is promoted together. Anywhere else, the promotion is an IA change
/// > and belongs in `RIBBON_IA.md` first.
///
/// `manifest::tests::large_items_already_lead_their_group` asserts it, so the
/// rule is checked rather than remembered. Two places where the mockup
/// disagrees with the shipped manifest were therefore **left alone**, and are
/// recorded here rather than silently skipped:
///
/// | mockup | shipped | why not promoted |
/// |---|---|---|
/// | Edit ▸ Content draws `Edit text` and `Add text` large, *after* a column of `Select all` / `Reflow paragraph` | all four Medium | promoting the two would hoist them **in front of** the column, which is the reordering this rule forbids. The mock authors its own column order; the band derives one. |
/// | View ▸ Navigate and Pages ▸ Transform draw their tools large | [`icon_only`] | these are the `RIBBON_SCALING.md` §5.1 icon-only clusters, whose whole argument is that *position in a labelled group teaches the meaning*. Reversing that is a decision about the tool strip, not about a glyph size. |
///
/// It is also where it reads best: a lone control in a captioned group looks
/// stranded at Medium, and Word gives exactly this treatment to its own
/// one-command groups — Dictate, Editor, Add-ins.
pub(super) fn large(id: &str) -> Item {
    Item::command(id).sized(ItemSize::Large)
}

// ===========================================================================
// The two registers — MOVED, 2026-08-27
//
// `PLANNED` (every command `RIBBON_IA.md` specifies that this manifest does
// not emit, and why) and `DIRECTED` (the ones emitted despite carrying no `G`
// mark) now live in `registers.rs`, and are re-exported below so that every
// call site still writes `manifest::PLANNED`.
//
// They left under R2 when the Format tab's Font group took this file to
// within eight lines of the ceiling, and they were the right ~640 lines to
// take: they are pure data, nothing in this file branches on them, and they
// change on a different occasion from everything else here. See that module's
// header for the seam and for why their tests deliberately stayed behind.
// ===========================================================================

mod registers;

pub use registers::{DIRECTED, PLANNED, TAB_SCOPED};

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::manifest::{Group, Item, Tab};
    use std::collections::BTreeSet;

    /// Every command id the manifest emits, in document order.
    fn emitted() -> Vec<String> {
        built_in()
            .command_references()
            .into_iter()
            .map(|(_, id)| id)
            .collect()
    }

    /// ★★★ **Every `Large` item already leads its group**, so promoting one
    /// never reorders the band.
    ///
    /// This is the rule that replaced [`super::large`]'s old *"only for a
    /// group whose single item it is"* restriction on 2026-09-04, when
    /// `mockups/pdfcer-shell.html` became this band's specification and a
    /// great many controls became Large.
    ///
    /// # Why the rule needs a test rather than a sentence
    ///
    /// `egui_shell::ribbon::sizing`'s layout rule is that **Large items lead
    /// their group** — they are drawn first, in a horizontal run at the
    /// group's left, and everything else wraps into the rows beside them. That
    /// is not a preference; a Large control spans the rows and therefore
    /// cannot live *inside* the row wrapping.
    ///
    /// The consequence is that writing `large("x")` in the middle of a group
    /// **silently hoists `x` to the front of it**. Nothing fails, nothing
    /// warns, and the only evidence is that the band's controls are in a
    /// different order from the one `RIBBON_IA.md` argued for — an order the
    /// operator reaches for by position, in groups like Cut / Copy / Paste,
    /// where reordering is the whole cost.
    ///
    /// So: a `Large` item is legal exactly where hoisting is a no-op, i.e.
    /// where the Large items are already a **prefix** of their group's item
    /// list. Separators and custom items count as non-Large for this purpose,
    /// which is the strict reading — a `Recent ⌄` gallery hoisted past is just
    /// as reordered as a command.
    ///
    /// ★ Written as a scan for the first non-Large item followed by a Large
    /// one, rather than as a whitelist of blessed groups. A whitelist is a
    /// second copy of the manifest and goes stale; this cannot, because it is
    /// derived from the manifest it checks.
    #[test]
    fn large_items_already_lead_their_group() {
        for tab in built_in().tabs() {
            for group in tab.groups() {
                let mut seen_other = None::<usize>;
                for (i, item) in group.items().iter().enumerate() {
                    let is_large = matches!(item, Item::Command { size, .. } if *size == egui_shell::manifest::ItemSize::Large);
                    match (is_large, seen_other) {
                        (true, Some(at)) => panic!(
                            "`{}` ▸ `{}` item {i} is Large but item {at} before it is not, so \
                             `ribbon::sizing` will hoist it to the front of the group and the \
                             band will draw this group in an order the manifest does not \
                             state. Either promote the whole leading run, or leave this item \
                             Medium and take the size question to RIBBON_IA.md",
                            tab.id, group.id
                        ),
                        (false, None) => seen_other = Some(i),
                        _ => {}
                    }
                }
            }
        }
    }

    /// ★★ **…and the manifest actually contains some Large items**, so the
    /// scan above is not vacuous.
    ///
    /// A manifest with no Large item at all satisfies
    /// [`large_items_already_lead_their_group`] perfectly, and would go on
    /// satisfying it after somebody deleted every `large(…)` call in the tree.
    /// The count is a floor rather than an exact number — the exact number is
    /// a manifest decision that will move, and pinning it here would make an
    /// IA change fail a test about hoisting.
    #[test]
    fn the_manifest_draws_large_controls_at_all() {
        let large = built_in()
            .tabs()
            .iter()
            .flat_map(|t| t.groups())
            .flat_map(|g| g.items())
            .filter(|i| {
                matches!(i, Item::Command { size, .. } if *size == egui_shell::manifest::ItemSize::Large)
            })
            .count();
        assert!(
            large >= 10,
            "the manifest emits {large} Large items. The mockup draws well over a \
             dozen, and a band with none of them is the flat row of identical \
             buttons the 2026-09-04 pass was asked to replace"
        );
    }

    /// The shape of the ribbon, pinned.
    ///
    /// Not a change-detector for its own sake: these numbers are quoted in
    /// prose, in five module headers, as the description of the layout
    /// `RIBBON_IA.md` §5 specifies. A count that drifts silently makes
    /// every one of them wrong, and the failure message says which way it
    /// moved.
    ///
    /// **Failing here means editing prose, not just the literal.** The
    /// group count went 31 → 32 with this test passing on the new number
    /// and five headers still saying "thirty-one", because pinning a value
    /// does not pin the sentences that repeat it. The sites are:
    ///
    /// - this module's header, and [`group`]'s;
    /// - [`crate::shell`]'s submodule table;
    /// - [`crate::shell::ron`]'s header (groups **and** key bindings);
    /// - [`crate::text::ribbon`]'s header.
    ///
    /// ★★ …and it happened AGAIN on 2026-08-27, in both directions at once.
    /// The literal below said 32 while four of the five prose sites said
    /// "thirty-one" and the fifth said "thirty-two" — so the sentences had
    /// been out of step with the number *and with each other* for an unknown
    /// stretch, and the Format tab's new Font group took it to 33. All five
    /// were re-measured against `built_in()` and rewritten together.
    ///
    /// ⇒ The instruction above — **failing here means editing prose** — is
    /// necessary and is not sufficient, because it only fires when the count
    /// moves. Four sites drifted while the count stood still. The only thing
    /// that catches that is re-measuring the sentence rather than trusting it,
    /// which is why they are enumerated by path below.
    ///
    /// ★ It went back **32 → 31** on 2026-08-14, and the same five sites were
    /// edited with it. The cause was a *deletion*, which is the direction that
    /// makes this test most valuable: the two text-copy commands moved to
    /// File ▸ Export, Edit ▸ Clipboard was left with no members, and an empty
    /// group is a captioned band offering nothing — the placeholder P3
    /// forbids. Deleting it is what the rule requires; editing the number in
    /// six places is what this test makes unavoidable.
    ///
    /// The keymap is counted here for the same reason: `ron`'s header
    /// argues that the format can express *the real ribbon* and then lists
    /// its parts, so a binding added without that list moving turns the
    /// argument into a claim about a smaller shell than the one shipped.
    #[test]
    fn the_ribbon_has_the_documented_shape() {
        let shell = built_in();
        assert_eq!(shell.tabs().len(), 7, "seven ordinary tabs");
        assert_eq!(shell.contextual_tabs().len(), 1, "one contextual tab");
        assert_eq!(
            shell.all_tabs().flat_map(Tab::groups).count(),
            35,
            "thirty-five groups. ★★ 34 → 35 on 2026-09-04: File ▸ Security (O119) — \
             `Encrypt…` and `Permissions…`, in their own band immediately after Export. \
             A new group rather than two rows under Document, and rather than a row on \
             Edit ▸ Protect where a reader would first look: every other command on Edit \
             is an undoable edit to page CONTENT, and these two rewrite every byte and \
             enter nothing in the undo log. \
             ★ 33 → 34 on 2026-08-29: Pages ▸ Clipboard (O59 item 2). \
             Its own band rather than three more entries under Organise, for the reason that \
             band's own note gives about Delete leading it — an operator comes to a band \
             because of what it is CALLED, and Cut/Copy/Paste under a caption reading \
             `Organise` are three commands nobody scanning for a clipboard would look at"
        );
        assert_eq!(shell.modes().len(), 3, "three modes");
        assert_eq!(
            shell.keymap.as_ref().expect("a keymap").len(),
            35,
            "thirty-five key bindings — Ctrl+A joined on 2026-09-01 for `edit.select_all`; \
             the four pointer tools took V, A, T and H on 
             2026-08-19, and the document tabs took Ctrl+Tab, Ctrl+Shift+Tab and 
             Ctrl+W the same day. Both are the layout every program in this class uses. 
             ★ 33 → 34 on 2026-08-29: Ctrl+Shift+V for `edit.paste_duplicate`, the 
             operator's own choice, following the Word/Excel/browser convention that 
             'paste, but differently' is the same key with Shift"
        );
    }

    /// ★★ **The chords every document application has, asserted as a LIST.**
    ///
    /// Added 2026-08-20, on the operator: *"still no ctrl+c, ctrl+v, ctrl+x or
    /// ctrl+p shortcuts that were requested ages ago."* Three of the four were
    /// bound. `Ctrl+P` was not, and had not been since the manifest was
    /// written.
    ///
    /// # Why the whole list, and not a line for the one that was missing
    ///
    /// Because the defect was never about Print. It was that **nothing
    /// anywhere asked the question**, and a test naming `Ctrl+P` would leave
    /// the question unasked for the next one. The count assertion above cannot
    /// help: it says how MANY bindings there are, and a keymap with the wrong
    /// thirty-two passes it exactly as well as the right thirty-two.
    ///
    /// These are the chords a person arriving from any other PDF or office
    /// application will press without looking. Every one of them is muscle
    /// memory, which means its absence is not experienced as a missing feature
    /// - it is experienced as the application ignoring the keyboard.
    ///
    /// A command here that this build does not register is a failure of THIS
    /// test rather than a silently dropped binding, which is the second half of
    /// the same argument: `no_registered_command_is_orphaned` catches a binding
    /// pointing nowhere, and this catches a chord that is simply not there.
    #[test]
    fn the_keymap_offers_the_chords_a_document_application_must() {
        let shell = built_in();
        let keymap = shell.keymap.as_ref().expect("a keymap");
        for (chord, command) in [
            ("Ctrl+N", "file.new"),
            ("Ctrl+O", "file.open"),
            ("Ctrl+A", "edit.select_all"),
            ("Ctrl+S", "file.save"),
            ("Ctrl+Shift+S", "file.save_copy"),
            ("Ctrl+P", "file.print"),
            ("Ctrl+W", "file.close"),
            ("Ctrl+Z", "edit.undo"),
            ("Ctrl+Y", "edit.redo"),
            ("Ctrl+X", "edit.cut"),
            ("Ctrl+C", "edit.copy"),
            ("Ctrl+V", "edit.paste"),
            ("Ctrl+F", "edit.find"),
        ] {
            assert_eq!(
                keymap.get(chord),
                Some(command),
                "{chord} must reach {command} - muscle memory, and its absence reads as the application ignoring the keyboard"
            );
        }
    }

    /// The tabs are the seven of `RIBBON_IA.md` §4, in its order.
    #[test]
    fn the_tabs_are_the_seven_in_specification_order() {
        let shell = built_in();
        let ids: Vec<&str> = shell.tabs().iter().map(|t| t.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "file", "view", "pages", "edit", "markup", "measure", "tools"
            ]
        );
        assert_eq!(shell.contextual_tabs()[0].id, "format");
    }

    /// **Every command id is prefixed with the tab that owns it.**
    ///
    /// The convention that makes P1 legible in the id itself. Two
    /// documented exceptions, and they are named here rather than
    /// hand-waved so that a third one has to be added deliberately.
    #[test]
    fn every_command_id_names_its_owning_tab() {
        for tab in built_in().all_tabs() {
            for group in tab.groups() {
                for id in group.items().iter().filter_map(Item::command_id) {
                    assert!(
                        id.starts_with(&format!("{}.", tab.id)),
                        "`{id}` is on tab `{}` but is not prefixed with it",
                        tab.id
                    );
                }
            }
        }
    }

    /// The two ids that live on no tab do live somewhere.
    #[test]
    fn the_two_tabless_commands_are_the_documented_ones() {
        let shell = built_in();
        let on_a_tab: BTreeSet<&str> = shell
            .all_tabs()
            .flat_map(Tab::groups)
            .flat_map(Group::items)
            .filter_map(Item::command_id)
            .collect();
        let qat = shell.qat.as_ref().expect("the QAT exists");
        for id in ["edit.undo", "edit.redo"] {
            assert!(!on_a_tab.contains(id));
            assert!(qat.ids().iter().any(|q| q == id));
        }
    }

    /// **The three View ▸ Render entries with no `G` mark, and the two new
    /// Window settings, are actually present.**
    ///
    /// [`DIRECTED`] is a claim about this manifest. If an entry were listed
    /// there and then not emitted, the list would be documenting a
    /// deviation that had been quietly reverted — which is worse than
    /// either state on its own, because the note would still be there
    /// explaining a decision nobody could see.
    #[test]
    fn every_directed_entry_is_emitted() {
        let emitted: BTreeSet<String> = emitted().into_iter().collect();
        for (id, why) in DIRECTED {
            assert!(
                emitted.contains(*id),
                "`{id}` is listed as a directed inclusion ({why}) but is not in the manifest"
            );
        }
    }

    /// `DIRECTED` and `PLANNED` are disjoint.
    ///
    /// An id in both would be claiming to be emitted and absent at once.
    #[test]
    fn directed_and_planned_do_not_overlap() {
        let planned: BTreeSet<&str> = PLANNED.iter().map(|(id, _)| *id).collect();
        for (id, _) in DIRECTED {
            assert!(
                !planned.contains(*id),
                "`{id}` is listed as both directed and planned"
            );
        }
    }

    /// The View ▸ Window group carries the two new settings, in the
    /// specified order, between the two existing window commands and the
    /// layout reset.
    ///
    /// Order is checked rather than mere presence because the group reads
    /// as a progression — what the window shows, then what may float in
    /// it, then how to put it all back — and the reset belongs last for
    /// the same reason a reset button always does.
    #[test]
    fn the_window_group_holds_the_commands_it_still_has() {
        let shell = built_in();
        let view = shell
            .tabs()
            .iter()
            .find(|t| t.id == "view")
            .expect("the View tab");
        let window = view
            .groups()
            .iter()
            .find(|g| g.id == "window")
            .expect("the Window group");
        let ids: Vec<&str> = window.items().iter().filter_map(Item::command_id).collect();
        assert_eq!(
            ids,
            [
                "view.previous_document",
                "view.next_document",
                "view.close_other_documents",
                "view.read_mode",
                "view.fullscreen",
                // ★ Before Reset layout: the cheap remedy above the
                // destructive one. See the manifest's own note at the item.
                "view.dock_all_panels",
                "view.reset_layout",
            ]
        );
    }

    /// The Markup ▸ Style band holds the colour swatch as a `Custom` item.
    ///
    /// Asserted because the alternative — modelling a colour picker as a
    /// `Command` — is the easy mistake, and it is the one that would push
    /// a `ColourSwatch` variant into `egui-shell` the first time the
    /// renderer needed to tell the two apart.
    #[test]
    fn the_markup_style_band_is_a_custom_item_not_a_command() {
        let shell = built_in();
        let style = shell
            .tabs()
            .iter()
            .find(|t| t.id == "markup")
            .expect("the Markup tab")
            .groups()
            .iter()
            .find(|g| g.id == "style")
            .expect("the Style group")
            .items()
            .to_vec();
        assert_eq!(style, vec![Item::custom(COLOUR_SWATCH)]);
    }

    /// The keymap binds every chord to a command the manifest knows, and
    /// binds the four chords that are new in this layout.
    #[test]
    fn the_keymap_binds_the_new_chords() {
        let shell = built_in();
        let keymap = shell.keymap.as_ref().expect("a keymap");
        assert_eq!(keymap.get("Ctrl+H"), Some("view.read_mode"));
        assert_eq!(keymap.get("F11"), Some("view.fullscreen"));
        assert_eq!(keymap.get("Ctrl+1"), Some("mode.read"));
        assert_eq!(keymap.get("Ctrl+2"), Some("mode.review"));
        assert_eq!(keymap.get("Ctrl+3"), Some("mode.edit"));
        // Two chords, one command: redo is reachable both ways, exactly as
        // the shipped shortcut window already promises.
        assert_eq!(keymap.get("Ctrl+Y"), Some("edit.redo"));
        assert_eq!(keymap.get("Ctrl+Shift+Z"), Some("edit.redo"));
    }

    /// No command is emitted twice anywhere, including across the QAT and
    /// the keymap.
    ///
    /// `Shell::validate` enforces one-command-one-*tab* and separately
    /// forbids the QAT listing one id twice. This is the remaining case:
    /// a command that appears once on a tab, once on the QAT and twice in
    /// the keymap is legal and intended (redo), so what is checked is the
    /// narrower thing — no id appears twice within the tab set.
    #[test]
    fn no_command_appears_twice_on_the_tabs() {
        let shell = built_in();
        let mut on_tabs: Vec<&str> = shell
            .all_tabs()
            .flat_map(Tab::groups)
            .flat_map(Group::items)
            .filter_map(Item::command_id)
            .collect();
        let total = on_tabs.len();
        on_tabs.sort_unstable();
        on_tabs.dedup();
        assert_eq!(on_tabs.len(), total, "a command is on the tabs twice");
    }
}
