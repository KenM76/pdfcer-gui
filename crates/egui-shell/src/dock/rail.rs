//! # `dock::rail` — the permanent vertical strip down a side's outer edge
//!
//! `OPERATOR_REQUESTS.md` **O123**, part 7, his words:
//!
//! > *"What I'd also added in the bar at the left side that we are adding: the
//! > navigate selectors and some other related selection controls (lasso tool
//! > when we implement one, etc) and these will fold up into a drop down arrow
//! > if space becomes scarce."*
//!
//! and **O126**'s addendum:
//!
//! > *"also add rotate pages to that area, and those should be available in
//! > every mode including read."*
//!
//! `mockups/pdfcer-shell.html` draws the result and he approved it. This
//! module is that drawing made buildable: **the geometry and the fold ladder**.
//! What goes in the strip is [`crate::manifest::Rail`] — data — and what a row
//! looks like is the application's, drawn through a handler.
//!
//! ## ★★★ The width is a CONSTANT, and that is the whole safety argument
//!
//! [`WIDTH_PTS`] is `52.0` at every rung, and nothing in this module can make
//! it anything else. What shrinks under pressure is the **row budget**, never
//! the width.
//!
//! This is not tidiness. `D:/dev/rag/egui/bottom_panel_height_change_retriggers_fit_to_viewport_zoom.md`
//! and `…/a_surface_may_not_change_size_in_response_to_a_gesture_aimed_at_it.md`
//! record the same loop twice on this project (R128): a chrome region whose
//! size depends on its own content sits beside a canvas whose zoom is derived
//! from the space left over, and the two chase each other frame after frame —
//! measured at 230 % → 224 % → 215 % on three consecutive frames, with the
//! operator's click coordinates going stale between the click and the redraw.
//!
//! ⚠ **A rail sized from its widest word is exactly that loop.** `Signatures`
//! is a wider word than `Sigs`; a font change, a theme change or a
//! localization would move the rail's width, which moves the canvas, which
//! re-fits the zoom. So the constant is load-bearing and a "helpful" change to
//! `WIDTH_PTS.max(widest_label)` re-opens a defect this project has already
//! paid for twice. The label is what gives way instead — see [`Rung::Tight`].
//!
//! ## The fold ladder, and why it is this order
//!
//! `RIBBON_SCALING.md` already answers *"what happens when the controls do not
//! fit"* for the ribbon, from photographs of Word, in three rungs: **re-wrap →
//! collapse → scroll**. The rail reuses that reasoning rather than inventing a
//! second answer to the same question one surface over:
//!
//! | rung | what changes | why here |
//! |---|---|---|
//! | [`Rung::Roomy`] | everything, captions and labels | the resting state |
//! | [`Rung::Tight`] | **the words go first** | Word's *re-wrap*: presentation is cheaper than reach, and every control is still one click away |
//! | [`Rung::Snug`] | every [`RailFold::Whole`] group folds | Word's *collapse*, in the **authored** order §3.2 measured — not right-to-left, and never the group the author marked as the floor |
//! | [`Rung::Cramped`] | every [`RailFold::PinArmed`] group collapses to the armed row | the last thing a tool strip may stop doing is say what you are holding |
//!
//! And below `Cramped` the strip **scrolls** — the third rung of that ladder,
//! and the application's `ScrollArea` rather than this planner's business. A
//! rail that simply cut its last entry off the bottom edge of a short window
//! would be the unreachable-control defect with a different cause.
//!
//! ## ★★ Two things never fold, and each has a reason that is not symmetry
//!
//! * **A [`RailFold::Never`] group.** For pdfcer that is the five panel tabs,
//!   and *"all five panels one click away"* is the rail's entire argument for
//!   existing. A rail that folds them is strictly worse than the horizontal
//!   tab stack it replaced.
//! * **The chevron.** Inkscape failure mode #8 — past about six tabs the
//!   overflow button itself gets hidden — is this control eating itself.
//!   Everything the rail dropped is behind it, so it is the one row that must
//!   survive. [`RailRow::Chevron`] is appended after the ladder has run and is
//!   never a candidate for folding.
//!
//! ## ★ R7 — this module does not know what is in the rail
//!
//! `tools/gates/check-shell-purity.sh` forbids `egui-shell` naming anything
//! from `pdfcer-*`. Everything here is a command id, a rectangle, a row height
//! and a closure. The planner cannot tell a page thumbnail from a hand tool,
//! and [`draw`] reserves a strip and hands over a `Ui` exactly as
//! [`super::banner`] does.
//!
//! ## Why the plan is a value rather than a draw call
//!
//! [`plan`] is pure: manifest data plus a [`ConditionSet`] plus a height, in;
//! a list of rows, out. No `Ui`, no fonts, no frame. That is what makes the
//! ladder testable at all — and this feature is the one that shipped the
//! 2026-08-10 defect where Bookmarks, Layers and Signatures were laid out,
//! published healthy rectangles, and could not be reached. The lesson recorded
//! from that (`crate::dock::report`'s header) is that a rect proves layout and
//! not visibility; the response here is that **which rows exist at which
//! budget is decided by a function a test can call**, and separately that the
//! application publishes its rows through `ui_rect_visible`.

