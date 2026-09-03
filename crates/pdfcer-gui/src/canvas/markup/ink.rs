//! # `canvas::markup::ink` — freehand, and the hundreds of points nobody asked
//! for
//!
//! `/Ink` (§12.5.6.12): **press, follow the pointer, release.** Drag-shaped, so
//! it fits the existing `DragKind` path with no new gesture — and *not*
//! band-shaped, because the thing being authored is the whole path the pointer
//! took and a band is two points. That difference is the whole of this module:
//! the trail, its lifetime, its simplification, and a preview that draws the
//! simplified trail rather than the raw one.
//!
//! ---
//!
//! ## 1. One drag is one annotation, and one annotation is one stroke
//!
//! `/InkList` is a list *of* strokes, so the format permits several drags to
//! accumulate into a single ink comment. This shell authors **one stroke per
//! drag, one annotation per drag**, and the reference applications decide it:
//!
//! | | freehand tool | what a second drag does |
//! |---|---|---|
//! | **Inkscape** | the pencil | a second, separate path object |
//! | **Acrobat** | Comment ▸ Draw free form | a separate comment; the strokes are not merged unless the operator explicitly groups them |
//! | **SolidWorks** | — | no freehand surface at all, so no vote (`HANDOFF.md` §3's *ask which of the three has the surface*) |
//!
//! Two of two applicable references say one drag, one mark. It is also the answer
//! that costs nothing to explain: the release commits, exactly as it does for the
//! four band kinds, so the whole family shares one sentence — *let go and it is
//! drawn* — and `Ctrl+Z` removes exactly the stroke the operator just made rather
//! than a session's worth of them.
//!
//! [`super::Geometry::Strokes`] is still a list of lists, because that is the
//! engine's shape and this module's job is not to editorialise about it. What is
//! deliberately absent is a *gesture* that fills more than one entry.
//!
//! ---
//!
//! ## 2. ★ THE TRAIL IS DERIVED, NEVER STORED — which is what makes its lifetime
//! ## impossible to get wrong
//!
//! A trail is state that outlives a frame, and this codebase has a standing
//! argument about that shape of state. [`crate::canvas::tool`]'s header makes it
//! about the space bar: the obvious implementation — remember on key-down, restore
//! on key-up — *"is the one that fails"*, because the restore step can be missed
//! and the failure is **sticky**.
//!
//! The trail has the identical hazard. A drag can end four ways — released,
//! Escaped, interrupted by focus loss, interrupted by the space bar borrowing the
//! hand — and only the first two are events this module could hook. An
//! implementation that cleared the trail on release and on Escape would leave a
//! stale trail after an interruption, and the operator's *next* freehand stroke
//! would begin with a segment jumping from wherever they last let go.
//!
//! So the trail is not cleared by an event. [`sync`] is called once per frame with
//! the gesture machine's own answer to *"is a freehand markup drag in flight?"*
//! ([`crate::canvas::gesture::GestureState::active`]), and the trail exists
//! exactly while that answer is yes. Every one of the four endings is covered by
//! the same line, because all four make `active()` answer `None` — there is no
//! restore step to miss, and a dropped frame costs nothing.
//!
//! ### ★ It is asked BEFORE the machine advances, and that is not a detail
//!
//! `GestureState::update` clears its own drag on the frame it reports
//! `Phase::Complete`. An `active()` read *after* it therefore answers `None` on
//! exactly the frame the release arrives — so the trail would be discarded a few
//! lines before the arm that commits it, and every stroke would author only the
//! two points `egui` happened to report on that last frame.
//!
//! **That is not a hypothetical: it is what the first version of this module
//! did, and driving the real binary is what found it.** The harness walks a drag
//! in eight increments across a 1584 x 1224 pt sheet, and the trace read
//! `markup-commit kind=Ink raw=2 kept=2` — a two-point stroke, which is a
//! straight line between the ends of a gesture the operator drew freehand. Every
//! unit test in this file passed, because they all call [`drag`] directly and
//! none of them can see the order `canvas::interact` calls two functions in. It
//! is `HANDOFF.md` §2's ninth defect exactly — a wiring order that a green suite
//! cannot express — and `ui-verify`'s `raw=` / `kept=` assertion is what will
//! catch it if it ever comes back.
//!
//! Read first, the answer is the state the *previous* frame left, which is what
//! *"is a stroke in progress?"* actually means.
//!
//! Cheap, too: one `egui::Id` lookup per frame, and an early return when there is
//! no trail, which is every frame nobody is drawing on.
//!
//! ---
//!
//! ## 3. ★ SIMPLIFICATION — what was measured, and the rule the tolerance comes
//! ## from
//!
//! A raw pointer trail is hundreds of points. Every one of them is written into
//! `/InkList` as two `Real`s **and** into the appearance stream as a `l` operator,
//! so the cost is paid twice in the file and again on every render.
//!
//! ### 3.1 What is actually captured
//!
//! One point per frame in which the pointer **moved**. Frames in which it did not
//! are dropped at capture, and that is not an optimisation: `egui` reports
//! `dragged` on every frame the button is down, so a pointer held still for two
//! seconds contributes ~120 identical points. That is the `canvas-pointer`
//! lesson — *fifty identical trace lines in nine seconds from a stationary
//! pointer* — arriving as file bytes instead of log lines, and a run of identical
//! points is also what turns [`super::action`]'s zero-extent guard into the only
//! thing standing between a held button and a 1-point blob.
//!
//! ### 3.2 The tolerance is derived from the pen, not chosen
//!
//! The tolerance is a **quarter of the stroke width**, and it is not a feel:
//! Ramer–Douglas–Peucker guarantees that no removed point lay further than ε from
//! the line that replaced it, so ε is a bound on how far the *drawn centreline*
//! can move. A stroke drawn at width *w* has a half-width of *w*/2. Setting ε to
//! **half of that half-width** means the simplified centreline stays strictly
//! inside the body of the stroke the raw trail would have drawn: no pixel of the
//! mark can move outside the mark. That is the strongest statement available
//! about a lossy simplification.
//!
//! At the shipped [`super::PEN_WIDTH_PTS`] of 2 pt that is
//! [`SIMPLIFY_TOLERANCE_PTS`] = **0.5 pt**, which is what §3.3 measures.
//!
//! #### ★ …and this paragraph used to say the rule, and then the rule was broken
//!
//! It read: *"it is a rule — if the pen ever becomes an operator control, the
//! tolerance follows it rather than being re-tuned by eye."* **The pen became an
//! operator control on 2026-08-17 and the tolerance did not follow**, because it
//! was a `const` and a `const` cannot follow anything. Between the Style group
//! landing and this fix — the same day, a few commits apart — every freehand
//! stroke was simplified against the *default* pen's width whatever pen the
//! operator had set.
//!
//! At the thin end that changes what is authored and does it silently: a
//! 0.25 pt pen — the width that exists to match a CAD sheet's own linework — has
//! a 0.125 pt half-width, against which a fixed 0.5 pt ε is **four times** too
//! loose. The centreline is then free to leave the stroke entirely, and the
//! operator gets a curve they did not draw. The claim in the paragraph above was
//! false for every pen but one.
//!
//! Fixed 2026-08-17: [`drag`] reads [`super::pen::Pen::simplify_tolerance_pts`],
//! the derivation now lives on `Pen` beside the width it derives from, and
//! `tests::the_guarantee_holds_at_every_width_the_operator_can_set` asserts the
//! bound at both ends of the operator's range rather than at the shipped middle.
//!
//! **The lesson is not about ink.** A constant derived from another value is
//! safe exactly as long as that value is also a constant. The moment the input
//! becomes settable, the derivation stays pinned to the old input and *nothing
//! about it looks wrong* — the expression still names the right thing. Writing
//! down the rule for that day, as this paragraph did, turned out not to be
//! enough; only a test that varies the input can notice.
//!
//! ### 3.3 What it measures out at
//!
//! Two measurements, both reproducible, and neither of them a feel.
//!
//! **Synthetic**, from `tests::the_measured_retention_at_the_shipped_tolerance` —
//! a 240-point trail sampled at 60 Hz along a hand-drawn-shaped curve: a sweeping
//! arc carrying a 1.2 pt lateral wander (the hand leaving the line it meant to
//! draw) and a +/-0.3 pt per-sample jitter (pointer quantisation between one frame
//! and the next). Run it with `--nocapture` and it prints the table it asserts:
//!
//! | tolerance (pt) | points kept, of 240 | measured max deviation (pt) |
//! |---:|---:|---:|
//! | 0.125 | 154 (64 %) | 0.125 |
//! | 0.25 | 91 (38 %) | 0.249 |
//! | **0.5 — shipped** | **31 (13 %)** | **0.477** |
//! | 1.0 | 16 (7 %) | 0.934 |
//! | 2.0 | 10 (4 %) | 1.767 |
//!
//! So the shipped tolerance discards **87 %** of a realistic trail, and the worst
//! any point of the original moves is **0.477 pt** — inside the 0.5 pt bound, and
//! well inside the 1 pt half-width the bound was derived from. That is the whole
//! claim of section 3.2 measured rather than argued.
//!
//! The same test walks the sweep and asserts the two properties that make the
//! table mean something: **retention falls monotonically** as the tolerance rises,
//! and **the measured deviation never exceeds the tolerance** — which is RDP's
//! guarantee, checked rather than assumed, and which is what licenses deriving the
//! tolerance from the pen at all.
//!
//! ★ **The first version of that fixture was wrong and its numbers were too
//! good**, which is worth recording because it is how a measured claim goes bad.
//! It offset both disturbances along `(sin, -cos)` — the arc's **tangent** — so
//! they only re-spaced the samples along a path whose shape they never changed,
//! and RDP removed them almost for free: 17 points kept at 0.5 pt and only 33 at
//! 0.125 pt, which is a suspiciously flat response to a sixteen-fold change in
//! tolerance. It was caught by computing the retention independently and noticing
//! the answer was better than the input deserved. The fixture now offsets
//! radially, which for a circular arc is the normal, and the test carries a
//! `worst > tolerance / 2` assertion so a future fixture that stops exercising the
//! bound fails rather than flattering it.
//!
//! **Real, through the driven binary** — `ui-verify`'s
//! `markup_freehand_and_vertex_kinds` reads `markup-commit … raw=N kept=M` off the
//! trace of an actual OS-injected drag and reports both. A synthetic curve can be
//! argued with; a number the running program printed cannot. The trace line is the
//! reason that measurement is possible at all: without `raw=` beside `kept=`, a
//! build whose simplification did nothing would emit a line indistinguishable from
//! one whose simplification worked perfectly.
//!
//! ### 3.4 Why the decision is made in CANVAS space
//!
//! The trail is captured in canvas space, simplified there, and only the surviving
//! points are converted to PDF user space. That is deliberate and it is what makes
//! *"the preview describes what will commit"* exact rather than approximate: the
//! preview draws the kept points and the file receives the same kept points, so
//! the two cannot disagree about which points survived.
//!
//! It is also numerically the same decision as simplifying in page space, because
//! [`crate::viewer::canvas_to_pdf_space`] at scale 1.0 is an **isometry** — a
//! translation, a multiple-of-90° rotation and a Y flip, all of which preserve
//! distance — so a tolerance in canvas units is a tolerance in PDF points. If that
//! ever stopped being true, the identity of the kept set would still hold, which
//! is the property rule 4 actually asks for.
//!
//! ### 3.5 What is deliberately NOT done
//!
//! **No smoothing, and no curve fitting.** Acrobat's ink is a polyline and so is
//! `pdfcer-core`'s builder (`ink()` emits `move_to` then `line_to`, §12.5.6.12), so
//! a Bézier fit here would be a shape the appearance stream cannot express and the
//! preview would be the only place it existed. **No minimum-step filter in screen
//! space**, either: it would make the captured detail depend on the magnification
//! at the moment of drawing, so the same gesture at 8× and at 0.5× would author
//! different geometry — the zoom-dependence
//! [`crate::canvas::mapping`]'s header exists to keep out of this crate.

