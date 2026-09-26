//! # `canvas::tool::arm` — how a tool is CHOSEN
//!
//! ## The seam against `super`
//!
//! `super` answers *"what IS a tool?"* — the enum, and the predicates that are
//! properties of a variant: which cursor it wants, whether it pans, which kind
//! it carries, which capability it needs. Every one of those is a pure
//! function of the value and none of them touches the world.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/tool/arm.md`.

use super::{CanvasTool, MarkupKind, MeasureKind, TOOL_MEMORY_KEY, TextEditKind};
use crate::app::modes::Capabilities;
use egui::{CursorIcon, Key};

/// **What the pointer looks like this frame** — the whole precedence, in one
/// pure function.
#[must_use]
pub fn cursor_for(
    tool: CanvasTool,
    gesture: Option<crate::canvas::gesture::DragKind>,
    hovered_grip: Option<crate::canvas::handles::Grip>,
    pointer_down: bool,
    over_canvas: bool,
) -> Option<CursorIcon> {
    use crate::canvas::gesture::DragKind;

    if let Some(icon) = tool
        .cursor(pointer_down)
        .filter(|_| over_canvas || pointer_down)
    {
        return Some(icon);
    }
    if let Some(kind) = gesture {
        return Some(match kind {
            // **One crosshair for every rubber band**, whatever the band will
            // become. All six are the same gesture — a rectangle dragged out —
            // and `gesture`'s header refuses a second set of pixels for it.
            // What says which of them is armed is off-canvas: the pressed
            // ribbon control, or for a placement (O66) the Tool panel's armed
            // instruction, because the window that would have said so has
            // stepped aside.
            //
            // Every kind is spelled rather than wildcarded, including the
            // ones rung 1 already claims. A drag cannot be in flight without
            // the tool that started it, so those arms are unreachable today;
            // naming them is what keeps the six answers one answer the day one
            // of them stops agreeing.
            DragKind::Marquee(_)
            | DragKind::Markup(_)
            | DragKind::TextAnnot(_)
            | DragKind::Form(_)
            | DragKind::Place(_)
            | DragKind::TextBox => CursorIcon::Crosshair,
            DragKind::Move => CursorIcon::Grabbing,
            DragKind::Resize(grip) => grip.cursor(),
            // `Grabbing` too, and it names the same limit `Grip::Rotate`'s
            // cursor records: egui 0.35 has no rotate cursor. What it says is
            // *"you are holding something"*, which is true; what it does not say
            // is *"and turning it"*, which `handles.md` H6 asks for. Spelled out
            // rather than folded into the `Move` arm above, so the day a rotate
            // cursor exists there is one line to change and it is findable.
            DragKind::Rotate => CursorIcon::Grabbing,
            // `Grabbing`, the same as a move, and deliberately NOT a bespoke
            // icon. A handle drag IS a move — of a control point rather than of
            // an object — and the operator learns one grammar: the closed hand
            // means "you have hold of something and it follows the pointer".
            // A distinct cursor would be teaching a distinction that changes
            // nothing about what the gesture does.
            //
            // A markup shape's node joins them, and it is stated rather than
            // wildcarded for the reason the marquee arm above states: a
            // deliberate answer that happens to equal its neighbour's is one
            // line to change the day it stops being equal, and a wildcard is
            // not.
            DragKind::Handle { .. }
            | DragKind::DimensionVertex { .. }
            | DragKind::MarkupVertex { .. } => CursorIcon::Grabbing,
            // The I-beam for a sweep that began under the MODE rule rather
            // than under an armed tool, which is the whole of what this arm is
            // for.
            //
            // With `CanvasTool::Text` armed, rung 1 answers `Text` on hover,
            // before any drag, over the grey surround as readily as the paper,
            // at the cost of one match arm per frame. But in **Read and
            // Review** a press means text with *no tool armed at all* — the
            // select tool, under `textsel::takes_the_press`'s first disjunct —
            // so rung 1 has nothing to answer with and this rung is the only
            // one that can.
            //
            // Making it hover in those modes would mean asking "is there a
            // glyph under the pointer?" on every frame the pointer moves: a hit
            // test against the page's extraction, paid on canvases nobody is
            // selecting on, which is most of them. Threading `Capabilities` in
            // here to synthesise a tool from the mode would put the mode gate
            // in a second place, which `canvas::textsel`'s header §3 argues
            // against at length.
            //
            // ⇒ So the rule is: **armed ⇒ I-beam always; un-armed ⇒ I-beam once
            // the sweep starts.** No operator sees that as an inconsistency,
            // because the two cases never coexist on one canvas: arming the
            // tool is what moves a mode from the second case to the first.
            DragKind::TextSelect => CursorIcon::Text,
        });
    }
    hovered_grip.map(crate::canvas::handles::Grip::cursor)
}

