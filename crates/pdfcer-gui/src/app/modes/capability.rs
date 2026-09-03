//! # `app::modes::capability` — what a mode lets the canvas do
//!
//! **The one place the rule "Read does not edit the document" is written
//! down.** Everything else — the gesture machine, the key handler, the
//! context menus, the tool arming — asks this module and branches on the
//! answer; none of them knows what `"read"` is.
//!
//! ## 1. The operator's ask, and what it actually requires
//!
//! > *"in read mode the document shouldn't allow editing and should allow
//! > only selecting of objects that acrobat reader would allow."*
//!
//! `MODES_AND_PANELS.md` Part 1 had already specified this, one row of its
//! table:
//!
//! | | **Read** | **Review** | **Edit** |
//! |---|---|---|---|
//! | **Canvas gestures** | pan, zoom, text selection for copy, follow links | + place and edit **your own** markup and dimensions | + full content selection and editing |
//!
//! and its safety rule:
//!
//! > **A mode changes what is *visible*. It never makes a visible control
//! > silently inert.**
//!
//! The ribbon already honoured that rule — Read is shown File and View
//! alone, so no editing *command* is reachable. What the ribbon cannot
//! reach is the **canvas**, where a gesture is not a control and there is
//! no tab to hide. Before this module, clicking a line in Read selected it,
//! dragging it moved it, and Delete deleted it: three edits in a mode whose
//! entire purpose is that it does not author anything.
//!
//! ## 2. ★ Capability is derived from the mode's TABS, not from its id
//!
//! The obvious implementation is `if mode == "read"`, and
//! [`crate::viewer::display::PageDisplay::default_for_mode`] is precedent
//! for it. This module deliberately does **not** do that, and the reason is
//! the safety rule above rather than taste.
//!
//! A capability keyed on the id is a *second*, independent statement of what
//! a mode contains. It can disagree with the manifest, and the shape of the
//! disagreement is exactly the failure the rule forbids: a mode that shows
//! the Edit tab while the canvas refuses to select is a visible control that
//! is silently inert, and a mode that hides the Edit tab while the canvas
//! still moves objects is the defect this module was written to fix. Both
//! are unreachable if the canvas and the ribbon read the *same* sentence.
//!
//! So the rule is:
//!
//! | Capability | Granted when the mode contains |
//! |---|---|
//! | [`Capabilities::edit_content`] | the **`edit`** tab |
//! | [`Capabilities::author_markup`] | the **`markup`** tab |
//! | [`Capabilities::author_measure`] | the **`measure`** tab |
//!
//! Against the built-in manifest (`crate::shell::manifest::built_in`) that
//! yields precisely the table in §1, and
//! [`tests::the_built_in_modes_match_the_specified_gesture_table`] pins it:
//!
//! | mode | tabs | content | markup | measure |
//! |---|---|:-:|:-:|:-:|
//! | `read` | file, view | ✗ | ✗ | ✗ |
//! | `review` | file, view, pages, markup, measure | ✗ | ✓ | ✓ |
//! | `edit` | file, view, pages, edit, markup, measure, tools | ✓ | ✓ | ✓ |
//!
//! It also means a customized manifest gets the behaviour it asked for
//! without this file learning its vocabulary: someone who adds the Markup
//! tab to Read gets markup gestures in Read, because they said so, and the
//! alternative — a canvas that ignores their manifest — is not a safer
//! product, it is a broken one.
//!
//! ## 3. ★ An unknown mode gets EVERYTHING, and that is not an oversight
//!
//! [`Capabilities::for_mode`] falls back to [`Capabilities::FULL`] when
//! there is no validated shell, no active mode, or an active mode the
//! manifest does not declare.
//!
//! Falling back to *restricted* is the tempting choice and it is wrong
//! twice:
//!
//! 1. **It fails in the direction the safety rule forbids.** An unknown mode
//!    still renders whatever tabs it declares. Refusing its gestures gives
//!    the operator a full ribbon over a dead canvas — the `editing_enabled`
//!    master toggle, rebuilt by accident, which `RIBBON_IA.md` §5.4 removed
//!    at the operator's instruction.
//! 2. **A mode is not a permissions system**, and `MODES_AND_PANELS.md`
//!    says so in those words: *"Read mode does not protect a document from
//!    anything; a determined operator moves the slider. It is an
//!    interface-complexity control."* Nothing here is load-bearing for
//!    safety, so there is no security argument for failing closed — and
//!    pretending there is would invite a future reader to rely on it.
//!
//! This is the same shape of answer `default_for_mode` gives for the page
//! display (*"an unrecognised mode is not evidence that the operator wants a
//! different one"*), pointed at a different default: there, the default is
//! `Single`; here, the default is the canvas this shell had before modes
//! existed.
//!
//! ## 4. What is deliberately NOT gated
//!
//! - **Filling a form field.** Operator decision, 2026-08-14: *Acrobat
//!   Reader fills forms in its default view, and replacing it is the stated
//!   goal.* `canvas::forms` reads no mode and must not learn to —
//!   `HANDOFF.md` §9 named it as *"the second place that would have to learn
//!   about it"* if a genuinely read-only mode were ever wanted, and the
//!   answer, now that one is, is that it stays out. Filling is not
//!   authoring; it is the primary reason most form documents exist.
//! - **Pan, zoom, the hand tool, marquee *zoom*, Find, guides, rulers,
//!   grid.** Navigation and inspection, none of which touches the document.
//!   A marquee-zoom band shares its rubber band with marquee-*select* and is
//!   branched only at release, so the gate is on the intent rather than on
//!   the band — see [`content_gesture`].
//! - **Selecting a page in the Pages panel**, and every panel's own
//!   contents. A mode governs which panels *mount* (`app::modes::defaults`),
//!   which is a layout question this module has no part in.
//!
//! ## 5. Why selection and editing are one capability rather than two
//!
//! [`Capabilities::edit_content`] gates the *selection* of page content as
//! well as the verbs that act on it, which conflates two things that are
//! separable in principle. It is one flag on purpose, for as long as both of
//! these hold:
//!
//! - **Selection is the only route to the verbs.** Move, resize, delete and
//!   the Format tab all take the selection as their operand, so a mode that
//!   could select but not edit would need every verb gated *again*,
//!   separately — and the day one was missed, that mode would edit.
//! - **A selection with nothing to read it is not inspection.** Read and
//!   Review mount no Objects panel and no full Properties panel
//!   (`app::modes::defaults::spec`), so a selection there would be an
//!   outline on the page and nothing else.
//!
//! The day canvas selection reaches **annotations** — which
//! `canvas::target`'s header already names as future work, and which Review
//! genuinely needs in order to *"edit your own markup"* — that is a
//! different operand space and it gets its own capability, gated on
//! `author_markup`. It is not this flag with a wider meaning.

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
///
/// Three independent facts rather than one ordered level, even though the
/// three built-in modes happen to form a ladder. The ladder is a property of
/// *that manifest*, not of the type: Review offers markup without content
/// editing, so the two are already independent in the shipped product, and a
/// customized manifest may offer any combination at all. An ordered
/// `enum { Read, Review, Edit }` would have to be re-derived — wrongly —
/// from any manifest that did.
///
/// Copied freely: three `bool`s, computed once per frame in
/// [`crate::app::PdfcerApp`] and passed down by value.
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
    ///
    /// Not used as a fallback anywhere. It exists so a test can name the
    /// expected value rather than spell three fields, and so the Read row of
    /// the module header's table has a name in code.
    pub const NONE: Self = Self {
        edit_content: false,
        author_markup: false,
        author_measure: false,
    };

    /// **What `mode_id` may do, according to `shell`.**
    ///
    /// The one place the derivation lives. `shell` is `Option` because
    /// `PdfcerApp::shell` is: a build whose manifest failed to validate has
    /// none, and that build gets [`Self::FULL`] along with every other
    /// unknown case (module header §3).
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
    ///
    /// The predicate a surface asks when it wants to know "is this a reading
    /// stance?" without caring which authoring verb it is about — the canvas
    /// context menu uses it to decide whether it has anything to offer at
    /// all.
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

