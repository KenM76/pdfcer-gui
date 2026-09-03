//! # `canvas::tool::arm` — how a tool is CHOSEN
//!
//! Split out of `canvas/tool.rs` on 2026-08-18, when the fourth tool family
//! (`TextAnnot`) took that file past rule R2's 1,500-line ceiling. `HANDOFF.md`
//! §10 had flagged it as one edit from the wall, and this is that edit.
//!
//! ## The seam is a real one
//!
//! `super` answers *"what IS a tool?"* — the enum, and the predicates that are
//! properties of a variant: which cursor it wants, whether it pans, which kind
//! it carries, which capability it needs. Every one of those is a pure
//! function of the value and none of them touches the world.
//!
//! This file answers *"which tool is chosen, and how does that change?"*. Every
//! function here reads or writes `egui::Memory`, and the interesting content is
//! the **transition rules**: pressing an armed button retires it, pressing Hand
//! from anything takes Hand rather than toggling through Select, a mode change
//! retires a tool the mode may not use.
//!
//! Those two subjects change for different reasons. A new variant is a `super`
//! change; a new rule about what pressing something does is a change here. The
//! test for whether a split was along a seam is whether the reasoning came with
//! it, and it did: every "why does pressing this twice do that?" argument is
//! now in one file.

use super::{CanvasTool, MarkupKind, MeasureKind, TOOL_MEMORY_KEY, TextEditKind};
use crate::app::modes::Capabilities;
use egui::{CursorIcon, Key};

