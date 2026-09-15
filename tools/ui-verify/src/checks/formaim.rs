//! `checks::formaim` — **where a check must click so that a form-field
//! selection CHANGES**, and the census parsing that finds it.
//!
//! # ★★★ The finding this module exists to encode
//!
//! Three driven checks — `form_field`, `widget_move` and `field_menu` — all
//! begin the same way: arm `edit.form_text_field`, click the page, and get a
//! field. Then all three clicked that field's centre and asserted the
//! application traced
//!
//! ```text
//! pdfcer-diag form-field-selected page=0 field=Text1 widget=0
//! ```
//!
//! On 2026-08-29 all three failed on that assertion, with the same sentence:
//! *"THE FIELD COULD NOT BE SELECTED"*. **The click was landing exactly where
//! the field is.** From `widget-move.trace.txt`:
//!
//! ```text
//! form-target page=0 field=Text1 widget=0 rect=(473.8,529.4)+(160.0,20.0)
//! canvas-pointer screen=(460.0,429.0) page=(555.08,539.21) …
//! ```
//!
//! The rect spans x ∈ [473.8, 633.8] and y ∈ [529.4, 549.4]; the click resolved
//! to page (555.08, 539.21), which is 1.3 pt from its centre in x and 0.2 pt in
//! y. Nothing missed.
//!
//! What actually happened is one frame earlier in the same trace:
//!
//! ```text
//! add-form-field page=0 n=1 epoch=1 …
//! ui-rect name=canvas.selection-outline rect=[[436.0 426.1] - [483.3 432.0]]
//! ```
//!
//! 47.3 × 5.9 px at zoom 0.2955 is 160 × 20 canvas units — **the new field,
//! already drawn selected**. `app::actions::forms`' authoring arm sets
//! `doc.selected_field` to what it just placed (`OPERATOR_REQUESTS.md` **O53**:
//! *"every program in this class leaves a newly drawn object selected"*), and
//! `canvas::forms::select_click` raises `FieldAction::Select` — and writes its
//! trace line — **only on a change**:
//!
//! ```text
//! if picked == doc.selected_field { return; }
//! ```
//!
//! ⇒ So the click was correct, the program was correct, and the *check* was
//! asking a question with no answer: it clicked a field that was already
//! selected and then required the program to announce a selection that had not
//! moved. Neither a stale coordinate mapping nor a broken hit test — the page
//! did not move between the two clicks (`paint=` is identical on every
//! `canvas-pos` line of `form_field.trace.txt`, from the placement through the
//! selection), and the widget was hit dead centre.
//!
//! # ★★ The repair, and why it makes the checks say MORE than they did
//!
//! A check that wants to observe *"clicking a widget selects it"* has to make
//! the selection different first. The program documents exactly one gesture
//! that does so, in `canvas::forms::select_click`'s own table:
//!
//! | | primary | secondary |
//! |---|---|---|
//! | over a field | select it | select it |
//! | over the selected field | no change | no change |
//! | **over blank paper** | **clear** | change nothing |
//!
//! So: click blank paper (the application traces `form-field-selected none`),
//! **assert that clearing line arrived**, then click the field and assert the
//! naming line. Two observations where there was one, and the first is what
//! makes the second admissible — `crate::checks`' rule 4, that an absence is
//! evidence only once the thing that would have produced a presence is shown
//! working. A build that stopped tracing selection at all now fails at the
//! clearing step, naming the trace channel rather than the hit test.
//!
//! # ★ Why "blank paper" is computed rather than named
//!
//! The three callers place their field at different points on different
//! documents — `form_field` at the sweep's `--doc-point`, the other two at page
//! fractions — and the sweep's document (`SW41177.pdf`, 36 sheets of CAD) may
//! carry widgets of its own. A constant offset would eventually land on one,
//! and the check would then fail reporting a selection that in fact changed
//! from one field to another. [`blank_canvas_point`] therefore consults the
//! application's **own** census of where every selectable widget is, tries a
//! short ring of candidates around the target, and returns the first that is
//! clear of all of them and comfortably inside the sheet.
//!
//! Everything here is in **canvas space** — y increasing downward from the top
//! of the sheet — because that is the space `form-target` publishes. Callers
//! flip to PDF space with `page.height_pt - y` before handing a point to
//! `CanvasMapping::doc_to_window`, exactly as they did before this module
//! existed.

use crate::coords::PageGeometry;
use crate::trace::Trace;

