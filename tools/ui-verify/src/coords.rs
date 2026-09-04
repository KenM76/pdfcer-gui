//! **The coordinate seam.** Document space in, screen pixels out — and the
//! rule that a check may never write down the screen pixels itself.
//!
//! # The rule
//!
//! > **Scripts are written in document coordinates. Never in absolute screen
//! > coordinates.**
//!
//! `PROJECT_PLAN.md` §4.2 lists this as one of three prerequisites that
//! "belong in S1, not later", ahead of the panel-flexibility work that would
//! otherwise invalidate it.
//!
//! # Why the rule is not merely tidiness
//!
//! Two reasons, and the second is the one that has already cost this project
//! real time.
//!
//! **1. Every screen coordinate in this application is about to become
//! variable.** `MODES_AND_PANELS.md` puts multi-column docks, a tab-overflow
//! menu, named workspaces, collapse-to-icon-rail and eventually tear-out on
//! the roadmap. Each one changes where the canvas begins. A harness whose
//! scripts say `click at 819,513` is a harness that has to be re-baselined
//! after every layout change — and the re-baselining is manual, so in practice
//! it does not happen and the checks quietly stop testing anything.
//!
//! **2. A stale screen coordinate is symptom-identical to a broken coordinate
//! conversion.** This is the part that matters. When a click lands on empty
//! canvas instead of on the object, the trace shows a hit test returning
//! nothing — which is *exactly* what a genuinely broken document-to-screen
//! conversion looks like. The recorded outcome in this codebase was a
//! coordinate-space defect filed and then retracted: the conversion was
//! correct all along and the harness was pointing at the wrong pixel.
//!
//! A false defect is worse than no defect. It consumes an investigation, and
//! it teaches everyone involved to distrust the harness — after which the
//! harness's true reports get discounted too.
//!
//! So the fix is structural rather than advisory: a check *cannot* write a
//! screen coordinate, because [`ScreenPoint`] has private fields and no
//! constructor. The only way to obtain one is to start from a [`DocPoint`] and
//! pass it through a [`CanvasMapping`] the application itself supplied this
//! run, and then through the [`WindowFrame`] measured from the live window.
//! If the application did not supply a mapping, there is no [`ScreenPoint`],
//! and the check SKIPs saying so — which is the honest answer, and is not the
//! same answer as "the click missed".
//!
//! # The four spaces
//!
//! ```text
//!   DocPoint          PDF user space. Page index + (x, y) in points,
//!                     origin BOTTOM-LEFT, y growing UP.
//!                     ── written by the check author. Stable across every
//!                        layout change, every window size, every DPI.
//!        │  CanvasMapping::doc_to_window   (needs: the page's height, and the
//!        ▼                                  canvas rect + zoom from the trace)
//!   WindowPoint       egui logical points, relative to the window's CLIENT
//!                     origin, y growing DOWN.
//!                     ── the space the application's own trace speaks in.
//!        │  WindowFrame::to_screen         (needs: the live window's client
//!        ▼                                  origin and its DPI scale)
//!   ScreenPoint       Physical desktop pixels. The only thing the OS input
//!                     API accepts, and the only space a check may not name.
//! ```
//!
//! (The fourth is [`crate::geom::PixRect`], the screenshot's own pixel space,
//! which shares an origin with the captured region rather than with the
//! desktop. It is handled by [`WindowFrame::client_pixels`].)
//!
//! # The y-flip happens exactly once
//!
//! PDF user space has its origin at the bottom-left with y growing up. egui
//! has its origin at the top-left with y growing down. That flip is performed
//! in [`CanvasMapping::doc_to_window`] and nowhere else in this crate. Every
//! codebase that flips y in two places eventually flips it twice on one path,
//! and the resulting bug is a mirror image that looks like a rounding problem.
//!
//! # What is verified, and what is assumed
//!
//! Stated separately, because this project has recorded the cost of a comment
//! that asserts a cause nobody tested.
//!
//! **Verified** (against `D:\Dev\pdfcer`'s trace and its `tools/gui-drive.ps1`
//! notes): the canvas trace line carries `rect=` (the image rect in window
//! logical points) and `zoom=`, and the conversion
//! `window = rect.min + canvas_point * zoom` with `canvas_y = page_height -
//! pdf_y` is the one that script's own header documents for picking points.
//!
//! **Assumed, and NOT verified here**: that `rect=` already accounts for the
//! scroll offset — i.e. that when the view is scrolled, the image rect moves
//! rather than the content moving inside a fixed rect. The canvas line also
//! carries an `off=` scroll offset, and if the assumption is wrong, every
//! conversion is wrong by exactly that offset whenever the view is scrolled.
//!
//! [`CanvasMapping::scroll`] exists to hold that correction, defaults to zero,
//! and is applied if a profile supplies it. **The falsification test**, for
//! whoever gets there first: drive the same document point twice, once
//! unscrolled and once after a `Scroll` step, and compare the resulting
//! `vector-click canvas=` values. If they differ by the scroll amount, the
//! assumption is wrong and the profile should name the scroll field. Until
//! someone runs it, the checks stay at scroll zero — which is why every check
//! in [`crate::checks`] operates on an unscrolled view and says so.