/// **What the pointer looks like this frame** — the whole precedence, in one
/// pure function.
///
/// Lifted out of `canvas::interact` when the markup tool arrived, along the
/// same seam [`crate::canvas::gesture::press_kind`] was: the first rung of this
/// decision was already [`CanvasTool::cursor`], so the remaining three rungs
/// were the rest of one question living in the wiring, where they could not be
/// tested and where a fourth tool would have had to be remembered.
///
/// # The order is the rule
///
/// 1. **The armed tool**, when the pointer is over the canvas or a button is
///    down. This rung is the whole of *"the cursor must change, and must change
///    back"*: it changes because this branch is taken while the tool is active,
///    and it changes back because the answer is recomputed every frame from
///    [`active`] with nothing stored to restore. A dropped key-up costs one
///    frame of hand, not a canvas stuck showing a grab cursor over a select
///    tool.
/// 2. **A gesture in flight**, which keeps its own cursor even once the pointer
///    has wandered off the thing it started on — otherwise a drag that outruns
///    its object looks like it stopped working.
/// 3. **A hovered grip**, which is how the eight resize handles are findable at
///    all.
/// 4. **Nothing**, leaving the cursor to whatever else set it.
///
/// `pointer_down` is *any* button, because a middle-drag pan must show the
/// closed hand too; `over_canvas` is measured against the scroll viewport
/// rather than the page, because the hand pans the grey surround as readily as
/// the paper and a hand tool that shows no hand over half the canvas reads as a
/// tool that is not armed.
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
            // One crosshair for both marquee intents: the band is the same band
            // and `gesture`'s header refuses a second set of pixels for it. What
            // tells the operator a zoom is armed is the ribbon control that
            // armed it, off-canvas, where a mode indicator belongs. A markup
            // band answers the same way, and is stated rather than wildcarded
            // even though rung 1 already claimed it — a drag cannot be in flight
            // without the tool that started it, so this is unreachable today and
            // spelling it keeps the two answers one answer if that changes.
            // The text-annotation band joins them: it is the same crosshair
            // for the same reason, and what tells the operator which tool is
            // armed is the pressed ribbon control, off-canvas.
            // ★ …and the text box joins them, which is the same crosshair for
            // the same reason: it is a rubber-band being dragged out, and what
            // tells the operator it will hold text rather than a comment is the
            // pressed ribbon control, off-canvas, where a mode indicator
            // belongs.
            // ★ …and a form control's band is the fifth, for the reason the
            // four above share: it is a rubber band being dragged out, and what
            // says which of the five kinds is armed is the pressed ribbon
            // control, off-canvas.
            // ★ …and a PLACEMENT band is the sixth (O66), on the same
            // argument: it is a rubber band being dragged out, and what says
            // what it will place is the Tool panel's armed instruction —
            // off-canvas, because the window that would have said so has
            // stepped aside.
            DragKind::Marquee(_)
            | DragKind::Markup(_)
            | DragKind::TextAnnot(_)
            | DragKind::Form(_)
            | DragKind::Place(_)
            | DragKind::TextBox => CursorIcon::Crosshair,
            DragKind::Move => CursorIcon::Grabbing,
            DragKind::Resize(grip) => grip.cursor(),
            // ★ `Grabbing` too, and it names the same limit `Grip::Rotate`'s
            // cursor records: egui 0.35 has no rotate cursor. What it says is
            // *"you are holding something"*, which is true; what it does not say
            // is *"and turning it"*, which `handles.md` H6 asks for. Spelled out
            // rather than folded into the `Move` arm above, so the day a rotate
            // cursor exists there is one line to change and it is findable.
            DragKind::Rotate => CursorIcon::Grabbing,
            // ★ `Grabbing`, the same as a move, and deliberately NOT a bespoke
            // icon. A handle drag IS a move — of a control point rather than of
            // an object — and the operator learns one grammar: the closed hand
            // means "you have hold of something and it follows the pointer".
            // A distinct cursor would be teaching a distinction that changes
            // nothing about what the gesture does.
            DragKind::Handle { .. } | DragKind::DimensionVertex { .. } => CursorIcon::Grabbing,
            // ★ The I-beam for a sweep that began under the MODE rule rather
            // than under an armed tool — and that distinction is now the whole
            // of what this arm is for.
            //
            // ★ **The paragraph that used to stand here has been half
            // discharged, and the discharged half is quoted rather than
            // deleted** because it predicted its own expiry: *"The hover I-beam
            // becomes free on the day a `CanvasTool::Text` lands, because it is
            // then rung 1's answer like every other tool's."* That day is
            // 2026-08-14. With the tool armed, `CanvasTool::Text`'s `cursor`
            // answers `Text` at rung 1, so the pointer is an I-beam from the
            // moment the tool is chosen — on hover, before any drag, over the
            // grey surround as readily as the paper — and it costs one match arm
            // per frame rather than a hit test.
            //
            // The undischarged half stands unchanged and is why this arm
            // survives: in **Read and Review** a press means text with *no tool
            // armed at all* (the select tool, under
            // `textsel::takes_the_press`'s original disjunct), so rung 1 has
            // nothing to answer with there and this rung is the only one that
            // can. Making it hover in those modes would still mean asking "is
            // there a glyph under the pointer?" on every frame the pointer moves
            // — a hit test against the page's extraction, paid on canvases
            // nobody is selecting on, which is most of them. And threading
            // `Capabilities` into this function to synthesise a tool from the
            // mode would put the mode gate in a second place, which is the thing
            // `canvas::textsel`'s header §3 spends its length arguing against.
            //
            // So the shipped rule is: **armed ⇒ I-beam always; un-armed ⇒ I-beam
            // once the sweep starts.** A reader may reasonably ask whether that
            // is an inconsistency an operator would notice, and the answer is
            // that they cannot: the two cases never coexist on one canvas,
            // because arming the tool is what moves a mode from the second to
            // the first.
            DragKind::TextSelect => CursorIcon::Text,
        });
    }
    hovered_grip.map(crate::canvas::handles::Grip::cursor)
}

/// Compose the chosen tool with the space bar — **the rule, and the only
/// place it exists**.
///
/// Space *borrows* the hand; it does not choose it. So this is a `max`, not a
/// swap: holding space over the hand tool changes nothing, and releasing it
/// returns whatever [`selected`] has said all along.
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
///
/// This is what a ribbon toggle or a tool palette should render as pressed:
/// showing the *active* tool there would make the button flicker under the
/// operator's thumb every time they held space.
#[must_use]
pub fn selected(ctx: &egui::Context) -> CanvasTool {
    let id = egui::Id::new(TOOL_MEMORY_KEY);
    ctx.data(|d| d.get_temp::<CanvasTool>(id).unwrap_or_default())
}

/// Choose a tool. **The entry point a `view.tool_hand` / `view.tool_select`
/// command calls.**
pub fn select(ctx: &egui::Context, tool: CanvasTool) {
    let id = egui::Id::new(TOOL_MEMORY_KEY);
    ctx.data_mut(|d| d.insert_temp(id, tool));
}

