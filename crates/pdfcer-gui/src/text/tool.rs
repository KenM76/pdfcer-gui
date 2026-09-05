//! # `text::tool` — the words the tools say, wherever they are said
//!
//! ## ★★★ This file OUTLIVED the panel it was written for
//!
//! It was `every word the Tool panel says`. `OPERATOR_REQUESTS.md` **O123**
//! dissolved that panel on 2026-09-04, and the copy went three ways rather than
//! away:
//!
//! | what | who says it now |
//! |---|---|
//! | the per-tool instructions and live stages | [`crate::app::toolstatus`] — the right dock's permanent one-line strip |
//! | the second sentence of the stages that had one | the same strip, in its hover |
//! | the text pen's labels, the measure pick list, the resize switches | [`crate::panels::properties::tool`] |
//! | the disclosure heading | [`crate::panels::properties::disclose`] |
//!
//! ⚠ **What was DELETED, and it is the only deletion**: the fifteen strings the
//! panel's tool LIST used — `tools_heading`, `tools_hint`, `row_home` and the
//! nine `row_*` sentences, plus `pointer_heading`, `armed_heading` and
//! `no_document`. Every one of them labelled a button that duplicated a ribbon
//! control, and the operator's instruction was *"its buttons duplicate the
//! ribbon and go."* They are gone rather than left orphaned, because an unused
//! catalog entry is a sentence nobody can find and nobody can retire.
//!
//! ★ Worth naming what that cost: those rows were the answer to a
//! discoverability defect — *"The feature works. He could not find it."* The
//! strip that replaced them cannot list what is NOT armed. That is a real
//! subtraction and it is the operator's own call; it is recorded in
//! `crate::app::toolstatus`'s header rather than argued here.
//!
//! ## ★ The three rules the whole file follows, unchanged
//!
//! **1. No label is written here that the command registry already owns.**
//! The armed tool's name comes from `CommandRegistry` through
//! `crate::shell::menus::MenuHost::label`, and the chord comes from the
//! operator's own keymap. A second copy of a label compiles, reads identically
//! the day it is written, and drifts the first time either is reworded —
//! invisibly, because nothing renders both at once. `NO_SURFACE.md` §1 records
//! that exact failure with a colour.
//!
//! **2. Every sentence states a fact about the program, never a tip.** The
//! operator's own report about the shell this replaces: *"the nagging and red
//! flagging in the original GUI made for a lot of extra bugs in the visibility
//! when editing."* *"Drag marquees objects on this page"* is a statement.
//! *"Try dragging to select several objects!"* is a tip, and there are none
//! here.
//!
//! **3. An instruction says how the gesture ENDS.** Half the gestures in this
//! application do not end by themselves — a run of clicks does not, a text
//! caret does not — and *"click each corner"* is not a complete instruction
//! because nothing in it says when to stop. Every instruction below that
//! describes an open-ended gesture names its ending.
use crate::canvas::markup::MarkupKind;
use crate::canvas::measure::MeasureKind;
use crate::canvas::textannot::TextAnnotKind;
use crate::canvas::textedit::TextEditKind;

// ===========================================================================
// What the pointer does right now — the resting tool's sentence
// ===========================================================================

/// What a press means in a mode that can select page content — Edit.
///
/// ★ **This sentence exists nowhere else in the application**, which is the
/// whole reason the unarmed panel is not a placeholder. The identical drag
/// means *marquee objects* here and *sweep text* in Read, decided by
/// `canvas::textsel::takes_the_press` reading the mode — and no surface has
/// ever said so. An operator who wonders why dragging behaves differently in
/// two modes has had no way to find out but to guess.
#[must_use]
pub const fn pointer_edit() -> &'static str {
    "Drag marquees objects on the page; click selects one. Hold Space to move \
     the paper."
}

/// What a press means in a mode that cannot select page content — Read and
/// Review.
#[must_use]
pub const fn pointer_reading() -> &'static str {
    "Drag selects text on the page; click puts the cursor in it. Hold Space to \
     move the paper."
}

// ===========================================================================
// Block B — the tools this mode has
// ===========================================================================

// ===========================================================================
// The armed frame
// ===========================================================================