use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::profile::Vocabulary;
use crate::trace::Trace;

/// A point in the document: PDF user space, origin bottom-left, y up.
///
/// **This is the only spatial literal a check may write.** It is stable under
/// every layout change the roadmap contemplates, because it describes the
/// document rather than the window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DocPoint {
    /// Zero-based page index.
    pub page: usize,
    /// Points from the left edge of the page.
    pub x: f64,
    /// Points from the **bottom** edge of the page. See the module docs on the
    /// y-flip.
    pub y: f64,
}

impl DocPoint {
    /// A document point.
    #[must_use]
    pub const fn new(page: usize, x: f64, y: f64) -> Self {
        Self { page, x, y }
    }
}

/// A page's size in PDF points, from the document rather than from the window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageGeometry {
    /// Page width, points.
    pub width_pt: f64,
    /// Page height, points. Needed for the y-flip, and for nothing else.
    pub height_pt: f64,
}

/// A point in egui logical points, relative to the window's client origin.
///
/// Fields are readable — a failure report should be able to say *where* it
/// clicked — but the type is only ever produced by
/// [`CanvasMapping::doc_to_window`]. There is deliberately no public
/// constructor: a check that could build one directly could write a window
/// coordinate literal, which is the same defect as a screen coordinate literal
/// wearing a different hat.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowPoint {
    x: f32,
    y: f32,
}

impl WindowPoint {
    /// Logical points from the client area's left edge.
    #[must_use]
    pub fn x(&self) -> f32 {
        self.x
    }
    /// Logical points from the client area's top edge.
    #[must_use]
    pub fn y(&self) -> f32 {
        self.y
    }
}

/// A point in physical desktop pixels — what the OS input API accepts.
///
/// Private fields, no constructor, produced only by [`WindowFrame::to_screen`].
/// That is the enforcement mechanism for this module's rule; everything above
/// is the explanation of why it is worth enforcing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenPoint {
    x: i32,
    y: i32,
}

impl ScreenPoint {
    /// Desktop x, pixels.
    #[must_use]
    pub fn x(&self) -> i32 {
        self.x
    }
    /// Desktop y, pixels.
    #[must_use]
    pub fn y(&self) -> i32 {
        self.y
    }
}

/// Document space to window space, for one page, as the application laid it
/// out on the frame the trace was written.
///
/// Constructed from a trace, never by hand. If the application did not emit
/// what this needs, construction fails and the caller SKIPs — see
/// [`CanvasMapping::from_trace`].
#[derive(Clone, Copy, Debug)]
pub struct CanvasMapping {
    /// The canvas image rect, in window logical points, as the application
    /// reported it.
    pub image_rect: LRect,
    /// The application's view magnification: document points per logical
    /// point.
    pub zoom: f32,
    /// A scroll correction, in logical points, subtracted from the result.
    ///
    /// Zero unless a profile names a scroll field. See the module docs'
    /// "assumed, and NOT verified" section — including the experiment that
    /// would settle it.
    pub scroll: Pt,
    /// The page's own size, from the document.
    pub page: PageGeometry,
    /// Which page [`Self::image_rect`] shows.
    pub page_index: usize,
}

