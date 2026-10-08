//! Tests for `crate::app::modes::capability`, kept in the gui because they reach gui modules.

use crate::app::modes::capability::*;
use egui_shell::manifest::Mode;
use egui_shell::manifest::Shell;

/// The manifest the product actually ships.
fn built_in() -> Shell {
    crate::shell::manifest::built_in()
}

/// **The `MODES_AND_PANELS.md` gesture table, asserted against the
/// shipped manifest.**
#[test]
fn the_built_in_modes_match_the_specified_gesture_table() {
    let shell = built_in();
    let caps = |id: &str| Capabilities::for_mode(Some(&shell), Some(id));

    // Read: pan, zoom, and form filling. Nothing that authors.
    assert_eq!(caps("read"), Capabilities::NONE, "Read authors nothing");

    // Review: its own markup and dimensions, but the page content is not
    // the reviewer's to alter.
    assert_eq!(
        caps("review"),
        Capabilities {
            edit_content: false,
            author_markup: true,
            author_measure: true,
        },
        "Review places markup and dimensions and does not edit content"
    );

    // Edit: everything.
    assert_eq!(caps("edit"), Capabilities::FULL, "Edit authors everything");
}

/// Read is the mode the operator asked about, so its refusal is asserted
/// on its own rather than only inside the table above.
#[test]
fn read_mode_refuses_every_content_gesture() {
    let shell = built_in();
    let read = Capabilities::for_mode(Some(&shell), Some("read"));
    assert!(!content_gesture(read), "no content gesture in Read");
    assert!(!read.author_markup, "no markup placement in Read");
    assert!(!read.author_measure, "no dimension placement in Read");
    assert!(!read.authors_anything(), "Read is a reading stance");
}

/// **Every unknown case lands on `FULL`** — module header §3.
#[test]
fn an_unknown_mode_gets_the_full_canvas() {
    let shell = built_in();
    assert_eq!(
        Capabilities::for_mode(None, Some("read")),
        Capabilities::FULL,
        "no validated shell: the canvas is not crippled by a missing manifest"
    );
    assert_eq!(
        Capabilities::for_mode(Some(&shell), None),
        Capabilities::FULL,
        "no active mode"
    );
    assert_eq!(
        Capabilities::for_mode(Some(&shell), Some("kiosk")),
        Capabilities::FULL,
        "a mode this build does not declare"
    );
}

/// A customized manifest is honoured rather than second-guessed —
/// module header §2's last paragraph, made mechanical.
#[test]
fn a_customized_mode_gets_the_capabilities_its_tabs_name() {
    let shell = Shell::default().with_mode(Mode::new(
        "reviewing-reader",
        "Reviewing reader",
        ["view", "markup"],
    ));
    let caps = Capabilities::for_mode(Some(&shell), Some("reviewing-reader"));
    assert_eq!(
        caps,
        Capabilities {
            edit_content: false,
            author_markup: true,
            author_measure: false,
        },
        "a mode offering Markup and nothing else places markup and nothing else"
    );
}

/// The default is permissive, so a test that does not mention modes is
/// not silently asserting one.
#[test]
fn the_default_is_full() {
    assert_eq!(Capabilities::default(), Capabilities::FULL);
}

// -----------------------------------------------------------------
// `offers_command` — the keymap's share of the gate
// -----------------------------------------------------------------

/// **The commands whose chords must keep working in Read**, and the
/// reason each one does: none of them lives on an ordinary tab.
#[test]
fn a_command_on_no_tab_is_offered_by_every_mode() {
    let shell = built_in();
    for id in [
        "edit.undo",
        "edit.redo",
        "edit.find",
        "view.read_mode",
        "view.fullscreen",
        "mode.read",
        "mode.review",
        "mode.edit",
    ] {
        for mode in ["read", "review", "edit"] {
            assert!(
                offers_command(Some(&shell), Some(mode), id),
                "`{id}` is on no ordinary tab, so `{mode}` must offer it"
            );
        }
    }
}

/// **Both text-copy commands are offered in every mode — the property
/// the File-tab placement exists to secure.**
#[test]
fn both_text_copy_commands_are_offered_by_every_mode() {
    let shell = built_in();
    for id in ["file.copy_page_text", "file.copy_document_text"] {
        for mode in ["read", "review", "edit"] {
            assert!(
                offers_command(Some(&shell), Some(mode), id),
                "`{id}` copies text out and authors nothing, so `{mode}` must offer it — \
                 Read most of all, which is measured against a reader that copies text"
            );
        }
    }
    // …and the ids they replaced are gone, not merely unreferenced. A build
    // that still registered the old ones would be one where a customized
    // manifest could put them back on the Edit tab and reopen the defect.
    let reg = {
        let mut reg = egui_shell::CommandRegistry::new();
        crate::shell::commands::register(&mut reg);
        reg
    };
    for id in ["edit.copy_page_text", "edit.copy_document_text"] {
        assert!(
            reg.get(id).is_none(),
            "`{id}` moved to the `file.` block on 2026-08-14 and must not be registered"
        );
    }
}