/// The put-the-tool-down button.
///
/// ★ Named for what it does to the **tool**, never "Close" — the dock tab's ✕
/// closes the panel and this does not, and two controls a click apart that both
/// read as closing something is how an operator loses a surface they wanted.
#[must_use]
pub const fn put_down_button() -> &'static str {
    "Put this tool down"
}

/// The hint on that button, naming the key that does the same thing.
#[must_use]
pub const fn put_down_hint() -> &'static str {
    "Esc does the same."
}

/// What the **Node tool** does, in the Tool panel's live stage.
///
/// ★ Written as the two gestures in the order an operator performs them, and
/// naming the *thing* rather than the rung. "Anchor" is what a draughtsman
/// calls the point; `SelectionLevel::Node` is what this program calls the
/// state, and the panel speaks the first vocabulary — `text::commands`' rule
/// that a label is the operator's word and an id is the format's.
///
/// ## ★★★ The second sentence, added 2026-09-05, and why it is unconditional
///
/// This tool has **two** subjects, not one — see
/// `canvas::tool::retire_forbidden`'s Node arm for the table. The first is an
/// anchor of a path on the page and needs `edit_content`; the second is a
/// corner of a measurement pdfcer authored, needs `author_measure`, and is
/// therefore the tool's only subject in Review.
///
/// The sentence names both in every mode rather than being swapped by
/// capability, and that is a decision with a cost attached. The precedent for
/// swapping is [`text_select_takes_the_press`], which is rendered only where it
/// is true — but the fact *that* sentence states is a **change** ("arming this
/// takes the press away from…"), which is either true or false. This one states
/// **where to aim**, and an operator in Edit who is shown only the anchor
/// sentence would never learn that his measurements have corners at all. R9
/// governs drawing a control that does nothing; naming a second subject that
/// exists in the mode the operator is in is the opposite of that.
///
/// ⇒ The wording therefore says *a measurement you have drawn*, which is true
/// in both modes and locates the subject without a mode word in it.
#[must_use]
pub const fn node_instruction() -> &'static str {
    "Click a shape to show its points. Click a point to select it, then drag to \
     move it. On a measurement you have drawn, drag a corner to reshape it."
}

/// The line under it: how to take more than one, and how to change how many
/// corners a measurement has.
///
/// ## ★★★ This is where the two chords are spelled, and it is the ONLY place
///
/// `canvas::dimdrag` gates adding and removing a corner on this tool being
/// armed — deliberately, so that a stray Ctrl during an ordinary corner drag
/// cannot destroy a corner. The cost of that safety is that the gesture is
/// invisible to anyone who has not armed the tool, and the payment is this
/// line: arm the tool, and the sentence under it tells you what the modifiers
/// do.
///
/// ★★ It is a **stopgap and is recorded as one.** The discoverable form of
/// these two verbs is a right-click menu on the shape — *"add a point here"*,
/// *"remove this point"* — which is the shape `pdfcer-core`'s own doc comments
/// describe them in (`edit.rs:37927`, `edit.rs:37955`). That surface is
/// `canvas::menus`, and the session that built these two verbs did not own that
/// file. Reported rather than half-built; see `OPERATOR_REQUESTS.md` O132.
#[must_use]
pub const fn node_shift() -> &'static str {
    "Shift-click to add more points. A point on a curve also shows its handles. \
     On a measurement: Ctrl-drag a corner to add one after it, Ctrl+Shift-drag \
     to take it away."
}

/// The Hand tool's instruction.
#[must_use]
pub const fn hand_instruction() -> &'static str {
    "Drag to move the paper. Nothing on the page changes."
}

/// The Hand tool's second line — the borrow every other tool can do.
#[must_use]
pub const fn hand_borrow() -> &'static str {
    "Holding Space borrows this from any other tool, so you rarely need to arm it."
}

/// The text-sweep tool's instruction.
#[must_use]
pub const fn text_select_instruction() -> &'static str {
    "Drag across the words you want. Click once to put the cursor in a word."
}

/// ★ What arming the sweep takes away, in the mode where it takes something.
///
/// Rendered only in a mode that can select page content. Everywhere else the
/// select tool already swept text, so there is nothing to disclose and the line
/// is **absent** rather than reworded — R9's rule applied to a sentence.
#[must_use]
pub const fn text_select_takes_the_press() -> &'static str {
    "While this is armed a drag selects text instead of marqueeing objects."
}

