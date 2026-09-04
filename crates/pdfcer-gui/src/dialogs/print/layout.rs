//! `dialogs::print::layout` — where the print dialog's two columns go.
//!
//! # Why this is its own file
//!
//! R2 (no source file over 1,500 lines), reached honestly: `print/mod.rs` went
//! past the limit on 2026-09-03 while four operator-reported defects were fixed
//! in it. The seam is real rather than convenient — **geometry is a different
//! subject from the transaction.** `mod.rs` owns what a print job IS (the
//! device, the plan, the commit, what the operator is told afterwards); this
//! file owns only where things are drawn and how wide they are.
//!
//! # ★★★ THE ONE RULE THIS FILE EXISTS TO HOLD
//!
//! > **Every width and height is derived from the space OUTSIDE the scroll area
//! > and from constants. Nothing is measured from inside it.**
//!
//! Breaking that rule is what produced the operator's report of 2026-09-03 —
//! *"I have two scroll bars in the pop up window that won't go away no matter
//! how"* — and it was broken in three different ways in succession, each of
//! which read as obviously correct:
//!
//! 1. the content was forced to `ui.available_width()` measured **outside** the
//!    scroll area, which is one scrollbar wider than the viewport the content
//!    is actually laid into;
//! 2. `auto_shrink([false, false])` *defines* the content to be at least the
//!    pre-bar viewport, so the content was again always at least one bar too
//!    wide — by construction, at every window size;
//! 3. the two `item_spacing` gaps that `horizontal_top` inserts between three
//!    children were not in the arithmetic, and the preview's control strip was
//!    laid out **40 pt wider than its own column** and pushed the total out.
//!
//! Each of those raised a horizontal bar; a horizontal bar consumes height;
//! that raised a vertical bar; the vertical bar consumed width, which kept the
//! horizontal one. **The two bars were each other's cause**, which is why no
//! amount of resizing dismissed them.
//!
//! ★★ And the failure was INVERTED, which walking the size series found and a
//! single screenshot would not have: bars at 1000x760 and 1300x900 where
//! nothing needed scrolling, and **no bar at all** at 700x520 where the Paper
//! section was clipped and unreachable.
//!
//! # The oracle
//!
//! `ui-verify`'s `print_dialog_body_does_not_deadlock_its_scrollbars`, which
//! reads egui's own `content_size` and `inner_rect` out of a running frame via
//! the `print-body` trace line. It was falsified by planting cause 1 back in.
//! Nothing in a unit test can produce those numbers, and no screenshot can say
//! which of the three causes is the live one — the unit tests in `mod.rs` pin
//! only the relationships that are genuinely between our own constants.

use egui::Ui;

use super::{PrintDialog, preview};
use crate::app::state::OpenDoc;
use crate::dialogs::print::spooler::Job;
use crate::text::print as t;

/// The **narrowest** the options column may be squeezed to, in egui points.
///
/// A floor, not a width. It used to be a fixed 400 pt whose stated reason was
/// that *"a fixed width is what gives the horizontal scrollbar something stable
/// to measure"* — and that reasoning is what produced the scrollbar deadlock
/// [`PrintDialog::body`] documents. The scrollbar does not need a stable number
/// to measure; it needs to be told the truth about how wide the content is.
///
/// Sized to hold the longest radio label in the three tabs without wrapping,
/// which is what makes it a floor worth having: below this the options start
/// reflowing and the tab strip wraps, and scrolling is the better answer.
pub(super) const OPTIONS_COLUMN_MIN_WIDTH_PTS: f32 = 400.0;

/// The **narrowest** the preview column may be dragged to, in egui points.
///
/// Below this the sheet is too small to judge a margin or a clipped edge by,
/// which is the only reason to look at a print preview. The operator can still
/// collapse the dialog itself; what they cannot do is drag the preview into a
/// sliver by accident and be left with a control that has no visible grab.
pub(super) const PREVIEW_MIN_WIDTH_PTS: f32 = 220.0;