/// **Review offers the whole clipboard**, as a headless assertion.
#[test]
fn review_offers_every_clipboard_chord() {
    let shell = built_in();
    for id in [
        "edit.copy",
        "edit.cut",
        "edit.paste",
        "edit.paste_duplicate",
    ] {
        assert!(
            offers_command(Some(&shell), Some("review"), id),
            "`{id}` must reach Review: the mode authors markup, and \
             `dispatch::clipboard` decides per press what the clipboard holds. \
             `edit.paste` refused here is the driven sweep's finding A1 — an operator \
             who copied a comment with nowhere to put it"
        );
        // …and it is offered *because it is on the escape list*, not by
        // some other accident. Named separately so a build that made every
        // command reachable everywhere would still be caught by the
        // negative tests, and a build that dropped the list would be caught
        // here with the id printed.
        assert!(
            GATED_BY_THEIR_DISPATCHER.contains(&id),
            "`{id}` reaches Review, and it is not on the escape list — so something \
             else is granting it and this test is measuring the wrong mechanism"
        );
    }
}

/// **…and Read still refuses all four — but in `dispatch::clipboard`,
/// not here.**
#[test]
fn read_mode_still_refuses_the_clipboard_verbs_it_should() {
    let shell = built_in();
    let read = Capabilities::for_mode(Some(&shell), Some("read"));
    assert_eq!(
        read,
        Capabilities::NONE,
        "Read grants neither gate `dispatch::clipboard` reads, so every cut and \
         every paste is refused there — with a sentence, which is more than the \
         chord gate gave"
    );
    // …and Review grants exactly one of the two, which is what makes the
    // paste it may do different from the paste it may not.
    let review = Capabilities::for_mode(Some(&shell), Some("review"));
    assert!(
        review.author_markup && !review.edit_content,
        "Review pastes a comment and not a drawing's geometry: {review:?}"
    );
}

/// **Every id that escapes its tab is one the clipboard dispatcher owns.**
#[test]
fn every_dispatcher_gated_command_is_one_the_clipboard_dispatcher_owns() {
    for id in GATED_BY_THEIR_DISPATCHER {
        assert!(
            crate::app::dispatch::clipboard::handles(id),
            "`{id}` escapes its tab and `app::dispatch::clipboard` does not claim it, \
             so nothing asks the mode question for it at all"
        );
    }
}

/// **…and it is not simply every id that dispatcher owns**, which is the
/// direction that would make the list vacuous.
#[test]
fn the_escape_list_is_narrower_than_the_dispatchers_own() {
    assert!(
        crate::app::dispatch::clipboard::handles("edit.copy_as_vector"),
        "the precondition"
    );
    assert!(
        !GATED_BY_THEIR_DISPATCHER.contains(&"edit.copy_as_vector"),
        "the escape list is a judgement about which verbs need it, not a copy of `handles`"
    );
}

/// **…and a command on a tab the mode hides is not offered.**
///
/// The other half, without which the test above passes on a build where
/// the filter returns `true` unconditionally.
#[test]
fn a_command_on_a_hidden_tab_is_not_offered() {
    let shell = built_in();
    // Edit-tab commands: reachable only in Edit.
    // Three ids rather than the one that would demonstrate the point: the
    // property under test is *a command on a hidden tab is not offered*,
    // and it needs more than one witness, or a build that offered exactly
    // one Edit command everywhere would still pass.
    for id in ["edit.text", "edit.add_text", "edit.reflow_block"] {
        assert!(!offers_command(Some(&shell), Some("read"), id), "read/{id}");
        assert!(
            !offers_command(Some(&shell), Some("review"), id),
            "review/{id}"
        );
        assert!(offers_command(Some(&shell), Some("edit"), id), "edit/{id}");
    }
    // Pages-tab commands: hidden in Read, shown in Review and Edit —
    // the row that proves this is per-tab rather than "Edit only".
    for id in ["pages.rotate_left", "pages.move_up"] {
        assert!(!offers_command(Some(&shell), Some("read"), id), "read/{id}");
        assert!(
            offers_command(Some(&shell), Some("review"), id),
            "review/{id}"
        );
    }
}