/// One markup kind's instruction. The gesture, and how it ends.
///
/// ★ These came from `MarkupKind`'s own variant doc comments, where they had
/// been written — correctly, and in the operator's words — since the day each
/// kind landed, with no surface able to render them. Moving them here is what
/// `check-ui-strings.sh` requires and is also what makes them reachable.
#[must_use]
pub const fn markup_instruction(kind: MarkupKind) -> &'static str {
    match kind {
        MarkupKind::Rectangle => "Drag from one corner to the other.",
        MarkupKind::Ellipse => "Drag out the box it fits inside.",
        MarkupKind::Arrow => "Drag from the tail to the head.",
        MarkupKind::PolyLine | MarkupKind::Polygon | MarkupKind::Cloud => {
            "Click each corner in turn, and double-click the last one."
        }
        MarkupKind::Ink => "Press and draw. Let go when you are done.",
        MarkupKind::Highlight => "Drag across what you want marked.",
    }
}

/// How many corners are down, and what ends the run.
///
/// ★ **The number is why this panel exists rather than a canvas readout.** A
/// rubber band and a snap indicator are the cursor and are welcome; a *number*
/// floated near the pointer would be pdfcer putting a surface over the drawing
/// on its own initiative, which `MODES_AND_PANELS.md` sets to **never**. So the
/// count has exactly one legal home, and it is a real need: a polygon and a
/// revision cloud both refuse at two corners, and an operator who double-clicks
/// one click early gets silence.
#[must_use]
pub fn vertices_placed(n: usize) -> String {
    match n {
        0 => "No corners placed yet.".to_owned(),
        1 => "1 corner placed. Double-click the last one to finish.".to_owned(),
        _ => format!("{n} corners placed. Double-click the last one to finish."),
    }
}

/// One text-annotation kind's instruction.
#[must_use]
pub const fn text_annot_instruction(kind: TextAnnotKind) -> &'static str {
    match kind {
        TextAnnotKind::TextBox => "Drag out the box, then type into it.",
        TextAnnotKind::Sticky => "Click where the note should sit, then type into it.",
        TextAnnotKind::Stamp => "Drag out the area the stamp should cover.",
    }
}

/// ★ What a release does for a text-bearing annotation, which is NOT what it
/// does for a shape.
///
/// The distinction `CanvasTool` was split for: *"A markup band authors on
/// release, from geometry alone. These cannot: releasing produces an empty box,
/// and an empty box is not an annotation."* An operator who does not know that
/// reads a release-that-authors-nothing as a broken tool — which is the same
/// failure shape as the text-editing complaint that produced this panel.
#[must_use]
pub const fn text_annot_release() -> &'static str {
    "Nothing is added to the page until you accept what you have typed."
}

/// Edit-text's instruction, before there is a caret.
#[must_use]
pub const fn text_edit_instruction(kind: TextEditKind) -> &'static str {
    match kind {
        TextEditKind::Edit => "Click a word already on the page to put the cursor in it.",
        TextEditKind::Add => "Click an empty spot to start typing new text there.",
    }
}

/// Edit-text's instruction while a caret is live.
#[must_use]
pub const fn text_edit_live() -> &'static str {
    "Enter commits what you have typed. Esc abandons it."
}

/// ★★ The heading over the refusal, when a click was declined.
///
/// The refusal sentences themselves are `crate::text::textedit::refusal`'s and
/// are **not** duplicated here. They were written well, are tested, and have
/// never had a surface wide enough to show them: their own module records that
/// they were aimed at the status bar, and that *"it shares the status row with
/// everything else and R128 forbids that row growing."* A dock panel's width is
/// the dock's, decided before the body draws, so that constraint does not apply
/// here at all.
///
/// This is very likely the actual cause of *"no text editing or adding text on
/// the canvas"*: on a dense CAD sheet the first click lands where the operator
/// wants text rather than where text is, the tool declines with an explanation
/// nobody could read, and they conclude the feature does not exist.
#[must_use]
pub const fn refusal_heading() -> &'static str {
    "That click was declined"
}