/// Flip between the hand and the select tool. **The entry point a single
/// `view.tool_hand` *toggle* command calls.**
///
/// Returns the tool now chosen, so a caller that wants to report or check the
/// new state does not have to ask again and risk reading a different frame's
/// answer.
pub fn toggle_hand(ctx: &egui::Context) -> CanvasTool {
    let next = match selected(ctx) {
        CanvasTool::Hand => CanvasTool::Select,
        // Any other tool is *left* by pressing Hand, not toggled through — the
        // operator asked for the hand, and returning them to Select would make
        // one press mean "put the pen down" and a second one mean "pick the
        // hand up". The text tool joins that arm rather than earning its own for
        // the identical reason: pressing Hand while sweeping text means Hand.
        // ★ `Place` joins this arm: pressing Hand while a placement is armed
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
///
/// [`toggle_hand`]'s twin, deliberately down to the shape of the `match`: these
/// are the two pointer tools that carry no kind, they sit in the same ribbon
/// group, and a single press of either is how an operator both enters and leaves
/// it. The same-press-retires rule is [`arm_markup`]'s argument applied to a tool
/// with one kind instead of four — *the button is pressed, so pressing it is how
/// you un-press it* — and without it an operator who armed Text by mistake would
/// have no way back to the select tool except by arming something else.
///
/// # Why it returns to `Select` and not to whatever was armed before
///
/// Because nothing is stored to return to, and that is the same refusal this
/// module's header makes about the space bar: a "previous tool" is state that can
/// be lost, and losing it leaves the canvas in a tool the operator never chose.
/// [`CanvasTool::Select`] is this enum's `#[default]` and the stance every other
/// retirement path in this file returns to ([`disarm_markup`],
/// [`disarm_measure`], [`retire_forbidden`]), so a reader has one answer to learn
/// rather than four.
///
/// Note what that means in a **reading** mode, and it is deliberate rather than a
/// gap: in Read and Review the select tool already sweeps text
/// ([`crate::canvas::textsel::takes_the_press`]'s original rule), so toggling
/// this off there changes the pressed control and changes no behaviour. The tool
/// is not suppressed in those modes for that reason — a control that vanished
/// from View in two of three modes would be a per-mode visibility rule invented
/// to hide a redundancy, and View is shown in every mode precisely so its
/// contents do not have to be.
///
/// Returns the tool now chosen, honouring the same report-rather-than-re-ask
/// contract [`toggle_hand`] and [`arm_markup`] do.
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
///
/// Same shape as [`arm_markup`] and deliberately so: pressing the armed button
/// again retires the tool, which is what makes a mis-click cheap and what stops
/// an operator hunting for a way to cancel.
///
/// ★ The trace line is not decoration. A canvas armed with a form tool and an
/// un-armed one are **the same picture** — a crosshair is a cursor — so this is
/// the only way a driven check can prove the ribbon button armed anything at
/// all. It is the lesson of defect 8, applied to a new tool before the defect
/// can recur.
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
///
/// # ★ Why pressing the armed button again retires the tool
///
/// *"Make it work the way other programs do"* is the operator's stated
/// tie-breaker, and every drawing application treats a tool button as a toggle:
/// the button is **pressed**, so pressing it is how you un-press it. The
/// alternative — a button that only ever arms — leaves an operator who armed
/// Rectangle by mistake with no way back to the select tool except Escape,
/// which they have to know about, or arming some other tool, which is not what
/// they want either.
///
/// Choosing a *different* kind is not a toggle; it is a change of kind, and it
/// arms. So the rule is: same kind ⇒ retire, different kind ⇒ re-arm. That is
/// what makes the four Markup buttons behave as a radio you can switch off,
/// which is what they look like once each renders pressed.
///
/// Returns the tool now chosen, so a caller that wants to report or check the
/// new state does not have to ask again and risk reading a different frame's
/// answer — the same contract [`toggle_hand`] honours.
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
///
/// [`arm_markup`]'s third sibling, with the identical same-kind-retires rule
/// and for the identical reason: a tool button is pressed, so pressing it is
/// how you un-press it.
///
/// The trace line is deliberately the **same event name** `arm_markup` emits.
/// From a harness's point of view — and from the operator's — these are markup
/// tools; that they take a different route to the document is an
/// implementation fact, and a second event name would make a check asking
/// *"did a markup tool arm?"* have to know which family it was about.
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
///
/// [`arm_markup`]'s twin, with the identical same-kind-retires rule and for the
/// identical reason — see that function's header, which is the argument for
/// both. The two are separate functions rather than one generic over the kind
/// because the tools are separate: a shared one would have to take a
/// `CanvasTool` already built, which moves the "which variant" decision back out
/// to the four call sites this pair exists to keep it away from.
///
/// **It arms a tool; it authors nothing.** The clicks are taken by
/// [`crate::canvas::measure`], and only the pick that completes a dimension
/// raises an `Action`.
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
///
/// **Escape's claimant, alongside [`disarm_markup`]**, and it sits at the same
/// rung for the same reason — see [`crate::canvas::keys`]'s precedence table.
///
/// ★ Note what this does **not** do: it does not discard a half-finished pick.
/// A linear dimension with point A taken and point B not is in-progress work
/// held by [`crate::canvas::measure::pick`], and Escape retires *one* thing per
/// press (decision 025's L1). So the first Escape abandons the pick and the
/// second puts the tool down — which is the order the operator means, because
/// the pick is the more transient of the two.
pub fn disarm_measure(ctx: &egui::Context) -> bool {
    if selected(ctx).measure_kind().is_none() {
        return false;
    }
    select(ctx, CanvasTool::Select);
    true
}

/// Retire the markup tool, returning to [`CanvasTool::Select`], and report
/// whether there was one to retire.
///
/// **Escape's claimant.** Reports rather than being asked twice, for the same
/// reason `zoom::disarm_region_zoom` does: the caller cannot know whether the
/// key was spent here without asking, and a caller that re-derived it would be
/// the version that retires the tool *and* ascends a selection rung. See
/// [`crate::canvas::keys`]'s precedence table for where this sits and why.
///
/// Deliberately reads [`selected`] rather than [`active`]: a held space bar
/// borrows the hand, and Escape pressed mid-space must retire the markup tool
/// underneath it rather than doing nothing because the *active* tool happened
/// to be the hand at that instant.
pub fn disarm_markup(ctx: &egui::Context) -> bool {
    if selected(ctx).markup_kind().is_none() {
        return false;
    }
    select(ctx, CanvasTool::Select);
    true
}

/// ★★★ **Put down whatever is armed**, and report whether anything was.
///
/// The operator, 2026-08-20: *"Escape should get me out of a tool."*
///
/// # Why this exists when `disarm_markup` and `disarm_measure` already did
///
/// Because between them they covered **two** of the seven tools. Escape put
/// down a pen and a measure tool and did nothing at all for the caret, the node
/// tool, the text tool or the hand — so the answer to *"how do I stop doing
/// this?"* depended on which tool you had picked, which is not something an
/// operator should have to know.
///
/// The convention is universal and has no exceptions worth carving: **Escape
/// returns you to the pointer.** Every drawing program, every CAD package,
/// every vector editor.
///
/// # It is the LAST rung of the tool group, not the first
///
/// A tool with a gesture in flight spends the first Escape on the gesture and
/// stays armed — an operator correcting a mistyped character must not also be
/// putting the pen down. `canvas::keys`' ladder enforces that ordering and this
/// function is only reached once every in-flight claimant has declined.
///
/// So the sequence an operator experiences is the one they expect from
/// everywhere else: **Escape abandons what you are doing; Escape again puts the
/// tool down; Escape again backs out of the selection.**
///
/// # Why it does not touch a SPACE-held hand
///
/// Because that hand is not armed — `resolve` composes it over the selected
/// tool for as long as the bar is down, and `selected` reports what the
/// operator actually chose. Retiring it here would be retiring something they
/// have not picked, and it would come straight back on the next frame anyway.
pub fn disarm_any(ctx: &egui::Context) -> bool {
    if selected(ctx) == CanvasTool::Select {
        return false;
    }
    select(ctx, CanvasTool::Select);
    // The caret is not part of `CanvasTool`, and a draft that outlived its tool
    // would take keystrokes with nothing to commit them to. `abandon` is a
    // no-op when there is none — and by the time this rung is reached the
    // ladder has already spent an Escape on any draft that WAS in flight, so
    // this cannot swallow one.
    crate::canvas::textedit::abandon(ctx);
    true
}

/// **Retire an armed tool the mode being entered does not permit**, and report
/// whether there was one.
///
/// Called from `PdfcerApp`'s mode-change arm, once, on the frame the operator
/// moves the selector.
///
/// # ★ Why arming has to be undone rather than merely refused
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
        // ★ **Text is on this arm and NOT on the markup arm below, and the
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
        // 2. **The operator settled it for the commands already.** On 2026-08-14
        //    both text-copy verbs moved off the authoring tab under the sentence
        //    *copying is not authoring*. A capability invented here would be that
        //    ruling restated in a second place, free to disagree with it — which
        //    is the same argument `canvas::textsel` §3 makes for why there is no
        //    `select_text` flag.
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
        // ★ **Node is on the OTHER side of the line the paragraph above draws.**
        //
        // It is the first tool in this enum whose whole purpose is to *change*
        // the document — an anchor is selected in order to be dragged — so
        // unlike Select, Hand and Text it must retire when the mode forbids
        // content editing. Leaving it armed in Review would put anchor marks on
        // a page whose every drag is refused, which is the "visible control,
        // silently inert" defect in its most literal form.
        CanvasTool::Node => caps.edit_content,
        // ★ Authoring a form field is a change to the DOCUMENT's content, not
        // an annotation over it — a `/Widget` and its field are page objects
        // the operator is adding. So it answers to `edit_content` and retires
        // when a mode that cannot edit is chosen, exactly like the node tool.
        // Pairing it with `author_markup` would let Review mode place form
        // controls, which is not a review activity.
        CanvasTool::Form(_) => caps.edit_content,
        // ★ A placement answers to the capability of the thing being placed,
        // which each kind states for itself — see `PlaceKind::capability`.
        // Asking here would put the mapping in a second place and let the two
        // disagree about whether Review may drop an image on a drawing.
        CanvasTool::Place(kind) => kind.capability(caps),
        CanvasTool::Markup(_) => caps.author_markup,
        // ★ Gated on `author_markup`, not on a capability of its own.
        //
        // A text box, a sticky and a stamp ARE markup — they are annotations
        // added on top of the page, they appear in the Comments panel beside
        // the geometric kinds, and a mode that may not author a rectangle has
        // no business authoring a callout. Giving them a fourth capability
        // would let the two drift, and there is no mode in `RIBBON_IA.md` that
        // wants one without the other.
        CanvasTool::TextAnnot(_) => caps.author_markup,
        CanvasTool::Measure(_) => caps.author_measure,
        // ★ …and the caret tool joins the *authoring* group, which is the
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
    // ★★★ …and a PENDING PLACEMENT goes with it, or the mode change leaves a
    // hidden dialog with nothing coming back for it — `OPERATOR_REQUESTS.md`
    // O66.
    //
    // The same argument the draft below makes, one step further: a placement is
    // a window that has stepped aside and is waiting. `canvas::placing` makes
    // the window's absence DERIVED from this record precisely so that clearing
    // it here is all "bring the window back" means — there is no second flag to
    // remember. Cheap on every other retirement: one `egui::Memory` read.
    crate::canvas::placing::cancel(ctx);
    // ★ …and the draft goes with the tool. A retirement that left one in
    // `egui::Memory` would leave a keystroke buffer aimed at a document the mode
    // being entered says is not the operator's to change — and it would still be
    // there on the way back, holding text typed against a revision that may have
    // moved. `app::gating`'s rule is *retire what was already there*, and a
    // half-typed word is squarely that.
    crate::canvas::textedit::abandon(ctx);
    true
}

/// Arm the caret tool with `kind`, or retire it if that kind is already armed.
/// **The entry point the `edit.text` and `edit.add_text` dispatch arms call.**
///
/// [`arm_markup`]'s twin, down to the same-press-retires rule and for the
/// identical reason — *the button is pressed, so pressing it is how you un-press
/// it* — and to the discarded return value at the call sites.
///
/// ★ **Changing the kind abandons the draft, and that is not the same as the
/// mid-drag rule above it.** `arm_markup` can be careless about a drag in flight
/// because a drag is owned by the gesture machine and carries the kind it
/// started with, so a kind change cannot reach it. A draft is not owned that
/// way: it sits in `egui::Memory` between frames, and an operator who types
/// three characters into a run and then presses **Add text** has asked for a
/// different verb against a different anchor. Committing it silently would write
/// a half-word; carrying it across would commit `Edit`'s text through `Add`'s
/// engine call. Abandoning is the only reading that is neither, and
/// `textedit::load`'s kind check enforces the same thing from the other end so
/// the two cannot disagree.
pub fn arm_text_edit(ctx: &egui::Context, kind: TextEditKind) -> CanvasTool {
    let next = if selected(ctx) == CanvasTool::TextEdit(kind) {
        CanvasTool::Select
    } else {
        CanvasTool::TextEdit(kind)
    };
    select(ctx, next);
    crate::canvas::textedit::abandon(ctx);
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
/// # ★★★ It asks `textedit::composing`, and asking anything narrower was a
/// # defect the operator hit within a day of text editing working
///
/// This read `!ctx.text_edit_focused()`, which is true for an operator typing
/// into the **canvas caret** — that caret is deliberately not an
/// `egui::TextEdit`, so egui reports no focused text field for somebody who is
/// visibly mid-word. The space bar is this tool's modifier, so the canvas took
/// it and panned the paper. **Text editing could not type a space.**
///
/// > *"I can edit text now, but there is no live preview of that either, and it
/// > doesn't accept spaces. Like how?"* — 2026-08-20
///
/// `app::keyboard` had the right predicate, written out with a paragraph
/// explaining the second claimant, and this call site had a different one. One
/// truth, two copies, one of them wrong — which is why the predicate now exists
/// exactly once and a gate refuses a second.
#[must_use]
pub fn space_held(ctx: &egui::Context) -> bool {
    !crate::canvas::textedit::composing(ctx) && ctx.input(|i| i.key_down(Key::Space))
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
/// [`SELECTED_KEY`]'s own note gives.
const CAPABILITIES_KEY: &str = "pdfcer.canvas.capabilities"; // ui-text-exempt: memory key, never displayed

/// **Park what this mode may do, so a surface that is not handed it can ask.**
///
/// # ★ Why this exists at all, when `Capabilities` is already threaded
///
/// It is threaded to everything that *gates a gesture* — `retire_forbidden`,
/// `takes_the_press`, `press_kind` — because those are called from the canvas
/// pass, which has `PdfcerApp` in scope. A **dock panel** does not: `Panel::show`
/// is handed `(ui, Option<&OpenDoc>, &mut PanelsState, Option<&MenuHost>,
/// &mut Vec<Action>)` and nothing else, which is the seam that keeps a panel
/// from reaching into the application.
///
/// `crate::panels::tool` has to answer *"what does a press mean in this
/// mode"* and *"which tools does this mode have"*, and both are this value.
/// The alternatives were worse in specific ways:
///
/// * **widen `Panel::show`** — a sixth parameter every panel takes and one
///   panel reads;
/// * **re-derive from the ribbon's active mode inside the panel** — a second
///   copy of `Capabilities::for_mode`, which would eventually disagree with the
///   canvas about what a mode may do, and the symptom would be a panel that
///   lies rather than a panel that crashes.
///
/// Parking it beside the armed tool is the smallest thing that works and it
/// keeps **one** derivation: `PdfcerApp::capabilities()` computes it,
/// `on_mode_capabilities_changed` stores it, everything else reads it.
pub fn store_capabilities(ctx: &egui::Context, caps: Capabilities) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(CAPABILITIES_KEY), caps));
}

/// What this mode may do, as last parked by [`store_capabilities`].
///
/// ★ Falls back to [`Capabilities::FULL`], and the fallback is the same
/// decision `Capabilities::for_mode` makes for an unknown mode, for the same
/// reason recorded there: a build with no validated manifest has no mode
/// taxonomy, and a shell that silently withheld every capability would be a
/// broken product rather than a safe one. Here it is also the honest answer for
/// the first frame, before any mode change has occurred — the application
/// starts in the manifest's first mode with its capabilities already applied by
/// `modes::start`, and the panel simply has not been told yet.
#[must_use]
pub fn capabilities(ctx: &egui::Context) -> Capabilities {
    ctx.data(|d| d.get_temp(egui::Id::new(CAPABILITIES_KEY)))
        .unwrap_or(Capabilities::FULL)
}