/// The census line the canvas publishes for every **selectable** widget.
///
/// ★ `form-target`, not `form-box`: the second lists what a click can FILL,
/// which excludes drop-downs, push buttons and any widget with no appearance.
/// `form_field`'s own constant carries the longer version of this note.
pub const TARGET_LINE: &str = "form-target";

/// One `form-target` line, parsed back into a canvas-space rectangle.
///
/// The application's numbers, never the fixture's: `canvas/forms.rs` publishes
/// this census precisely so a harness can aim at where the *program* says the
/// box is. A check that recomputed the rect from the PDF would be asserting
/// that two independent derivations agree, and would report a disagreement as
/// a hit-test failure.
#[derive(Clone, Debug)]
pub struct WidgetBox {
    /// Zero-based page index.
    pub page: usize,
    /// Fully-qualified field name, as the census writes it.
    pub field: String,
    /// Canvas-space top-left corner.
    pub min: (f64, f64),
    /// Width and height in canvas units.
    pub size: (f64, f64),
}

impl WidgetBox {
    /// The canvas-space centre — what a selecting click aims at.
    #[must_use]
    pub fn centre(&self) -> (f64, f64) {
        (
            self.min.0 + self.size.0 / 2.0,
            self.min.1 + self.size.1 / 2.0,
        )
    }

    /// Is `p` inside this box, grown by `margin` on every side?
    ///
    /// ★ The margin is what makes "outside" mean *comfortably* outside. A
    /// point one unit clear of an edge survives this test and can still land
    /// inside the widget once the canvas mapping has rounded it to a whole
    /// screen pixel, which at a fit zoom of 0.29 is three canvas units wide.
    #[must_use]
    pub fn contains(&self, p: (f64, f64), margin: f64) -> bool {
        p.0 >= self.min.0 - margin
            && p.0 <= self.min.0 + self.size.0 + margin
            && p.1 >= self.min.1 - margin
            && p.1 <= self.min.1 + self.size.1 + margin
    }
}

/// Every widget the canvas has named, newest census first in trace order.
#[must_use]
pub fn targets(trace: &Trace) -> Vec<WidgetBox> {
    trace
        .events(TARGET_LINE)
        .filter_map(|l| {
            let page = l.get("page")?.parse().ok()?;
            let field = l.get("field")?.to_owned();
            // `rect=(x,y)+(w,h)` — the canvas rect, as the census writes it.
            let raw = l.get("rect")?;
            let (min, size) = raw.split_once(")+(")?;
            let (x, y) = min.trim_start_matches('(').split_once(',')?;
            let (w, h) = size.trim_end_matches(')').split_once(',')?;
            let (x, y): (f64, f64) = (x.trim().parse().ok()?, y.trim().parse().ok()?);
            let (w, h): (f64, f64) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
            Some(WidgetBox {
                page,
                field,
                min: (x, y),
                size: (w, h),
            })
        })
        .collect()
}

/// How far outside a widget's edge a point must be before it counts as clear
/// of it, in canvas units. See [`WidgetBox::contains`].
const CLEARANCE: f64 = 8.0;

/// How far inside the sheet's own edge a candidate must stay, as a fraction of
/// the page. A click on the extreme margin is still on paper, but it is where
/// a popup would be repositioned and where a fit view is most likely to have
/// clipped the sheet against the viewport.
const INSET: f64 = 0.04;