/// The perimeter tool's LIVE sentence - vertices so far, and the running total
/// in the authoring group's own units.
///
/// # Why the count is in it as well as the length
///
/// Because the two answer different worries. The length says *"this is what I
/// have measured"*; the count says *"this is how much of the shape I have
/// traced"*, which is the one an operator loses track of on a footprint with
/// twenty corners - and the one that tells them whether a click registered at
/// all. A tool with no fixed arity has nothing else on screen that says so.
///
/// The length is formatted by the caller through the engine's own
/// `format_measurement`, so this function never sees a number it could round
/// differently from the committed label.
///
/// A verb rather than a bare pair of numbers: this replaces the instruction
/// once tracing starts, and a line reading only "4 - 12.40 m" gives an operator
/// who has looked away nothing to reattach to.
#[must_use]
pub fn measure_perimeter_live(vertices: usize, length: &str) -> String {
    format!("{vertices} points so far, {length} around. Click the first point to close it.")
}

/// The radius/diameter tool's LIVE sentence — how many points are in the fit,
/// and what circle they currently make.
///
/// # ★★ Why the SIZE is in it, and why that is the whole ask
///
/// `OPERATOR_REQUESTS.md` O105: *"selecting more points around a hole doesn't
/// always get it to narrow down to the size of the hole."* An operator adding
/// points to a fit is watching a number converge, and until 2026-09-03 there
/// was no number to watch — the fitted circle was drawn on the canvas and its
/// value appeared only once the dimension had been placed. So the tool could
/// not be steered: every correction was a commit-and-undo.
///
/// The count is in it for the reason it is in the perimeter's sentence — it is
/// the only thing on screen that says *whether the last click registered at
/// all*, which for a tool with no fixed arity nothing else answers.
///
/// The measurement is formatted by the caller through the engine's own
/// `format_measurement`, so this function never sees a number it could round
/// differently from the committed label.
#[must_use]
pub fn measure_circular_live(points: usize, measurement: &str) -> String {
    format!("{points} points, {measurement}. Add more, or finish it.")
}

/// The radius/diameter tool's sentence while the fit is still degenerate.
///
/// ★ It states the count AND what is missing, because "nothing yet" is exactly
/// the report that sent the operator looking for a broken tool. Two points on
/// an arc is not a failure, it is halfway.
#[must_use]
pub fn measure_circular_needs_more(points: usize) -> String {
    match points {
        0 => "No points yet. Click around the arc — three or more.".to_owned(),
        1 => "1 point. Two more at least, spread around the arc.".to_owned(),
        n => format!("{n} points, and no circle through them yet — spread them around the arc."),
    }
}

/// The heading over the list of points in the circular fit.
#[must_use]
pub const fn measure_points_heading() -> &'static str {
    "Points in this measurement"
}

/// One row in that list: its position in the set, where it came from, and where
/// it is.
///
/// # ★★ Why the ORIGIN is on the row
///
/// Because a point snapped to the drawing's own geometry and a point the
/// operator placed by eye on a scanned image produce the same numbers and are
/// **not** the same evidence. `OPERATOR_REQUESTS.md` O106 asks for the second
/// deliberately — it is what makes a bitmap measurable — and the honest
/// consequence is that the operator can see which of their points are which.
///
/// ★★★ That disclosure is here and **not on the canvas**, and the placement is
/// the rule rather than a preference. Rule 4: applied content renders exactly
/// as saved content will, and a tint or a dashed marker saying *"this one is a
/// guess"* would be pdfcer marking its own uncertainty into the page view. The
/// list is off-canvas, non-blocking, and positioned relative to nothing in the
/// document — which is where a disclosure belongs.
///
/// The coordinates are page units to one decimal. Not the group's scale: these
/// are *positions*, not a measurement, and running them through
/// `format_measurement` would print a length unit beside something that is not
/// a length.
#[must_use]
pub fn measure_point_row(ordinal: usize, origin: &str, x: f64, y: f64) -> String {
    format!("{ordinal}. {origin} — {x:.1}, {y:.1}")
}