/// Where the splitter sits on a freshly opened dialog, and where a double-click
/// on it returns to.
///
/// The same 340 pt the column was fixed at before it became draggable, so the
/// dialog opens looking exactly as it did and the new freedom is opt-in.
pub(super) const PREVIEW_DEFAULT_WIDTH_PTS: f32 = 340.0;

/// The splitter's hit width, in egui points.
///
/// Wider than the line it draws, on purpose: the line is 1-2 pt and a 1 pt drag
/// target is not a drag target. 8 pt is the smallest band that can be hit
/// reliably without hunting, and it is what the dock's own splitter uses.
pub(super) const SPLITTER_WIDTH_PTS: f32 = 8.0;

/// How far the splitter's drawn line stops short of the column's ends.
///
/// Purely visual: a divider that runs edge to edge reads as a border round the
/// preview rather than as a control between two things.
pub(super) const SPLITTER_INSET_PTS: f32 = 4.0;

/// The space reserved out of the body so that a scrollbar's appearance on one
/// axis cannot raise one on the other.
///
/// # ★★★ This constant is the fix for the operator's "two scroll bars"
///
/// Wider than [`SCROLLBAR_WIDTH_PTS`] on purpose. egui reserves slightly more
/// than the bar's drawn width for a solid bar — measured at **14 pt** for a
/// 10 pt bar, by reading `available_width` inside the scroll area against the
/// width handed to it. A reservation equal to the drawn width would leave the
/// content one or two points wider than the viewport, which is a scrollbar just
/// as surely as a hundred points would be.
///
/// 16 is that measurement rounded up, and the rounding is the point: this
/// number's job is to be **comfortably more** than whatever egui takes, so that
/// the common case has strictly less content than viewport and neither bar is
/// drawn at all. Being a few points generous costs a few unused points at the
/// window edge; being a point mean costs a scrollbar that cannot be dismissed.
pub(super) const SCROLLBAR_ALLOWANCE_PTS: f32 = 16.0;

/// The narrowest the body's content may become before it scrolls, in egui
/// points: both column floors plus the splitter.
///
/// Below this the columns would have to reflow into each other, and horizontal
/// scrolling is the better answer — the operator can see a whole column at a
/// time rather than two half ones. The window's own 520 pt floor is below this,
/// deliberately: a dialog dragged to its minimum should scroll, not shred.
pub(super) const MIN_CONTENT_WIDTH_PTS: f32 =
    PREVIEW_MIN_WIDTH_PTS + SPLITTER_WIDTH_PTS + OPTIONS_COLUMN_MIN_WIDTH_PTS;

/// The width a solid scrollbar occupies, in egui points.
///
/// ★ Named rather than inlined because it is used **twice for one reason**: it
/// is what the bar is drawn at, and it is what the body reserves out of the
/// column height so that a horizontal bar appearing cannot raise a vertical one
/// as a side effect. Those two uses must agree or the deadlock in
/// [`PrintDialog::body`] returns, so they read one constant.
pub(super) const SCROLLBAR_WIDTH_PTS: f32 = 10.0;

/// The floor on the body's height, in egui points.
///
/// The window has its own 380 pt minimum, so this is reached only transiently —
/// during a resize, or on the first frame before the viewport reports its real
/// size. It exists so that arithmetic on `available_height` can never hand a
/// negative or absurd height to a column that will allocate it.
pub(super) const MIN_BODY_HEIGHT_PTS: f32 = 200.0;

/// The splitter's declared region, for the driven harness.
///
/// A drag target's position cannot be computed from outside the process, and
/// this one exists *because* the operator asked for it — so a check that it is
/// present, has area, and moves the split needs somewhere to aim.
pub(super) const REGION_SPLITTER: &str = "print.splitter";

/// Height reserved under the scrolling body for the footer row.
///
/// The footer is drawn AFTER the scroll area, so the scroll area must be told
/// not to eat the whole window. Reserved as a constant for the same reason
/// [`preview`]'s strip height is: the commit button's position must not depend
/// on how much the body happens to contain this frame.
pub(super) const FOOTER_HEIGHT_PTS: f32 = 46.0;