use egui::Pos2;
use pdfcer_core::page_tree::Page;

use super::{Geometry, MarkupKind};
use crate::app::actions::Action;
use crate::canvas::gesture::{DragKind, Phase};
use crate::canvas::mapping::PageMapping;

/// Where the trail lives between the frames of one drag.
// ui-text-exempt: an `egui::Id` source string, never displayed.
const INK_MEMORY_KEY: &str = "pdfcer-markup-ink-trail";

/// **The simplification tolerance of the SHIPPED pen**, in PDF points — the
/// largest distance a removed point may have lain from the line that replaced
/// it.
///
/// **0.5 pt, derived from the pen rather than chosen.** See §3.2: it is half of
/// [`super::PEN_WIDTH_PTS`]'s half-width, so the simplified centreline stays
/// strictly inside the body of the stroke the raw trail would have drawn.
///
/// # ★ This is no longer what the running code reads — 2026-08-17
///
/// [`drag`] calls [`super::pen::Pen::simplify_tolerance_pts`], which derives
/// the same quarter-width from the pen the operator actually set. This constant
/// remains as **the value the shipped pen implies**, which is what §3.3's
/// measurement table is measured at and what the tests below sweep around; it
/// is deliberately *not* deleted, because the measurements are meaningless
/// without a named value to attach them to.
///
/// It was the live value until 2026-08-17 and by then it was stale: the pen
/// became an operator control on 2026-08-17 and this `const` went on deriving
/// itself from the *default* width. §3.2 had already written the rule for that
/// day — *"if the pen ever becomes an operator control, the tolerance follows
/// it"* — so the module predicted its own defect and then had it anyway,
/// because a `const` cannot follow anything.
///
/// **The generalisable half:** a constant derived from another value is safe
/// exactly as long as that value is also a constant. The moment the input
/// becomes settable, the derivation is silently pinned to its old input, and
/// nothing about it looks wrong — the expression still names the right thing.
/// [`tests::the_shipped_constant_matches_the_shipped_pen`] is what now welds
/// the two, so this cannot drift a second time.
///
/// Measured retention at this value is in §3.3 and is asserted, with the RDP
/// deviation bound, by [`tests::the_measured_retention_at_the_shipped_tolerance`].
pub const SIMPLIFY_TOLERANCE_PTS: f32 = (super::PEN_WIDTH_PTS as f32) / 4.0;