impl CanvasMapping {
    /// Build a mapping from the application's most recent canvas trace line.
    ///
    /// # Errors
    ///
    /// Every failure here is a **precondition**, not an assertion: it means
    /// the harness cannot aim at all, so the caller must SKIP rather than FAIL.
    /// Each message names the specific missing field, because "no mapping" is
    /// useless to whoever has to add it.
    pub fn from_trace(
        trace: &Trace,
        vocab: &Vocabulary,
        page: PageGeometry,
        page_index: usize,
    ) -> Result<Self> {
        let line = trace.last(vocab.canvas_event).ok_or_else(|| {
            Error::new(format!(
                "the trace carries no `{}` event, so the harness has no canvas rect to \
                 convert document coordinates against. Either the diagnostic variable did \
                 not reach the process, or this build does not trace its canvas layout yet.",
                vocab.canvas_event
            ))
        })?;

        let image_rect = line.get_rect(vocab.canvas_rect_field).ok_or_else(|| {
            Error::new(format!(
                "the `{}` event has no parsable `{}=` field (fields present: {:?}). \
                 Without the canvas rect there is no document-to-window conversion.",
                vocab.canvas_event,
                vocab.canvas_rect_field,
                line.field_names()
            ))
        })?;

        // ═══════════════════════════════════════════════════════════════════
        // ★★★ THE PAGE THE APPLICATION IS SHOWING, AGAINST THE PAGE THE CALLER
        // ASKED FOR — 2026-09-04
        // ═══════════════════════════════════════════════════════════════════
        //
        // `doc_to_window` below refuses when a point's page differs from
        // `self.page_index`, and its doc comment explains why: *"converting it
        // against the wrong page's rect would produce a confident, wrong
        // click."* Entirely correct, and **it could never fire**, because every
        // caller does this:
        //
        //     CanvasMapping::from_trace(&trace, vocab, page, target.page)
        //                                                   ^^^^^^^^^^^
        //
        // — the mapping is told its page index BY THE POINT it is about to
        // check. `p.page != self.page_index` was comparing a number against
        // itself. A tautology wearing a guard's clothing.
        //
        // ⇒ On 2026-09-04 a sweep ran with `--doc-point 1,300,400` against a
        // ONE-PAGE fixture. Page `1` is the second page. Nothing refused it,
        // and it produced **six confident, detailed, plausible failure
        // reports** — resize, rotate, shift-constrained resize, multi-node
        // move and two more — each naming real functions and real trace
        // events. **Four were filed as defects.** Re-run on the same fixture at
        // the same zoom with a valid page, every one passes.
        //
        // ★ This is the third time this project has recorded the same shape: a
        // **proxy condition** standing in for the real one, where the stand-in
        // is derived from the thing it is meant to be checking. The rule it
        // keeps re-learning: *ask what the mechanism READS.* A guard reads a
        // number; the question is where that number came from.
        //
        // The application publishes the page it is actually showing on the same
        // line as the rect. That is an INDEPENDENT quantity, and comparing
        // against it is a real comparison.
        if let Some(shown) = line.get_usize("page")
            && shown != page_index
        {
            return Err(Error::new(format!(
                "the harness was asked to convert a point on page {page_index} (0-based) and \
                 the application is showing page {shown}.\n  \
                 Refused rather than converted: the rect on the `{}` line describes the page on \
                 SCREEN, so mapping another page's coordinates through it yields a click that \
                 is plausible, precise and in the wrong place — which is indistinguishable from \
                 a broken feature and costs an investigation to disprove.\n  \
                 ★ PAGE IS 0-BASED. If this came from `--doc-point PAGE,X,Y`, the first page is \
                 `0`.",
                vocab.canvas_event
            )));
        }

        if !image_rect.is_substantial() {
            return Err(Error::new(format!(
                "the canvas rect {image_rect:?} has no area — the canvas was not laid out on \
                 the traced frame. Converting against it would produce a click at the window \
                 corner, which lands on the wrong widget rather than on nothing."
            )));
        }

        let zoom = line.get_f32(vocab.canvas_zoom_field).ok_or_else(|| {
            Error::new(format!(
                "the `{}` event has no parsable `{}=` field (fields present: {:?}).",
                vocab.canvas_event,
                vocab.canvas_zoom_field,
                line.field_names()
            ))
        })?;

        if !(zoom.is_finite() && zoom > 0.0) {
            return Err(Error::new(format!(
                "traced zoom {zoom} is not a usable magnification."
            )));
        }

        // Zero unless the profile names a scroll field AND the line carries a
        // parsable value for it. A named-but-absent field is deliberately NOT
        // an error: it means the application did not report a scroll this
        // frame, which is the common case and is correctly read as zero. A
        // named-but-UNPARSABLE field would be worth complaining about, but it
        // is indistinguishable from absent here, which is one more reason the
        // falsification experiment in the module docs is worth running before
        // anyone relies on this path.
        let scroll = vocab
            .canvas_scroll_field
            .and_then(|field| line.get_vec2(field))
            .unwrap_or(Pt::new(0.0, 0.0));

        Ok(Self {
            image_rect,
            zoom,
            scroll,
            page,
            page_index,
        })
    }