impl PrintDialog {
    /// The two-column body: preview on the left, a draggable splitter, options
    /// on the right.
    ///
    /// # ★★★ THE TWO SCROLLBARS THAT WOULD NOT GO AWAY — 2026-09-03
    ///
    /// The operator: *"I have two scroll bars in the pop up window that won't
    /// go away no matter how."* They would not, and the previous version of
    /// this function is why. It did:
    ///
    /// ```text
    /// let body_width = ui.available_width();
    /// ...
    /// ui.set_width(BODY_CONTENT_WIDTH_PTS.max(body_width));
    /// ```
    ///
    /// — i.e. it forced the content to be **exactly as wide as the space it had
    /// been offered**. But `available_width` is measured *before* the
    /// `ScrollArea` reserves room for its own vertical bar. So the sequence at
    /// any window wider than the two columns was:
    ///
    /// 1. content width := viewport width;
    /// 2. the vertical bar takes 10 pt, leaving a viewport 10 pt narrower than
    ///    the content;
    /// 3. a horizontal bar therefore appears, and takes 10 pt of height;
    /// 4. the columns had been allocated at exactly `body_height`, so the
    ///    content was now taller than what remained — keeping the vertical bar
    ///    alive, which is where step 2 came from.
    ///
    /// **The two bars were each other's cause.** Measured at 1000 x 760 and
    /// 1300 x 900: both bars present, two thirds of the window empty, nothing
    /// needing to scroll.
    ///
    /// ★★ And the failure was **inverted**, which a single screenshot would
    /// have missed and walking the size series found: at 700 x 520 the options
    /// column was clipped below "Landscape" — the entire Paper section
    /// unreachable — with **no vertical bar at all**. Bars where nothing needed
    /// scrolling; no bar where content was genuinely cut off.
    ///
    /// # How it works now
    ///
    /// The columns are laid out to the width actually available and the
    /// `ScrollArea` is told nothing about how wide the content should be. It
    /// therefore does the only honest thing: a bar appears when, and only when,
    /// the content does not fit.
    ///
    /// - the preview column's width is [`Self::preview_width`], which the
    ///   operator drags (his 2026-09-03 request), clamped so neither column can
    ///   be squeezed out of existence;
    /// - the options column takes the remainder, never less than
    ///   [`OPTIONS_COLUMN_MIN_WIDTH_PTS`];
    /// - a **horizontal** bar appears only when the window is too narrow for
    ///   both minimums together, which is the one case where scrolling is the
    ///   right answer;
    /// - a **vertical** bar appears only when a column's own content is taller
    ///   than the body, which for the options column is a real possibility on a
    ///   short window and for the preview never is.
    ///
    /// ★ `context` is the frame's one cache context — see
    /// [`super::verdicts::Context`]. It is `Some` exactly when `job` is, and
    /// the two are zipped below rather than unwrapped separately: the preview
    /// needs both or neither, and a `job` drawn against a context from a
    /// different device is the staleness the whole type exists to prevent.
    pub(super) fn body(
        &mut self,
        ui: &mut Ui,
        doc: &OpenDoc,
        job: Option<&Job>,
        page_sizes: &[(f64, f64)],
        context: Option<&super::verdicts::Context>,
    ) {
        // ═══════════════════════════════════════════════════════════════════
        // EVERY NUMBER BELOW IS DERIVED FROM THE OUTER SPACE AND CONSTANTS.
        // Nothing is measured from inside the scroll area. That is the whole
        // fix; see this function's doc comment for the deadlock it breaks.
        // ═══════════════════════════════════════════════════════════════════
        //
        // The footer is reserved out of the height because it is drawn AFTER
        // the body, and a scroll area given the whole window would push the
        // commit button off the bottom of a dialog whose purpose is that
        // button.
        let body_height = (ui.available_height() - FOOTER_HEIGHT_PTS).max(MIN_BODY_HEIGHT_PTS);
        let outer_width = ui.available_width();

        // ★ The allowance, subtracted ONCE, from the outer size.
        //
        // In the common case this makes the content strictly narrower and
        // shorter than the viewport, so neither bar is drawn and the ten points
        // are simply unused. In the narrow case the content hits its floor,
        // genuinely exceeds the viewport, and one bar appears — into space that
        // was already reserved for it, so its appearance cannot change the
        // other axis. **The allowance is what decouples the two axes.**
        //
        // ★★★ THE TWO ITEM GAPS ARE IN THE ARITHMETIC, and leaving them out was
        // this defect's third layer.
        //
        // `horizontal_top` inserts `spacing.item_spacing.x` between every child,
        // and the row has three children — preview, splitter, options — so it
        // adds **two** gaps that no column's width accounts for. Measured:
        // `content_w=1260` against an `outer_w=1276` that should have left 16 pt
        // spare, with the horizontal bar still drawn. The 16 the allowance
        // reserved and the 16 the spacing consumed were the same 16.
        //
        // ★ Read from the style rather than assumed to be 8, because this
        // shell's `Metrics::gutter` differs per theme preset — so a hard-coded
        // gap would be right in one preset and reintroduce the bar in another,
        // which is precisely the kind of defect that gets reported as
        // "sometimes".
        //
        // ★★ Setting `item_spacing.x = 0` was tried first and is WORSE, though
        // it also removes the bar. The zero inherits into every child `Ui`, so
        // the radio rows inside the options column lost their spacing too —
        // *"Subset ●Every page ○Odd only"*, *"Sheet 1 of 1Next"*. Fixing a
        // layout defect by removing the layout is how one defect becomes
        // several, and it was visible in the very next capture.
        let gap = ui.spacing().item_spacing.x;
        let content_width =
            (outer_width - SCROLLBAR_ALLOWANCE_PTS - gap * 2.0).max(MIN_CONTENT_WIDTH_PTS);
        let column_height = (body_height - SCROLLBAR_ALLOWANCE_PTS).max(MIN_BODY_HEIGHT_PTS);

        // ★ The split, clamped and written back. Written back so a drag cannot
        // accumulate past the bound: if the stored width were left at 900 while
        // the window only allows 400, every later frame would re-clamp to 400
        // and the operator's first drag back would appear to do nothing until
        // it had unwound 500 pt of invisible travel. Storing the clamped value
        // keeps the control and the state in step, which is the rule the zoom
        // controls beside it follow too.
        // ★★★ THE CLAMP DOES NOT WRITE BACK, and the first version of this did.
        //
        // Writing the clamped value into `self.preview_width` looks like the
        // careful thing — it keeps the state and the layout in step, which is
        // what the zoom controls beside it do. It is wrong here, and the trace
        // caught it within one run: narrowing the dialog to 540 pt clamped the
        // stored width down to the 220 pt floor, and **widening it again left
        // the preview at 220**. The operator's chosen width was destroyed by a
        // resize they may not even have meant, with no way back except dragging
        // it out again from memory.
        //
        // So `preview_width` is a PREFERENCE and the clamp is applied only to
        // the value used for layout. A window too narrow to honour it squeezes
        // the preview for as long as it is too narrow, and widening restores
        // exactly what was asked for.
        //
        // ★ The drag then has to be expressed against the CLAMPED value rather
        // than the preference, or a drag begun while clamped would move an
        // invisible number and appear to do nothing until it had caught up. See
        // [`Self::splitter`], which is handed the effective width for that
        // reason.
        let widest_preview = (content_width - OPTIONS_COLUMN_MIN_WIDTH_PTS - SPLITTER_WIDTH_PTS)
            .max(PREVIEW_MIN_WIDTH_PTS);
        let preview_width = self
            .preview_width
            .clamp(PREVIEW_MIN_WIDTH_PTS, widest_preview);
        let options_width =
            (content_width - preview_width - SPLITTER_WIDTH_PTS).max(OPTIONS_COLUMN_MIN_WIDTH_PTS);

        // ★ SOLID SCROLLBARS, not egui's floating default.
        //
        // `ScrollStyle::default()` is `floating()`: a 2 pt sliver that
        // allocates no space and fades out when the pointer is elsewhere.
        // Functionally the body scrolls either way — but the operator's report
        // was that a too-small dialog cuts content off, and a scrollbar nobody
        // can see does not answer it.
        //
        // `foreground_color` on top of `solid()`, and that second step is not
        // cosmetic either: a solid handle defaults to
        // `widgets.inactive.bg_fill`, which in a light theme is a near-white
        // against a near-white panel — measured on a capture, the bar was
        // present, opaque, correctly sized, and invisible. `foreground_color`
        // draws the handle from the same visuals' TEXT colour instead, so it
        // inherits whatever contrast the active theme gives its text.
        let mut scroll = egui::style::ScrollStyle::solid();
        scroll.foreground_color = true;
        scroll.bar_width = SCROLLBAR_WIDTH_PTS;
        ui.style_mut().spacing.scroll = scroll;

        let out = egui::ScrollArea::both()
            // ★★★ `auto_shrink` TRUE ON BOTH AXES, and the previous value was
            // the defect rather than a setting near it.
            //
            // `auto_shrink([false, false])` means *"the content is at least as
            // big as the viewport"* — and egui takes "the viewport" to be the
            // area BEFORE its own bars are subtracted. So the content was, by
            // construction, always at least one scrollbar wider than the space
            // it was being measured against, and a bar could never be
            // dismissed by any amount of resizing.
            //
            // Measured at 1300 x 900 after the widths had already been fixed:
            // `print-body avail=1262 content=1262` — content exactly equal to
            // the viewport, and both bars still drawn. That measurement is what
            // moved the diagnosis from the widths to this flag.
            //
            // With it true the content is whatever the columns actually need,
            // which is the only number a scrollbar decision can honestly be
            // made from. The body no longer stretches to fill the window; it
            // does not need to, because the columns are laid out to
            // `content_width` and that is already the window less the
            // allowance.
            .auto_shrink([true, true])
            .max_height(body_height)
            .id_salt("print-dialog-body")
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(preview_width, column_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| match job.zip(context) {
                            Some((job, context)) => preview::column(
                                ui,
                                &preview::Inputs {
                                    doc,
                                    job,
                                    page_sizes,
                                    context,
                                },
                                self,
                                column_height,
                                preview_width,
                            ),
                            // The device would not describe itself. Everything
                            // this column draws — sheet, printable rectangle,
                            // margins — comes from that description, and a
                            // guessed rectangle is exactly the confidently
                            // wrong preview the feature exists to prevent.
                            None => {
                                ui.label(t::device_unavailable());
                            }
                        },
                    );
                    self.splitter(ui, column_height, preview_width);
                    ui.allocate_ui_with_layout(
                        egui::vec2(options_width, column_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| self.options_column(ui, job, doc.pages.len()),
                    );
                });
            });

        // ★ The numbers this layout turned on, so a driven check can assert the
        // deadlock is gone WITHOUT reading pixels — and so that "which width
        // did it use" is answerable from outside the process, which is the
        // question this defect turned on twice. Costs nothing when
        // `PDFCER_DIAG` is unset.
        // ★ `content_w` is the LAID-OUT width — the three columns plus the two
        // item gaps egui inserts between them — not the sum of the columns.
        // Reporting the sum would report a number that is not what the scroll
        // area measures, which is the mistake this whole defect was made of.
        let content = preview_width + SPLITTER_WIDTH_PTS + options_width + gap * 2.0;
        crate::diag::trace(|| {
            format!(
                "print-body outer_w={outer_width:.1} content_w={content:.1} \
                 preview_w={preview_width:.1} options_w={options_width:.1} gap={gap:.1} egui_content={:?} egui_view={:?} \
                 body_h={body_height:.1} column_h={column_height:.1}",
                out.content_size,
                out.inner_rect.size()
            )
        });
    }

    /// The draggable divider between the preview and the options.
    ///
    /// # ★ Why a real splitter and not a `ui.separator()`
    ///
    /// Operator request, 2026-09-03: *"the preview should be adjustable
    /// size."* The preview column was a hard-coded 340 pt, so widening the
    /// dialog widened the empty space and left the sheet postage-stamp sized —
    /// which is the wrong way round, because the preview is the reason the
    /// dialog exists.
    ///
    /// # ★★ The affordance is a CURSOR, and nothing is drawn on the preview
    ///
    /// Rule 4's pre-commit clause: a resize cursor over the divider and a
    /// hover-lift on the divider itself are the *pointer*, which is welcome. No
    /// grip dots on the sheet, no outline round the preview, nothing that
    /// changes what a screenshot of the previewed page looks like.
    ///
    /// ★ `drag_delta()` rather than the pointer's absolute position, because
    /// the two differ by wherever inside the divider the press landed —
    /// absolute tracking makes the divider jump to centre itself under the
    /// cursor on the first pixel of movement.
    fn splitter(&mut self, ui: &mut Ui, height: f32, effective_width: f32) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(SPLITTER_WIDTH_PTS, height),
            egui::Sense::click_and_drag(),
        );
        // ★ Against the EFFECTIVE width, not the stored preference. If the
        // window is currently too narrow to honour the preference the two
        // differ, and adding the delta to the preference would move a number
        // nothing is drawing — the operator would drag and see nothing happen
        // until the hidden value had unwound back into range. Adding it to what
        // is on screen makes the first pixel of movement visible, always.
        if response.dragged_by(egui::PointerButton::Primary) {
            self.preview_width = effective_width + response.drag_delta().x;
        }
        // ★ Double-click restores the default, which is the convention every
        // splitter on this machine carries and costs one line. Without it a
        // width dragged to an extreme has no way back except by feel.
        if response.double_clicked() {
            self.preview_width = PREVIEW_DEFAULT_WIDTH_PTS;
        }
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        // The line itself, centred in the hit area. The hit area is wider than
        // the line on purpose: a 1 pt drag target is a 1 pt drag target.
        let visuals = ui.visuals();
        let stroke = if response.hovered() || response.dragged() {
            egui::Stroke::new(2.0, visuals.widgets.hovered.fg_stroke.color)
        } else {
            egui::Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color)
        };
        let x = rect.center().x;
        ui.painter().line_segment(
            [
                egui::pos2(x, rect.top() + SPLITTER_INSET_PTS),
                egui::pos2(x, rect.bottom() - SPLITTER_INSET_PTS),
            ],
            stroke,
        );
        crate::diag::ui_rect(REGION_SPLITTER, rect);
    }
}