/// The pointer trail of one freehand drag, in **canvas space**.
///
/// Canvas space rather than page space, for §3.4's reason: the simplification
/// decides *which points survive* and it decides in the space the pointer lives
/// in, so the preview and the file receive the same surviving points rather than
/// two derivations that agree by construction until they do not.
#[derive(Debug, Clone, PartialEq)]
struct Trail {
    /// Every distinct position the pointer has been at, in order, including the
    /// press origin as the first entry.
    points: Vec<Pos2>,
}

/// Read the trail without creating one.
fn read(ctx: &egui::Context) -> Option<Trail> {
    ctx.data_mut(|d| d.get_temp::<Trail>(egui::Id::new(INK_MEMORY_KEY)))
}

/// Write the trail back.
fn store(ctx: &egui::Context, trail: Trail) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(INK_MEMORY_KEY), trail));
}

/// Forget the trail.
fn clear(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Trail>(egui::Id::new(INK_MEMORY_KEY)));
}

/// **Keep the trail alive exactly while a freehand drag is** — §2's derived
/// lifetime, in one line.
///
/// Called once per frame from `canvas::interact` with
/// [`crate::canvas::gesture::GestureState::active`]'s answer. Every way a drag can
/// end — released, Escaped, interrupted by focus loss, interrupted by the space
/// bar borrowing the hand — makes that answer `None`, so all four are handled by
/// this one comparison and none of them needs a hook of its own.
///
/// The early return is what makes it free: on every frame nobody is drawing on,
/// `read` finds nothing and the function does nothing.
pub(in crate::canvas) fn sync(ctx: &egui::Context, active: Option<DragKind>) {
    let freehand = matches!(active, Some(DragKind::Markup(kind)) if kind.is_freehand());
    if !freehand && read(ctx).is_some() {
        clear(ctx);
    }
}