use egui::{Layout, Rect, UiBuilder};

use crate::commands::ConditionSet;
use crate::manifest::{Item, Rail, RailFold};

use super::model::DockSide;
use super::{Ctx, report};

/// What an application draws into a side's rail.
///
/// Called at most once per side per frame, with a [`egui::Ui`] whose
/// `max_rect` **and clip rectangle** are the strip. The clip is the
/// load-bearing half, for [`super::banner::BannerHandler`]'s reason: a caller
/// that draws a wider row than the strip gets it clipped rather than pushing
/// the panel body sideways, which is the R128 feedback loop this crate is
/// arranged to make unwritable.
pub type RailHandler<'a> = dyn FnMut(&mut egui::Ui) + 'a;

/// **The rail's width, in points. A constant at every rung.**
///
/// 52 pt, which is `mockups/pdfcer-shell.html`'s own value and wide enough for
/// a 16 pt glyph with a short word under it. See the module header for why
/// this may not become a function of the content.
pub const WIDTH_PTS: f32 = 52.0;

/// Height of one entry drawn with its word under it.
pub const ROW_LABELLED_PTS: f32 = 34.0;

/// Height of one entry drawn as a picture alone.
pub const ROW_ICON_ONLY_PTS: f32 = 26.0;

/// Height of a group caption — `navigate`, `select`.
pub const CAPTION_PTS: f32 = 14.0;

/// Height of the rule drawn between two groups, including its margins.
pub const RULE_PTS: f32 = 10.0;

/// Height of the overflow chevron.
pub const CHEVRON_PTS: f32 = 22.0;

/// The strip's own padding, top and bottom together.
pub const PADDING_PTS: f32 = 12.0;

/// One rung of the fold ladder. Widest first.
///
/// Ordered, and the order is the ladder: [`plan`] walks these in sequence and
/// takes the first that fits. See the module header's table for what each one
/// gives up and why it is that one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Rung {
    /// Everything, with captions and words. The resting state.
    #[default]
    Roomy,
    /// ★ **The words go first.** Captions and labels are dropped; every
    /// control is still exactly where it was and still one click away.
    ///
    /// This is `RIBBON_SCALING.md` §3.1's *item size* mechanism — Word's
    /// Large → Medium → Small — applied to a vertical strip, where the only
    /// step available is icon-above-label → icon-only. Giving up the word is
    /// strictly cheaper than giving up the control, and it is the rung that
    /// buys the most: 34 pt a row becomes 26.
    Tight,
    /// Every [`RailFold::Whole`] group folds into the chevron.
    Snug,
    /// Every [`RailFold::PinArmed`] group collapses to its armed row as well.
    ///
    /// The floor of the ladder. Below this the strip scrolls; it does not
    /// shrink further, and it never fails to show what is armed.
    Cramped,
}

impl Rung {
    /// The ladder, widest first.
    pub const LADDER: [Rung; 4] = [Rung::Roomy, Rung::Tight, Rung::Snug, Rung::Cramped];

    /// Whether entries carry their word at this rung.
    #[must_use]
    pub fn shows_words(self) -> bool {
        self == Rung::Roomy
    }

    /// Whether a [`RailFold::Whole`] group is still drawn at this rung.
    #[must_use]
    pub fn keeps_whole_groups(self) -> bool {
        self <= Rung::Tight
    }

