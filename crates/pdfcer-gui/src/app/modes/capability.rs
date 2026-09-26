//! # `app::modes::capability` — what a mode lets the canvas do
//!
//! **The one place the rule "Read does not edit the document" is written
//! down.** Everything else — the gesture machine, the key handler, the
//! context menus, the tool arming — asks this module and branches on the
//! answer; none of them knows what `"read"` is.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/modes/capability.md`.

use egui_shell::manifest::{Item, Shell};

/// The `edit` tab's id in the manifest.
// ui-text-exempt: manifest identifier, never displayed.
const TAB_EDIT: &str = "edit";
/// The `markup` tab's id in the manifest.
// ui-text-exempt: manifest identifier, never displayed.
const TAB_MARKUP: &str = "markup";
/// The `measure` tab's id in the manifest.
// ui-text-exempt: manifest identifier, never displayed.
const TAB_MEASURE: &str = "measure";

/// **What the active mode lets the canvas do to the document.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// Whether page **content** — paths, text objects, images — may be
    /// selected on the canvas and changed.
    ///
    /// Gates the click hit test, marquee *select*, the move drag, the resize
    /// grips and the Delete key. See the module header §5 for why selection
    /// is inside this flag rather than beside it.
    pub edit_content: bool,
    /// Whether markup annotations may be **placed** on the canvas.
    ///
    /// Gates arming `CanvasTool::Markup` and the markup drag itself. Does
    /// not gate *reading* markup: the Comments panel is an inspection
    /// surface and mounts in every mode.
    pub author_markup: bool,
    /// Whether dimensions may be **placed** on the canvas.
    ///
    /// Gates arming the measure tools and their picks. Does not gate the
    /// ruler and grid, which *read* the document's dimension scale in every
    /// mode — reading a scale is not authoring one.
    pub author_measure: bool,
}

impl Capabilities {
    /// Everything permitted — the canvas this shell had before modes gated
    /// it, and the fallback for a mode nothing declares.
    ///
    /// See the module header §3 for why the unknown case lands here rather
    /// than on [`Self::NONE`].
    pub const FULL: Self = Self {
        edit_content: true,
        author_markup: true,
        author_measure: true,
    };

    /// Nothing but navigation and form filling — what Read grants.
    pub const NONE: Self = Self {
        edit_content: false,
        author_markup: false,
        author_measure: false,
    };

    /// **What `mode_id` may do, according to `shell`.**
    #[must_use]
    pub fn for_mode(shell: Option<&Shell>, mode_id: Option<&str>) -> Self {
        let (Some(shell), Some(mode_id)) = (shell, mode_id) else {
            return Self::FULL;
        };
        let Some(mode) = shell.modes().iter().find(|m| m.id == mode_id) else {
            return Self::FULL;
        };
        let has = |tab: &str| mode.tabs().iter().any(|t| t == tab);
        Self {
            edit_content: has(TAB_EDIT),
            author_markup: has(TAB_MARKUP),
            author_measure: has(TAB_MEASURE),
        }
    }

    /// Whether **any** authoring gesture is permitted.
    #[must_use]
    pub fn authors_anything(self) -> bool {
        self.edit_content || self.author_markup || self.author_measure
    }
}

impl Default for Capabilities {
    /// [`Capabilities::FULL`] — see the module header §3. A `Default` that
    /// restricted would make every `..Default::default()` in a test a silent
    /// assertion about modes.
    fn default() -> Self {
        Self::FULL
    }
}

/// **The commands whose chord reaches every mode because their own
/// dispatcher gates them.**
const GATED_BY_THEIR_DISPATCHER: [&str; 5] = [
    // ui-text-exempt: registered command ids, never displayed.
    "edit.copy",
    "edit.cut",
    "edit.paste",
    "edit.paste_duplicate",
    // **`edit.duplicate`** — Ctrl+D.
    //
    // It is registered on the **Edit** tab, beside the four above and for the
    // same reason: the Clipboard group is where an operator looks for *"make
    // another one of this"*. **Review is not shown that tab** — and Review is
    // the mode whose entire purpose is marking up somebody else's drawing,
    // i.e. the mode in which an operator is most likely to be laying out a row
    // of identical revision marks. Without this line `Ctrl+D` traces
    // `chord-not-offered id=edit.duplicate mode=review` and does nothing,
    // which is the same defect `edit.paste` has without its own entry.
    //
    // Membership is a promise, not a decoration: `app::dispatch::clipboard`'s
    // `duplicate` arm gates on `capabilities().author_markup` and words the
    // refusal through `ModeRefusal::DuplicateMarkup`, so Read is still stopped
    // — with a sentence rather than with silence.
    "edit.duplicate",
];