/// The one command a chord reaches from **every** mode, whichever tab draws it.
///
/// See [`offers_command`] for the argument. A constant rather than a literal in
/// the comparison so the exception has a name, and so a reader grepping for
/// `edit.copy` finds the rule as well as the registration.
const COPY_IN_EVERY_MODE: &str = "edit.copy"; // ui-text-exempt: a registered command id, never displayed

/// **Whether the active mode offers `command_id` at all.**
///
/// The rule a **keyboard chord** is filtered through, so that a chord cannot
/// reach a command the operator cannot see. Operator decision, 2026-08-14.
///
/// # ★ The problem this closes, and why it was not the gesture gate's job
///
/// [`Capabilities`] governs the **canvas**; the ribbon governs itself by
/// hiding tabs. Between them sat the keymap, which dispatches by command id
/// and consults neither: `app::keyboard::commands` looked a chord up and
/// handed the id straight to the dispatcher. So Read mode hid the Edit tab and
/// `Ctrl+E` still reached `edit.text`.
///
/// It was **latent rather than live** and that was checked rather than
/// assumed — every chord-bound Edit command reaches `command-unimplemented` at
/// the time of writing. Phase 5 is what makes it live, which is why it is
/// closed now: a defect that becomes real on the day someone lands an
/// unrelated feature is worse than one that is real today, because nothing
/// about that day points at this file.
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
/// # ★ The one command this gate refused that it should not have
///
/// `Ctrl+Shift+C` was bound to `edit.copy_page_text`, which sat on the Edit
/// tab, so this function refused it in Read — correctly *by the rule*, and
/// wrongly *about the product*: Acrobat Reader copies text, and replacing
/// Acrobat Reader is what Read is for. The answer was **not** an exception in
/// this file. It was that the command was on the wrong tab: **copying is not
/// authoring** — it reads the page and writes to the clipboard, changing
/// nothing — so on 2026-08-14 the operator moved both text-copy verbs to
/// File ▸ Export as `file.copy_page_text` and `file.copy_document_text`, and
/// the chord followed the command. Read shows File, so the rule now yields the
/// right answer with no clause added to it.
///
/// That is the shape every future case of this should take. A chord refused in
/// a mode where the operator plainly needs it is evidence about the **taxonomy**
/// — it says the command's tab is wrong — and an exception list here would
/// convert that evidence into a second, quieter statement of which tab owns
/// what, free to disagree with the manifest. `edit.form_fill` →
/// `view.panel_forms` was the first instance, this is the second, and the fix
/// was the same both times.
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
    // ★★★ **COPY ESCAPES ITS TAB**, 2026-08-31 — `OPERATOR_REQUESTS.md` O71.
    //
    // The rule above is right and this is the one exception to it: a chord
    // reaches a command only where the mode shows the tab that owns it, so an
    // operator in Read cannot press a key belonging to a tab they cannot see.
    // `edit.copy` lives on the Edit tab because that is where the Clipboard
    // group belongs, and copying is available in **every** mode by the
    // operator's own ruling — *copying is not authoring*, 2026-08-14, which
    // already moved both text-copy verbs off the authoring tab.
    //
    // ⇒ Without this, `Ctrl+C` in Read traced `chord-not-offered id=edit.copy
    // mode=read` and did nothing, which is exactly what O71 reported: a picture
    // that can be selected while reading and not copied. Found by driving it —
    // `dispatch::clipboard` permits copy in every mode and had never been
    // reached from one that could not open the Edit tab.
    //
    // ★ **Copy only.** `edit.cut` and `edit.paste` change the document and stay
    // behind their tab, which is the same asymmetry `dispatch::clipboard`'s
    // rung 3 already enforces one layer down: the gate follows what the verb
    // DOES, not which group it is drawn in.
    if command_id == COPY_IN_EVERY_MODE {
        return true;
    }
    let Some(mode) = shell.modes().iter().find(|m| m.id == mode_id) else {
        return true;
    };
    mode.tabs().contains(&owning_tab.id)
}