#[cfg(test)]
mod tests {
    //! The relationships between this file's own constants.
    //!
    //! ★★ These are deliberately the ONLY thing asserted here. Whether a
    //! scrollbar actually appears is egui's decision, made against a viewport
    //! that exists only in a laid-out frame — see this module's header. It is
    //! asserted by `ui-verify`'s
    //! `print_dialog_body_does_not_deadlock_its_scrollbars`, which reads
    //! egui's own numbers out of a running process and was falsified by
    //! planting the defect back in.

    use super::*;

    // ★★★ `the_body_width_holds_both_columns` WAS HERE, AND IT WAS GREEN
    // THROUGHOUT — retired 2026-09-03.
    //
    // It asserted `BODY_CONTENT_WIDTH_PTS > COLUMN + COLUMN`, and its stated
    // purpose was that a forgotten change *"would silently reintroduce the
    // squeezed-column defect the `set_width` call exists to fix."* The
    // relationship it pinned was true for every one of the many frames in which
    // the operator was looking at two scrollbars he could not dismiss.
    //
    // ★★ It was a test of the WRONG QUANTITY, and that is worth keeping. A
    // scrollbar appears when the content exceeds the **viewport**, and the
    // viewport is egui's, is smaller than the space the widths were derived
    // from, and does not exist until a frame has been laid out. No relationship
    // between our own constants can decide it. The test could only ever have
    // confirmed that we had been self-consistent about a number that was not
    // the number.
    //
    // What replaced it is in two halves, because the question has two halves:
    //
    //  · the constants keep a property that is genuinely theirs — below;
    //  · whether a bar is actually drawn is read out of a **running frame** by
    //    `ui-verify`'s `print_dialog_body_does_not_deadlock_its_scrollbars`,
    //    which reads egui's own `content_size` and `inner_rect` and was
    //    falsified by planting this defect back in.