    /// Whether a [`RailFold::PinArmed`] group is still drawn entire.
    #[must_use]
    pub fn keeps_pinned_groups_entire(self) -> bool {
        self <= Rung::Snug
    }
}

/// One row of the planned strip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RailRow {
    /// A control.
    Entry {
        /// The id of the group it came from — the region name's middle part.
        group: String,
        /// The command id.
        id: String,
        /// Whether the word is drawn under the picture at this rung.
        with_label: bool,
        /// Whether `selected:<id>` is set — the ribbon's own convention.
        selected: bool,
        /// ★ True when this row is a [`RailFold::PinArmed`] group collapsed to
        /// one entry. The application draws it differently — it stands for a
        /// group rather than for itself — and a test can assert the pinning
        /// happened without re-deriving which tool was armed.
        pinned: bool,
    },
    /// The group's word, drawn at [`Rung::Roomy`] only.
    Caption(String),
    /// The rule between two groups.
    Rule,
    /// ★ The overflow chevron. Always last, never folded.
    Chevron {
        /// How many entries are behind it.
        folded: usize,
    },
}

impl RailRow {
    /// This row's height in points.
    #[must_use]
    pub fn height_pts(&self) -> f32 {
        match self {
            RailRow::Entry { with_label, .. } => {
                if *with_label {
                    ROW_LABELLED_PTS
                } else {
                    ROW_ICON_ONLY_PTS
                }
            }
            RailRow::Caption(_) => CAPTION_PTS,
            RailRow::Rule => RULE_PTS,
            RailRow::Chevron { .. } => CHEVRON_PTS,
        }
    }
}

/// What the rail will draw this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct RailPlan {
    /// The rung the ladder settled on.
    pub rung: Rung,
    /// The rows, top to bottom.
    pub rows: Vec<RailRow>,
    /// The command ids behind the chevron, **in the order they went** — which
    /// is what the chevron's tooltip says, so an operator can see what the
    /// strip gave up rather than discovering it by opening the menu.
    pub folded: Vec<String>,
    /// The strip's width. [`WIDTH_PTS`], always. See the module header.
    pub width_pts: f32,
    /// The height the rows want. May exceed the budget at [`Rung::Cramped`],
    /// which is the case the application's `ScrollArea` answers.
    pub height_pts: f32,
}

impl RailPlan {
    /// Whether there is nothing to draw.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// The visible command ids of one group, after `visible_when` is applied.
///
/// ★ Filtering happens **before** the ladder runs, exactly as
/// [`crate::ribbon::trailing`] filters before it measures, and for the same
/// reason: a hidden item that was counted would make the rail fold a group to
/// make room for a control nobody can see.
///
/// This is where **mode gating** lands. `pdfcer-gui` marks the Points tool
/// `visible_when("mode.edit_content")`, so in Read it is not in this list, is
/// not measured, and is not folded — it is simply absent, which is R9: *an
/// unavailable capability renders nothing.*
fn visible_ids(items: &[Item], conditions: &ConditionSet) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| match item {
            Item::Command {
                id, visible_when, ..
            } => {
                let shown = visible_when
                    .as_deref()
                    .is_none_or(|name| conditions.is_set(name));
                shown.then(|| id.clone())
            }
            // A separator or a custom item draws nothing here. See
            // `RailGroup::items` on why the variants are tolerated rather than
            // forbidden by the type.
            Item::Separator | Item::Custom { .. } => None,
        })
        .collect()
}

