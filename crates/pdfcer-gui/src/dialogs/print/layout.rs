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

/// The preview column's own rect, for the driven harness.
///
/// # ★★★ It exists to be able to GO AWAY
///
/// Every other region in this shell is published so that something can be
/// aimed at, measured or clicked. This one is published so that
/// `diag::end_ui_frame` can emit `ui-rect-gone name=print.preview.column` on
/// the frame the operator pops the preview out — which is the only evidence
/// from outside the process that the column **collapsed** rather than merely
/// gaining a sibling in another window.
///
/// ★★ Without it, the honest-looking check *"a second window appeared"* passes
/// on a build that opens the pop-out **and keeps drawing the column too** —
/// two previews of one sheet, which is precisely the shape O112 asked against.
/// A presence assertion cannot see that; an absence assertion can, and an
/// absence assertion is only worth anything when the run has first been driven
/// into the state where the absence is the claim. See `ui-verify`'s
/// `the_print_preview_pops_into_its_own_window`, which asserts the region is
/// there before the click and gone after it.
pub(super) const REGION_PREVIEW_COLUMN: &str = "print.preview.column";

/// **The body's horizontal division, decided once and in one place.**
///
/// # Why a type rather than four `let`s
///
/// Because there are now **two** layouts — preview beside options, and options
/// alone with the preview in its own window (O112 ask 2) — and every number in
/// the second differs from the first: the number of `item_spacing` gaps egui
/// inserts, the floor the content may not go below, and both column widths.
/// Four `let`s with an `if` threaded through them is how the two cases come to
/// disagree about one of the four, and the failure of a width in this file has
/// twice been a scrollbar the operator could not dismiss.
///
/// # ★★★ It is PURE, and that is what makes the collapse falsifiable
///
/// The one thing a unit test genuinely can assert about this dialog's layout is
/// a relationship between numbers we own — `layout::tests`' own header says so,
/// and says why the *presence of a scrollbar* is not such a relationship. The
/// collapse **is** one: *"when the preview is popped out, the preview column is
/// zero wide, there is no splitter, and the options column is the whole
/// content."* That is an assertion about arithmetic, it needs no window, and a
/// build that popped the preview out and left the column standing fails it.
///
/// ⇒ Which matters here more than usual, because a layout change's only true
/// oracle is a rendered frame and this session could not render one. Pushing
/// as much of the claim as possible into arithmetic is what is left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Columns {
    /// The width the columns are laid out into, after the scrollbar allowance
    /// and the item gaps have been taken out of the window.
    pub(super) content: f32,
    /// The preview column's width. **Exactly zero** when it is popped out.
    pub(super) preview: f32,
    /// The options column's width — the whole of [`Self::content`] when the
    /// preview is elsewhere.
    pub(super) options: f32,
    /// How many `item_spacing` gaps egui will insert between the children of
    /// the body's row: two for three children, none for one.
    pub(super) gaps: f32,
    /// The splitter's width, or zero when there is nothing to split.
    pub(super) splitter: f32,
}