/// Every chord the shipped keymap binds, resolved against every mode —
/// so the *actual* consequence of the gate is visible in one place rather
/// than inferred from two rules.
#[test]
fn every_bound_chord_is_offered_by_the_fullest_mode() {
    let shell = built_in();
    let keymap = shell
        .keymap
        .as_ref()
        .expect("the built-in manifest binds chords");
    for (chord, id) in keymap.iter() {
        assert!(
            offers_command(Some(&shell), Some("edit"), id),
            "`{chord}` -> `{id}` is bound and Edit does not offer it, so it is bound to something no mode can reach"
        );
    }
}

/// A contextual tab's command is treated as tab-less: the tab is governed
/// by its own `visible_when`, not by mode membership, and gating it twice
/// would be two rules for one thing.
#[test]
fn a_contextual_tabs_command_is_not_gated_by_the_mode() {
    let shell = built_in();
    assert!(offers_command(Some(&shell), Some("read"), "format.delete"));
}

/// The permissive fallbacks, asserted as three separate routes because
/// they are three separate `return`s.
#[test]
fn an_unknown_shell_mode_or_command_is_offered() {
    let shell = built_in();
    assert!(offers_command(None, Some("read"), "edit.text"));
    assert!(offers_command(Some(&shell), None, "edit.text"));
    assert!(offers_command(Some(&shell), Some("kiosk"), "edit.text"));
    assert!(offers_command(Some(&shell), Some("read"), "not.a.command"));
}

/// **The whole consequence of the gate, in one table.**
#[test]
fn read_mode_refuses_exactly_these_bound_chords() {
    let shell = built_in();
    let keymap = shell
        .keymap
        .as_ref()
        .expect("the built-in manifest binds chords");
    let mut refused: Vec<String> = keymap
        .iter()
        .filter(|(_, id)| !offers_command(Some(&shell), Some("read"), id))
        .map(|(_, id)| id.to_string())
        .collect();
    refused.sort_unstable();
    refused.dedup();
    assert_eq!(
        refused,
        [
            // Authoring the page's own content — correctly refused. Read
            // is the mode that does not author.
            "edit.add_text",
            "edit.align",
            "edit.align_bottom",
            "edit.align_centre",
            "edit.align_centre_x",
            "edit.align_centre_y",
            "edit.align_left",
            "edit.align_right",
            "edit.align_top",
            // **No clipboard id belongs in this list**
            // (`OPERATOR_REQUESTS.md` O71). A chord refused here traces
            // `chord-not-offered` and does nothing, which in Read is a
            // picture the operator may select and may not copy into Word,
            // and in Review a comment with nowhere to paste it. Every
            // clipboard verb instead reaches `dispatch::clipboard`, which
            // gates the EFFECT on the operand and refuses it in Read anyway
            // — in words, on the `⊗` slot, which is more than this list can
            // give. `read_mode_still_refuses_the_clipboard_verbs_it_should`
            // keeps that true, and it asserts the outcome rather than the
            // route, which is the only form of the claim that survives the
            // gate moving.
            //
            // Read refuses `edit.select_all` because it selects CONTENT.
            // Text selection has its own Ctrl+A and is unaffected — which is
            // the distinction this list is for.
            "edit.select_all",
            "edit.text",
            // **The four Markup ▸ Arrange chords** — `Ctrl+[`, `Ctrl+]`
            // and their Shift forms.
            //
            // Refused in Read for the same structural reason as the page
            // verbs below rather than for a reason of their own: Read's tab
            // list is File and View, so the Markup tab is not there, and
            // `offers_command` answers `false` for every id on a tab the
            // mode does not show. **Nothing was added to the gate.**
            //
            // And it is the right answer on the merits, which is worth
            // checking rather than inheriting: changing which mark is drawn
            // on top **is an edit to the document** — it permutes the page's
            // `/Annots` and enters the undo log — and Read is the mode that
            // does not edit. It is not the copying-is-not-authoring case
            // three notes up, where the refusal was wrong because the act
            // changed nothing.
            //
            // In **Review** all four reach the dispatcher, which is where
            // they belong: Review is the markup stance, it has the Markup
            // tab, and `author_markup` is the capability
            // `dispatch::arrange` asks.
            "markup.bring_forward",
            "markup.bring_to_front",
            "markup.send_backward",
            "markup.send_to_back",
            // Structural page verbs. Read shows no Pages tab, which is
            // `MODES_AND_PANELS.md`'s own decision, not this gate's.
            "pages.move_down",
            "pages.move_up",
            "pages.rotate_left",
            "pages.rotate_right",
            // The node tool's `A`: its View ▸ Navigate item is shown only
            // under `mode.edit_content`, and `offers_command` refuses such an
            // item wherever the mode lacks the Edit tab. The same rule keeps
            // Security ▸ Protect's redaction out of Read and Review.
            "view.tool_node",
        ]
        .map(str::to_owned),
        "the set of chords Read refuses has changed"
    );
}