/// Build the strip at one rung, without asking whether it fits.
///
/// Split out from [`plan`] so the ladder is a loop over a pure function rather
/// than four branches that could each drift.
#[must_use]
pub fn build(rail: &Rail, conditions: &ConditionSet, rung: Rung) -> RailPlan {
    let mut rows: Vec<RailRow> = Vec::new();
    let mut folded: Vec<String> = Vec::new();
    let mut drawn_a_group = false;

    for group in rail.groups() {
        let ids = visible_ids(&group.items, conditions);
        if ids.is_empty() {
            // ★ R9 in the layout: a group whose every member is hidden by the
            // mode draws no caption and no rule. An empty captioned run is a
            // heading offering nothing — the same call `pdfcer-gui` made for
            // Edit ▸ Clipboard, and the same one `SideLayout::is_empty` makes
            // for a side with no columns.
            continue;
        }

        let selected = |id: &str| conditions.is_set(&crate::ribbon::band::selected_condition(id));

        let shown: Vec<(String, bool)> = match group.fold {
            RailFold::Never => ids.iter().map(|id| (id.clone(), false)).collect(),
            RailFold::Whole => {
                if rung.keeps_whole_groups() {
                    ids.iter().map(|id| (id.clone(), false)).collect()
                } else {
                    folded.extend(ids.iter().cloned());
                    continue;
                }
            }
            RailFold::PinArmed => {
                if rung.keeps_pinned_groups_entire() {
                    ids.iter().map(|id| (id.clone(), false)).collect()
                } else {
                    // ★★★ The pinned row is whatever is ARMED — and the
                    // fallback when nothing is, deliberately, is the group's
                    // FIRST member rather than no row at all. A pinned group
                    // that vanished when nothing was armed would be a strip
                    // that flickers a row in and out as the operator picks
                    // tools up and puts them down, at the rung where there is
                    // least room to absorb it.
                    let armed = ids
                        .iter()
                        .find(|id| selected(id))
                        .cloned()
                        .unwrap_or_else(|| ids[0].clone());
                    folded.extend(ids.iter().filter(|id| **id != armed).cloned());
                    vec![(armed, true)]
                }
            }
        };

        if drawn_a_group {
            rows.push(RailRow::Rule);
        }
        if rung.shows_words()
            && let Some(caption) = &group.caption
        {
            rows.push(RailRow::Caption(caption.clone()));
        }
        for (id, pinned) in shown {
            rows.push(RailRow::Entry {
                group: group.id.clone(),
                selected: selected(&id),
                id,
                with_label: rung.shows_words(),
                pinned,
            });
        }
        drawn_a_group = true;
    }

    // ★ The chevron is appended AFTER the ladder has run and is drawn only
    // when it holds something. A chevron over an empty overflow is the dead
    // control R9 forbids — an error `mockups/pdfcer-shell.html` avoids and
    // this build does not re-introduce.
    if !folded.is_empty() {
        rows.push(RailRow::Chevron {
            folded: folded.len(),
        });
    }

    let height_pts = if rows.is_empty() {
        0.0
    } else {
        PADDING_PTS + rows.iter().map(RailRow::height_pts).sum::<f32>()
    };

    RailPlan {
        rung,
        rows,
        folded,
        width_pts: WIDTH_PTS,
        height_pts,
    }
}

/// Walk the ladder and return the widest rung that fits in `height_budget`.
///
/// Falls through to [`Rung::Cramped`] when nothing fits, and that plan may be
/// taller than the budget — deliberately. The rail does not keep shrinking
/// past the floor; it **scrolls**, which is `RIBBON_SCALING.md` §3.3's third
/// rung, and the application wraps the rows in a `ScrollArea` to honour it.
/// The alternative — dropping rows until they fit — is the unreachable-control
/// defect, and it is the one this project already shipped on this exact
/// surface.
#[must_use]
pub fn plan(rail: &Rail, conditions: &ConditionSet, height_budget: f32) -> RailPlan {
    for rung in Rung::LADDER {
        let candidate = build(rail, conditions, rung);
        if candidate.height_pts <= height_budget {
            return candidate;
        }
    }
    build(rail, conditions, Rung::Cramped)
}

/// How wide the rail actually gets, given the side it must share.
///
/// Returns `0.0` when reserving the strip would leave the panel body below
/// [`super::plan::MIN_COLUMN_WIDTH`] — **absent rather than squeezed**, which
/// is [`super::banner::resolve_height`]'s rule and reasoning verbatim: a strip
/// that publishes a rectangle beside a panel too narrow to read is the shape
/// that let three panels ship unreachable with every gate green.
#[must_use]
pub fn resolve_width(side_width: f32) -> f32 {
    if !side_width.is_finite() {
        return 0.0;
    }
    if side_width - WIDTH_PTS < super::plan::MIN_COLUMN_WIDTH {
        return 0.0;
    }
    WIDTH_PTS
}