/// What a point's row does when it is clicked.
///
/// ★ A row that removes on a single click needs to say so before it is pressed,
/// because the gesture is not recoverable through undo — a pick set is
/// pre-commit state and never enters the document's history. One click to put
/// it back is the whole cost, and the tooltip says which click.
#[must_use]
pub const fn measure_point_remove_hint() -> &'static str {
    "Click to take this point out of the measurement"
}

/// The line drawn in place of the list when nothing has been picked.
///
/// ★ Not a placeholder row and not a greyed one: R9 reserves greying for a
/// *temporarily* unavailable control, and an empty set is not that. This is a
/// sentence, and it is the only thing this section renders until there is
/// something to list.
#[must_use]
pub const fn measure_points_empty() -> &'static str {
    "Nothing picked yet."
}

/// The operator-facing name for where a picked point came from.
///
/// # ★ Why this is a separate vocabulary from the trace's
///
/// `canvas::measure::circular::origin_tag` produces short machine tags that a
/// driven check matches on. Those are a contract with `tools/ui-verify` and must
/// not move when a word is reworded here; these are what the operator reads and
/// must be free to. One function serving both would tie a harness assertion to
/// a translatable string.
///
/// **Free position** is worded as a statement of fact rather than as a warning.
/// It is a legitimate and often the only available pick — see
/// `OPERATOR_REQUESTS.md` O106 — and language like *"unsnapped"* or
/// *"approximate"* would be pdfcer editorialising about a choice the operator
/// made deliberately.
#[must_use]
pub const fn measure_point_origin(
    origin: crate::canvas::measure::pick::PickOrigin,
) -> &'static str {
    use pdfcer_core::vector::snap::SnapKind;

    use crate::canvas::measure::pick::PickOrigin;
    match origin {
        PickOrigin::Free => "Free position",
        PickOrigin::Snapped(kind) => match kind {
            SnapKind::Node => "Node",
            SnapKind::Endpoint => "Endpoint",
            SnapKind::Center => "Centre",
            SnapKind::Midpoint => "Midpoint",
            SnapKind::Intersection => "Intersection",
            SnapKind::SegmentCenterline => "On a line",
            SnapKind::DerivedCenterline => "Centreline",
            SnapKind::Axis => "Axis",
        },
    }
}

/// One measure kind's instruction, before any pick.
#[must_use]
pub const fn measure_instruction(kind: MeasureKind) -> &'static str {
    match kind {
        MeasureKind::Linear => {
            "Click the first point, then the second, then where the \
                                dimension line should sit."
        }
        MeasureKind::Circular => "Click three or more points around the arc, then finish it.",
        // ★ All three endings, in one sentence, in the order an operator meets
        // them. A tool with three ways to stop needs to say so before the first
        // click - discovering the closing convention by accident works, and
        // discovering it AFTER tracing thirty vertices the wrong way does not.
        MeasureKind::Perimeter => {
            "Click around the shape. Click the first point again to close it, or double-click to finish an open path."
        }
        // ★ Two endings, not three - and the sentence says so, because the
        // difference between this tool and Perimeter IS the missing ending.
        MeasureKind::PathLength => {
            "Click along what you are measuring. Double-click the last point to finish."
        }
        MeasureKind::TwoLine => "Click one line, then the other.",
        // ★ The calibration pick, which is armed from inside the Set-scale
        // window rather than from the Measure tab — it is deliberately absent
        // from `MeasureKind::ALL` for that reason.
        //
        // It gets its own sentence rather than borrowing Linear's, even though
        // it reuses `LinearPick` verbatim, because the two picks mean opposite
        // things: Linear AUTHORS a ce dimension onto the page and this one
        // authors nothing at all — it measures a length the operator is about
        // to tell pdfcer the real-world value of. An operator who read
        // "then where the dimension line should sit" would wait for a third
        // click that never comes.
        MeasureKind::Scale => "Click each end of something whose real length you know.",
    }
}

/// The label over the group the next dimension will join.
///
/// ★ **Read-only here, and the button beside it is a route rather than a
/// picker.** A second group picker would be two copies of the one control that
/// decides where every ce dimension goes, which is precisely the duplication
/// this project has already been bitten by. The panel that owns it is one click
/// away and is the only place it can be changed.
#[must_use]
pub const fn draw_into_label() -> &'static str {
    "Drawing into"
}