/// **Everything one frame of a freehand drag is resolved against.**
///
/// A struct rather than seven parameters, and not merely to satisfy a lint —
/// [`crate::canvas::measure::Pick`] is the precedent and its own docs carry the
/// argument, which applies here word for word: the fields that describe *where
/// this frame's pointer is* are only meaningful together, and a call site that
/// had six of them and reached for the seventh from somewhere else would be
/// resolving a stroke against a page it did not come from — the class of defect
/// [`crate::canvas::mapping`]'s header exists to make unavailable.
///
/// The `ctx` is in here for a reason of its own: it is where the trail lives
/// (§2), so a caller that supplied a *different* context from the one the
/// gesture machine's `active()` was read against would be syncing one trail and
/// extending another.
pub(in crate::canvas) struct Stroke<'a> {
    /// Where the trail is stored between the frames of this drag.
    pub ctx: &'a egui::Context,
    /// Which markup kind is armed. Checked rather than assumed — see [`drag`]'s
    /// family guard.
    pub kind: MarkupKind,
    /// Where the button went down, in canvas space. Seeded as the trail's first
    /// point; see [`drag`]'s section on why it is appended rather than used as
    /// an anchor.
    pub from: Pos2,
    /// Where the pointer is now, in canvas space.
    pub to: Pos2,
    /// Draw the trail, or commit the stroke.
    pub phase: Phase,
    /// The page the stroke is authored onto.
    pub page_index: usize,
    /// That page, for the canvas→PDF transform. `None` when the frame has none,
    /// which is refused rather than authored.
    pub page: Option<&'a Page>,
}

/// Apply one frame of a freehand drag: extend the trail, or commit the stroke.
///
/// [`super::band::drag`]'s twin, and deliberately the same shape — one function
/// that touches the frame, returning the preview while the pointer is down and
/// pushing exactly one [`Action::CommitMarkup`] on release.
///
/// What differs is what it returns: a **polyline** rather than a band, already
/// [`simplify`]-ed, because the simplified trail is what the release will author
/// and rule 4 says the affordance has to describe that rather than the raw input.
///
/// # Why `from` is appended and not merely used as an anchor
///
/// [`crate::canvas::gesture::PointerFrame::press_origin`] exists because
/// `drag_started` fires only once the pointer has travelled far enough for `egui`
/// to call the interaction a drag — **measured at 94 page points on an A1 sheet
/// at 0.21× zoom**. For a band that offset moved a corner; for a stroke it would
/// remove the beginning of it. So the origin is seeded as the trail's first point,
/// which is the same fix the band gets from the same field.
pub(in crate::canvas) fn drag(
    pen: super::pen::Pen,
    stroke: Stroke<'_>,
    actions: &mut Vec<Action>,
) -> Option<Vec<Pos2>> {
    let Stroke {
        ctx,
        kind,
        from,
        to,
        phase,
        page_index,
        page,
    } = stroke;
    if !kind.is_freehand() {
        return None;
    }
    let mut trail = read(ctx).unwrap_or_else(|| Trail { points: vec![from] });
    // ★ Distinct positions only — §3.1. A stationary pointer under a held button
    // reports `dragged` on every frame, and each of those would otherwise be two
    // more `Real`s in `/InkList` and one more `l` in the appearance stream.
    if trail.points.last() != Some(&to) {
        trail.points.push(to);
    }
    let raw = trail.points.len();
    // ★ The PEN's tolerance, not the shipped constant — §3.2's rule, honoured.
    //
    // This read `SIMPLIFY_TOLERANCE_PTS` until 2026-08-17, which was a `const`
    // derived from the pen's *default* 2 pt width. At a 0.25 pt pen — the
    // width that exists to match a CAD sheet's own linework — a fixed 0.5 pt
    // tolerance is four times the stroke's half-width, so the simplified
    // centreline can leave the body of the stroke entirely and the operator
    // gets a visibly different curve from the one they drew. See
    // `Pen::simplify_tolerance_pts` for the table.
    //
    // Read here rather than inside `simplify`, so that function stays a pure
    // `(points, tolerance)` and its measurement tests can sweep the tolerance
    // without constructing a pen.
    let kept = simplify(&trail.points, pen.simplify_tolerance_pts());
    store(ctx, trail);

    if phase == Phase::InFlight {
        return Some(kept);
    }

    // From here on the gesture is over: the trail must not survive into the next
    // stroke whatever happens below, including every refusal.
    clear(ctx);
    let Some(page) = page else {
        super::decline(kind, page_index, super::Refusal::NoPage);
        return None;
    };
    let mut points: Vec<(f64, f64)> = Vec::with_capacity(kept.len());
    for at in &kept {
        let Some(point) = super::vertex::page_point(*at, page) else {
            super::decline(kind, page_index, super::Refusal::DegeneratePage);
            return None;
        };
        points.push(point);
    }
    match super::action(kind, page_index, Geometry::Strokes(vec![points]), pen) {
        Ok(raised) => {
            super::trace_commit(
                kind,
                page_index,
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                //
                // ★ `raw` BESIDE `kept` — §3.3. A build whose simplification did
                // nothing at all would emit an otherwise identical line, and the
                // only external evidence that this feature works is the two
                // numbers differing. It is also how the *real* retention figure
                // in §3.3 is obtained, as opposed to the synthetic one.
                &format!("raw={raw} kept={}", kept.len()),
            );
            actions.push(raised);
        }
        Err(reason) => super::decline(kind, page_index, reason),
    }
    None
}