/// **A point on the same sheet that is blank paper**, in canvas space.
///
/// `from` is the canvas-space centre of the widget the caller is about to
/// select; the returned point is somewhere near it that no widget occupies, so
/// that a primary click there CLEARS the form selection (`select_click`'s
/// table, quoted in the module header) and the caller's next click on `from`
/// is a genuine change.
///
/// ## The search, and why it is a ring rather than one offset
///
/// Candidates are tried in order: **above** the widget first, then below, then
/// left, then right, each at two distances. Above is first because it is the
/// direction with the most room in the two shapes this is used on — a field
/// placed at 55 % of the page height, and one placed at the sweep's
/// `--doc-point` near the bottom edge of a landscape CAD sheet. Each candidate
/// must clear every box in `boxes` on page `page` by [`CLEARANCE`] and sit at
/// least [`INSET`] of the sheet in from every edge.
///
/// Returns `None` when every candidate is occupied or off-sheet — which the
/// caller must report as a SKIP, not a failure: a document whose widgets crowd
/// out every candidate is a fixture problem, and the gesture under test was
/// never attempted.
#[must_use]
pub fn blank_canvas_point(
    boxes: &[WidgetBox],
    geometry: PageGeometry,
    page: usize,
    from: (f64, f64),
) -> Option<(f64, f64)> {
    let inset_x = geometry.width_pt * INSET;
    let inset_y = geometry.height_pt * INSET;
    // Two step lengths, both a fraction of the sheet so they scale with it: a
    // near one that stays in the same neighbourhood as the widget (and so is
    // certainly still on screen if the widget was), and a far one for when the
    // near ring is occupied.
    let near = (geometry.width_pt * 0.06, geometry.height_pt * 0.06);
    let far = (geometry.width_pt * 0.12, geometry.height_pt * 0.12);

    let candidates = [
        (0.0, -near.1),
        (0.0, near.1),
        (-near.0, 0.0),
        (near.0, 0.0),
        (0.0, -far.1),
        (0.0, far.1),
        (-far.0, 0.0),
        (far.0, 0.0),
    ];

    candidates
        .into_iter()
        .map(|(dx, dy)| (from.0 + dx, from.1 + dy))
        .find(|&p| {
            p.0 >= inset_x
                && p.0 <= geometry.width_pt - inset_x
                && p.1 >= inset_y
                && p.1 <= geometry.height_pt - inset_y
                && !boxes
                    .iter()
                    .any(|b| b.page == page && b.contains(p, CLEARANCE))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geometry() -> PageGeometry {
        PageGeometry {
            width_pt: 1584.0,
            height_pt: 1224.0,
        }
    }

    fn boxed(page: usize, min: (f64, f64), size: (f64, f64)) -> WidgetBox {
        WidgetBox {
            page,
            field: "Text1".to_owned(),
            min,
            size,
        }
    }

    /// The sweep's own case: one field, and the first candidate — straight up —
    /// is clear, so that is what comes back.
    #[test]
    fn the_first_candidate_is_taken_when_it_is_clear() {
        let boxes = vec![boxed(0, (1140.6, 1141.8), (160.0, 20.0))];
        let point = blank_canvas_point(&boxes, geometry(), 0, (1220.6, 1151.8))
            .expect("a 1584x1224 sheet with one 160x20 widget has blank paper on it");
        assert!(
            !boxes[0].contains(point, CLEARANCE),
            "{point:?} must be clear of the widget it is meant to deselect"
        );
        assert!((point.1 - (1151.8 - 1224.0 * 0.06)).abs() < 0.001);
    }

    /// A second widget sitting on the first candidate pushes the search on,
    /// which is the whole reason the census is consulted rather than an offset
    /// being assumed.
    #[test]
    fn an_occupied_candidate_is_skipped() {
        let target = boxed(0, (700.0, 600.0), (160.0, 20.0));
        // Directly above the target, where the first candidate lands.
        let blocker = boxed(0, (700.0, 600.0 - 1224.0 * 0.06), (160.0, 20.0));
        let boxes = vec![target.clone(), blocker.clone()];
        let point = blank_canvas_point(&boxes, geometry(), 0, target.centre())
            .expect("the ring has seven more candidates");
        assert!(!blocker.contains(point, CLEARANCE));
        assert!(!target.contains(point, CLEARANCE));
    }

    /// A widget on ANOTHER page constrains nothing: the census is document-wide
    /// and the click is on one sheet.
    #[test]
    fn boxes_on_other_pages_are_ignored() {
        let target = boxed(0, (700.0, 600.0), (160.0, 20.0));
        let elsewhere = boxed(3, (700.0, 600.0 - 1224.0 * 0.06), (160.0, 20.0));
        let point =
            blank_canvas_point(&[target.clone(), elsewhere], geometry(), 0, target.centre())
                .expect("page 3's widgets do not occupy page 0's paper");
        assert!((point.1 - (610.0 - 1224.0 * 0.06)).abs() < 0.001);
    }

    /// The census parser, against a line in the exact shape the canvas writes.
    #[test]
    fn the_census_line_parses() {
        let trace = Trace::parse(
            "pdfcer-diag form-target page=0 field=Text1 widget=0 rect=(1140.6,1141.8)+(160.0,20.0)\n",
            "pdfcer-diag",
        );
        let boxes = targets(&trace);
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].page, 0);
        assert_eq!(boxes[0].field, "Text1");
        assert!((boxes[0].centre().0 - 1220.6).abs() < 0.001);
        assert!((boxes[0].centre().1 - 1151.8).abs() < 0.001);
    }
}