    /// Convert a document point to a window point.
    ///
    /// The whole conversion, and the only place this crate flips y:
    ///
    /// ```text
    /// canvas_x = doc.x                       (PDF x and canvas x agree)
    /// canvas_y = page_height - doc.y         (the flip: PDF y is up, egui y is down)
    /// window   = image_rect.min + canvas * zoom - scroll
    /// ```
    ///
    /// # Errors
    ///
    /// * The point is on a page this mapping does not describe. Silently
    ///   converting it against the wrong page's rect would produce a
    ///   confident, wrong click.
    /// * The point is outside the page box. That is almost always a typo in
    ///   the check, and it is caught here rather than becoming a click on
    ///   whatever chrome happens to be there.
    /// * The result is outside the canvas rect — the point is real but
    ///   currently scrolled or zoomed out of view. Refused rather than
    ///   clamped: a clamped click lands on the canvas edge and hit-tests
    ///   nothing, which reads as "the object is not selectable" instead of
    ///   "the harness could not reach it".
    pub fn doc_to_window(&self, p: DocPoint) -> Result<WindowPoint> {
        if p.page != self.page_index {
            return Err(Error::new(format!(
                "the mapping describes page {} and the point is on page {}",
                self.page_index, p.page
            )));
        }
        if p.x < 0.0 || p.y < 0.0 || p.x > self.page.width_pt || p.y > self.page.height_pt {
            return Err(Error::new(format!(
                "document point ({}, {}) is outside the {}x{} pt page box",
                p.x, p.y, self.page.width_pt, self.page.height_pt
            )));
        }

        let canvas_x = p.x as f32;
        let canvas_y = (self.page.height_pt - p.y) as f32; // the flip, once

        let wx = self.image_rect.min.x + canvas_x * self.zoom - self.scroll.x;
        let wy = self.image_rect.min.y + canvas_y * self.zoom - self.scroll.y;

        if wx < self.image_rect.min.x
            || wx > self.image_rect.max.x
            || wy < self.image_rect.min.y
            || wy > self.image_rect.max.y
        {
            return Err(Error::new(format!(
                "document point ({}, {}) maps to window ({wx:.1}, {wy:.1}), which is outside \
                 the canvas rect {:?}. The point is not currently on screen: scroll or zoom to \
                 fit before driving it. Refusing to clamp — a clamped click lands on the canvas \
                 edge and hit-tests nothing, which reads as a broken feature.",
                p.x, p.y, self.image_rect
            )));
        }

        Ok(WindowPoint { x: wx, y: wy })
    }

    /// **The same conversion, for a point deliberately OUTSIDE the page box.**
    ///
    /// \ ★★★ Why this exists, and why it is a second entry point rather than a flag
    ///
    /// `OPERATOR_REQUESTS.md` O92: *"we should be able to select things offside
    /// of the page, especially since I sometimes drop objects there, and when I
    /// do I can't get them back."* A check for that has to drive a rubber band
    /// **into the grey margin**, and [`Self::doc_to_window`] refuses every point
    /// outside the media box — correctly, for every other caller.
    ///
    /// That refusal is not softened and its reasoning is untouched: a point
    /// outside the page is almost always a check aiming at the wrong sheet or
    /// converting against the wrong geometry, and silently allowing it would let
    /// those land somewhere plausible and wrong. **This is the narrow, named
    /// exception**, and a caller has to say the words to get it.
    ///
    /// \ ★★ What is NOT relaxed
    ///
    /// A bound stays, and it is the **viewport's**, passed in — see the ★★★
    /// comment in the body for why bounding against `image_rect` instead
    /// rejects the entire class this function exists for. A point off the page
    /// can still be clicked, because the grey margin is part of the canvas
    /// widget; a point off the *viewport* cannot be clicked by anybody, and
    /// clamping it would land on an edge and hit-test nothing.
    ///
    /// ★ Coordinates may be negative. The flip is the same one line, and a
    /// negative `x` produces a window position left of the page's own origin,
    /// which is exactly where a dropped object sits.
    ///
    /// \ Errors
    ///
    /// Wrong page, or a point that is off the **viewport** rather than merely
    /// off the page.
    pub fn doc_to_window_off_page(&self, p: DocPoint, viewport: LRect) -> Result<WindowPoint> {
        if p.page != self.page_index {
            return Err(Error::new(format!(
                "the mapping describes page {} and the point is on page {}",
                self.page_index, p.page
            )));
        }
        let canvas_x = p.x as f32;
        let canvas_y = (self.page.height_pt - p.y) as f32; // the flip, once
        let wx = self.image_rect.min.x + canvas_x * self.zoom - self.scroll.x;
        let wy = self.image_rect.min.y + canvas_y * self.zoom - self.scroll.y;
        // ★★★ **THE VIEWPORT, NOT `image_rect`** — and getting this wrong was
        // the first thing that happened.
        //
        // `image_rect` is the **page's** rectangle. Every off-page point is
        // outside it by construction, so bounding against it rejects the entire
        // class this function exists for — and rejects it with a message about
        // "not enough margin on screen", which is plausible and completely
        // wrong.
        //
        // The bound that means something is the scroll area the page is drawn
        // inside, published as `ui-rect name=canvas-viewport`. Its grey margin
        // IS where a dropped object lives, and a point outside it cannot be
        // clicked by anybody.
        if wx < viewport.min.x || wx > viewport.max.x || wy < viewport.min.y || wy > viewport.max.y
        {
            return Err(Error::new(format!(
                "off-page document point ({}, {}) maps to window ({wx:.1}, {wy:.1}), which is \
                 outside the canvas VIEWPORT {viewport:?} — not merely outside the page \
                 ({:?}), which this conversion permits. There is not enough margin on screen \
                 to reach it: zoom out so the sheet occupies less of the viewport, or choose a \
                 point nearer it. Refusing to clamp, for the reason `doc_to_window` gives.",
                p.x, p.y, self.image_rect
            )));
        }
        Ok(WindowPoint { x: wx, y: wy })
    }
}