/// Whether a gesture that acts on page **content** may proceed.
///
/// Free function rather than a method because it is the exact predicate the
/// gesture machine needs at three call sites and it reads as a sentence
/// there: `content_gesture(caps)`. The marquee is the interesting caller —
/// a *zoom* band is not a content gesture even though it is the same rubber
/// band, so the branch is on the release intent, not on the band.
#[must_use]
pub fn content_gesture(caps: Capabilities) -> bool {
    caps.edit_content
}

/// The memory slot [`publish_edit_content`] writes and [`edit_content_now`]
/// reads.
const EDIT_CONTENT_KEY: &str = "pdfcer.caps.edit-content"; // ui-text-exempt: a memory key, never displayed

/// **Publish whether this frame's mode edits page content**, for the canvas
/// helpers that have no `Capabilities` to hand.
///
/// # ★★ Why a published value rather than a fifth parameter
///
/// `canvas::pressing::grabbable` decides which grips a selection offers, and as
/// of `OPERATOR_REQUESTS.md` O71 that answer depends on the mode: a content
/// selection is reachable in **Read**, where every grip would commit an edit
/// the mode forbids. It has four callers and only two of them hold a
/// `Capabilities`, so the alternative was threading a boolean through two call
/// chains that have no other interest in it.
///
/// ★ This is the same shape `canvas::tool` uses for the armed tool and
/// `crate::pagedrag` for the active document, and it carries the same
/// obligation: **one writer**. `app::frame` publishes it once per frame before
/// any surface draws, so a reader cannot get last frame's answer.
pub fn publish_edit_content(ctx: &egui::Context, on: bool) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(EDIT_CONTENT_KEY), on));
}