/// Compose the chosen tool with the space bar — **the rule, and the only
/// place it exists**.
#[must_use]
pub fn resolve(selected: CanvasTool, space_held: bool) -> CanvasTool {
    if space_held {
        CanvasTool::Hand
    } else {
        selected
    }
}

/// The tool the operator chose — the persistent half, unaffected by the space
/// bar.
#[must_use]
pub fn selected(ctx: &egui::Context) -> CanvasTool {
    let id = egui::Id::new(TOOL_MEMORY_KEY);
    ctx.data(|d| d.get_temp::<CanvasTool>(id).unwrap_or_default())
}

/// Choose a tool. **The entry point a `view.tool_hand` / `view.tool_select`
/// command calls, and the only writer of the armed tool in the crate.**
pub fn select(ctx: &egui::Context, tool: CanvasTool) {
    let previous = selected(ctx);
    let id = egui::Id::new(TOOL_MEMORY_KEY);
    ctx.data_mut(|d| d.insert_temp(id, tool));
    if previous != tool {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            format!("canvas-tool armed={tool:?} from={previous:?}")
        });
    }
}

/// Flip between the hand and the select tool. **The entry point a single
/// `view.tool_hand` *toggle* command calls.**
pub fn toggle_hand(ctx: &egui::Context) -> CanvasTool {
    let next = match selected(ctx) {
        CanvasTool::Hand => CanvasTool::Select,
        // Any other tool is *left* by pressing Hand, not toggled through — the
        // operator asked for the hand, and returning them to Select would make
        // one press mean "put the pen down" and a second one mean "pick the
        // hand up". The text tool joins that arm rather than earning its own for
        // the identical reason: pressing Hand while sweeping text means Hand.
        // `Place` joins this arm: pressing Hand while a placement is armed
        // means Hand. The pending record is cleared by `retire_forbidden`'s
        // sibling below and by `canvas::keys`' Escape claimant, so the window
        // it was hiding comes straight back — see `canvas::placing`.
        CanvasTool::Select
        | CanvasTool::Node
        | CanvasTool::Markup(_)
        | CanvasTool::Measure(_)
        | CanvasTool::TextAnnot(_)
        | CanvasTool::Place(_)
        | CanvasTool::Text
        | CanvasTool::TextEdit(_)
        | CanvasTool::Form(_) => CanvasTool::Hand,
    };
    select(ctx, next);
    next
}

/// Flip between the text tool and the select tool. **The entry point the
/// `view.tool_text` toggle command calls.**
pub fn toggle_text(ctx: &egui::Context) -> CanvasTool {
    let next = match selected(ctx) {
        CanvasTool::Text => CanvasTool::Select,
        // …and from any other tool this *takes* the text tool rather than
        // returning to Select, which is `toggle_hand`'s rule above and
        // `arm_markup`'s different-kind-re-arms rule, spelled once more.
        CanvasTool::Select
        | CanvasTool::Node
        | CanvasTool::Hand
        | CanvasTool::Place(_)
        | CanvasTool::Markup(_)
        | CanvasTool::Measure(_)
        | CanvasTool::TextAnnot(_)
        | CanvasTool::TextEdit(_)
        | CanvasTool::Form(_) => CanvasTool::Text,
    };
    select(ctx, next);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // The same argument `markup-tool` and `measure-tool` carry, and it is
        // sharper here than for either: an armed text tool changes the CURSOR and
        // nothing else on screen, so an armed canvas and an un-armed one are not
        // merely the same screenshot — they are the same screenshot even with the
        // pointer in it, because a captured window does not carry the cursor.
        // This line is the only way a harness can prove the ribbon button armed
        // anything.
        format!("text-tool tool={next:?}")
    });
    next
}
/// **Arm a form-field tool**, or put it down if it is already armed.
pub fn arm_form(ctx: &egui::Context, kind: crate::canvas::formfield::FormFieldKind) -> CanvasTool {
    let next = if selected(ctx) == CanvasTool::Form(kind) {
        CanvasTool::Select
    } else {
        CanvasTool::Form(kind)
    };
    select(ctx, next);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!("form-tool-armed kind={kind:?} now={next:?}")
    });
    next
}