    /// **The allowance must exceed the bar it allows for**, which is the single
    /// property that decouples the two scroll axes.
    ///
    /// The body reserves [`SCROLLBAR_ALLOWANCE_PTS`] out of both its width and
    /// its height before laying anything out. If that reservation were merely
    /// *equal* to the bar's drawn width the content would land exactly on the
    /// viewport, and egui reserves slightly more than the drawn width for a
    /// solid bar — measured at 14 pt for a 10 pt bar — so equality is not even
    /// the boundary, it is already over it.
    ///
    /// ★ This is the one thing in that arithmetic a constant CAN pin, and it is
    /// pinned here so that a later change tuning the bar's width cannot quietly
    /// make the allowance too small. It does not, and cannot, prove no bar
    /// appears; see the retired test above for why.
    #[test]
    fn the_scrollbar_allowance_exceeds_the_scrollbar() {
        // Bound to locals rather than compared as constant paths: comparing the
        // paths directly is const-folded (`clippy::assertions_on_constants`),
        // which makes a test pass by being erased rather than by being true.
        let allowance = SCROLLBAR_ALLOWANCE_PTS;
        let bar = SCROLLBAR_WIDTH_PTS;
        assert!(
            allowance > bar,
            "the reservation ({allowance}) must exceed the bar's drawn width ({bar}), and by \
             more than egui's own extra padding for a solid bar. At or below it, a bar on one \
             axis consumes space the other axis was counting on and raises a second bar that \
             cannot be dismissed."
        );
    }