/// The live window's measured geometry: where its client area sits on the
/// desktop, and how many physical pixels one logical point is.
///
/// Measured per run, never assumed. A window that the window manager placed
/// somewhere other than where it was asked, a DPI change between runs, a
/// second monitor at a different scale — all of these change these numbers,
/// and all of them are invisible to a harness that hard-codes them.
#[derive(Clone, Copy, Debug)]
pub struct WindowFrame {
    /// Desktop position of the client area's top-left corner, in pixels.
    pub client_origin: (i32, i32),
    /// Client area size, in **physical pixels**.
    pub client_size: (u32, u32),
    /// Physical pixels per logical point — 1.0 at 100%, 1.5 at 150%.
    pub scale: f32,
}

impl WindowFrame {
    /// Convert a window point to a screen point.
    ///
    /// The only constructor of [`ScreenPoint`] in the crate.
    #[must_use]
    pub fn to_screen(&self, p: WindowPoint) -> ScreenPoint {
        ScreenPoint {
            x: self.client_origin.0 + (p.x * self.scale).round() as i32,
            y: self.client_origin.1 + (p.y * self.scale).round() as i32,
        }
    }

    /// Convert a rectangle the **application** reported into a rectangle in
    /// the **captured screenshot**.
    ///
    /// This is the counterpart of [`Self::to_screen`] for areas rather than
    /// points, and it is what makes the application's own `ui-rect`
    /// declarations usable as a pixel check's region source.
    ///
    /// ## The two spaces, and the one thing that separates them
    ///
    /// A traced rect is in **window logical points**, relative to the client
    /// area's top-left corner — that is where egui's coordinate system starts,
    /// so `[[8.0 8.0] - [1092.0 792.0]]` means "eight points in from the left
    /// and top of the client area". A capture taken by [`crate::capture`] is
    /// of exactly the client area, so its pixel origin is the *same* corner.
    ///
    /// Therefore the whole conversion is the DPI scale. No origin term appears
    /// here, and its absence is deliberate rather than forgotten: adding
    /// `client_origin` would be correct for a desktop-space rectangle and is
    /// wrong for this one, and the resulting regions would be offset by the
    /// window's position on the desktop — which is zero on a maximised window
    /// at the top-left of the primary monitor, i.e. exactly the configuration
    /// a developer would test on.
    ///
    /// ## Clipping, and what an empty result means
    ///
    /// The result is clamped to the capture. A region that lies **entirely**
    /// outside it comes back with zero area, and the caller must treat that as
    /// a finding rather than as a measurement: the application declared a
    /// region that is not on screen, which is the clipped-out-of-its-pane
    /// defect `PROJECT_PLAN.md` §4.2 prerequisite 2 cites two recorded cases
    /// of. It is emphatically not "contrast 1.0".
    #[must_use]
    pub fn logical_to_capture_pixels(&self, r: LRect) -> PixRect {
        let (cw, ch) = self.client_size;
        let to_px = |v: f32| -> u32 {
            if v <= 0.0 {
                0
            } else {
                (v * self.scale).round() as u32
            }
        };
        let x0 = to_px(r.min.x).min(cw);
        let y0 = to_px(r.min.y).min(ch);
        let x1 = to_px(r.max.x).min(cw);
        let y1 = to_px(r.max.y).min(ch);
        PixRect::new(x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0))
    }

    /// The **centre of a rectangle the application declared this frame**, as a
    /// screen point the input driver can be aimed at.
    ///
    /// # Why this does not break this module's rule
    ///
    /// The rule is that a check may write only a [`DocPoint`] or a
    /// [`crate::geom::FracRect`] literal, and the enforcement is that
    /// [`ScreenPoint`] has no public constructor. That rule is about
    /// **literals** — a number somebody typed, which was true when they typed
    /// it and goes quietly wrong the first time a panel moves.
    ///
    /// An [`LRect`] parsed out of a `ui-rect` line is the opposite of a
    /// literal. It is the application stating, on the frame it drew the
    /// control, where it put it; there is no interval between the measurement
    /// and the claim, so there is nothing for a layout change to invalidate. A
    /// ribbon that reflows, collapses to an icon rail or gains an eighth tab
    /// publishes different rects and a check aiming through this function
    /// follows it with no edit. That is region source 1 from [`crate::profile`]
    /// — the *preferred* source — applied to aiming rather than to measuring.
    ///
    /// So the constructor stays private and this is the second and last way
    /// through it. [`Self::layout_probe_point`] is the first, and the two are
    /// different in kind: that one is an assumption about where the canvas
    /// probably is, this one is a fact the application published.
    ///
    /// # Why the centre
    ///
    /// A control's rect includes its border stroke and its rounded corners, so
    /// a point on the edge can miss the hit area or land on the neighbour a
    /// gutter away. The centre is the only point in a convex control that is
    /// inside it under every corner radius, padding and rounding error the
    /// theme can produce.
    ///
    /// # Degenerate rects
    ///
    /// A zero-area rect still yields a point, and that is deliberate: the
    /// caller is the one who knows whether "the application declared this
    /// control at no size" is a SKIP or a FAIL, and silently returning `None`
    /// here would turn a finding into a missing measurement. Callers are
    /// expected to test the rect before aiming at it —
    /// [`crate::geom::LRect::is_substantial`] is the question.
    #[must_use]
    /// A point a fixed number of screen pixels from another one.
    ///
    /// # ★ Why this is on `Frame` and not a method on `ScreenPoint`
    ///
    /// `coords`' standing rule is that **a coordinate is produced by a
    /// conversion and never assembled**, and a bare `ScreenPoint::offset` would
    /// be exactly the assembly the rule forbids — it would let any caller
    /// invent a screen position out of arithmetic and a hope.
    ///
    /// This is deliberately narrower: a *displacement in screen pixels from a
    /// point the application itself published*. That is what a drag is, and what
    /// a sweep looking for a neighbouring control is. Living on `Frame` keeps it
    /// beside `declared_at`, which is the other member of the same family —
    /// both take an application-supplied anchor and move within it.
    ///
    /// It does not clamp. A sweep that walks off the window is caught by the
    /// application not responding, which is the honest answer; clamping would
    /// silently retry the same point and report a false negative.
    #[allow(clippy::cast_possible_truncation)]
    pub fn offset_from(&self, from: ScreenPoint, dx: f32, dy: f32) -> ScreenPoint {
        let _ = self;
        ScreenPoint {
            x: from.x() + dx.round() as i32,
            y: from.y() + dy.round() as i32,
        }
    }

    pub fn declared_center(&self, r: LRect) -> ScreenPoint {
        self.to_screen(WindowPoint {
            x: (r.min.x + r.max.x) / 2.0,
            y: (r.min.y + r.max.y) / 2.0,
        })
    }

    /// A point **inside** a declared rectangle, given as fractions of its
    /// width and height.
    ///
    /// `(0.5, 0.5)` is [`Self::declared_center`]; `(0.75, 0.5)` is
    /// three-quarters across, vertically centred.
    ///
    /// # Why this exists rather than callers building a `WindowPoint`
    ///
    /// Because `WindowPoint`'s fields are private and deliberately so — this
    /// module's rule is that a coordinate is produced by a conversion, never
    /// assembled — and a check that needs to aim at *part* of a control was
    /// otherwise stuck reaching for the centre.
    ///
    /// The case that forced it is real rather than hypothetical:
    /// `checks::pages_drag` has to release the pointer over the **right half**
    /// of a page tile, because the Pages panel resolves the nearer vertical
    /// edge and the two halves mean two different landing boundaries. Aiming
    /// at the centre would be aiming at the one place the answer is undefined.
    ///
    /// The fractions are **not** clamped. A caller asking for `1.5` means a
    /// point outside the control and is entitled to it — that is how a check
    /// aims *beside* a widget rather than at it — and silently correcting it
    /// would produce a click at an edge the caller did not choose, which is
    /// the class of failure that reads as the application misbehaving.
    #[must_use]
    pub fn declared_at(&self, r: LRect, fx: f32, fy: f32) -> ScreenPoint {
        self.to_screen(WindowPoint {
            x: r.min.x + (r.max.x - r.min.x) * fx,
            y: r.min.y + (r.max.y - r.min.y) * fy,
        })
    }

    /// The client area in **logical points**, which is the space `ui-rect`
    /// publishes in.
    ///
    /// # ★ Why this is not [`Self::client_pixels`] divided by the scale
    ///
    /// It is, arithmetically — and the point of having it as its own accessor
    /// is that a caller comparing a published rect against the window must not
    /// have to remember which of the two spaces it is in. `ui-rect` carries
    /// logical points; `client_pixels` carries desktop pixels; a check that
    /// compared one against the other would pass or fail by the display's scale
    /// factor, which is a property of the machine the suite happens to run on.
    ///
    /// Used by the "is it actually visible" assertions. **Drawn is not seen** —
    /// a widget below the fold publishes a perfectly good rect.
    #[must_use]
    pub fn client_logical(&self) -> LRect {
        let w = self.client_size.0 as f32 / self.scale;
        let h = self.client_size.1 as f32 / self.scale;
        LRect::new(crate::geom::Pt::new(0.0, 0.0), crate::geom::Pt::new(w, h))
    }

    /// The client area as a desktop-pixel rectangle, for the screen grabber.
    #[must_use]
    pub fn client_pixels(&self) -> PixRect {
        PixRect::new(
            self.client_origin.0.max(0) as u32,
            self.client_origin.1.max(0) as u32,
            self.client_size.0,
            self.client_size.1,
        )
    }

    /// The centre of the client area, as a screen point.
    ///
    /// # This is the ONE exception to this module's rule, and it is narrow
    ///
    /// Everything else in this crate refuses to turn a window-relative
    /// quantity into a screen point without a document-space mapping. This
    /// function does exactly that, and it exists for one reason: **some
    /// applications only report their layout when something happens.**
    ///
    /// The old GUI is one of them. Its `canvas` trace fires on
    /// `pressed || released || down || zoom` and on nothing else, so a freshly
    /// opened document produces no canvas rect at all — and without a canvas
    /// rect there is no document-to-window mapping, and without that mapping
    /// there is no click. The harness cannot aim until the application has
    /// spoken, and the application will not speak until the harness clicks.
    ///
    /// So: one **layout probe**. A click at the client-area centre, whose only
    /// purpose is to make the application report where its canvas is. It is
    /// explicitly not an assertion, it does not care what it hits, and the
    /// real document-space click follows it and replaces whatever it selected.
    ///
    /// ## The assumption it rests on, stated
    ///
    /// That the centre of the client area is the canvas. True of every layout
    /// in `MODES_AND_PANELS.md` — the document is the centre of a document
    /// editor — and it fails safe: if the probe lands on chrome, no canvas
    /// event appears, and the check SKIPs with a reason naming exactly that.
    /// It cannot produce a *wrong* mapping, only no mapping.
    ///
    /// ## The right fix — which has landed, for one of the two binaries
    ///
    /// The application should trace its canvas layout **unconditionally**, at
    /// least once per document open, rather than only on pointer events. Then
    /// the harness reads the rect from a quiet trace and the probe disappears.
    /// That was `PROJECT_PLAN.md` §4.3 requirement 1, discovered by building
    /// this harness rather than by reading the code, and the new application
    /// implements it: its `canvas` line is built every frame and emitted
    /// through a de-duplicating gate that is cleared on document open, so
    /// there is a line before any input is delivered. A run against it never
    /// reaches this function — confirmed by the absence of the "no `canvas`
    /// event yet" note in its reports.
    ///
    /// The probe stays for the **old** binary, which still traces only on
    /// pointer events and is still the thing the D1 reproduction is driven
    /// against. Deleting it would delete the acceptance evidence.
    #[must_use]
    pub fn layout_probe_point(&self) -> ScreenPoint {
        ScreenPoint {
            x: self.client_origin.0 + (self.client_size.0 / 2) as i32,
            y: self.client_origin.1 + (self.client_size.1 / 2) as i32,
        }
    }

    /// A frame describing a plain image rather than a live window.
    ///
    /// Used by `--image` mode, where the "window" is a PNG somebody captured
    /// earlier. Its origin is (0, 0) and its scale is 1.0, so fractional
    /// regions resolve against the image exactly as they would against a
    /// client area — which is what lets one region set be checked against a
    /// dated screenshot and against a live run without being written twice.
    #[must_use]
    pub fn for_image(width: u32, height: u32) -> Self {
        Self {
            client_origin: (0, 0),
            client_size: (width, height),
            scale: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Vocabulary;

    fn mapping() -> CanvasMapping {
        CanvasMapping {
            image_rect: LRect::new(Pt::new(200.0, 100.0), Pt::new(1000.0, 900.0)),
            zoom: 1.0,
            scroll: Pt::new(0.0, 0.0),
            page: PageGeometry {
                width_pt: 612.0,
                height_pt: 792.0,
            },
            page_index: 0,
        }
    }

    /// The y-flip, stated as a test so it cannot be "simplified" away: the
    /// TOP of the page (`y = height`) is the TOP of the canvas rect.
    #[test]
    fn the_top_of_the_page_is_the_top_of_the_canvas() {
        let m = mapping();
        let top = m.doc_to_window(DocPoint::new(0, 0.0, 792.0)).unwrap();
        assert_eq!((top.x(), top.y()), (200.0, 100.0));

        let bottom = m.doc_to_window(DocPoint::new(0, 0.0, 0.0)).unwrap();
        assert_eq!((bottom.x(), bottom.y()), (200.0, 892.0));
    }

    #[test]
    fn zoom_scales_from_the_canvas_origin() {
        let mut m = mapping();
        m.zoom = 2.0;
        // 100 pt right and 100 pt down from the page's top-left corner.
        let p = m.doc_to_window(DocPoint::new(0, 100.0, 692.0)).unwrap();
        assert_eq!((p.x(), p.y()), (400.0, 300.0));
    }

    #[test]
    fn a_point_on_another_page_is_refused_not_silently_converted() {
        let m = mapping();
        assert!(m.doc_to_window(DocPoint::new(3, 10.0, 10.0)).is_err());
    }

    #[test]
    fn a_point_outside_the_page_box_is_refused() {
        let m = mapping();
        assert!(m.doc_to_window(DocPoint::new(0, 5000.0, 10.0)).is_err());
    }

    /// Refusing rather than clamping is the whole point: a clamped click lands
    /// on the canvas edge, hit-tests nothing, and reads exactly like a broken
    /// feature.
    #[test]
    fn a_point_scrolled_out_of_the_canvas_is_refused_not_clamped() {
        let mut m = mapping();
        m.zoom = 4.0; // the page no longer fits the 800x800 canvas rect
        let err = m.doc_to_window(DocPoint::new(0, 600.0, 10.0)).unwrap_err();
        assert!(err.message().contains("outside the canvas rect"));
    }

    #[test]
    fn window_to_screen_applies_origin_and_dpi_scale() {
        let m = mapping();
        let f = WindowFrame {
            client_origin: (40, 60),
            client_size: (2400, 1500),
            scale: 1.5,
        };
        let w = m.doc_to_window(DocPoint::new(0, 0.0, 792.0)).unwrap();
        let s = f.to_screen(w);
        assert_eq!((s.x(), s.y()), (40 + 300, 60 + 150));
    }

    /// A traced region converts by scale alone, because the capture and the
    /// application share an origin.
    #[test]
    fn a_traced_region_scales_into_the_capture_without_an_origin_term() {
        let f = WindowFrame {
            client_origin: (400, 300),
            client_size: (2400, 1500),
            scale: 1.5,
        };
        let r = f.logical_to_capture_pixels(LRect::new(Pt::new(8.0, 8.0), Pt::new(108.0, 28.0)));
        assert_eq!(
            r,
            PixRect::new(12, 12, 150, 30),
            "the window's desktop position must not appear in a capture-relative rect"
        );
    }

    /// Aiming at a declared rect goes through the origin **and** the scale,
    /// unlike measuring one, which goes through the scale alone.
    ///
    /// The two conversions are next to each other and differ by exactly the
    /// origin term, which is the mistake worth pinning: a capture is of the
    /// client area and shares its corner, whereas the input driver works in
    /// desktop pixels and does not. Getting them the wrong way round is
    /// invisible on a maximised window at the top-left of the primary monitor —
    /// i.e. on the machine anybody would test it on.
    #[test]
    fn aiming_at_a_declared_rect_takes_its_centre_in_desktop_pixels() {
        let f = WindowFrame {
            client_origin: (400, 300),
            client_size: (2400, 1500),
            scale: 1.5,
        };
        // A control at 100..180 x 30..54 logical: centre (140, 42).
        let p = f.declared_center(LRect::new(Pt::new(100.0, 30.0), Pt::new(180.0, 54.0)));
        assert_eq!(
            (p.x(), p.y()),
            (400 + 210, 300 + 63),
            "a click is aimed in desktop pixels, so the window's position IS part of it"
        );
    }

    /// A region the application declared off the edge of its own client area
    /// must come back empty, so the caller can report "declared off-screen"
    /// rather than measure a sliver and call it a contrast.
    #[test]
    fn a_region_declared_off_the_capture_resolves_to_nothing() {
        let f = WindowFrame {
            client_origin: (0, 0),
            client_size: (800, 600),
            scale: 1.0,
        };
        let r =
            f.logical_to_capture_pixels(LRect::new(Pt::new(900.0, 10.0), Pt::new(1000.0, 30.0)));
        assert_eq!(r.area(), 0);
    }

    #[test]
    fn a_missing_canvas_event_is_a_precondition_error_naming_the_event() {
        let trace = Trace::parse("pdfcer-diag start argv1=None", "pdfcer-diag");
        let v = Vocabulary::pdfcer_gui();
        let err = CanvasMapping::from_trace(
            &trace,
            &v,
            PageGeometry {
                width_pt: 612.0,
                height_pt: 792.0,
            },
            0,
        )
        .unwrap_err();
        assert!(err.message().contains(v.canvas_event));
    }

    #[test]
    fn a_canvas_event_without_a_rect_names_the_fields_that_were_present() {
        let trace = Trace::parse("pdfcer-diag canvas zoom=1.0 sel=0", "pdfcer-diag");
        let v = Vocabulary::pdfcer_gui();
        let err = CanvasMapping::from_trace(
            &trace,
            &v,
            PageGeometry {
                width_pt: 612.0,
                height_pt: 792.0,
            },
            0,
        )
        .unwrap_err();
        assert!(err.message().contains("zoom"), "{}", err.message());
    }
}