/// Arm the markup tool with `kind`, or retire it if that kind is already
/// armed. **The entry point every `markup.*` shape command calls.**
pub fn arm_markup(ctx: &egui::Context, kind: MarkupKind) -> CanvasTool {
    let next = if selected(ctx) == CanvasTool::Markup(kind) {
        CanvasTool::Select
    } else {
        CanvasTool::Markup(kind)
    };
    select(ctx, next);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // The tool a canvas is armed with is otherwise invisible from outside:
        // a crosshair is a cursor, and a screenshot of an armed canvas and an
        // un-armed one are the same picture — which is defect 8's lesson
        // exactly. This line is how a harness proves the button armed anything.
        format!("markup-tool tool={next:?}")
    });
    next
}

/// Arm a **text-annotation** tool with `kind`, or retire it if that kind is
/// already armed. The entry point `markup.text_box`, `markup.sticky_note` and
/// `markup.stamp` call.
pub fn arm_text_annot(
    ctx: &egui::Context,
    kind: crate::canvas::textannot::TextAnnotKind,
) -> CanvasTool {
    let next = if selected(ctx) == CanvasTool::TextAnnot(kind) {
        CanvasTool::Select
    } else {
        CanvasTool::TextAnnot(kind)
    };
    select(ctx, next);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("markup-tool tool={next:?}")
    });
    next
}

/// Arm the measure tool with `kind`, or retire it if that kind is already
/// armed. **The entry point every `measure.*` tool command calls.**
pub fn arm_measure(ctx: &egui::Context, kind: MeasureKind) -> CanvasTool {
    let next = if selected(ctx) == CanvasTool::Measure(kind) {
        CanvasTool::Select
    } else {
        CanvasTool::Measure(kind)
    };
    select(ctx, next);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // Same argument as `markup-tool` above: an armed canvas and an un-armed
        // one are the same screenshot, so this line is the only way a harness
        // can prove the ribbon button armed anything.
        format!("measure-tool tool={next:?}")
    });
    next
}

/// Retire the measure tool, returning to [`CanvasTool::Select`], and report
/// whether there was one to retire.
pub fn disarm_measure(ctx: &egui::Context) -> bool {
    if selected(ctx).measure_kind().is_none() {
        return false;
    }
    select(ctx, CanvasTool::Select);
    true
}

/// Retire the markup tool, returning to [`CanvasTool::Select`], and report
/// whether there was one to retire.
pub fn disarm_markup(ctx: &egui::Context) -> bool {
    if selected(ctx).markup_kind().is_none() {
        return false;
    }
    select(ctx, CanvasTool::Select);
    true
}

/// **Put down whatever is armed**, and report whether anything was.
pub fn disarm_any(ctx: &egui::Context) -> bool {
    if selected(ctx) == CanvasTool::Select {
        return false;
    }
    select(ctx, CanvasTool::Select);
    // The caret is not part of `CanvasTool`, so a draft can outlive the tool it
    // was typed under. It is not torn down here: `app::frame`'s step 2d settles
    // one whose caret tool is no longer armed, which this call has just made
    // true, and settling writes the operator's text instead of discarding it.
    true
}