/// The button that opens the panel which owns the group picker.
#[must_use]
pub const fn manage_groups_button() -> &'static str {
    "Groups…"
}

// ===========================================================================
// The Select tool — what rides along with a resize
// ===========================================================================

/// The heading over the Select tool's three scale switches.
///
/// ★★ *"When you resize something"*, not *"Scaling"* or *"Transform options"*.
/// It names the **gesture** these modify, because that is how the operator will
/// arrive: they have just dragged a grip and something did or did not come with
/// it. A noun heading would be correct and would not connect to anything they
/// did.
#[must_use]
pub const fn scale_heading() -> &'static str {
    "When you resize something"
}

/// The stroke-width switch.
///
/// ★★★ **His own vocabulary.** `OPERATOR_REQUESTS.md` O51 says *"scaling line
/// weight, etc with resize"* — *line weight*, which is the drafting term and
/// the one on every CAD program's layer table. The PDF calls it a border width
/// and Inkscape calls it a stroke width; neither is what he said.
#[must_use]
pub const fn scale_stroke_label() -> &'static str {
    "Scale line weight"
}

/// The `/RD` switch.
///
/// ★★★ **Phrased as KEEPING, matching the field it sets.** `/RD` scales by
/// default, so the switch is an opt-out, and `canvas::scaling` spells it that
/// way deliberately so `Default::default()` is correct in every field.
///
/// ⇒ A label reading *"Scale the inner margins"* would read better and would
/// put an inversion between the words and the value. That is the single easiest
/// way to ship a control that does the opposite of what it says, and no test
/// catches it — both states are legal and both produce a plausible picture.
///
/// ★ *"inner margins"* rather than *"rect differences"*: `/RD` is the gap
/// between an annotation's rectangle and the drawing inside it, which is a
/// margin, and nobody outside the specification says *rect difference*.
#[must_use]
pub const fn scale_insets_label() -> &'static str {
    "Keep the inner margins the same size"
}

/// The distortion escape.
///
/// ★★★ **It says the result will be uneven, in the label itself.** This is the
/// one switch whose ON state makes the output worse, and O51's ruling on it is
/// explicit: proceed and **state** the residual distortion, *"never silently
/// pick a fudge factor, which is the one thing the parity reference does."*
/// A label reading *"Allow non-uniform resize"* would be true, neutral, and
/// would hide the cost inside a word the operator does not have to decode.
#[must_use]
pub const fn scale_distort_label() -> &'static str {
    "Allow the artwork to distort (borders may come out uneven)"
}

/// The note under the three switches.
///
/// ★★ It states the default in the operator's terms and names **why** the line
/// weight stays put — *a drafting standard* — because on his documents that is
/// not a preference, it is a convention his drawings are read against. Without
/// the reason, "off by default" reads as an arbitrary choice somebody made.
///
/// ★ It also says the switches apply to the **next** resize, which is the fact
/// that makes them a per-drag modifier rather than a setting, and the fact an
/// operator needs in order to use them at all: tick, then drag.
#[must_use]
pub const fn scale_note() -> &'static str {
    "These apply to the next resize. Line weight stays put by default, because on a drawing it is a drafting standard rather than decoration — the same default Acrobat, Illustrator and Inkscape all ship."
}

// ===========================================================================
// The text pen — what NEW page text is written in
// ===========================================================================

/// The heading over the Add-text options.
///
/// ★ Says **new text**, not "text", and the distinction is the whole reason
/// these controls are in the Tool panel rather than on the Format tab: they
/// decide what the *next* thing typed looks like, not what a run already on the
/// page looks like. An operator who reads "Text" here and expects it to restyle
/// the word they clicked has been misled by one word.
#[must_use]
pub const fn text_pen_heading() -> &'static str {
    "New text"
}

/// The font combo's label.
#[must_use]
pub const fn text_pen_font_label() -> &'static str {
    "Font"
}