/// Paint the freehand trail.
///
/// The **simplified** trail, in the pen's own colour and width, which is the
/// whole of rule 4's requirement here: what is drawn is the polyline that is about
/// to be written into `/InkList`, point for point, rather than the raw input it
/// was derived from. A preview of the raw trail would be a promise the file does
/// not keep — at any zoom where the difference is visible, the mark would
/// visibly *change* on release, which is the one thing a pre-commit affordance
/// must not do.
///
/// Round joins and caps, matching `pdfcer-core`'s `ink()` builder, which sets
/// `LineCap::Round` and `LineJoin::Round` — so a corner of the preview and a
/// corner of the annotation are the same corner.
pub(in crate::canvas) fn draw_preview(
    painter: &egui::Painter,
    map: &PageMapping,
    trail: &[Pos2],
    pen: super::pen::Pen,
) {
    if trail.len() < 2 {
        return;
    }
    // DOCUMENT COLOUR: the pen, read from the one place `spec` reads it.
    let stroke = egui::Stroke::new(
        super::pen_px(map, pen),
        super::pen_color(MarkupKind::Ink, pen),
    );
    let screen: Vec<Pos2> = trail.iter().map(|p| map.to_screen(*p)).collect();
    painter.add(egui::Shape::line(screen, stroke));
}

/// **Ramer–Douglas–Peucker**, iteratively.
///
/// Returns the subsequence of `points` that survives at `tolerance`, always
/// including the first and last. The guarantee — and the reason
/// [`SIMPLIFY_TOLERANCE_PTS`] can be derived from the pen rather than tuned — is
/// that **no removed point lay further than `tolerance` from the segment that
/// replaced it**, which bounds how far the drawn centreline can move.
///
/// Iterative rather than recursive on purpose. The recursive form is shorter and
/// its depth is the *number of retained points*, which for a long slow stroke is
/// in the hundreds; a canvas that overflowed its stack because somebody drew a
/// spiral would be a spectacular way to lose an unsaved document. The explicit
/// stack costs four lines.
///
/// `tolerance <= 0` returns the input unchanged rather than looping, and a run of
/// fewer than three points has nothing to remove.
#[must_use]
fn simplify(points: &[Pos2], tolerance: f32) -> Vec<Pos2> {
    // `is_nan()` beside `<= 0.0` rather than `!(tolerance > 0.0)`, which says the
    // same thing and which clippy refuses on a partially ordered type. A NaN
    // tolerance would make every comparison below false and silently keep every
    // point, which is a simplification that did nothing wearing a green test.
    if points.len() < 3 || tolerance <= 0.0 || tolerance.is_nan() {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0_usize, points.len() - 1)];
    while let Some((first, last)) = stack.pop() {
        if last <= first + 1 {
            continue;
        }
        let (mut worst, mut worst_at) = (0.0_f32, first);
        for (i, p) in points.iter().enumerate().take(last).skip(first + 1) {
            let d = distance_to_segment(*p, points[first], points[last]);
            if d > worst {
                worst = d;
                worst_at = i;
            }
        }
        if worst > tolerance {
            keep[worst_at] = true;
            stack.push((first, worst_at));
            stack.push((worst_at, last));
        }
    }
    points
        .iter()
        .zip(keep)
        .filter_map(|(p, k)| k.then_some(*p))
        .collect()
}