    /// **The default preview width sits inside its own bounds**, so a fresh
    /// dialog and a double-clicked splitter both land somewhere legal.
    ///
    /// [`PREVIEW_DEFAULT_WIDTH_PTS`] is what the dialog opens at and what a
    /// double-click on the splitter restores to. A default below the floor
    /// would be silently clamped, so the restore gesture would not restore what
    /// the operator saw on opening — the two would differ by however far out of
    /// range the constant had drifted, and nothing would say so.
    #[test]
    fn the_default_preview_width_is_within_its_own_floor() {
        let default = PREVIEW_DEFAULT_WIDTH_PTS;
        let floor = PREVIEW_MIN_WIDTH_PTS;
        assert!(
            default >= floor,
            "the opening width ({default}) must be at least the floor ({floor}), or opening the \
             dialog and double-clicking the splitter give two different layouts"
        );
    }

    /// **The content floor is exactly its parts**, so the narrow case scrolls
    /// rather than shredding a column.
    ///
    /// [`MIN_CONTENT_WIDTH_PTS`] is what the body refuses to lay out below,
    /// and it must equal the two column floors plus the splitter — not
    /// approximate them. A floor smaller than its parts would let a column be
    /// squeezed under its own minimum with no scrollbar offered, which is the
    /// *"content clipped and unreachable, no bar anywhere"* half of the
    /// operator's report — the half a single screenshot at one size would have
    /// missed entirely.
    #[test]
    fn the_content_floor_is_the_sum_of_the_column_floors() {
        let floor = MIN_CONTENT_WIDTH_PTS;
        let parts = PREVIEW_MIN_WIDTH_PTS + SPLITTER_WIDTH_PTS + OPTIONS_COLUMN_MIN_WIDTH_PTS;
        assert!(
            (floor - parts).abs() < f32::EPSILON,
            "the content floor ({floor}) must be exactly its parts ({parts}); anything less \
             squeezes a column below its minimum without offering a scrollbar"
        );
    }
}
