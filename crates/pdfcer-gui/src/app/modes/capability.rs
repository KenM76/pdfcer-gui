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

/// ★★★ **The commands whose chord reaches every mode because their own
/// dispatcher gates them** — 2026-09-05.
///
/// See [`offers_command`]'s §"The one class that escapes its tab" for the whole
/// argument. In short: the tab gate is a *proxy* for *"may this mode do this?"*,
/// and where a dispatcher asks the real question — of the operand, per press —
/// the proxy is not merely redundant, it answers a **different** question and
/// gets it wrong.
///
/// A named constant rather than literals in the comparison so the class has a
/// name, and so a reader grepping for `edit.paste` finds the rule as well as
/// the registration.
///
/// ⚠ **Membership is not a decoration.** Every id here must be one
/// `app::dispatch::clipboard` gates on `PdfcerApp::capabilities`, or this list
/// hands a mode a verb nothing stops.
/// [`tests::every_dispatcher_gated_command_is_one_the_clipboard_dispatcher_owns`]
/// binds the two ends mechanically rather than by this paragraph.
const GATED_BY_THEIR_DISPATCHER: [&str; 5] = [
    // ui-text-exempt: registered command ids, never displayed.
    "edit.copy",
    "edit.cut",
    "edit.paste",
    "edit.paste_duplicate",
    // ★★★ **`edit.duplicate`, 2026-09-06** — Ctrl+D, and it joins the class on
    // its first day rather than after a driven sweep found it silent, which is
    // the whole value of the class having a name.
    //
    // It is registered on the **Edit** tab, beside the four above and for the
    // same reason: the Clipboard group is where an operator looks for *"make
    // another one of this"*. **Review is not shown that tab** — and Review is
    // the mode whose entire purpose is marking up somebody else's drawing,
    // i.e. the mode in which an operator is most likely to be laying out a row
    // of identical revision marks. Without this line `Ctrl+D` would trace
    // `chord-not-offered id=edit.duplicate mode=review` and do nothing, which
    // is character for character the defect the 2026-09-05 sweep found for
    // `edit.paste`.
    //
    // ⚠ Membership is a promise, not a decoration: `app::dispatch::clipboard`'s
    // `duplicate` arm gates on `capabilities().author_markup` and words the
    // refusal through `ModeRefusal::DuplicateMarkup`, so Read is still stopped
    // — with a sentence rather than with silence.
    "edit.duplicate",
];

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
/// # ★★★ The one class that escapes its tab — and why it is a class, not a list
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
/// tab. So the proxy refused all four in Review, and for cut and paste it
/// refused something the mode is **allowed to do**:
///
/// > ```text
/// > chord-command      chord="Ctrl+C" id=edit.copy  via=clipboard-event
/// > clipboard-copy     kind=selection page=0 objects=0 annots=1 thin=0 bytes=395
/// > chord-command      chord="Ctrl+V" id=edit.paste via=clipboard-event
/// > chord-not-offered  id=edit.paste mode=review
/// > ```
///
/// **In the mode whose entire purpose is marking up somebody else's drawing, an
/// operator could copy a comment and had nowhere to put it.** Two independent
/// driven checks hit that line
/// (`copying_a_sticky_note_carries_the_whole_comment` failed on it,
/// `a_note_can_be_written_onto_a_shape_that_exists` skipped on it), and O71 had
/// found the identical shape one layer over for copy five days earlier.
///
/// ## ★★ Why this is NOT the taxonomy evidence the section above describes
///
/// The paragraph above says a chord refused where the operator plainly needs it
/// is evidence the command's **tab is wrong**, and that the fix is to move it.
/// That was right for `edit.form_fill` → `view.panel_forms` and for the two
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
/// ## ★★★ The debt this takes on, and it is paid in `dispatch::clipboard`
///
/// A chord refused *here* traces `chord-not-offered id=… mode=…`. A chord that
/// reaches a dispatcher which silently `return`s traces **nothing on any
/// surface**. Pushing the chord through blind therefore obliges the dispatcher
/// to word every refusal it can now meet, and on the same day it did: both mode
/// gates in `app::dispatch::clipboard` now call
/// `app::status::decline::record_mode_refusal`, which draws in the `⊗` slot
/// that means *this did not happen*. Without that half this change would have
/// traded a defect for a quieter one.
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
    // ★★★ **THE WHOLE CLIPBOARD ESCAPES ITS TAB**, 2026-09-05 — and it was
    // `edit.copy` alone from 2026-08-31 (`OPERATOR_REQUESTS.md` O71) until the
    // driven sweep proved that half of it was the defect. See
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

    /// ★★★ **Review offers the whole clipboard** — the driven sweep's finding
    /// A1, as a headless assertion.
    ///
    /// > *In the mode whose entire purpose is marking up somebody else's
    /// > drawing, an operator could copy a comment and had nowhere to put it.*
    ///
    /// `edit.copy` was offered and `edit.paste` was not, and two independent
    /// driven checks traced `chord-not-offered id=edit.paste mode=review`. All
    /// four are asserted rather than paste alone, because the defect was an
    /// **asymmetry**: a build that fixed paste and left cut behind would put the
    /// same trap one keystroke away.
    ///
    /// # ⚠ The four ids are LITERALS here, and that is the whole test
    ///
    /// It was written as `for id in GATED_BY_THEIR_DISPATCHER` and **the
    /// falsification caught it**: planting the pre-fix state — shrinking that
    /// constant back to `["edit.copy"]` — left this test *passing*, because it
    /// then asserted "the one thing in the list is offered", which was true.
    ///
    /// ⇒ **A test that iterates the mechanism it is testing cannot fail by that
    /// mechanism being narrowed**, which is the exact regression this test
    /// exists to catch. The property is about *these four commands in this
    /// mode*, so these four commands are written out; the constant is asserted
    /// separately, below, so the two cannot drift without a named failure.
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

    /// ★★ **…and Read still refuses all four — but in `dispatch::clipboard`,
    /// not here.**
    ///
    /// The other half of the change above, and it is asserted at the layer that
    /// now owns the answer rather than at this one. `Capabilities::NONE` is what
    /// Read gets, and that is what both of the dispatcher's gates read:
    /// `edit_content` for content and a field, `author_markup` for markup and
    /// for an empty clipboard. So every operand Read can present is refused.
    ///
    /// ★ It asserts the **capability**, not the gate's code, because the gate is
    /// a match on `Clipped` that this module cannot construct without a
    /// document. What it pins is the premise the gate rests on: if Read ever
    /// gained either flag, this fails and names it — which is the warning worth
    /// having, since the dispatcher would then quietly permit the paste.
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

    /// ⚠ **Every id that escapes its tab is one the clipboard dispatcher owns.**
    ///
    /// The list in [`super::GATED_BY_THEIR_DISPATCHER`] is safe only because
    /// each member's effect is gated somewhere else. This binds the two ends
    /// mechanically: an id added to that list that no dispatcher claims would be
    /// a command handed to every mode with nothing standing under it, and the
    /// symptom would be silence rather than a failure.
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

    /// ★ **…and it is not simply every id that dispatcher owns**, which is the
    /// direction that would make the list vacuous.
    ///
    /// `edit.copy_as_vector` is routed by the same dispatcher and is **not** on
    /// the escape list, because it needs no escape: it takes no mode gate at all
    /// (*copying is not authoring*) and it lives on the Edit tab, where its
    /// button is absent outside Edit — visibility doing the work, which is the
    /// rule `app::modes` states. A list that had simply been derived from
    /// `handles` would have included it and would have been documenting nothing.
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
                // ★ Authoring the page's own content — correctly refused. Read
                // is the mode that does not author.
                "edit.add_text",
                // ★★★ THE WHOLE CLIPBOARD LEFT THIS LIST, in two steps.
                //
                // `edit.copy` went on 2026-08-31 (O71): a picture became
                // selectable in Read so it could be pasted into Word, and
                // `Ctrl+C` traced `chord-not-offered id=edit.copy mode=read` and
                // did nothing — permitted by the dispatcher, unreachable by the
                // keyboard. Found by driving it.
                //
                // `edit.cut`, `edit.paste` and `edit.paste_duplicate` went on
                // 2026-09-05, and the note that stood here said the opposite in
                // as many words: *"Copy escapes its tab; **cut and paste do
                // not**, and the asymmetry is the operator's own copying-is-not-
                // authoring ruling."* That reasoning was about **Read**, where
                // it is still true, and it was applied as a rule about the
                // **command**, where it is false — so it also refused paste in
                // **Review**, which authors markup and is the mode the whole
                // feature exists for. Two driven checks hit
                // `chord-not-offered id=edit.paste mode=review`.
                //
                // ⇒ The three chords now reach `dispatch::clipboard`, which
                // gates the EFFECT on the operand and refuses them in Read
                // anyway — in words, on the `⊗` slot, which is more than this
                // list ever gave. `read_mode_still_refuses_the_clipboard_verbs_
                // it_should` is the test that keeps that true, and it asserts
                // the outcome rather than the route, which is the only form of
                // the claim that survives the gate moving.
                //
                // ★ Read refuses `edit.select_all` because it selects CONTENT.
                // Text selection has its own Ctrl+A and is unaffected — which is
                // the distinction this list is for.
                "edit.select_all",
                "edit.text",
                // ★★★ **The four Markup ▸ Arrange chords**, joined 2026-09-06 —
                // `Ctrl+[`, `Ctrl+]` and their Shift forms.
                //
                // Refused in Read for the same structural reason as the page
                // verbs below rather than for a reason of their own: Read's tab
                // list is File and View, so the Markup tab is not there, and
                // `offers_command` answers `false` for every id on a tab the
                // mode does not show. **Nothing was added to the gate.**
                //
                // ★★ And it is the right answer on the merits, which is worth
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