/// Perpendicular distance from `p` to the segment `a`–`b`.
///
/// Segment, not infinite line: RDP's guarantee is about the polyline that
/// replaces the removed run, and a point beyond an endpoint is further from the
/// *segment* than from the line it lies on. Using the line would under-report
/// exactly at a hairpin, which is where a hand-drawn stroke doubles back and is
/// the one place the operator can see the difference.
///
/// A degenerate segment (`a == b`) falls back to the distance to `a`, which is
/// the correct answer and avoids a division by zero.
fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_sq();
    if len2 <= 0.0 || len2.is_nan() {
        return (p - a).length();
    }
    let t = (((p - a).x * ab.x + (p - a).y * ab.y) / len2).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trail shaped like a hand-drawn stroke: a sweeping arc with two
    /// lateral disturbances on it, sampled at 60 Hz for four seconds.
    ///
    /// The two disturbances are the fixture, and they are chosen to sit on
    /// **opposite sides of the tolerance** so that it is asked a question it
    /// could get wrong in either direction:
    ///
    /// | component | amplitude | what it stands for | what must happen to it |
    /// |---|---|---|---|
    /// | a slow undulation, 7½ cycles across the stroke | **1.2 pt** | the hand wandering off the line it meant to draw | **kept** — it is above the pen's 1 pt half-width, so it is visible in the mark and is detail the simplification is obliged to preserve |
    /// | a fast per-sample jitter | **±0.3 pt** | pointer quantisation and tremor between one frame and the next | **removed** — it is below the 0.5 pt tolerance, so no pixel of the drawn stroke moves when it goes |
    ///
    /// A smooth arc alone would flatter the tolerance: it would simplify to
    /// almost nothing and prove only that RDP works on lines. The jitter is what
    /// makes the retention figure in §3.3 mean something, because it is the
    /// component a real trail is mostly made of.
    ///
    /// The jitter is a deterministic hash of the sample index rather than a
    /// random number, so the measured figures in §3.3 are reproducible: a
    /// measurement quoted in prose that changes between runs is a measurement
    /// nobody can check.
    fn hand_drawn_trail(n: usize) -> Vec<Pos2> {
        (0..n)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let t = i as f32 / (n as f32 - 1.0);
                let angle = t * std::f32::consts::PI * 0.75;
                let (x, y) = (200.0 + 160.0 * angle.cos(), 200.0 + 160.0 * angle.sin());
                // The slow wander, across the direction of travel.
                let wander = (t * 47.0).sin() * 1.2;
                // …and the fast jitter: a cheap deterministic hash of the sample
                // index, mapped to ±0.3 pt. Deterministic so the numbers §3.3
                // quotes can be re-measured.
                #[allow(clippy::cast_precision_loss)]
                let hashed = ((i as u32).wrapping_mul(2_654_435_761) >> 8) % 601;
                #[allow(clippy::cast_precision_loss)]
                let jitter = (hashed as f32 / 1000.0) - 0.3;
                // ★ Applied along the RADIAL direction, which for a circular arc
                // is the normal — i.e. across the direction of travel. The first
                // version of this fixture used `(sin, -cos)`, which is the
                // *tangent*, so both disturbances merely re-spaced the samples
                // along a path whose shape they never changed; RDP removed them
                // almost for free and the fixture flattered the tolerance. The
                // measurement is only worth quoting because that was caught by
                // computing the retention independently and finding it too good.
                let off = wander + jitter;
                Pos2::new(x + off * angle.cos(), y + off * angle.sin())
            })
            .collect()
    }

    /// ★ **The measured retention at the shipped tolerance, and the bound RDP
    /// promises.**
    ///
    /// §3.3's synthetic measurement, asserted rather than quoted, plus the two
    /// properties that make the tolerance a *rule* instead of a number:
    ///
    /// 1. **The deviation never exceeds the tolerance**, at any tolerance in the
    ///    sweep. That is RDP's guarantee, and it is what licenses deriving the
    ///    tolerance from the pen's half-width — if it did not hold, "the
    ///    centreline stays inside the stroke" would be an unsupported claim.
    /// 2. **Retention falls monotonically** as the tolerance rises. A build whose
    ///    simplification was subtly wrong — comparing against the infinite line
    ///    rather than the segment, say — can still pass a single-point retention
    ///    assertion and fails this one.
    ///
    /// The exact retention figure is printed in the assertion message so that a
    /// reader who changes the pen can see what it did rather than only that it
    /// broke a bound.
    #[test]
    fn the_measured_retention_at_the_shipped_tolerance() {
        let trail = hand_drawn_trail(240);
        assert_eq!(trail.len(), 240, "the fixture is 4 s at 60 Hz");

        let mut previous = usize::MAX;
        for tolerance in [0.125_f32, 0.25, 0.5, 1.0, 2.0] {
            let kept = simplify(&trail, tolerance);
            // Property 1: every dropped point lay within `tolerance` of the
            // polyline that replaced it. Measured against the SIMPLIFIED
            // polyline, which is the thing that is actually drawn.
            let worst = trail
                .iter()
                .map(|p| {
                    kept.windows(2)
                        .map(|w| distance_to_segment(*p, w[0], w[1]))
                        .fold(f32::INFINITY, f32::min)
                })
                .fold(0.0_f32, f32::max);
            assert!(
                worst <= tolerance + 1e-3,
                "tolerance {tolerance}: a point lay {worst} from the simplified polyline, \
                 which breaks the bound the shipped tolerance is derived from"
            );
            // Property 2: monotonic.
            assert!(
                kept.len() <= previous,
                "tolerance {tolerance} kept {} where {previous} were kept at a tighter one",
                kept.len()
            );
            previous = kept.len();
            // Printed, not merely asserted, so the figures quoted in the module
            // header's section 3.3 can be RE-MEASURED rather than trusted:
            // `cargo test -p pdfcer-gui --lib the_measured_retention -- --nocapture`.
            // A number in prose that nobody can reproduce is the shape of claim
            // this project has had to correct four times.
            eprintln!(
                // ui-text-exempt: a test measurement, printed to the test
                // harness's stderr and never to an operator.
                "ink-simplify tolerance={tolerance} kept={} of {} worst={worst}",
                kept.len(),
                trail.len()
            );
            if (tolerance - SIMPLIFY_TOLERANCE_PTS).abs() < 1e-6 {
                // The shipped value. Asserted as a band rather than an exact
                // count: the figure quoted in section 3.3 is what this fixture
                // produces, and a fixture-sensitive equality would fail on a
                // compiler that rounds `sin` one ulp differently.
                assert!(
                    kept.len() < trail.len() / 4,
                    "at the shipped tolerance {tolerance} pt the trail retained {} of {} \
                     points ({}%), which is not the reduction section 3.3 records",
                    kept.len(),
                    trail.len(),
                    kept.len() * 100 / trail.len()
                );
                assert!(
                    worst > tolerance / 2.0,
                    "at the shipped tolerance the worst deviation was only {worst} pt, so this \
                     fixture is not exercising the bound and the table in section 3.3 is \
                     measuring an easier curve than it claims — which is exactly how the first \
                     version of this fixture was wrong"
                );
                assert!(
                    kept.len() >= 8,
                    "at the shipped tolerance the trail retained only {} points, which is a \
                     simplification that has eaten the stroke rather than tidied it",
                    kept.len()
                );
            }
        }
        // The ends always survive: a stroke that lost its first or last point
        // would start or stop somewhere the operator did not.
        let kept = simplify(&trail, SIMPLIFY_TOLERANCE_PTS);
        assert_eq!(kept.first(), trail.first());
        assert_eq!(kept.last(), trail.last());
    }

    /// A straight run collapses to its two ends, and a zero tolerance keeps
    /// everything — the two extremes that say the algorithm is doing anything at
    /// all.
    #[test]
    fn a_straight_run_collapses_and_a_zero_tolerance_keeps_everything() {
        let straight: Vec<Pos2> = (0..50).map(|i| Pos2::new(i as f32 * 3.0, 100.0)).collect();
        assert_eq!(simplify(&straight, SIMPLIFY_TOLERANCE_PTS).len(), 2);
        assert_eq!(simplify(&straight, 0.0).len(), 50);
        assert_eq!(simplify(&straight, -1.0).len(), 50);
        // Fewer than three points has nothing to remove.
        let two = vec![Pos2::ZERO, Pos2::new(10.0, 10.0)];
        assert_eq!(simplify(&two, 100.0), two);
    }

    /// ★ **A hairpin keeps its point**, which is what distinguishes the segment
    /// distance from the line distance.
    ///
    /// A stroke that doubles back on itself has its apex *on* the line through
    /// its two ends, so a build measuring against the infinite line would compute
    /// a deviation of zero and delete the apex — turning a fold into a straight
    /// line the operator never drew, and doing it silently.
    #[test]
    fn a_hairpin_keeps_its_apex_where_an_infinite_line_would_lose_it() {
        let hairpin = vec![
            Pos2::new(0.0, 0.0),
            Pos2::new(50.0, 0.0),
            Pos2::new(100.0, 0.0),
            Pos2::new(50.0, 0.0),
            Pos2::new(0.0, 0.0),
        ];
        let kept = simplify(&hairpin, SIMPLIFY_TOLERANCE_PTS);
        assert!(
            kept.contains(&Pos2::new(100.0, 0.0)),
            "the fold's apex was deleted: {kept:?}"
        );
        // …and the distance function is why.
        let apex = Pos2::new(100.0, 0.0);
        let (a, b) = (Pos2::ZERO, Pos2::ZERO);
        assert!(distance_to_segment(apex, a, b) > 99.0);
    }

    /// ★ **A stationary pointer contributes one point, not one per frame.**
    ///
    /// §3.1, at the capture end. Without the duplicate filter a held button emits
    /// ~60 identical points a second into `/InkList`, and the resulting run of
    /// identical coordinates is also what [`super::action`]'s zero-extent guard
    /// would then be the only defence against.
    #[test]
    fn a_stationary_pointer_contributes_one_point() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        let at = Pos2::new(40.0, 40.0);
        for _ in 0..50 {
            let _ = drag(
                crate::canvas::markup::pen::Pen::default(),
                Stroke {
                    ctx: &ctx,
                    kind: MarkupKind::Ink,
                    from: at,
                    to: at,
                    phase: Phase::InFlight,
                    page_index: 0,
                    page: None,
                },
                &mut actions,
            );
        }
        let trail = read(&ctx).expect("a trail exists");
        assert_eq!(
            trail.points.len(),
            1,
            "fifty identical frames produced {} points",
            trail.points.len()
        );
        assert!(actions.is_empty());
    }

    /// ★ **The trail is derived, so every way a drag can end discards it.**
    ///
    /// §2's argument, asserted through the one line that implements it. The
    /// interruption row is the one an event-hooked implementation gets wrong: the
    /// window loses focus, `egui` stops reporting the drag without ever reporting
    /// a stop, and the next stroke would begin with a segment jumping from
    /// wherever the operator last let go.
    #[test]
    fn every_way_a_drag_can_end_discards_the_trail() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        for step in 0..5 {
            let _ = drag(
                crate::canvas::markup::pen::Pen::default(),
                Stroke {
                    ctx: &ctx,
                    kind: MarkupKind::Ink,
                    from: Pos2::ZERO,
                    to: Pos2::new(step as f32 * 10.0, 5.0),
                    phase: Phase::InFlight,
                    page_index: 0,
                    page: None,
                },
                &mut actions,
            );
        }
        assert!(read(&ctx).is_some(), "a trail is in flight");

        // Still in flight: nothing is discarded.
        sync(&ctx, Some(DragKind::Markup(MarkupKind::Ink)));
        assert!(read(&ctx).is_some());

        // Escaped, interrupted, or simply over — all three are `None`.
        sync(&ctx, None);
        assert!(read(&ctx).is_none(), "the trail must not outlive its drag");

        // …and another kind of drag is not this one, so a band started after an
        // interrupted stroke does not inherit its points.
        let _ = drag(
            crate::canvas::markup::pen::Pen::default(),
            Stroke {
                ctx: &ctx,
                kind: MarkupKind::Ink,
                from: Pos2::ZERO,
                to: Pos2::new(9.0, 9.0),
                phase: Phase::InFlight,
                page_index: 0,
                page: None,
            },
            &mut actions,
        );
        sync(&ctx, Some(DragKind::Markup(MarkupKind::Rectangle)));
        assert!(read(&ctx).is_none());
    }

    /// A non-freehand kind is refused here, exactly as a non-band kind is refused
    /// by [`super::band::drag`] — the two guards are the two halves of one
    /// routing rule, and a build that lost the routing would otherwise author a
    /// one-segment `/Ink` for every rectangle drawn.
    #[test]
    fn a_non_freehand_kind_is_refused_by_the_freehand_gesture() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        for kind in [
            MarkupKind::Rectangle,
            MarkupKind::Highlight,
            MarkupKind::Polygon,
        ] {
            assert_eq!(
                drag(
                    crate::canvas::markup::pen::Pen::default(),
                    Stroke {
                        ctx: &ctx,
                        kind,
                        from: Pos2::ZERO,
                        to: Pos2::new(10.0, 10.0),
                        phase: Phase::InFlight,
                        page_index: 0,
                        page: None,
                    },
                    &mut actions,
                ),
                None,
                "{kind:?}"
            );
        }
        assert!(actions.is_empty());
        assert!(read(&ctx).is_none());
    }

    /// ★ **The preview draws the points that will be committed**, not the raw
    /// trail.
    ///
    /// Rule 4's honesty requirement, asserted where it can be: the value handed
    /// back for painting is the *same* `simplify` output the release converts and
    /// authors. A build that previewed the raw trail would show a mark that
    /// visibly changed shape at the moment of release, which is precisely what a
    /// pre-commit affordance exists to prevent.
    #[test]
    fn the_preview_is_the_simplified_trail_that_will_be_authored() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        let trail = hand_drawn_trail(120);
        let mut last = None;
        for (i, p) in trail.iter().enumerate() {
            last = drag(
                crate::canvas::markup::pen::Pen::default(),
                Stroke {
                    ctx: &ctx,
                    kind: MarkupKind::Ink,
                    from: trail[0],
                    to: *p,
                    phase: Phase::InFlight,
                    page_index: 0,
                    page: None,
                },
                &mut actions,
            );
            assert!(last.is_some(), "frame {i} drew nothing");
        }
        let previewed = last.expect("a preview");
        let raw = read(&ctx).expect("a trail").points;
        assert!(
            previewed.len() < raw.len(),
            "the preview drew every raw point ({} of {})",
            previewed.len(),
            raw.len()
        );
        assert_eq!(
            previewed,
            simplify(&raw, SIMPLIFY_TOLERANCE_PTS),
            "the preview must be the exact polyline the release authors"
        );
    }

    /// The tolerance is derived from the pen and stays so — a test rather than a
    /// comment, because §3.2's whole claim is that the two move together.
    #[test]
    fn the_tolerance_is_half_the_pens_half_width() {
        #[allow(clippy::cast_possible_truncation)]
        let half_width = (super::super::PEN_WIDTH_PTS as f32) / 2.0;
        assert!(
            (SIMPLIFY_TOLERANCE_PTS - half_width / 2.0).abs() < 1e-6,
            "the simplified centreline must stay strictly inside the drawn stroke"
        );
    }

    /// ★ **The guarantee holds at EVERY width the operator can set**, not just
    /// at the shipped one.
    ///
    /// The test above was true and insufficient, and the gap between them is
    /// the defect this pair now pins. It asserts a relation between two
    /// *constants*, so it went on passing unchanged on 2026-08-17 when the pen
    /// width became an operator control from 0.25 to 12 pt and the tolerance
    /// stayed welded to the default 2 pt.
    ///
    /// At the thin end that is not a rounding difference: a 0.25 pt pen has a
    /// 0.125 pt half-width, and a fixed 0.5 pt ε is **four times** it — so
    /// Ramer–Douglas–Peucker is free to move the centreline clean outside the
    /// stroke, and the operator gets a curve they did not draw. §3.2's whole
    /// claim is *"no pixel of the mark can move outside the mark"*, and that
    /// claim was false for every pen but one.
    ///
    /// This asserts it across the range, which is the only form that can
    /// notice the input becoming settable again.
    #[test]
    fn the_guarantee_holds_at_every_width_the_operator_can_set() {
        use super::super::pen::{MAX_WIDTH_PTS, MIN_WIDTH_PTS, Pen};

        // The two ends and the shipped middle. The ends are what matter: a
        // derivation that had been pinned to the default would pass at the
        // middle and fail at both ends, which is exactly the shape of the
        // defect.
        for width in [MIN_WIDTH_PTS, super::super::PEN_WIDTH_PTS, MAX_WIDTH_PTS] {
            let pen = Pen {
                width_pts: width,
                ..Pen::default()
            };
            #[allow(clippy::cast_possible_truncation)]
            let half_width = (width as f32) / 2.0;
            let tolerance = pen.simplify_tolerance_pts();
            assert!(
                (tolerance - half_width / 2.0).abs() < 1e-6,
                "at a {width} pt pen the tolerance is {tolerance} pt but the \
                 half-width is {half_width} pt — the simplified centreline can \
                 leave the stroke it is meant to stay inside"
            );
        }
    }

    /// ★ The shipped constant and the shipped pen agree.
    ///
    /// The weld between the two halves of this module's tolerance story:
    /// [`SIMPLIFY_TOLERANCE_PTS`] is the value §3.3's measurement table was
    /// measured at, and [`super::super::pen::Pen::simplify_tolerance_pts`] is
    /// what the running code reads. If they ever disagreed, the table would be
    /// documenting a tolerance the shipped build does not use — a measurement
    /// that is still *reproducible* and no longer *about* anything.
    ///
    /// The two are separate on purpose (the constant names a value, the method
    /// derives one), so this is what stops them being separate in effect.
    #[test]
    fn the_shipped_constant_matches_the_shipped_pen() {
        assert!(
            (SIMPLIFY_TOLERANCE_PTS - super::super::pen::Pen::default().simplify_tolerance_pts())
                .abs()
                < 1e-6,
            "§3.3's measurements are quoted against a tolerance the shipped \
             build no longer uses"
        );
    }
}