/// **Whether the active mode offers `command_id` at all.**
///
/// The rule a **keyboard chord** is filtered through, so that a chord cannot
/// reach a command the operator cannot see. Operator decision.
///
/// # The problem this closes, and why it is not the gesture gate's job
///
/// [`Capabilities`] governs the **canvas**; the ribbon governs itself by
/// hiding tabs. Between them sits the keymap, which dispatches by command id
/// and consults neither: `app::keyboard::commands` looks a chord up and hands
/// the id to the dispatcher. Without this filter Read hides the Edit tab and
/// `Ctrl+E` still reaches `edit.text`.
///
/// The gate is in place ahead of the verbs it filters being implemented,
/// because a defect that becomes real on the day someone lands an unrelated
/// feature is worse than one that is real today: nothing about that day
/// points at this file.
///
/// # The rule, and the case that decides its shape
///
/// > A chord may reach a command the active mode **shows**, or a command that
/// > **lives on no ordinary tab at all**.
///
/// The second clause is the whole design, and it is what makes an exception
/// list unnecessary:
///
/// | command | where it lives | reachable in Read |
/// |---|---|:-:|
/// | `edit.undo`, `edit.redo` | the **QAT** and the keymap — no tab | ✅ |
/// | `edit.find` | the **status bar** and the keymap — no tab | ✅ |
/// | `view.read_mode`, `view.fullscreen`, `mode.*` | the keymap — no tab | ✅ |
/// | `edit.text`, `edit.add_text` | the **Edit** tab | ❌ |
/// | `pages.rotate_left`, `pages.move_up` | the **Pages** tab | ❌ in Read, ✅ in Review |
///
/// Undo and redo are the case that would have forced an exception, and they do
/// not, because they were **already on no tab** — they sit on the quick access
/// toolbar, which every mode draws. That is not luck; it is the same taxonomy
/// rule that moved `edit.form_fill` to `view.panel_forms`, applied earlier by
/// someone else. A command's id prefix says which tab *owns* it, and `edit.`
/// commands that are not authoring do not live on the Edit tab.
///
/// # Why the text-copy verbs live on File and not on Edit
///
/// `Ctrl+Shift+C` is bound to `file.copy_page_text`. On the Edit tab this
/// function would refuse it in Read — correctly *by the rule*, and wrongly
/// *about the product*: Acrobat Reader copies text, and replacing Acrobat
/// Reader is what Read is for. The answer is **not** an exception in this
/// file; it is that the command does not belong on an authoring tab.
/// **Copying is not authoring** — it reads the page and writes to the
/// clipboard, changing nothing — so the operator placed both text-copy verbs
/// on File ▸ Export as `file.copy_page_text` and `file.copy_document_text`,
/// and the chord follows the command. Read shows File, so the rule yields the
/// right answer with no clause added to it.
///
/// That is the shape every case of this should take. A chord refused in a mode
/// where the operator plainly needs it is evidence about the **taxonomy** — it
/// says the command's tab is wrong — and an exception list here would convert
/// that evidence into a second, quieter statement of which tab owns what, free
/// to disagree with the manifest. `edit.form_fill` → `view.panel_forms` is the
/// same move, and the fix is the same one.
///
/// # The one class that escapes its tab — and why it is a class, not a list
///
/// The rule above states a **proxy**. "Does this mode show the tab that owns
/// this command?" stands in for "may this mode do this?", and it is a good
/// proxy because a mode is *defined* by its tabs and because a command an
/// operator cannot see is one they should not be able to press.
///
/// It is the wrong question for a verb whose answer depends on **what the
/// operator is pointing at rather than on which mode they are in**, and
/// `app::dispatch::clipboard` contains four:
///
/// | verb | what its dispatcher gates on |
/// |---|---|
/// | `edit.copy` | nothing — *copying is not authoring*, the operator's ruling |
/// | `edit.cut` | **what is selected**: an annotation takes `author_markup`, page content takes `edit_content` |
/// | `edit.paste`, `edit.paste_duplicate` | **what is on the clipboard**, by the same split |
///
/// All four live in the Edit tab's Clipboard group, which is the right home for
/// them — a tab is a place to *find* a command — and Review is not shown that
/// tab. So without an escape the proxy refuses all four in Review, and for cut
/// and paste it refuses something the mode is **allowed to do**:
///
/// > ```text
/// > chord-command      chord="Ctrl+C" id=edit.copy  via=clipboard-event
/// > clipboard-copy     kind=selection page=0 objects=0 annots=1 thin=0 bytes=395
/// > chord-command      chord="Ctrl+V" id=edit.paste via=clipboard-event
/// > chord-not-offered  id=edit.paste mode=review
/// > ```
///
/// **In the mode whose entire purpose is marking up somebody else's drawing,
/// an operator could copy a comment and have nowhere to put it.**
/// `OPERATOR_REQUESTS.md` O71 is the same shape one layer over, for copy.
///
/// ## Why this is NOT the taxonomy evidence the section above describes
///
/// The paragraph above says a chord refused where the operator plainly needs it
/// is evidence the command's **tab is wrong**, and that the fix is to move it.
/// That is right for `edit.form_fill` → `view.panel_forms` and for the two
/// text-copy verbs, and it is **not** right here, which is why this is an
/// escape rather than a third tab move:
///
/// - Paste **is** authoring. Those two moves worked because the command turned
///   out not to belong on an authoring tab at all; Paste belongs on one.
/// - `RIBBON_IA.md` P1 — one command on at most one tab — means moving Paste to
///   Markup would *take it away from Edit*, where it plainly belongs, and
///   Review needs cut and paste for **markup** while Edit needs them for
///   **content**. No single tab is the answer, because the tab is not what
///   varies.
///
/// ⇒ So the exception is stated as the class it is: **a command whose own
/// dispatcher asks the mode question per press does not need this one asked for
/// it, and is harmed by it.** That is checkable — every member of
/// [`GATED_BY_THEIR_DISPATCHER`] is a command
/// `app::dispatch::clipboard::handles` claims — where "a list of ids somebody
/// added" is not.
///
/// ## The debt this takes on, and it is paid in `dispatch::clipboard`
///
/// A chord refused *here* traces `chord-not-offered id=… mode=…`. A chord that
/// reaches a dispatcher which silently `return`s traces **nothing on any
/// surface**. Pushing the chord through blind therefore obliges the dispatcher
/// to word every refusal it can now meet: both mode gates in
/// `app::dispatch::clipboard` call
/// `app::status::decline::record_mode_refusal`, which draws in the `⊗` slot
/// that means *this did not happen*. Without that half the escape would trade
/// a defect for a quieter one.
///
/// **Contextual tabs are treated as no tab**, deliberately. The Format tab is
/// not in any mode's list — it is governed by its own `visible_when`, which is
/// `selection.any`. In a mode that cannot select there is no selection, so the
/// tab never appears and its commands are unreachable anyway; gating them
/// again here would be a second rule saying the same thing, and the two would
/// eventually disagree.
///
/// An unknown shell, mode or command falls through to `true`, for the reason
/// the module header §3 gives: this is an interface-complexity control, not a
/// permissions system, and failing closed would produce a keyboard that
/// silently does nothing.
#[must_use]
pub fn offers_command(shell: Option<&Shell>, mode_id: Option<&str>, command_id: &str) -> bool {
    let (Some(shell), Some(mode_id)) = (shell, mode_id) else {
        return true;
    };
    // Which ordinary tab owns this command? `None` means no tab does, which is
    // the second clause of the rule above.
    let owning_tab = shell.tabs().iter().find(|tab| {
        tab.groups().iter().any(|group| {
            group
                .items()
                .iter()
                .any(|item| matches!(item, Item::Command { id, .. } if id == command_id))
        })
    });
    let Some(owning_tab) = owning_tab else {
        return true;
    };
    // **The whole clipboard escapes its tab**, not `edit.copy` alone
    // (`OPERATOR_REQUESTS.md` O71): cut and paste are refused in Review by the
    // tab proxy while the mode is allowed to perform them. See
    // [`GATED_BY_THEIR_DISPATCHER`] and this function's §"The one class that
    // escapes its tab".
    if GATED_BY_THEIR_DISPATCHER.contains(&command_id) {
        return true;
    }
    let Some(mode) = shell.modes().iter().find(|m| m.id == mode_id) else {
        return true;
    };
    mode.tabs().contains(&owning_tab.id)
}

/// Whether a gesture that acts on page **content** may proceed.
#[must_use]
pub fn content_gesture(caps: Capabilities) -> bool {
    caps.edit_content
}

/// The memory slot [`publish_edit_content`] writes and [`edit_content_now`]
/// reads.
const EDIT_CONTENT_KEY: &str = "pdfcer.caps.edit-content"; // ui-text-exempt: a memory key, never displayed

/// **Publish whether this frame's mode edits page content**, for the canvas
/// helpers that have no `Capabilities` to hand.
pub fn publish_edit_content(ctx: &egui::Context, on: bool) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(EDIT_CONTENT_KEY), on));
}

/// Whether this frame's mode edits page content. Defaults to `false`.
#[must_use]
pub fn edit_content_now(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::new(EDIT_CONTENT_KEY)))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::manifest::Mode;

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
            ]
            .map(str::to_owned),
            "the set of chords Read refuses has changed"
        );
    }
}