/// **Retire an armed tool the mode being entered does not permit**, and report
/// whether there was one.
///
/// Called from `PdfcerApp`'s mode-change arm, once, on the frame the operator
/// moves the selector.
///
/// # Why arming has to be undone rather than merely refused
///
/// The armed tool lives in `egui::Memory` and is **application**-scoped, not
/// per-mode — see this module's header. So a Rectangle armed in Edit is still
/// armed after a switch to Read, and it survives a switch back. That is the
/// right lifetime for a tool (an operator who returns to Edit expects their
/// pen), and it is exactly wrong across a mode that forbids the pen.
///
/// [`crate::canvas::gesture::press_kind`] already refuses to give a forbidden
/// tool a meaning, so nothing would be drawn either way. What that refusal
/// cannot fix is the **cursor**: [`CanvasTool::cursor`] gives an armed markup
/// tool a crosshair, so without this the operator would be shown a drawing
/// cursor over every page of a document they cannot draw on — a promise the
/// canvas has already decided not to keep. Retiring the tool is what makes the
/// pointer tell the truth.
///
/// Returns to [`CanvasTool::Select`] rather than to `Hand`, matching
/// [`disarm_markup`]: Select is this enum's `#[default]` and the stance every
/// other retirement path returns to, and a mode change that silently swapped in
/// a *different* tool would be a second surprise on top of the first.
pub fn retire_forbidden(ctx: &egui::Context, caps: Capabilities) -> bool {
    let armed = selected(ctx);
    let permitted = match armed {
        // None of the three touches the document: Select is inert in a mode that
        // cannot select (`press_kind` gives its presses no meaning), Hand only
        // pans, and Text reads the page and writes to the clipboard. Retiring any
        // of them would take a navigation or reading tool away from the mode that
        // navigates and reads.
        //
        // **Text is on this arm and NOT on the markup arm below, and the
        // difference is the operator's own ruling rather than a judgement made
        // here.** The obvious move when adding a tool is to copy the line above
        // the cursor and swap the capability — `CanvasTool::Text => caps.???` —
        // and there is no capability to put there. Three steps:
        //
        // 1. **Selecting text authors nothing.** It changes no byte, bumps no
        //    `edit_epoch`, and touches no `EditSession`. `app::modes::capability`
        //    §4's not-gated list is exactly that class — *"pan, zoom, the hand
        //    tool, marquee zoom, Find, guides, rulers, grid: navigation and
        //    inspection, none of which touches the document"* — and its nearest
        //    neighbour there is Find, which also extracts the page's text, also
        //    derives quads from it, and also washes the result.
        // 2. **The operator settled it for the commands already**, under the
        //    sentence *copying is not authoring*, which is why both text-copy
        //    verbs sit off the authoring tab. A capability invented here would
        //    be that ruling restated in a second place, free to disagree with
        //    it — the same argument `canvas::textsel` §3 makes for why there is
        //    no `select_text` flag.
        // 3. **The retirement would be actively wrong in both directions.**
        //    Retiring it on the way into Read would take away a tool that mode
        //    plainly permits (its select tool already sweeps text). Retiring it on
        //    the way into **Edit** would be worse: Edit is the one mode this tool
        //    exists for, so a capability check that failed there would delete the
        //    feature on the frame the operator entered the mode that needs it.
        //
        // So the honest answer is *none*, and it is written as membership of this
        // arm — where the reason is stated — rather than as a `true` on a line of
        // its own, so that a future reader adding a fifth tool has to decide which
        // of the two groups it joins.
        CanvasTool::Select | CanvasTool::Hand | CanvasTool::Text => true,
        // **Node is on the OTHER side of the line the paragraph above
        // draws, and it answers to TWO capabilities.**
        //
        // It is the first tool in this enum whose whole purpose is to *change*
        // the document — a point is selected in order to be dragged — so unlike
        // Select, Hand and Text it retires when the mode forbids the change.
        // The disjunct must name both of the things it changes, or the tool is
        // retired in a mode that has one of them:
        //
        // * an anchor of a **path on the page** — `move_nodes`, `move_handle`
        //   — **`edit_content`** — drawn by `canvas::painting::draw_anchors`;
        // * a corner of a **ce dimension** — `move_dimension_vertex`,
        //   `insert_dimension_vertex`, `remove_dimension_vertex` —
        //   **`author_measure`** — drawn by `canvas::painting`'s vertex-handle
        //   loop.
        //
        // The second row is the one Review has. `canvas::gesture::press_kind`
        // gates the ce-dimension rung on `author_measure`, on the ruling that
        // *reshaping a ce dimension is a measure edit — it writes the sidecar
        // and one annotation and touches no page content* — and
        // `canvas::dimdrag` gates **adding and removing** a corner on this tool
        // being armed, which is what makes the arming mean something here
        // rather than being a widened gate for its own sake.
        //
        // **Keeping the tool armed in Review does NOT put anchor marks on a
        // page whose every drag is refused**, and this is the thing that must
        // not regress. Three independent gates say so:
        //
        // 1. `canvas::clicking`'s node-tool branch is `is_node() &&
        //    caps.edit_content`, so a Node-tool click in Review picks no
        //    anchor;
        // 2. `press_kind`'s content rung is gated on `edit_content`, so no
        //    anchor drag can start;
        // 3. `draw_anchors` needs `selection.entered_object()` or a selected
        //    content outline, and Review can produce neither — content cannot
        //    be selected there at all — so it declines with `not-entered` and
        //    paints nothing.
        //
        // ⇒ In Review the tool draws exactly what R9 says an unavailable
        // capability draws: **nothing** on page content, and the ce dimension's
        // own corner handles, which are on screen and draggable there anyway.
        // What the arming buys in that mode is reach to the tool whose sentence
        // explains those handles, and a Ctrl-drag that adds or removes one.
        CanvasTool::Node => caps.edit_content || caps.author_measure,
        // Authoring a form field is a change to the DOCUMENT's content, not
        // an annotation over it — a `/Widget` and its field are page objects
        // the operator is adding. So it answers to `edit_content` and retires
        // when a mode that cannot edit is chosen, exactly like the node tool.
        // Pairing it with `author_markup` would let Review mode place form
        // controls, which is not a review activity.
        CanvasTool::Form(_) => caps.edit_content,
        // A placement answers to the capability of the thing being placed,
        // which each kind states for itself — see `PlaceKind::capability`.
        // Asking here would put the mapping in a second place and let the two
        // disagree about whether Review may drop an image on a drawing.
        CanvasTool::Place(kind) => kind.capability(caps),
        CanvasTool::Markup(_) => caps.author_markup,
        // Gated on `author_markup`, not on a capability of its own.
        //
        // A text box, a sticky and a stamp ARE markup — they are annotations
        // added on top of the page, they appear in the Comments panel beside
        // the geometric kinds, and a mode that may not author a rectangle has
        // no business authoring a callout. Giving them a fourth capability
        // would let the two drift, and there is no mode in `RIBBON_IA.md` that
        // wants one without the other.
        CanvasTool::TextAnnot(_) => caps.author_markup,
        CanvasTool::Measure(_) => caps.author_measure,
        // …and the caret tool joins the *authoring* group, which is the
        // decision the paragraph above asks a fifth tool's author to make.
        //
        // It joins it on the same three steps read the other way. Step 1: this
        // one **does** touch the document — it rewrites a show operator or
        // appends new page content, bumps `edit_epoch`, and lands one
        // `EditSession` command. Step 2: the operator's *copying is not
        // authoring* ruling is the reason `Text` is exempt, and it says nothing
        // about typing, which is authoring by any reading. Step 3: retiring it
        // is right in both directions here — Read plainly does not permit it,
        // and Edit's `edit_content` is true, so the mode this tool exists for
        // keeps it.
        //
        // `edit_content` and not `author_markup`: a markup annotation is a
        // comment layered over the page, and this changes the page itself.
        CanvasTool::TextEdit(_) => caps.edit_content,
    };
    if permitted {
        return false;
    }
    select(ctx, CanvasTool::Select);
    // …and a PENDING PLACEMENT goes with it, or the mode change leaves a
    // hidden dialog with nothing coming back for it — `OPERATOR_REQUESTS.md`
    // O66.
    //
    // The same argument the draft below makes, one step further: a placement is
    // a window that has stepped aside and is waiting. `canvas::placing` makes
    // the window's absence DERIVED from this record precisely so that clearing
    // it here is all "bring the window back" means — there is no second flag to
    // remember. Cheap on every other retirement: one `egui::Memory` read.
    crate::canvas::placing::cancel(ctx);
    // …and the draft goes with the tool, but it is **written** on the way
    // out rather than dropped. A retirement that left one in `egui::Memory`
    // would leave a keystroke buffer aimed at a document the mode being entered
    // says is not the operator's to change, and it would still be there on the
    // way back holding text typed against a revision that may have moved. One
    // that discarded it would charge the operator their typing for moving the
    // mode selector. `app::frame`'s step 2d takes the third reading — the
    // select above makes the draft's caret tool un-armed, so the text lands on
    // the page and `Ctrl+Z` is there if it was not wanted.
    true
}