impl Columns {
    /// Divide `outer_width` between the two columns.
    ///
    /// `gap` is the live `item_spacing.x` — read from the style rather than
    /// assumed, because this shell's `Metrics::gutter` differs per theme preset
    /// and a hard-coded gap is right in one preset and reintroduces a scrollbar
    /// in another.
    ///
    /// `preference` is [`PrintDialog::preview_width`] — the width the operator
    /// dragged the splitter to. It is **clamped for layout and never written
    /// back**, which is why this function returns a value instead of taking
    /// `&mut`: writing the clamp back destroyed the operator's chosen width the
    /// first time a narrow window clamped it, and the fix was to keep the
    /// preference and the layout apart. See [`PrintDialog::body`].
    pub(super) fn split(outer_width: f32, gap: f32, preference: f32, popped: bool) -> Self {
        if popped {
            // One child, so no gaps; and the only floor still in play is the
            // options column's own, because there is no second column left to
            // squeeze. Using `MIN_CONTENT_WIDTH_PTS` here would refuse to lay
            // the body out below 628 pt when 400 is the truth, and the refusal
            // presents as a horizontal scrollbar on a comfortable window.
            let content = (outer_width - SCROLLBAR_ALLOWANCE_PTS).max(OPTIONS_COLUMN_MIN_WIDTH_PTS);
            return Self {
                content,
                preview: 0.0,
                options: content,
                gaps: 0.0,
                splitter: 0.0,
            };
        }
        let gaps = gap * 2.0;
        let content = (outer_width - SCROLLBAR_ALLOWANCE_PTS - gaps).max(MIN_CONTENT_WIDTH_PTS);
        let widest_preview = (content - OPTIONS_COLUMN_MIN_WIDTH_PTS - SPLITTER_WIDTH_PTS)
            .max(PREVIEW_MIN_WIDTH_PTS);
        let preview = preference.clamp(PREVIEW_MIN_WIDTH_PTS, widest_preview);
        let options = (content - preview - SPLITTER_WIDTH_PTS).max(OPTIONS_COLUMN_MIN_WIDTH_PTS);
        Self {
            content,
            preview,
            options,
            gaps,
            splitter: SPLITTER_WIDTH_PTS,
        }
    }