/// Whether this frame's mode edits page content. Defaults to `false`.
///
/// ★ `false` when nothing has been published — a unit test with a bare
/// `egui::Context`, or a frame before the publication. That is the safe
/// direction: the consequence of a wrong `false` is a selection that offers no
/// grips, and of a wrong `true` is eight controls whose drag is refused.
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

    /// ★ **The `MODES_AND_PANELS.md` gesture table, asserted against the
    /// shipped manifest.**
    ///
    /// This is the test that makes §2's claim true rather than merely
    /// argued: the derivation is from tabs, so this asserts the *outcome*
    /// for the three real modes. Change a mode's tab list and this fails,
    /// which is correct — the canvas capability moved with it.
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

    /// ★ **Every unknown case lands on `FULL`** — module header §3.
    ///
    /// Asserted as three separate routes to the same answer, because they
    /// are three separate `return`s and a refactor could easily fix one and
    /// break another.
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

    /// ★ **The commands whose chords must keep working in Read**, and the
    /// reason each one does: none of them lives on an ordinary tab.
    ///
    /// This is the test that makes the exception list unnecessary. If any of
    /// these ever moves onto a tab, this fails and names it — which is the
    /// warning you want, because moving `edit.undo` onto the Edit tab would
    /// silently take undo away from Read.
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

    /// ★ **Both text-copy commands are offered in every mode — the property
    /// the 2026-08-14 tab move exists to restore.**
    ///
    /// > *Acrobat Reader copies text, and replacing Acrobat Reader is what Read
    /// > is for. Copying is not authoring.*
    ///
    /// That sentence is the whole reason `edit.copy_page_text` and
    /// `edit.copy_document_text` became `file.copy_page_text` and
    /// `file.copy_document_text`. It is asserted here **directly**, for every
    /// mode including the two where it was never in doubt, because the move is
    /// otherwise invisible to the suite in the direction that matters: nothing
    /// else fails if a later edit puts these two back on the Edit tab, or
    /// invents a `clipboard` group on a tab Read does not show. The registry
    /// count would not move, the group count would, and both are numbers a
    /// reverting change edits on its way past.
    ///
    /// `read_mode_refuses_exactly_these_bound_chords` covers the *chord* half
    /// for the page-text command alone and only because a chord happens to be
    /// bound to it; this covers **both commands**, chord or no chord, which is
    /// the property the operator actually asked for. The document-text command
    /// has no chord at all, so this test is the only thing standing under it.
    ///
    /// Deliberately asserted through `offers_command` rather than by looking the
    /// ids up on the File tab: the tab is *how* it is true today, and the
    /// requirement is that the mode offers them however that comes about.
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

    /// ★ **…and a command on a tab the mode hides is not offered.**
    ///
    /// The other half, without which the test above passes on a build where
    /// the filter returns `true` unconditionally.
    #[test]
    fn a_command_on_a_hidden_tab_is_not_offered() {
        let shell = built_in();
        // Edit-tab commands: reachable only in Edit.
        // `edit.objects` was the third id here until 2026-08-31 (O69,
        // deleted). `edit.reflow_block` replaces it rather than the list
        // shrinking to two: the property under test is *a command on a hidden
        // tab is not offered*, and it needs more than one witness or a build
        // that offered exactly one Edit command everywhere would still pass.
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
    ///
    /// It asserts the shape rather than a fixed list: every bound chord must
    /// be offered by Edit, because Edit shows every tab. A binding that failed
    /// that would be one pointing at a command on no tab of any mode, i.e. a
    /// chord bound to something unreachable.
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

    /// ★ **The whole consequence of the gate, in one table.**
    ///
    /// Every chord the shipped keymap binds, resolved against Read — the
    /// mode that hides the most. Asserted as an exact set rather than a
    /// spot-check, so that adding a binding, moving a command between tabs, or
    /// changing a mode's tab list all fail here and print what changed.
    ///
    /// ★ **It was also the record of a taxonomy question, and that question is
    /// now answered.** The note here used to read: *"`edit.copy_page_text` is on
    /// the Edit tab, so `Ctrl+Shift+C` is refused in Read — and Acrobat Reader
    /// copies text, which is the standard this mode is measured against.
    /// Copying is not authoring, so by the same argument that moved
    /// `edit.form_fill` to `view.panel_forms` it does not belong on the
    /// authoring tab… the destination tab is an operator decision."*
    ///
    /// The operator decided on 2026-08-14: **File ▸ Export**. Both text-copy
    /// commands are now `file.copy_page_text` and `file.copy_document_text`, the
    /// chord moved with the page-text one, and File is in every mode's tab list
    /// — so the id has dropped out of the set below, which is the *whole*
    /// visible consequence of the move and the reason this test asserts an exact
    /// set rather than a spot check. Nothing was added to `offers_command` to
    /// achieve it. See [`super::offers_command`]'s header, and
    /// [`both_text_copy_commands_are_offered_by_every_mode`] for the property
    /// that now has a test of its own.
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
                // ★ The object clipboard, added 2026-08-19. Refused in Read
                // because all three sit on the **Edit tab**, which Read does not
                // show — the structural gate, not a special case.
                //
                // ★★ This does NOT take text copying away from Read, and the
                // distinction matters because Acrobat Reader copies text and
                // this mode is measured against it. `Ctrl+C` over a swept range
                // is `canvas::textsel::clipboard`'s, read before the command
                // dispatcher sees the key at all. What Read refuses is copying
                // an *annotation*, which it could not paste anywhere.
                // Authoring the page's own content — correctly refused.
                "edit.add_text",
                // ★★★ `edit.copy` was HERE until 2026-08-31, and its removal is
                // the one exception `offers_command` carries —
                // `OPERATOR_REQUESTS.md` O71.
                //
                // The paragraph above says Read still copies TEXT, and that was
                // true and sufficient while text was the only thing Read could
                // select. O71 made a picture selectable in Read so it could be
                // pasted into Word, and then `Ctrl+C` traced
                // `chord-not-offered id=edit.copy mode=read` and did nothing:
                // the command was permitted by the dispatcher and unreachable by
                // the keyboard. Found by driving it, not by reading it.
                //
                // ⇒ Copy escapes its tab; **cut and paste do not**, and the
                // asymmetry is the operator's own *copying is not authoring*
                // ruling applied one layer up from where
                // `dispatch::clipboard`'s rung 3 already applies it.
                "edit.cut",
                "edit.paste",
                // ★ Added 2026-08-29 with `edit.paste_duplicate` (O58). Read
                // refuses it for the same structural reason as its three
                // siblings: the whole Clipboard group lives on the **Edit
                // tab**, which Read does not show. Nothing special-cases the
                // new chord — it inherits the gate by being in the group.
                "edit.paste_duplicate",
                // ★ Read refuses it because it selects CONTENT, and Read is the
                // mode that does not edit content. Text selection has its own
                // Ctrl+A and is unaffected — which is the distinction this list is
                // for.
                "edit.select_all",
                "edit.text",
                // ★ The object clipboard, added 2026-08-19 — listed after the
                // two text verbs because this array is SORTED (the assertion
                // sorts and dedups), not grouped by subject.
                //
                // Refused in Read because all three sit on the **Edit tab**,
                // which Read does not show: the structural gate, not a special
                // case.
                //
                // ★★ It does NOT take text copying away from Read, and that
                // distinction matters because Acrobat Reader copies text and
                // this mode is measured against it. `Ctrl+C` over a swept range
                // belongs to `canvas::textsel::clipboard`, which reads the key
                // before the command dispatcher sees it. What Read refuses is
                // copying an ANNOTATION, which it could not paste anywhere.
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
