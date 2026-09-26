//! # `modecapability` — what a mode lets the canvas do
//!
//! **The one place the rule "Read does not edit the document" is written
//! down.** Everything else — the gesture machine, the key handler, the
//! context menus, the tool arming — asks this module and branches on the
//! answer; none of them knows what `"read"` is.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/modecapability.md`.

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
#[doc(hidden)]
pub const GATED_BY_THEIR_DISPATCHER: [&str; 5] = [
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