/// One bundled face's name, as an operator would say it.
///
/// ★ *"Helvetica Bold"*, not `HelveticaBold` — the engine's identifier is a
/// Rust variant and this is a font menu. The four Courier faces say
/// *"Courier Oblique"* rather than *"Courier Italic"*, because oblique is what
/// the Standard-14 set actually contains and a menu that renamed it would
/// promise a true italic pdfcer cannot write.
#[must_use]
pub const fn text_pen_font_name(face: pdfcer_core::fontdata::Std14) -> &'static str {
    use pdfcer_core::fontdata::Std14 as F;
    match face {
        F::Helvetica => "Helvetica",
        F::HelveticaBold => "Helvetica Bold",
        F::HelveticaOblique => "Helvetica Oblique",
        F::HelveticaBoldOblique => "Helvetica Bold Oblique",
        F::TimesRoman => "Times Roman",
        F::TimesBold => "Times Bold",
        F::TimesItalic => "Times Italic",
        F::TimesBoldItalic => "Times Bold Italic",
        F::Courier => "Courier",
        F::CourierBold => "Courier Bold",
        F::CourierOblique => "Courier Oblique",
        F::CourierBoldOblique => "Courier Bold Oblique",
        F::Symbol => "Symbol",
        F::ZapfDingbats => "Zapf Dingbats",
        // ★ NO wildcard, and its absence is deliberate. `Std14` is not
        // `#[non_exhaustive]` — checked, rather than assumed from its
        // neighbours in that module, several of which are — so this match is
        // exhaustive by the compiler's own count and a fifteenth face would be
        // a build error here rather than a combo entry reading "Another
        // bundled face". That is the stronger arrangement and it is available,
        // so it is taken.
    }
}

/// The size control's label.
#[must_use]
pub const fn text_pen_size_label() -> &'static str {
    "Size"
}

/// The suffix on the size control.
#[must_use]
pub const fn text_pen_size_suffix() -> &'static str {
    " pt"
}

/// The colour swatch's label.
#[must_use]
pub const fn text_pen_colour_label() -> &'static str {
    "Colour"
}

/// ★ The sentence under the three controls.
///
/// It says what they DO NOT do, because that is the thing an operator will
/// otherwise assume: these set the next run's appearance and change nothing
/// already on the page. `Edit text` beside them replaces the words in a run and
/// keeps its existing face — pdfcer cannot restyle a placed run at all yet, and
/// a control group that stayed silent about that would be read as offering it.
#[must_use]
pub const fn text_pen_note() -> &'static str {
    "These apply to the next text you add. They do not change text already on \
     the page — pdfcer cannot restyle a run it did not write."
}

// ===========================================================================
// Block C — what pdfcer last inferred
// ===========================================================================

/// The heading over the disclosures block.
///
/// ★ Rendered only when there is something under it. R9: an unavailable
/// capability renders **nothing**, and a heading over an empty region is the
/// placeholder that rule exists to forbid.
#[must_use]
pub const fn disclosures_heading() -> &'static str {
    "What pdfcer worked out"
}

// ===========================================================================
// The empty case
// ===========================================================================

/// What the Tool panel says while a form-field tool is armed.
///
/// ★★★ It names BOTH gestures, and that is the point of the line. The whole
/// feature is *"click to place, or drag for the exact size"*, and an operator
/// who clicks once, gets a standard-sized box and is never told about dragging
/// will conclude that sizing is not offered. A panel that teaches only the
/// gesture just performed teaches half the tool.
#[must_use]
pub const fn form_instruction() -> &'static str {
    "Click the page to place one at a standard size, or drag out the exact size you want."
}

/// The second line: what happens next, and what this kind needs.
///
/// ★ Radio buttons get their own sentence because they are the only kind whose
/// behaviour depends on ANOTHER field — two sharing a group name are one
/// control. An operator who does not know that places two buttons that both
/// stay on and reasonably reports it as a bug.
#[must_use]
pub const fn form_kind_hint(kind: crate::canvas::formfield::FormFieldKind) -> &'static str {
    use crate::canvas::formfield::FormFieldKind as K;
    match kind {
        K::Radio => {
            "Nothing is added until you fill in the box that appears. Give buttons the same group name to make them alternatives."
        }
        _ => "Nothing is added until you fill in the box that appears. Escape cancels.",
    }
}