impl<'a> super::Dock<'a> {
    /// Draw a permanent vertical strip down `side`'s outer edge.
    ///
    /// The strip's width is [`WIDTH_PTS`] and is not negotiable — see this
    /// module's header. It is reserved off the side's rectangle **before** the
    /// columns are resolved, so the panels below it lose that width once
    /// rather than being painted over.
    ///
    /// The handler is called only when the side is drawn and wide enough to
    /// afford the strip — see [`resolve_width`].
    ///
    /// ```no_run
    /// # use egui_shell::dock::{Dock, DockSide, DockState};
    /// # fn frame(ui: &mut egui::Ui, state: &mut DockState) {
    /// let mut rail = |ui: &mut egui::Ui| {
    ///     ui.label("▤");
    /// };
    /// Dock::new()
    ///     .with_side_rail(DockSide::Left, &mut rail)
    ///     .show(ui, state, |_panel, _ui| {});
    /// # }
    /// ```
    ///
    /// # Borrowing
    ///
    /// [`Dock::with_tab_menu`](super::Dock::with_tab_menu)'s rule: this
    /// handler and the `body` closure both live across
    /// [`super::Dock::show`], so they cannot both capture `&mut` to the same
    /// thing. Record into a local and act after `show` returns.
    #[must_use]
    pub fn with_side_rail(
        mut self,
        side: DockSide,
        handler: &'a mut (impl FnMut(&mut egui::Ui) + 'a),
    ) -> Self {
        self.rail = Some((side, handler));
        self
    }
}