    /// What the row will actually measure, for the `print-body` trace line.
    ///
    /// The columns **plus** the gaps egui inserts between them — never the sum
    /// of the columns alone. Reporting the sum was the mistake the whole
    /// two-scrollbar defect was made of, and it is not made twice because there
    /// is one function that knows how many gaps there are.
    pub(super) fn laid_out(self) -> f32 {
        self.preview + self.splitter + self.options + self.gaps
    }
}

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
        // ★★★ THE POPPED-OUT CASE CHANGES THE ARITHMETIC AND NOT ONLY THE
        // DRAWING — 2026-09-05, operator request O112 ask 2.
        //
        // With the preview in its own window the row has **one** child, not
        // three, so `horizontal_top` inserts **no** gaps and the two column
        // floors are no longer both in play. Carrying the three-child numbers
        // into the one-child case would reserve 16 pt of spacing that nothing
        // consumes and refuse to lay the body out below 628 pt of content when
        // 400 pt is now the honest floor — i.e. it would raise a horizontal
        // scrollbar on a dialog narrow enough to be perfectly comfortable.
        // That is the same class of defect as the one this whole file was
        // rewritten for, arrived at from the opposite direction.
        //
        // ⇒ [`Columns`] is where both cases are decided, together, as a pure
        // function of three numbers. See its own docs for why that is worth a
        // type.
        let split = Columns::split(outer_width, gap, self.preview_width, self.preview_popped);
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
        //
        // ★ Both numbers now come out of [`Columns::split`] rather than being
        // derived here, so the popped-out case cannot be given a second,
        // slightly different arithmetic by whoever adds the next branch. The
        // clamp-without-write-back described above is inside that function and
        // is asserted by its own unit tests.
        let preview_width = split.preview;
        let options_width = split.options;

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
                    // ★★★ R9, AND IT IS THE WHOLE OF ASK 2's DESIGN: WHILE THE
                    // PREVIEW IS POPPED OUT, THIS COLUMN DRAWS NOTHING AT ALL.
                    //
                    // Not a greyed rectangle. Not a *"the preview is in another
                    // window"* placard holding the same 340 pt open. Not a
                    // dotted outline where it used to be. The column and its
                    // splitter are **absent**, and the options take every point
                    // they were using — which is why [`Columns::split`] gives
                    // `options` the whole content width in that case rather
                    // than merely giving the preview a width of zero.
                    //
                    // A stub here would be the exact thing this project's
                    // no-placeholders rule exists to forbid: a surface that
                    // occupies space, answers no question, and teaches the
                    // operator that the dialog has a dead region in it.
                    //
                    // ★ The route back is the popped window's own close button
                    // — `Frame::closed`, which `popout::popped_preview` turns
                    // straight into `preview_popped = false`. So there is
                    // nothing for this branch to draw a control for either: the
                    // window is in the taskbar, and closing it is the gesture.
                    if !self.preview_popped {
                        let placed = ui.allocate_ui_with_layout(
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
                                    preview::Placement::InDialog,
                                ),
                                // The device would not describe itself.
                                // Everything this column draws — sheet,
                                // printable rectangle, margins — comes from
                                // that description, and a guessed rectangle is
                                // exactly the confidently wrong preview the
                                // feature exists to prevent.
                                None => {
                                    ui.label(t::device_unavailable());
                                }
                            },
                        );
                        // ★★ The column's own rect, and it is published with
                        // the UNGATED `ui_rect` on purpose.
                        //
                        // Its job is to answer *"was this column laid out at
                        // all"*, and `diag::end_ui_frame` turns the frame it
                        // stops being laid out into a `ui-rect-gone` line. That
                        // line is the oracle for the absence half of
                        // `ui-verify`'s `the_print_preview_pops_into_its_own_
                        // window` — the half that separates "the pop-out window
                        // appeared" from "the pop-out window appeared AND the
                        // column went away", which is the difference between the
                        // feature and a second copy of the preview.
                        //
                        // The visibility-gated form would be wrong here: a
                        // column scrolled out of view is still laid out, and
                        // reporting it as gone would make the check pass for a
                        // reason that has nothing to do with popping out.
                        crate::diag::ui_rect(REGION_PREVIEW_COLUMN, placed.response.rect);
                        self.splitter(ui, column_height, preview_width);
                    }
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
        // ★ `content_w` is the LAID-OUT width — the columns plus the item gaps
        // egui inserts between them — not the sum of the columns. Reporting the
        // sum would report a number that is not what the scroll area measures,
        // which is the mistake this whole defect was made of.
        //
        // ★★ `popped=` travels with it since 2026-09-05, and it is not
        // decoration. `preview_w=0.0` alone is ambiguous — it is also what a
        // build with a broken clamp would print — whereas `popped=true
        // preview_w=0.0 options_w=<content>` is the single line that says the
        // column collapsed *and* the options took the room. A driven check that
        // could only read the width would pass on a build that zeroed the
        // preview and left a 340 pt hole where it had been.
        let content = split.laid_out();
        let popped = self.preview_popped;
        crate::diag::trace(|| {
            format!(
                "print-body outer_w={outer_width:.1} content_w={content:.1} popped={popped} \
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

    // ═══════════════════════════════════════════════════════════════════════
    // THE FOOTER MOVED HERE FROM `mod.rs` ON 2026-09-05, and the seam is the
    // one this file was split on rather than a new one.
    //
    // `mod.rs` owns what a print job IS — the device, the plan, the commit,
    // what the operator is told afterwards. This file owns **where things are
    // drawn and how wide they are**. The footer is a `ui.horizontal` that
    // places two things in a row and picks no policy: the label it shows is
    // chosen by `verdicts::ClipClaim::commit_label`, the ordering and the
    // theme fill are `Host::buttons`', and all it does itself is set two
    // request flags that `mod.rs` acts on after the frame. That is layout.
    //
    // ★ It was moved because `mod.rs` stood at 1,492 lines against R2's 1,500
    // and the pop-out preview needed a field, a call site and their reasons.
    // Growing a file past a ceiling to add a feature is how a file gets a
    // second reason to exist; moving the block whose subject this file already
    // names — [`FOOTER_HEIGHT_PTS`] has lived here since the split — is how it
    // keeps one. Nothing about the footer changed in the move; the diff is a
    // cut and a paste plus two `super::` qualifications on `verdicts`.
    // ═══════════════════════════════════════════════════════════════════════

    /// The footer: Close, the commit button, and the last outcome.
    ///
    /// # ★ The commit button is ABSENT, not greyed, when there is nothing to print
    ///
    /// The no-placeholders rule's own distinction: greying is for
    /// *temporarily* unavailable, and there are two genuinely different
    /// reasons this button might not act.
    ///
    /// - **No device, or no pages selected** — the job does not exist. The
    ///   button is not drawn. Something else on screen already says why (the
    ///   preview column's own sentence), so a disabled button would be a
    ///   second, quieter statement of a fact already made loudly.
    /// - There is no third case. A job that exists can always be sent; whether
    ///   it *should* be is the operator's call, and the clip count in the
    ///   label is how they make it.
    ///
    /// # ★★★ The label's count is corrected by what the preview has seen
    ///
    /// Operator request O113, 2026-09-04. It used to be [`Job::clipped`] —
    /// a geometric count of page boxes exceeding the printable rectangle —
    /// which on a 1:1 CAD sheet read *"Print — 1 sheet will be clipped"* over
    /// a preview showing nothing hatched and saying the overhang was blank.
    ///
    /// It is now the geometric count **minus the sheets the preview has
    /// examined and found blank**, with every sheet nobody has looked at still
    /// counted. [`super::verdicts::ClipClaim`] carries both the number and how well
    /// it is known, and picks the sentence that number can support; this
    /// function does not choose wording, so the button and the preview's own
    /// caption cannot come to say different things about one job.
    ///
    /// ★ Drawn AFTER the body, which is what makes the sheet on screen count
    /// as examined on the same frame it is drawn. The alternative — the
    /// footer reading a cache the preview has not written yet — would make the
    /// button lag the picture beside it by exactly one frame, which is a
    /// contradiction that flickers rather than one that persists, and is
    /// therefore harder to notice and worse.
    pub(super) fn footer(
        &mut self,
        ui: &mut Ui,
        job: Option<&Job>,
        page_sizes: &[(f64, f64)],
        context: Option<&super::verdicts::Context>,
    ) {
        ui.horizontal(|ui| {
            // ★★ G4 — ENTER PRESSES PRINT, AND PRINT LOOKS LIKE THE DEFAULT.
            //
            // The operator's second item, and the failure mode
            // `ui-conventions/dialogs.md` names: *"the operator types into the
            // last field, presses Enter out of habit, and nothing happens."*
            // In this dialog the last field is the page range, which is exactly
            // the box somebody types into and then expects Enter to act on.
            //
            // ★ The pair is drawn only when there is a job to send. That is not
            // a styling decision, it is the no-placeholders invariant: a
            // default button for a print that cannot happen is a control the
            // operator would press and be ignored by — and worse, it would make
            // ENTER silently do nothing while looking like it should do
            // something, which is the very complaint. With no job, the footer
            // keeps its plain Close and Enter is honestly inert.
            //
            // ★ `Host::buttons` owns the ordering, the theme fill and the Enter
            // guard, so no dialog can implement two of the three. It puts
            // Cancel to the LEFT of the affirmative in the right-to-left
            // layout, which is Windows' order and the order every dialog on
            // this machine uses.
            // ★★★ THE OUTCOME IS DRAWN FIRST, AND THE ORDER IS THE BUG FIX.
            //
            // Operator report, 2026-08-25: *"when I press print, instead of
            // closing after printing it just keeps expanding its size in
            // little steps to infinity."*
            //
            // It did, and the cause was this block sitting AFTER the button
            // block rather than before it. [`Host::buttons`] lays its pair out
            // with `Layout::right_to_left`, and a right-to-left child inside a
            // left-to-right `horizontal` is anchored to the RIGHT EDGE of
            // whatever space it was offered — so its `min_rect` reaches that
            // edge whether it needed the room or not. Appending anything after
            // it therefore places that widget **past the edge of the available
            // width**, by its own width plus one item spacing.
            //
            // On its own that is only an overflowing row. What made it
            // unbounded is [`crate::dialogs::host::Host::fit`], which grows a
            // dialog whose content is wider than its window:
            //
            // 1. the row overflows by the label's width, `w`;
            // 2. `fit` grows the window by `w`;
            // 3. the wider window offers a wider row, the RTL block reaches the
            //    NEW right edge, and the label is placed `w` past it again;
            // 4. goto 2, for ever, in steps of exactly `w`.
            //
            // ★ Note what this is NOT. It is not the once-per-size guard
            // failing — every size in the sequence is genuinely new, so the
            // guard is satisfied every time. It is not `FIT_MARGIN` being too
            // small — the step is a whole label wide. **It is a measurement fed
            // back into the size that produced it**, which is R128's shape and
            // the third time this project has met it. `fit`'s own doc comment
            // asserted the print dialog was immune because it scrolls; that was
            // true of its BODY and said nothing about its footer.
            //
            // Drawing the outcome first fixes it completely and needs no
            // arithmetic: the label consumes width from the left, the button
            // block is then offered what remains and anchors to the right edge
            // of THAT, and the row ends exactly at the edge. It is also the
            // conventional arrangement — status left, actions right — which is
            // what every dialog on this machine does, so the fix costs nothing
            // in layout terms and gains the Windows idiom.
            //
            // ★ `truncate()` is not decoration either: `t::failed` carries a
            // driver's own error text and can be arbitrarily long. Untruncated
            // it would push the buttons off the row and re-create the overflow
            // by a different route — a *bounded* one, since the text does not
            // grow with the window, but bounded overflow is still overflow.
            // The full text is not lost; it is what the trace records.
            match &self.outcome {
                // ★ A SUCCESS DRAWS NOTHING HERE, and the arm is kept rather
                // than folded into `None` so the reason is where somebody
                // looking for the missing receipt will find it.
                //
                // Since 2026-09-03 a successful commit sets `close_requested`,
                // so this window is gone by the next frame and anything drawn
                // here would be shown for at most one. The receipt — the page
                // count, and the `Synthesised` disclosure when the driver held
                // settings pdfcer does not model — goes to the application's
                // disclosure row instead, where it OUTLIVES the dialog. See
                // `show`'s commit block.
                //
                // Drawing it in both places was considered and refused: two
                // copies of one sentence is how they come to disagree, and the
                // footer's copy would be the one nobody could act on.
                Some(Ok(_)) => {}
                Some(Err(detail)) => {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(t::failed(detail))
                                .color(ui.visuals().error_fg_color),
                        )
                        .truncate(),
                    );
                }
                None => {}
            }
            match job.filter(|j| !j.plans.is_empty()) {
                Some(job) => {
                    // ★ `ClipClaim::None` when there is no context, which
                    // happens only when there is no job — and this arm has
                    // one. The plain label is therefore not a fallback that
                    // could hide a clip; it is what an unclipped job says.
                    let label = context
                        .map(|context| self.verdicts.claim(context, job, page_sizes))
                        .unwrap_or(super::verdicts::ClipClaim::None)
                        .commit_label()
                        .unwrap_or_else(|| t::commit().to_owned());
                    let (accepted, cancelled) =
                        crate::dialogs::host::Host::buttons(ui, &label, t::close());
                    if accepted {
                        self.commit_requested = true;
                    }
                    if cancelled {
                        self.close_requested = true;
                    }
                }
                None => {
                    if ui.button(t::close()).clicked() {
                        self.close_requested = true;
                    }
                }
            }
        });
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
    /// **With the preview popped out the column is GONE, not hidden** — O112
    /// ask 2, and this is the assertion the whole of R9 rests on here.
    ///
    /// Three separate claims, and all three have to hold or the operator gets
    /// something this project's no-placeholders rule forbids:
    ///
    /// 1. `preview == 0.0` — no width is reserved for it;
    /// 2. `splitter == 0.0` — no divider is drawn beside a column that is not
    ///    there, which would be a control that moves nothing;
    /// 3. `options == content` — **the room is taken**, which is the half a
    ///    "hide the column" implementation gets wrong. A build that zeroed the
    ///    preview and left the options at their old width would leave a 340 pt
    ///    hole in the dialog, which is a placeholder made of nothing at all and
    ///    is exactly as bad as a greyed rectangle.
    ///
    /// ★ Claim 3 is the one worth the test. Claims 1 and 2 are what anybody
    /// would write; claim 3 is what makes the difference between *collapsing*
    /// the column and merely *emptying* it, and it is invisible in a screenshot
    /// of a wide dialog where the extra room is not obviously anybody's.
    #[test]
    fn popping_the_preview_out_collapses_its_column_and_gives_the_room_away() {
        let split = Columns::split(1000.0, 8.0, PREVIEW_DEFAULT_WIDTH_PTS, true);
        assert!(
            (split.preview - 0.0).abs() < f32::EPSILON,
            "the preview column must be exactly zero wide while the preview is in its own \
             window, and it was {}. Any positive width is a reserved hole in the dialog.",
            split.preview
        );
        assert!(
            (split.splitter - 0.0).abs() < f32::EPSILON,
            "there is nothing to split, so the splitter must be zero wide and it was {}",
            split.splitter
        );
        assert!(
            (split.options - split.content).abs() < f32::EPSILON,
            "the options column must take the whole content width ({}) when the preview is \
             popped out, and it took {}. The difference is a hole the operator can see and \
             cannot use.",
            split.content,
            split.options
        );
        assert!(
            (split.laid_out() - split.content).abs() < f32::EPSILON,
            "a one-child row has no item gaps, so what is laid out ({}) must equal the content \
             width ({}); anything more is width the scroll area will raise a bar for",
            split.laid_out(),
            split.content
        );
    }

    /// **And with it in place, both columns are there** — the other position of
    /// the same switch.
    ///
    /// Asserted beside the one above rather than left implicit, because an
    /// absence test alone passes on a build that has lost the preview
    /// altogether. `ui-verify`'s driven check makes the same pairing through
    /// the OS: the column's region is declared before the click and retired
    /// after it.
    #[test]
    fn with_the_preview_in_place_both_columns_are_laid_out() {
        let split = Columns::split(1000.0, 8.0, PREVIEW_DEFAULT_WIDTH_PTS, false);
        assert!(
            split.preview >= PREVIEW_MIN_WIDTH_PTS,
            "the preview column must be at least its floor ({}) and was {}",
            PREVIEW_MIN_WIDTH_PTS,
            split.preview
        );
        assert!(
            split.options >= OPTIONS_COLUMN_MIN_WIDTH_PTS,
            "the options column must be at least its floor ({}) and was {}",
            OPTIONS_COLUMN_MIN_WIDTH_PTS,
            split.options
        );
        assert!(
            (split.splitter - SPLITTER_WIDTH_PTS).abs() < f32::EPSILON,
            "the splitter must be drawn between two columns"
        );
        assert!(
            split.laid_out() <= split.content + split.gaps + f32::EPSILON,
            "the row must not be laid out wider than the content width it was given plus the \
             gaps that width already accounts for: {} against {} + {}",
            split.laid_out(),
            split.content,
            split.gaps
        );
    }

    /// **A narrow window with the preview popped out must not scroll**, which
    /// is the reason the popped case has a floor of its own.
    ///
    /// Carrying [`MIN_CONTENT_WIDTH_PTS`] — both column floors plus the
    /// splitter — into the one-column case would refuse to lay the body out
    /// below 628 pt of content, so a dialog dragged to 560 pt would be told its
    /// content is 628 pt wide and would raise a horizontal scrollbar. That is
    /// the operator's original complaint, re-entering through the new feature.
    #[test]
    fn a_narrow_dialog_with_the_preview_popped_out_still_fits_its_options() {
        // 560 pt outer: comfortably above the window's own 520 pt floor and
        // comfortably below `MIN_CONTENT_WIDTH_PTS`.
        let split = Columns::split(560.0, 8.0, PREVIEW_DEFAULT_WIDTH_PTS, true);
        assert!(
            split.laid_out() <= 560.0 - SCROLLBAR_ALLOWANCE_PTS + f32::EPSILON,
            "with the preview elsewhere, a 560 pt dialog must lay its options out in {} pt or \
             less and it wanted {}",
            560.0 - SCROLLBAR_ALLOWANCE_PTS,
            split.laid_out()
        );
    }

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