/// **The Points tool was asked for in a mode that cannot change page content.**
///
/// `OPERATOR_REQUESTS.md` row **O69**, the operator:
///
/// > *"I'm still not entirely clear how to reliably get to a point where I can
/// > edit nodes."*
///
/// One of the two reasons it felt unreliable. The Points tool needs
/// `Capabilities::edit_content`, which only Edit has, and its dispatch arm
/// declined into the trace and said nothing on screen. The ribbon item is now
/// withheld outside Edit (`shell::manifest::view`), so the only way left to
/// reach the decline is the bare `A` chord — a chord is filtered by TAB
/// visibility, not by item visibility, and View is in every mode.
///
/// ★ **So this sentence exists for a route the ribbon can no longer produce**,
/// and that is deliberate rather than belt-and-braces. R83's rule is *a
/// refusal must be a sentence, never a silence*, and a chord that does nothing
/// is the worst kind of silence: there is no control to look at, so there is
/// nowhere for the operator to discover why.
///
/// # Why it names the remedy rather than the rule
///
/// *"Editing points needs Edit mode"* would be a true statement of the rule
/// and useless at the moment it is read — the operator pressed a key and
/// nothing happened, and what they need is the next act, not a diagnosis.
/// This names the control that fixes it, in the words printed on it, which is
/// the same choice `resize_not_rebuildable` makes and for the same reason.
///
/// # Why it is here and not in `crate::text::status`
///
/// `text::status` is at 1,482 lines against R2's 1,500, and its own header
/// records the seam being noticed rather than trimmed. This module already
/// owns the tool panel's sentences, and this sentence is about a tool.
///
/// # ★★★ WHERE THIS NOW FIRES, corrected 2026-09-05 — and it had become a
/// misdirection
///
/// The gate above it changed. `retire_forbidden`'s Node arm and
/// `app::dispatch::navigate`'s `view.tool_node` arm both read
/// `edit_content || author_measure` now, because the tool's second subject is a
/// ce dimension's corners and reshaping one is a **measure** edit. So this
/// sentence is reachable in **Read alone**, where nothing is editable, and
/// naming Edit is the correct next act.
///
/// ⚠ It was **actively wrong in Review**, which is the state that made the
/// correction urgent rather than tidy. Review is the mode a measurement is
/// drawn in; an operator there who pressed `A` was told to switch to Edit —
/// and switching would not have helped him, because what he was reaching for
/// was either a corner he already had (his own measurement, draggable in
/// Review all along) or a markup shape's points, which **Edit cannot edit
/// either**: `pdfcer-core` models no `/Vertices` or `/InkList`, filed as
/// `request_a_markup_shapes_vertices_cannot_be_read_or_edited.md`. A refusal
/// that names a remedy which does not work is worse than one that names none,
/// because it spends the operator's time before it fails.
#[must_use]
pub fn node_tool_needs_edit_mode() -> &'static str {
    "Switch to Edit to work on points."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every markup kind has an instruction, and every instruction says how the
    /// gesture ends.
    ///
    /// ★ The second half is the assertion worth having. *"Click each corner"*
    /// is not a complete instruction — nothing in it says when to stop — and
    /// the failure it produces is an operator clicking forever, which is
    /// exactly what the two endings exist to prevent. Asserted as a property
    /// (the sentence names a release, a double-click or a stop) rather than
    /// against the literals, which would pass just as well if every kind
    /// returned the same string.
    #[test]
    fn every_markup_instruction_says_how_the_gesture_ends() {
        for kind in MarkupKind::ALL.iter().copied() {
            let s = markup_instruction(kind);
            assert!(!s.is_empty(), "{kind:?} has no instruction");
            let ends = s.contains("double-click")
                || s.contains("Let go")
                || s.contains("Drag from")
                || s.contains("Drag out")
                || s.contains("Drag across");
            assert!(
                ends,
                "{kind:?}'s instruction {s:?} never says how the gesture ends, so an \
                 operator following it has no way to know when to stop"
            );
        }
    }

    /// The corner count reads as English at one and at many.
    #[test]
    fn the_corner_count_reads_as_english() {
        assert!(vertices_placed(1).starts_with("1 corner placed"));
        assert!(vertices_placed(3).starts_with("3 corners placed"));
        assert!(!vertices_placed(0).contains('0'));
    }
}