/// Reserve and draw the rail for `side`, returning the rectangle the columns
/// get.
///
/// Returns `area` unchanged when there is no rail for this side or the width
/// resolved to zero, so the no-rail path costs one comparison and changes no
/// geometry — which is what keeps every existing dock layout test valid.
///
/// # ★★ The region is published against the SIDE's `Ui`, not the child's
///
/// [`report::Reporter::report`]'s own doc states the rule: reporting a region
/// against a clip derived from itself is *"the tautology `visible == 1.0`
/// dressed up as a measurement"*. The question asked of
/// `dock.<side>.toolrail` is *can the operator reach this strip*, and only the
/// side's clip can answer it.
///
/// # ★ Why the region is not called `dock.<side>.rail`
///
/// That name is taken, by [`report::rail`], for a **different feature**: the
/// sliver a *collapsed* side leaves behind as the way back. The mockup's
/// legend draws the distinction explicitly — *"This one replaces the dock's
/// arrangement while the dock is open. VS Code's activity bar, not its
/// collapsed sidebar."* Two surfaces sharing one trace name is how a driven
/// check reads the wrong one, which
/// `D:/dev/rag/egui/two_trace_lines_sharing_an_event_name_make_a_check_read_the_wrong_one.md`
/// records costing a day.
pub(super) fn draw(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    side: DockSide,
    area: Rect,
    rail: Option<&mut (DockSide, &mut RailHandler<'_>)>,
) -> Rect {
    let Some((wanted_side, handler)) = rail else {
        return area;
    };
    if *wanted_side != side {
        return area;
    }
    let width = resolve_width(area.width());
    if width <= 0.0 {
        return area;
    }

    // The strip hugs the OUTER edge — the window edge, away from the document
    // — because that is where a permanent activity bar goes in every program
    // that has one, and because the inner edge already carries the side
    // splitter. Two draggable-looking things on one edge is failure mode #1.
    let (strip, rest) = match side {
        DockSide::Left => (
            Rect::from_min_max(area.min, egui::pos2(area.left() + width, area.bottom())),
            Rect::from_min_max(egui::pos2(area.left() + width, area.top()), area.max),
        ),
        DockSide::Right => (
            Rect::from_min_max(egui::pos2(area.right() - width, area.top()), area.max),
            Rect::from_min_max(area.min, egui::pos2(area.right() - width, area.bottom())),
        ),
    };
    ctx.reporter.report(ui, strip, || report::tool_rail(side));

    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(strip)
            .layout(Layout::top_down(egui::Align::Center)),
    );
    // Set explicitly rather than inherited, for `banner::draw`'s reason: a
    // child built from `max_rect` alone keeps its parent's clip, so a row
    // wider than 52 pt would paint over the panel body — visible, unclickable,
    // and indistinguishable in a screenshot from a layout fault.
    child.set_clip_rect(strip.intersect(ui.clip_rect()));
    handler(&mut child);

    rest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::RailGroup;

    /// The rail the mockup draws, minus the lasso — which does not exist. See
    /// the report and `pdfcer-gui`'s own rail module on that choice.
    fn pdfcer_rail() -> Rail {
        [
            RailGroup::new(
                "tabs",
                [
                    Item::command("view.panel_pages"),
                    Item::command("view.panel_bookmarks"),
                    Item::command("view.panel_layers"),
                    Item::command("view.panel_signatures"),
                    Item::command("view.panel_fonts"),
                ],
            ),
            RailGroup::new(
                "navigate",
                [
                    Item::command("view.tool_select"),
                    Item::command("view.tool_node").shown_when("mode.edit_content"),
                    Item::command("view.tool_text"),
                    Item::command("view.tool_hand"),
                ],
            )
            .with_caption("navigate")
            .with_fold(RailFold::PinArmed),
            RailGroup::new(
                "pages",
                [
                    Item::command("pages.rotate_left"),
                    Item::command("pages.rotate_right"),
                ],
            )
            .with_caption("rotate")
            .with_fold(RailFold::Whole),
        ]
        .into_iter()
        .collect()
    }

    fn edit() -> ConditionSet {
        ConditionSet::default().with("mode.edit_content")
    }

    fn ids(plan: &RailPlan) -> Vec<&str> {
        plan.rows
            .iter()
            .filter_map(|r| match r {
                RailRow::Entry { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect()
    }

    fn tall_enough(rail: &Rail, conditions: &ConditionSet, rung: Rung) -> f32 {
        build(rail, conditions, rung).height_pts
    }

    // ---------------------------------------------------------------------
    // The ladder
    // ---------------------------------------------------------------------

    /// ★★★ **The fold ladder, rung by rung, in the order the mockup shows.**
    ///
    /// Roomy has words; Tight drops them and keeps every control; Snug folds
    /// the `Whole` group; Cramped pins the navigate group to one row.
    #[test]
    fn the_ladder_gives_up_words_then_whole_groups_then_pins_the_armed_tool() {
        let rail = pdfcer_rail();
        let c = edit();

        let roomy = build(&rail, &c, Rung::Roomy);
        assert!(
            roomy
                .rows
                .iter()
                .any(|r| matches!(r, RailRow::Caption(w) if w == "navigate")),
            "roomy draws captions: {roomy:?}"
        );
        assert!(
            roomy.rows.iter().all(|r| !matches!(
                r,
                RailRow::Entry {
                    with_label: false,
                    ..
                }
            )),
            "roomy draws every entry with its word: {roomy:?}"
        );
        assert!(roomy.folded.is_empty(), "roomy folds nothing");

        let tight = build(&rail, &c, Rung::Tight);
        assert_eq!(ids(&tight), ids(&roomy), "tight keeps every control");
        assert!(
            tight.rows.iter().all(|r| !matches!(r, RailRow::Caption(_))),
            "tight drops the captions: {tight:?}"
        );
        assert!(
            tight.rows.iter().all(|r| !matches!(
                r,
                RailRow::Entry {
                    with_label: true,
                    ..
                }
            )),
            "tight drops the words: {tight:?}"
        );
        assert!(tight.folded.is_empty(), "tight still folds nothing");

        let snug = build(&rail, &c, Rung::Snug);
        assert_eq!(
            snug.folded,
            vec!["pages.rotate_left", "pages.rotate_right"],
            "snug folds the Whole group and only it"
        );
        assert!(
            ids(&snug).contains(&"view.tool_hand"),
            "snug keeps the navigate group entire"
        );

        let cramped = build(&rail, &c, Rung::Cramped);
        let nav_rows: Vec<&str> = ids(&cramped)
            .into_iter()
            .filter(|id| id.starts_with("view.tool_"))
            .collect();
        assert_eq!(
            nav_rows.len(),
            1,
            "cramped collapses navigate to a single row: {cramped:?}"
        );
    }

    /// ★★ **The `Never` group is drawn entire at every rung.**
    ///
    /// The five panel tabs are the rail's whole argument for existing.
    #[test]
    fn the_panel_tabs_never_fold_at_any_rung() {
        let rail = pdfcer_rail();
        let c = edit();
        for rung in Rung::LADDER {
            let plan = build(&rail, &c, rung);
            for id in [
                "view.panel_pages",
                "view.panel_bookmarks",
                "view.panel_layers",
                "view.panel_signatures",
                "view.panel_fonts",
            ] {
                assert!(
                    ids(&plan).contains(&id),
                    "{id} must be drawn at {rung:?}: {plan:?}"
                );
                assert!(
                    !plan.folded.iter().any(|f| f == id),
                    "{id} must never be folded, and was at {rung:?}"
                );
            }
        }
    }

    /// ★ **The chevron never folds itself, and never draws over nothing.**
    #[test]
    fn the_chevron_is_last_present_only_when_it_holds_something_and_never_folded() {
        let rail = pdfcer_rail();
        let c = edit();
        for rung in Rung::LADDER {
            let plan = build(&rail, &c, rung);
            let chevrons: Vec<&RailRow> = plan
                .rows
                .iter()
                .filter(|r| matches!(r, RailRow::Chevron { .. }))
                .collect();
            if plan.folded.is_empty() {
                assert!(chevrons.is_empty(), "no chevron over an empty overflow");
            } else {
                assert_eq!(chevrons.len(), 1, "exactly one chevron");
                assert!(
                    matches!(
                        plan.rows.last(),
                        Some(RailRow::Chevron { folded }) if *folded == plan.folded.len()
                    ),
                    "the chevron is the last row and counts what it holds: {plan:?}"
                );
            }
        }
    }

    // ---------------------------------------------------------------------
    // The armed tool
    // ---------------------------------------------------------------------

    /// ★★★ **The pinned row is whatever is armed** — including a tool armed
    /// from a ribbon tab that is not open, which is exactly what the
    /// `selected:` condition already reports and this planner reads.
    #[test]
    fn the_pinned_row_is_the_armed_tool() {
        let rail = pdfcer_rail();
        let c = edit().with("selected:view.tool_hand");

        let plan = build(&rail, &c, Rung::Cramped);
        let pinned: Vec<&str> = plan
            .rows
            .iter()
            .filter_map(|r| match r {
                RailRow::Entry {
                    id, pinned: true, ..
                } => Some(id.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(pinned, vec!["view.tool_hand"], "{plan:?}");
        assert!(
            plan.folded.iter().any(|f| f == "view.tool_select"),
            "the rest of the group went behind the chevron: {plan:?}"
        );
        assert!(
            !plan.folded.iter().any(|f| f == "view.tool_hand"),
            "the armed tool is not also folded"
        );
    }

    /// With nothing armed the group still shows one row — its first — rather
    /// than flickering in and out as tools are picked up and put down.
    #[test]
    fn a_pinned_group_with_nothing_armed_shows_its_first_member() {
        let rail = pdfcer_rail();
        let plan = build(&rail, &edit(), Rung::Cramped);
        assert!(
            matches!(
                plan.rows.iter().find(|r| matches!(
                    r,
                    RailRow::Entry { pinned: true, .. }
                )),
                Some(RailRow::Entry { id, .. }) if id == "view.tool_select"
            ),
            "{plan:?}"
        );
    }

    // ---------------------------------------------------------------------
    // Mode gating
    // ---------------------------------------------------------------------

    /// ★★ **Read drops Points**, and it is absent rather than folded.
    ///
    /// R9: an unavailable capability renders nothing. Folding it would put it
    /// behind the chevron, where the operator could reach a control the
    /// mode's own dispatch refuses.
    #[test]
    fn read_mode_drops_the_points_tool_entirely() {
        let rail = pdfcer_rail();
        let read = ConditionSet::default(); // `mode.edit_content` not set
        for rung in Rung::LADDER {
            let plan = build(&rail, &read, rung);
            assert!(
                !ids(&plan).contains(&"view.tool_node"),
                "Points is not drawn in Read at {rung:?}: {plan:?}"
            );
            assert!(
                !plan.folded.iter().any(|f| f == "view.tool_node"),
                "Points is not reachable behind the chevron in Read at {rung:?}"
            );
        }
    }

    /// ★ **Rotate is in the rail in every mode, Read included** — O126.
    #[test]
    fn rotate_is_present_in_read_mode() {
        let rail = pdfcer_rail();
        let read = ConditionSet::default();
        let plan = build(&rail, &read, Rung::Roomy);
        assert!(ids(&plan).contains(&"pages.rotate_left"), "{plan:?}");
        assert!(ids(&plan).contains(&"pages.rotate_right"), "{plan:?}");
    }

    /// A group whose every member the mode hid draws no caption and no rule.
    #[test]
    fn a_group_hidden_entirely_by_the_mode_draws_no_caption() {
        let rail: Rail = [
            RailGroup::new("tabs", [Item::command("view.panel_pages")]),
            RailGroup::new(
                "select",
                [Item::command("edit.lasso").shown_when("mode.edit_content")],
            )
            .with_caption("select"),
        ]
        .into_iter()
        .collect();
        let plan = build(&rail, &ConditionSet::default(), Rung::Roomy);
        assert!(
            plan.rows
                .iter()
                .all(|r| !matches!(r, RailRow::Caption(_) | RailRow::Rule)),
            "no heading and no rule over nothing: {plan:?}"
        );
    }

    // ---------------------------------------------------------------------
    // The width
    // ---------------------------------------------------------------------

    /// ★★★ **The width is the same constant at every rung and every budget.**
    ///
    /// The R128 argument in a test: nothing about the content, the rung, the
    /// number of folded entries or the length of a caption can move it. A
    /// build that sized the rail from its widest word would fail here.
    #[test]
    fn the_width_is_constant_at_every_rung_and_every_budget() {
        let mut rail = pdfcer_rail();
        // A caption far wider than the strip, on purpose: if width were ever
        // derived from content this is what would move it.
        rail.0[1].caption = Some("an extremely long caption nobody would author".to_owned());
        let c = edit();

        for rung in Rung::LADDER {
            assert!(
                (build(&rail, &c, rung).width_pts - WIDTH_PTS).abs() < f32::EPSILON,
                "width moved at {rung:?}"
            );
        }
        let mut budget = 0.0_f32;
        while budget <= 2_000.0 {
            let plan = plan(&rail, &c, budget);
            assert!(
                (plan.width_pts - WIDTH_PTS).abs() < f32::EPSILON,
                "width moved at budget {budget}: {plan:?}"
            );
            budget += 7.0;
        }
    }

    // ---------------------------------------------------------------------
    // The budget walk
    // ---------------------------------------------------------------------

    /// [`plan`] takes the widest rung that fits, and falls through to
    /// `Cramped` rather than dropping rows when nothing does.
    #[test]
    fn the_planner_takes_the_widest_rung_that_fits_and_never_drops_rows() {
        let rail = pdfcer_rail();
        let c = edit();

        assert_eq!(
            plan(&rail, &c, tall_enough(&rail, &c, Rung::Roomy)).rung,
            Rung::Roomy
        );
        assert_eq!(
            plan(&rail, &c, tall_enough(&rail, &c, Rung::Tight)).rung,
            Rung::Tight
        );
        assert_eq!(
            plan(&rail, &c, tall_enough(&rail, &c, Rung::Snug)).rung,
            Rung::Snug
        );

        // Nothing fits. The floor is Cramped and it is still taller than the
        // budget — the strip scrolls rather than shedding controls.
        let starved = plan(&rail, &c, 10.0);
        assert_eq!(starved.rung, Rung::Cramped);
        assert!(
            starved.height_pts > 10.0,
            "the floor plan is allowed to overflow: {starved:?}"
        );
        assert_eq!(
            ids(&starved)
                .iter()
                .filter(|id| id.starts_with("view.panel_"))
                .count(),
            5,
            "and it still holds all five panel tabs: {starved:?}"
        );
    }

    /// The width is refused outright when reserving it would leave the panel
    /// body unreadable — absent rather than squeezed.
    #[test]
    fn a_side_too_narrow_for_both_gets_no_rail() {
        assert_eq!(resolve_width(WIDTH_PTS + 10.0), 0.0);
        assert_eq!(resolve_width(f32::NAN), 0.0);
        assert!(
            (resolve_width(WIDTH_PTS + super::super::plan::MIN_COLUMN_WIDTH) - WIDTH_PTS).abs()
                < f32::EPSILON
        );
        assert!((resolve_width(360.0) - WIDTH_PTS).abs() < f32::EPSILON);
    }
}