/// Arm the caret tool with `kind`, or retire it if that kind is already armed.
/// **The entry point the `edit.text` and `edit.add_text` dispatch arms call.**
pub fn arm_text_edit(ctx: &egui::Context, kind: TextEditKind) -> CanvasTool {
    let next = if selected(ctx) == CanvasTool::TextEdit(kind) {
        CanvasTool::Select
    } else {
        CanvasTool::TextEdit(kind)
    };
    select(ctx, next);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // `markup-tool`'s argument, and sharper: an armed caret tool changes the
        // cursor and nothing else on screen, and a captured window does not carry
        // the cursor — so an armed canvas and an un-armed one are the same
        // picture even with the pointer in it. This line is the only way a
        // harness can prove the ribbon button armed anything.
        format!("text-edit-tool tool={next:?}")
    });
    next
}

/// Whether the space bar is down **and the canvas is entitled to it**.
///
/// # It must ask `textedit::composing`, and nothing narrower
///
/// `ctx.text_edit_focused()` is **false** for an operator typing into the
/// canvas caret: that caret is deliberately not an `egui::TextEdit`, so egui
/// reports no focused text field for somebody who is visibly mid-word. The
/// space bar is this tool's modifier, so a canvas asking egui takes the space
/// and pans the paper — and text editing cannot type a space at all.
///
/// > *"I can edit text now, but there is no live preview of that either, and it
/// > doesn't accept spaces. Like how?"* — the operator
///
/// `app::keyboard` asks the same question about the same second claimant, so
/// the predicate exists exactly **once** and `tools/gates/check-typing-guard.sh`
/// refuses a second copy.
///
/// # The form ring is a third claimant, and it is a different question
///
/// A check box on the Tab ring reads Space as *toggle me*, and it holds no
/// caret, so `composing` is false while it does. [`forms::ring_takes_space`]
/// is not a second copy of the typing predicate the gate forbids -- that one
/// asks whether a caret is open, this one asks whether the canvas owns the
/// keyboard at all.
///
/// [`forms::ring_takes_space`]: crate::canvas::forms::ring_takes_space
#[must_use]
pub fn space_held(ctx: &egui::Context) -> bool {
    !crate::canvas::textedit::composing(ctx)
        && !crate::canvas::forms::ring_takes_space(ctx)
        && ctx.input(|i| i.key_down(Key::Space))
}

/// What the primary button means on this frame — [`resolve`] applied to the
/// live context.
///
/// The one call the canvas makes. Everything downstream branches on the
/// result and nothing downstream reads the space bar for itself.
#[must_use]
pub fn active(ctx: &egui::Context) -> CanvasTool {
    resolve(selected(ctx), space_held(ctx))
}

/// The `egui::Memory` key the mode's capabilities are parked under.
///
/// Salted like every other key in this module, for the reason
/// `super::TOOL_MEMORY_KEY`'s own note gives.
const CAPABILITIES_KEY: &str = "pdfcer.canvas.capabilities"; // ui-text-exempt: memory key, never displayed

/// **Park what this mode may do, so a surface that is not handed it can ask.**
pub fn store_capabilities(ctx: &egui::Context, caps: Capabilities) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(CAPABILITIES_KEY), caps));
}

/// What this mode may do, as last parked by [`store_capabilities`].
#[must_use]
pub fn capabilities(ctx: &egui::Context) -> Capabilities {
    ctx.data(|d| d.get_temp(egui::Id::new(CAPABILITIES_KEY)))
        .unwrap_or(Capabilities::FULL)
}
