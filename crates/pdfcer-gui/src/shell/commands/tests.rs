//! `shell::commands::tests` — the properties every registration in this
//! catalogue must hold.
//!
//! Split out of [`super`] under **R2** on 2026-09-06, when the five Format ▸
//! Markup registrations took that file past 1,500 lines. The seam is the
//! standard one this tree already uses for `text::commands::tests`,
//! `panels::tests` and `app::actions::tests`, and `text::commands::tests`' own
//! header states it:
//!
//! > **the assertions about a catalog are a different subject from the
//! > catalog**, and the catalog is the half a reader opens to find out what a
//! > control says.
//!
//! ★ It is the same cut [`super::catalog`] took a week earlier, one level up.
//! That split moved the *entries*; this one moves the *rules about them*, and
//! what is left in [`super`] is what a caller of this module actually reaches
//! for: `register`, `FILE_RECENT`, the `mapping` re-exports and the `reach`
//! declaration.
//!
//! ## ★★ The second cut, 2026-09-10 — [`super::ledger`]
//!
//! This file went past 1,500 lines again, and the entry that did it was
//! `file.stamp_collection`'s (O169). Nine hundred of those lines were the two
//! **counters** — `registration_succeeds_and_registers_every_command` and
//! `the_icon_coverage_split_adds_up_to_the_registry` — and almost all of that
//! was their ledgers rather than their code.
//!
//! Those two moved whole to [`super::ledger`]. What stayed is everything that
//! asserts a *property* rather than a *count*: the handler-token blocks, the
//! condition vocabulary, the with-nothing-open enabled set, the tooltip rule
//! and the icon-key rules.
//!
//! ⚠ **The cut is between whole tests, never between a literal and the notes
//! that justify it.** The ledgers stay with their assertions rather than moving
//! to the registrations: the running commentary is written *at the literal it
//! explains*, and that literal is an assertion. Splitting a number from its
//! argument is exactly the drift those ledgers exist to record — which is why
//! `every_handler_token_is_unique` and `every_handler_token_is_in_its_tabs_block`
//! are still here, carrying their own block table, rather than being swept into
//! a file called *ledger* because the word fits.
//!
//! ## Nothing moved but the module wrapper
//!
//! Of the first cut, on 2026-09-06: every test was byte-identical to what it
//! was, de-indented by one level, so a failure here reads exactly as it did
//! before the split. `use super::*;` still resolves to `shell::commands`, and
//! so does every `super::super::` path inside — a `mod tests { … }` and a
//! `mod tests;` have the same parent, which is why that cut was safe to make
//! mechanically. The same is true of [`super::ledger`], which is a sibling of
//! this module and not a child of it, for exactly that reason.
//!
//! ## ★ `#![cfg(test)]` as well as the parent's `#[cfg(test)] mod tests;`
//!
//! Redundant to the compiler and load-bearing to a gate.
//! `tools/gates/check-ui-strings.sh` stops scanning a file at
//! `#[cfg(test)]` — assertion messages are prose read by whoever is staring at
//! a failing test, never by an operator — and a whole-file test module has no
//! such line to stop at. Its own header records what happened when
//! `canvas/selection/tests.rs` was split without one: **28 assertion messages
//! reported as operator-facing copy**, and *"the noise is the actual hazard"*,
//! because a report full of false positives trains people to ignore it.
//!
//! The inner attribute is the marker that gate recognises, and
//! `check-theme-colors.sh` recognises the same one from the AST. Both state why
//! it is the marker rather than the filename: the property that earns the
//! exemption is *"not in the shipped binary"*, and a filename is a restatement
//! of that which goes stale the moment a third such module is written.

#![cfg(test)]

use super::*;
use egui_shell::commands::ConditionSet;
use std::collections::BTreeSet;

fn registry() -> CommandRegistry {
    let mut reg = CommandRegistry::new();
    register(&mut reg);
    reg
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
        // ★★★ **A markup annotation is selected, and this mode may author
        // markup** — one fused fact, published by `app::conditions` since
        // 2026-09-06 for the five Format ▸ Markup controls.
        //
        // Fused rather than split into a mode condition and a selection
        // condition, unlike the Font group's `mode.edit_content` +
        // `selection.text` pair, because the two halves are answered by
        // **absence** here and by absence and greying there. R9's
        // discriminator is whether hovering could tell the operator
        // anything: a greyed Font control is the one surface that can say
        // *press T first*, and a greyed Markup control could only say
        // *select a mark*, which the operator has already done or the tab
        // would not be drawn. `Enable::When` takes one name, so an `A && B`
        // predicate is a **named fact** — see `manifest::format`'s
        // `MARKUP_VISIBLE_WHEN`, which carries the whole argument.
        //
        // ★ It is NOT a refinement of `selection.any`. That is the object
        // selection — a paint-order index into page content — and an
        // annotation is an `ObjId`; nothing maps between the two index
        // spaces, which is why `AnnotTarget` exists at all.
        //
        // ★★ The **lock** is deliberately outside it. §12.5.3 Table 165 bit
        // 8 is a fact about one annotation rather than about the build or
        // the mode, so it greys with a sentence rather than making the
        // group vanish — and `app::markupband` does that greying itself,
        // which it must, since the shell evaluates no predicate for an
        // `Item::Custom`.
        "selection.markup_restylable",
        // ★★★ **The two markup-node predicates**, published since 2026-09-06 —
        // and they are the first entries in this list that `app::conditions`
        // never sets.
        //
        // Both are corrected **per right-click** by `canvas::menus`, through
        // `MenuHost::with_conditions`, and the reason they cannot live in the
        // frame-top condition set is the same reason `panel.docked` cannot: the
        // frame's set describes the frame, and these describe **one click on one
        // edge of one shape**. There is no ribbon control whose greying they
        // could get wrong, because `markup.add_node` and `markup.remove_node`
        // are in `manifest::TAB_SCOPED` and have no ribbon control at all.
        //
        // ★★ The answer itself is the ENGINE's, per frame the popup is open:
        // `EditSession::reshape_annotation_preview`, asked with the exact
        // `VertexEdit` the row would commit. `Ok` sets the name; the one
        // *temporary* refusal (`ReshapeWouldBreachVertexFloor` — draw another
        // corner and it comes back) clears it and leaves the row drawn and
        // greyed; every other refusal is a property of the shape's kind and
        // takes the row away through the item's `visible_when` instead. That is
        // R9's two halves on one row, derived rather than declared — see
        // `canvas::annotnodes::menu`.
        //
        // ★ Named here rather than exempted: this list asserts what a COMMAND
        // may wait on, and these two are waited on by two commands. Where the
        // name is *set* is a different question, and the answer being "a menu
        // host rather than `conditions`" is exactly the kind of fact this list
        // is read to discover.
        "markup.node_insertable",
        "markup.node_removable",
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
        // ★ The two auto-hide toggles are document-free ON PURPOSE. They
        // are settings about the application's own chrome, and a chrome
        // setting that could only be changed with a drawing open would be
        // unreachable in the state the application STARTS in — which is
        // defect D1's shape, and is what `enabled_when("doc.pages")` on a
        // view command has to justify each time it is written.
        "view.rail_auto_hide",
        "view.read_mode",
        "view.reset_layout",
        "view.ribbon_auto_hide",
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
