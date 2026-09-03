//! # `canvas::guides` — draggable alignment lines, whose home is a page and whose life is a file
//!
//! The third of `RIBBON_IA.md` §5.2's *"Rulers · Grid · Guides"*, and the one
//! `super::manifest::PLANNED` singled out with a condition attached:
//!
//! > `view.guides` — *"N — draggable guides, which need a per-document store
//! > to survive a reopen."*
//!
//! That sentence is the specification for this module. It names the two hard
//! parts — **draggable**, and **survives a reopen** — and this header answers
//! both, plus the two questions they imply: *what does a guide belong to*, and
//! *where does it live on disk*.
//!
//! ---
//!
//! ## ★ 1. A guide belongs to a PAGE, and that follows from the grid
//!
//! [`super::rulers`]' header §2 settles the space the grid is drawn in: page
//! space, per page, anchored to the sheet's own corner, because a reference
//! that is not attached to the sheet cannot make a statement about the
//! drawing. A guide is the same kind of object for the same reason, only
//! placed by hand instead of by a ladder — so it is stored as
//! `(page, axis, canvas-space coordinate)` and nothing else.
//!
//! The alternatives, and why each is worse:
//!
//! | model | what breaks |
//! |---|---|
//! | **viewport-space** (a line at a window position) | scrolling moves it off whatever it was aligned to; it means nothing the moment the operator scrolls |
//! | **document-wide, applied to every page** | tempting for a 36-sheet set of identical drawings, and wrong the first time a set is mixed A3 and A1 — a guide at *y* = 500 is inside one sheet and off the end of another, and there is nothing to say which |
//! | **per page** *(this)* | a guide is where the operator put it, on the sheet they put it on, for as long as that sheet exists |
//!
//! The per-page answer is also Acrobat's, which matters because an operator
//! coming from the comparison product should not have to relearn what a guide
//! is attached to. A document-wide *copy* verb ("put this guide on every
//! sheet") is a plausible future convenience and is a **different feature**:
//! it would place N guides, one per page, and every one of them would still be
//! a page's guide.
//!
//! ---
//!
//! ## ★ 2. Where guides live on disk — a fourth file, following Phase 4's precedent
//!
//! Phase 4 landed `page-display.txt` beside `layout.ron` and `recent.txt` as a
//! *third* store, and [`crate::viewer::remembered`]'s header carries the
//! argument in full. Its three reasons transfer here one for one, so this is a
//! **fourth** file, `guides.txt`, rather than a field in any of the three:
//!
//! 1. **The lifetimes differ.** `recent.txt` is capped at ten because it is
//!    *drawn* in a menu. `page-display.txt` is capped at two hundred because
//!    the cap is about disk. This one is capped at [`CAP`] documents for the
//!    disk reason as well, but its *rows are unbounded in width* — a document
//!    can carry many guides — which neither of the others can express.
//! 2. **Forgetting means different things.** "Clear recent files" must not
//!    delete the guides an operator ruled up on a drawing, and clearing the
//!    guides on a sheet must not evict it from the recent menu or reset its
//!    page-display mode. Separate files make that true by construction rather
//!    than by a rule somebody has to honour.
//! 3. **The format cannot serve either.** `recent.txt` is one path per line
//!    and nothing else, deliberately; `page-display.txt` is exactly one mode
//!    id and one path. Adding a variable-length payload to either makes every
//!    existing line ambiguous with the format it replaced.
//!
//! **Why guides persist when the three toggles do not.** `view.rulers`,
//! `view.grid` and `view.guides` are per-document view state that starts off
//! and is not written anywhere — see [`crate::viewer::ViewState`]. The guides
//! themselves *are* written. The distinction is not inconsistency, it is the
//! difference between a switch and a work product: switching the grid on again
//! costs one click, while re-placing six guides means measuring six positions
//! again, and losing work is a different class of loss from losing a switch.
//!
//! The consequence that falls out of it, and is deliberate: **a document with
//! remembered guides opens with `view.guides` already on.** The presence of
//! the work *is* the preference, so it does not need storing separately, and
//! the alternative — restoring invisible guides and waiting for the operator
//! to discover a toggle — would be a feature that appears not to have worked.
//! See [`crate::app::state::OpenDoc::new`].
//!
//! ### The format
//!
//! ```text
//! 0:h:120.5 0:v:64 3:h:200<TAB>D:\Drawings\job-4471\sheet-set.pdf
//! 0:v:306<TAB>C:\Users\ken\Documents\report.pdf
//! ```
//!
//! (`<TAB>` stands for one U+0009; the real file carries the character.)
//!
//! One line per document, most recently written first, UTF-8, no header. The
//! separator between the payload and the path is a **tab** — the one ASCII
//! character a Windows path cannot contain, so it needs no escaping — and the
//! payload comes **first** because the path is the part that may contain
//! spaces and must therefore be the whole remainder of the line. Each guide is
//! `page:axis:coordinate`, space separated, with the axis spelled `h` or `v`
//! by [`GuideAxis::id`].
//!
//! A malformed guide is **dropped**, a malformed line is **dropped**, and a
//! document with no guides is **not written at all**. A corrupt file therefore
//! degrades into fewer guides rather than into an error the operator has to
//! dismiss about a preference — exactly as `recent.txt` and
//! `page-display.txt` do.
//!
//! Flat text rather than RON for the reason `remembered.rs` gives: **this
//! crate cannot serialize.** `serde` and `ron` are dependencies of
//! `egui-shell`, not of `pdfcer-gui`.
//!
//! ---
//!
//! ## ★ 3. Dragging, and why it cannot disturb the selection
//!
//! Two gestures create and move guides, and both are the ones every peer uses:
//!
//! * **drag out of a ruler** — the top ruler yields a horizontal guide, the
//!   left ruler a vertical one;
//! * **drag the guide itself** on the page, to move it;
//! * **release anywhere that is not a page** — the grey between sheets, a
//!   ruler, off the window — and the guide is discarded (if it was being
//!   created) or **deleted** (if it existed). One rule, both cases.
//! * **double-click a guide** to delete it without a drag, which is the only
//!   route that works with the rulers switched off.
//!
//! ### The interaction hazard, and the two-line fix
//!
//! The canvas's primary button is spoken for: it selects, it marquees, and
//! under the hand tool it pans. A guide drag that reached
//! [`super::gesture::GestureState`] would be a drag that moved a guide **and**
//! rubber-banded a selection.
//!
//! It cannot, and the mechanism is egui's own rather than a check anybody has
//! to remember: [`canvas_drag`] registers each guide's catch band **after**
//! every page widget in the same layer, so the band is the topmost widget
//! under the pointer and wins the interaction outright. The page's own
//! `Response` then reports no press at all, `interact`'s step 1 builds an
//! empty [`super::gesture::PointerFrame`], and the gesture machine sees an
//! idle frame. Nothing is suppressed, because nothing was offered — which is
//! the same shape as the hand tool's fix (`canvas`'s header: *"the gesture
//! simply is not offered, which is the only version of this that cannot leave
//! a half-applied selection behind"*).
//!
//! The ruler-started drag has the same property for free: a gutter is outside
//! the scroll area and outside every page widget, so a press there was never
//! the canvas's to begin with.
//!
//! ### Why the in-flight drag is read from raw pointer input rather than from
//! a `Response`
//!
//! Because the two entry points must resolve identically, and because a
//! `Response` is keyed on an `egui::Id` that encodes the guide's index — and
//! the index moves when a guide is added or removed. Reading `primary_released`
//! from the input state once the drag has started makes the release path one
//! function that neither entry point can diverge from, and makes it immune to
//! the widget the drag started on disappearing mid-gesture.
//!
//! **Escape does not cancel a guide drag, and that is stated rather than
//! hidden.** `canvas::keys` owns Escape's precedence between the gesture
//! machine and the selection ladder, and a third claimant would need a rule
//! there. Releasing over a ruler or over the grey already cancels, which is
//! the peer convention, so the gap costs the operator nothing they cannot do
//! another way.
//!
//! ---
//!
//! ## ★ 4. Rule 4
//!
//! A guide is a **pre-commit affordance in `overlay`'s second category** — the
//! cursor, describing where the operator has decided something belongs. It is
//! not keyed on any property of the content, it is placed by hand rather than
//! inferred, it changes nothing a save would write, and it disappears the
//! instant `view.guides` is switched off. `overlay`'s one-line test — *would a
//! screenshot of the editing canvas differ from a screenshot of the same
//! document saved and reopened?* — answers **yes, because the operator asked
//! for it**, which is the answer rule 4 admits. The version that would fail is
//! a guide pdfcer placed *itself*, on a margin or a frame it detected. There is
//! no such code path and there must not be.

use std::path::{Path, PathBuf};

use egui::{Context, Id, Pos2, Rect, Sense, Stroke, Ui, pos2};
use pdfcer_core::settings;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::rulers::{CanvasGeometry, Gutters};
use crate::canvas::strip::PageView;

/// The file's name, inside the settings directory.
///
/// `.txt` because the format is one line per document and an operator who
/// opens it should find exactly what they expect — the same argument
/// `recent.txt` and `page-display.txt` make. `.ron` would promise a structure
/// that is not there and a parser this crate does not have.
pub const GUIDES_FILE: &str = "guides.txt"; // ui-text-exempt: a file name, never displayed as copy

/// The separator between the guide payload and the path.
///
/// A tab, because it is the one ASCII character a Windows path cannot contain
/// and therefore the one that needs no escaping. See the module header.
const SEPARATOR: char = '\t';

/// The separator between a guide's three fields.
const FIELD: char = ':';

/// How many documents are remembered.
///
/// Two hundred, matching [`crate::viewer::remembered::CAP`] and for the same
/// reason: nothing draws this list, so the cap that governs the recent menu
/// ("what fits without becoming a scroll view") does not apply, and the only
/// constraint is disk. A cap exists at all because the file is rewritten on
/// every guide change and an uncapped one would grow for the life of an
/// installation.
pub const CAP: usize = 200;

/// How many guides one document may carry.
///
/// Not a disk limit — a thousand guides is a few kilobytes — but a **legibility
/// and cost** one. Every guide is a line drawn across its page and a catch band
/// registered as a widget, so an operator who has somehow accumulated hundreds
/// has a canvas they cannot see the drawing through and a frame that registers
/// hundreds of widgets. Refusing further guides at a number far above any real
/// use is cheaper than discovering the ceiling from a frame time.
///
/// The refusal is silent, which is the one place this module is knowingly
/// short: it belongs on the edit-disclosure surface `FEATURES.md` still lists
/// as unbuilt, alongside the other worded declines.
pub const MAX_PER_DOCUMENT: usize = 256;

/// The half-width, in logical points, of the band that catches a guide drag.
///
/// A **screen**-space radius, like [`super::mapping::SELECT_SCREEN_TOLERANCE_PX`]
/// and for the identical reason recorded there: a page-space catch radius is
/// `radius × zoom` pixels on screen, so a guide that is easy to grab at 100 %
/// is un-grabbable at 25 % — exactly the zoom an operator uses to see a whole
/// sheet.
///
/// Deliberately smaller than the 6-point selection radius. A guide is a line
/// the operator can see, so aiming at it is easy; and the cost of a miss is
/// asymmetric — missing a guide starts a marquee the operator can abandon with
/// Escape, while catching a guide the operator did not aim at moves something
/// they had positioned deliberately.
const CATCH_PTS: f32 = 4.0;

/// The alpha, out of 255, of a placed guide.
///
/// High enough to read over dense linework — a guide the operator cannot see
/// on a CAD sheet is a guide that is not there — and short of opaque, because
/// a guide crosses the whole page and an opaque line would compete with the
/// drawing along its entire length. The same trade `overlay::GHOST_ALPHA`
/// records for the move ghost, at the same order of magnitude.
const GUIDE_ALPHA: u8 = 170;

/// The alpha of a guide preview that would be **discarded** on release.
///
/// Half again fainter than a placed guide, and that difference *is* the
/// feedback: while the pointer is over the grey or over a ruler, the line the
/// operator is dragging says "release here and this does not happen". No
/// second colour, no second shape, no wording — emphasis, which is the same
/// answer `overlay`'s current-find-hit reached when it could not have a second
/// hue.
const DISCARD_ALPHA: u8 = 60;

/// `egui::Memory` key for the in-flight guide drag.
///
/// In `Memory` for the reason `canvas`'s `GESTURE_MEMORY_KEY` states and not
/// the one the selection was moved off it for: a drag that is happening *right
/// now* is genuinely frame-local UI state with no meaning across a document,
/// and keying it here means a document change starts the next frame with no
/// drag in flight, by construction. What the drag *produces* is a
/// [`Action::SetGuides`] applied after the frame, through the one funnel.
const DRAG_KEY: &str = "pdfcer-canvas-guide-drag"; // ui-text-exempt: internal memory id, never displayed

/// `egui::Id` base for the guides' catch bands.
const BAND_KEY: &str = "pdfcer-canvas-guide-band"; // ui-text-exempt: internal widget id, never displayed

// ---------------------------------------------------------------------------
// The model
// ---------------------------------------------------------------------------

/// Which way a guide runs.
///
/// Named for the **line**, not for the coordinate it fixes, because that is
/// what the operator sees: a *horizontal* guide is a horizontal line, and it
/// is dragged out of the horizontal (top) ruler. The coordinate it pins is the
/// other axis, which is stated once, here, and nowhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideAxis {
    /// A horizontal line, pinning a canvas **y**. Dragged from the top ruler.
    Horizontal,
    /// A vertical line, pinning a canvas **x**. Dragged from the left ruler.
    Vertical,
}

impl GuideAxis {
    /// The on-disk spelling.
    ///
    /// Beside the type, exactly as [`crate::viewer::PageDisplay::id`] is, and
    /// for the reason `remembered.rs`'s header gives: a variant added without
    /// a spelling is then a compile error rather than a silently unsaveable
    /// guide.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            // ui-text-exempt: on-disk spelling, never displayed as copy
            GuideAxis::Horizontal => "h",
            // ui-text-exempt: on-disk spelling, never displayed as copy
            GuideAxis::Vertical => "v",
        }
    }

    /// The axis an on-disk spelling names, or `None`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "h" => Some(GuideAxis::Horizontal),
            "v" => Some(GuideAxis::Vertical),
            _ => None,
        }
    }

    /// The component of a canvas point this axis pins.
    fn of(self, p: Pos2) -> f32 {
        match self {
            GuideAxis::Horizontal => p.y,
            GuideAxis::Vertical => p.x,
        }
    }
}

/// One guide: a line at a fixed place on one page.
///
/// `at` is in **canvas space** — Y-down, origin at the page's top-left,
/// `/Rotate` applied — which is the space the ruler reads in, the space the
/// `canvas-pointer` trace calls `page=`, and the space the selection outlines
/// are cached in. Storing it in screen coordinates is the first of the three
/// failures `GUI_ROADMAP.md` Phase 1 names for a selection model, and a guide
/// is subject to the identical hazard: a screen coordinate is meaningless the
/// moment the operator zooms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Guide {
    /// The 0-based page it belongs to.
    pub page: usize,
    /// Which way the line runs.
    pub axis: GuideAxis,
    /// The canvas-space coordinate it pins.
    pub at: f32,
}

impl Guide {
    /// Its line on screen, spanning the page it belongs to.
    ///
    /// Across the **page**, not across the viewport, and that is the visual
    /// half of "a guide belongs to a sheet": a line that ran on into the grey
    /// would look like a property of the window.
    fn segment(self, map: PageMapping) -> [Pos2; 2] {
        let page = map.image_rect();
        match self.axis {
            GuideAxis::Horizontal => {
                let y = map.to_screen(pos2(0.0, self.at)).y;
                [pos2(page.min.x, y), pos2(page.max.x, y)]
            }
            GuideAxis::Vertical => {
                let x = map.to_screen(pos2(self.at, 0.0)).x;
                [pos2(x, page.min.y), pos2(x, page.max.y)]
            }
        }
    }

    /// The screen-space band a press must land in to grab this guide.
    ///
    /// [`CATCH_PTS`] either side of the line, and no further along it than the
    /// page goes — so a guide cannot be grabbed from the grey beside its own
    /// sheet, which under a continuous mode would mean grabbing a guide the
    /// operator cannot see.
    fn band(self, map: PageMapping) -> Rect {
        let [a, b] = self.segment(map);
        Rect::from_two_pos(a, b).expand2(match self.axis {
            GuideAxis::Horizontal => egui::vec2(0.0, CATCH_PTS),
            GuideAxis::Vertical => egui::vec2(CATCH_PTS, 0.0),
        })
    }
}

/// Every guide one document carries.
///
/// A flat `Vec` rather than a map keyed by page, because the whole collection
/// is small (bounded by [`MAX_PER_DOCUMENT`]), because every consumer either
/// wants one page's worth or all of it, and because the on-disk format is a
/// flat list — one shape end to end is one fewer place to get an ordering
/// wrong.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Guides(Vec<Guide>);

impl Guides {
    /// Whether there are none at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// How many there are, across every page.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Every guide, in the order they will be written.
    pub fn iter(&self) -> impl Iterator<Item = Guide> + '_ {
        self.0.iter().copied()
    }

    /// The guides on `page`, with their index in the whole collection.
    ///
    /// The index rides along because it is the identity a drag names: a guide
    /// has no id of its own, and comparing on the coordinate would confuse two
    /// guides an operator had deliberately placed together.
    pub fn on_page(&self, page: usize) -> impl Iterator<Item = (usize, Guide)> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter(move |(_, g)| g.page == page)
            .map(|(i, g)| (i, *g))
    }

    /// Add `guide`, unless the document is already at [`MAX_PER_DOCUMENT`].
    ///
    /// Returns whether it was added, so a caller that wants to word the
    /// refusal later has something to word it from — see
    /// [`MAX_PER_DOCUMENT`]'s note on the disclosure this is currently short
    /// of.
    pub fn add(&mut self, guide: Guide) -> bool {
        if self.0.len() >= MAX_PER_DOCUMENT || !guide.at.is_finite() {
            return false;
        }
        self.0.push(guide);
        true
    }

    /// Replace the guide at `index`, if there is one.
    pub fn replace(&mut self, index: usize, guide: Guide) {
        if guide.at.is_finite()
            && let Some(slot) = self.0.get_mut(index)
        {
            *slot = guide;
        }
    }

    /// Remove the guide at `index`, if there is one.
    pub fn remove(&mut self, index: usize) {
        if index < self.0.len() {
            self.0.remove(index);
        }
    }

    /// The on-disk payload — the part of a line before the tab.
    fn encode(&self) -> String {
        self.0
            .iter()
            .map(|g| format!("{}{FIELD}{}{FIELD}{}", g.page, g.axis.id(), g.at))
            .collect::<Vec<_>>()
            .join(" ") // ui-text-exempt: the on-disk field separator, never displayed
    }

    /// Parse an on-disk payload, dropping anything that does not parse.
    ///
    /// Dropping rather than failing, for the reason the module header gives:
    /// every malformed state means the same thing to the caller — there is no
    /// guide there to restore — and a preference is not worth an error path.
    fn decode(payload: &str) -> Self {
        let mut out = Self::default();
        for token in payload.split_whitespace() {
            let mut parts = token.split(FIELD);
            let (Some(page), Some(axis), Some(at), None) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let (Ok(page), Some(axis), Ok(at)) = (
                page.parse::<usize>(),
                GuideAxis::from_id(axis),
                at.parse::<f32>(),
            ) else {
                continue;
            };
            out.add(Guide { page, axis, at });
        }
        out
    }
}

// ---------------------------------------------------------------------------
// The store
// ---------------------------------------------------------------------------

/// The path this store reads and writes, or `None` when `pdfcer-core` found no
/// writable location.
///
/// Derived from the same `pdfcer_core::settings::resolve_store()` call that
/// decides where `settings.txt`, `layout.ron`, `recent.txt` and
/// `page-display.txt` go — never a directory computed here.
/// `persistence.rs`'s header carries the reasons in full; the short one is
/// that a second resolution is how two of an application's files end up in
/// different folders.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    settings::resolve_store()
        .directory()
        .map(|dir| dir.join(GUIDES_FILE))
}

/// **The guides remembered for `document`**, or none.
///
/// Never fails. A missing file, an unreadable one and a corrupt one all answer
/// an empty set, because every one of them means the same thing to the caller.
#[must_use]
pub fn recall(document: &Path) -> Guides {
    recall_at(default_path().as_deref(), document)
}

/// **Remember `guides` against `document`.**
///
/// Read-modify-write of the whole file: the entry moves to the front, any
/// previous entry for the same document is replaced rather than duplicated,
/// and the list is truncated to [`CAP`]. A document whose guides are now empty
/// has its line **removed** rather than written empty, so clearing the last
/// guide genuinely forgets the document instead of leaving a marker behind.
///
/// Writes immediately rather than debouncing, for the reason `recent.rs` gives
/// and which holds more strongly here: a guide change is a discrete gesture's
/// *release*, not a drag reporting sixty changes a second — [`canvas_drag`]
/// and [`ruler_drag`] raise nothing at all until the pointer comes up.
///
/// Failures are traced and otherwise ignored. There is no operator-facing
/// consequence worth a dialog: the guide is on the canvas either way, and the
/// only loss is that it will not be there on the next open.
pub fn remember(document: &Path, guides: &Guides) {
    remember_at(default_path().as_deref(), document, guides);
}

/// ★ **What a freshly opened document starts with**: its remembered guides,
/// and the view state that shows them.
///
/// The two halves are returned together because the rule joining them is the
/// point, and it belongs here rather than in
/// [`crate::app::state::OpenDoc::new`]: **a document that has remembered
/// guides opens with `view.guides` already on.**
///
/// The presence of the work *is* the preference — see this module's header §2
/// on why the three View ▸ Display toggles are not persisted while the guides
/// are. The alternative, restoring guides and leaving them invisible until the
/// operator finds a switch, is a feature that appears not to have worked; and
/// storing a fourth flag to say "show the things I just restored" would be
/// storing something derivable.
///
/// Every other field of the returned [`crate::viewer::ViewState`] is
/// `Default`, which is the conservative one — the same division of labour
/// `ViewState::default`'s own docs describe for Read mode's continuous
/// default: the path that knows the document is the path that may know better.
#[must_use]
pub fn opening(document: &Path) -> (Guides, crate::viewer::ViewState) {
    let guides = recall(document);
    let view = crate::viewer::ViewState {
        guides: !guides.is_empty(),
        ..crate::viewer::ViewState::default()
    };
    (guides, view)
}

/// [`recall`], against an explicit file — the seam tests use.
///
/// The twin of `viewer::remembered::recall_at` and of
/// `pdfcer_core::settings::store_in`, and it exists for the same two reasons:
/// tests, and a future `--user-data-dir` override.
#[must_use]
pub fn recall_at(file: Option<&Path>, document: &Path) -> Guides {
    let wanted = absolute(document);
    let Some(file) = file else {
        return Guides::default();
    };
    let Ok(text) = std::fs::read_to_string(file) else {
        return Guides::default();
    };
    for (payload, path) in parse(&text) {
        if path == wanted {
            return Guides::decode(&payload);
        }
    }
    Guides::default()
}

/// [`remember`], against an explicit file.
pub fn remember_at(file: Option<&Path>, document: &Path, guides: &Guides) {
    let Some(file) = file else {
        return;
    };
    let wanted = absolute(document);
    let previous = std::fs::read_to_string(file).unwrap_or_default();
    let mut lines: Vec<String> = Vec::with_capacity(CAP);
    if !guides.is_empty() {
        lines.push(format!(
            "{}{SEPARATOR}{}",
            guides.encode(),
            wanted.display()
        ));
    }
    for (payload, path) in parse(&previous) {
        if path == wanted || lines.len() >= CAP {
            continue;
        }
        lines.push(format!("{payload}{SEPARATOR}{}", path.display()));
    }

    // The parent directory may not exist on a first run — the same situation
    // `recent.rs` and `remembered.rs` handle, and the same answer: create it,
    // and let the write's own failure be the one that is reported.
    if let Some(dir) = file.parent()
        && !dir.as_os_str().is_empty()
    {
        let _ = std::fs::create_dir_all(dir);
    }
    let mut body = lines.join("\n");
    if !body.is_empty() {
        body.push('\n');
    }
    if let Err(err) = std::fs::write(file, body) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("guides-write-failed path={} err={err}", file.display())
        });
    }
}

/// Split a file's text into `(payload, path)` pairs, dropping unparseable
/// lines.
///
/// The payload is everything before the **first** tab and the path is the
/// whole remainder, which is what lets a path contain spaces. A line with no
/// tab, or with an empty path, is dropped.
fn parse(text: &str) -> Vec<(String, PathBuf)> {
    text.lines()
        .filter_map(|line| {
            let (payload, path) = line.split_once(SEPARATOR)?;
            let path = path.trim_end_matches(['\r']);
            if path.is_empty() {
                return None;
            }
            Some((payload.to_owned(), PathBuf::from(path)))
        })
        .collect()
}

/// A document's path as this store keys it.
///
/// ★ **`std::path::absolute`, and not `std::fs::canonicalize`** — the same
/// normalisation [`crate::viewer::remembered`] uses, and it has to stay the
/// same or the two stores would disagree about whether two spellings name one
/// document.
///
/// The difference is not cosmetic, and the first draft here got it wrong.
/// `canonicalize` touches the filesystem, which means it **fails on a path
/// that does not exist** — and on Windows it returns the verbatim `\\?\D:\…`
/// form, which is what actually landed in `guides.txt` on the driven run:
/// a line no operator opening the file would recognise as their drawing, and
/// one that would not match the same document's entry in `page-display.txt`.
fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

// ---------------------------------------------------------------------------
// The drag
// ---------------------------------------------------------------------------

/// A guide drag in flight.
///
/// `Copy` and three small fields, held in [`egui::Memory`] between frames. See
/// the module header §3 for why the *release* is read from raw pointer input
/// rather than from the `Response` the press came from.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Drag {
    /// Which way the line being dragged runs.
    axis: GuideAxis,
    /// The guide being moved, as `(page, index into the collection)`, or
    /// `None` when one is being created from a ruler.
    ///
    /// The index is the identity — see [`Guides::on_page`] — and the page
    /// rides with it so a move that lands on a *different* page can remove the
    /// old entry and add the new one without searching for it.
    moving: Option<(usize, usize)>,
}

/// Plant a drag in flight, for tests in sibling modules.
///
/// `canvas::keys` owns Escape's precedence and has to assert that a guide
/// drag outranks an armed region zoom. It cannot start a real one — that
/// needs a pointer press inside a laid-out ruler gutter — so the state it
/// must react to is planted directly.
///
/// `#[cfg(test)]` so it cannot become a second way for production code to
/// begin a drag: the real one is [`ruler_drag`] and [`canvas_drag`], and a
/// second entry point is how two code paths come to disagree about what a
/// drag means.
#[cfg(test)]
pub(super) fn plant_drag_for_test(ctx: &Context) {
    store(
        ctx,
        Some(Drag {
            axis: GuideAxis::Horizontal,
            moving: None,
        }),
    );
}

/// Read the in-flight guide drag.
fn load(ctx: &Context) -> Option<Drag> {
    ctx.data(|d| d.get_temp::<Drag>(Id::new(DRAG_KEY)))
}

/// Write the in-flight guide drag, or clear it.
fn store(ctx: &Context, drag: Option<Drag>) {
    let id = Id::new(DRAG_KEY);
    ctx.data_mut(|d| match drag {
        Some(drag) => {
            d.insert_temp(id, drag);
        }
        None => d.remove::<Drag>(id),
    });
}

/// **Abandon a guide drag in flight.** Returns whether there was one.
///
/// # ★ It reports rather than being asked
///
/// The return value is the whole interface. `canvas::keys` cannot know
/// whether a guide is being dragged — the drag lives in this module's own
/// `egui::Memory` slot — and a version that re-derived it there would be the
/// version that cancels a drag *and* ascends a selection rung, which is the
/// defect that module's three-claimant table exists to prevent. Each claimant
/// says whether it took the key; none of them guesses about another.
///
/// # Why a cancelled drag leaves nothing behind
///
/// Clearing the memory slot is the entire operation, and that is a property
/// of how the drag was built rather than a convenience. A guide being dragged
/// is not a guide that has moved: the drag holds the *proposed* position, and
/// the committed set only changes when [`settle`] raises
/// `Action::SetGuides` on release. So there is no half-applied state to roll
/// back — abandoning the drag abandons a proposal.
///
/// That is why this is safe to call unconditionally on Escape, and why it
/// cannot be reached by anything else: a *committed* guide is removed by
/// double-clicking it, which is a different verb with a different undo story.
pub(super) fn cancel_drag(ctx: &Context) -> bool {
    if load(ctx).is_none() {
        return false;
    }
    store(ctx, None);
    true
}

/// Where the pointer is, and which page it is over — the two facts every
/// resolution needs.
fn pointer_page(ctx: &Context, geometry: &CanvasGeometry) -> Option<(usize, PageMapping, Pos2)> {
    let p = ctx.pointer_latest_pos()?;
    if !geometry.viewport.contains(p) {
        return None;
    }
    let (page, map) = geometry.page_at(p)?;
    Some((page, map, p))
}

/// Finish an in-flight drag, if the pointer has come up.
///
/// **The one release path**, shared by both entry points, and the whole of the
/// create / move / delete rule:
///
/// | drag started as | released over a page | released anywhere else |
/// |---|---|---|
/// | new (from a ruler) | the guide is created there | nothing happens |
/// | an existing guide | it moves there, page included | it is **deleted** |
///
/// Raises at most one [`Action::SetGuides`] carrying the whole next
/// collection. One action rather than three verbs because the operand is
/// small, because the apply then has exactly one thing to persist, and because
/// "compute the next value from the previous one and hand it over" is the same
/// shape the canvas already uses for the selection.
fn release(ctx: &Context, doc: &OpenDoc, geometry: &CanvasGeometry, actions: &mut Vec<Action>) {
    let Some(drag) = load(ctx) else {
        return;
    };
    if !ctx.input(|i| i.pointer.any_released()) {
        return;
    }
    store(ctx, None);

    let landed = pointer_page(ctx, geometry);
    let mut next = doc.guides.clone();
    match (drag.moving, landed) {
        // A new guide, dropped on a page.
        (None, Some((page, map, p))) => {
            next.add(Guide {
                page,
                axis: drag.axis,
                at: drag.axis.of(map.to_page(p)),
            });
        }
        // An existing guide, dropped on a page — possibly a different one.
        (Some((_, index)), Some((page, map, p))) => {
            next.replace(
                index,
                Guide {
                    page,
                    axis: drag.axis,
                    at: drag.axis.of(map.to_page(p)),
                },
            );
        }
        // An existing guide, dropped anywhere that is not a page.
        (Some((_, index)), None) => next.remove(index),
        // A new guide that never reached a page. Nothing to do, and
        // deliberately no action: raising one would rewrite `guides.txt` for a
        // gesture that changed nothing.
        (None, None) => return,
    }
    if next != doc.guides {
        actions.push(Action::SetGuides(next));
    }
}

/// The ruler gutters' half of the gesture: **drag out of a ruler to create a
/// guide.**
///
/// Called from [`super::show`] after the canvas has been drawn, so a press in
/// a gutter is registered in the same layer as — and later than — the page
/// widgets, exactly as [`canvas_drag`]'s bands are.
///
/// Starts a drag and does nothing else: the preview and the release belong to
/// [`settle`], because a drag started **on the canvas** must resolve whether
/// or not there are rulers. Putting the release here was the first draft, and
/// it left a guide moved with the rulers hidden stuck to the pointer with no
/// way to put it down.
///
/// Registers nothing when the rulers are hidden, which is why the guides
/// toggle is usable on its own but *creating* a guide needs rulers — the same
/// relationship every peer has, and the reason the two commands sit next to
/// each other in View ▸ Display.
pub(super) fn ruler_drag(ui: &mut Ui, doc: &OpenDoc, gutters: Gutters) {
    if !doc.view.guides {
        return;
    }
    let (Some(top), Some(left)) = (gutters.top, gutters.left) else {
        return;
    };
    for (rect, axis, salt) in [
        (top, GuideAxis::Horizontal, 0u8),
        (left, GuideAxis::Vertical, 1u8),
    ] {
        let response = ui.interact(rect, Id::new((BAND_KEY, salt)), Sense::click_and_drag());
        if response.hovered() {
            ui.ctx().set_cursor_icon(cursor(axis));
        }
        if response.drag_started() {
            store(ui.ctx(), Some(Drag { axis, moving: None }));
        }
    }
}

/// Draw the in-flight guide, and commit it when the pointer comes up.
///
/// **Called unconditionally** from [`super::show`], whatever the toggles say
/// and whether or not there are rulers — because a drag that has started has
/// to be able to end. The two things it does are both no-ops when nothing is
/// in flight, so the cost on an ordinary frame is one `Memory` lookup.
pub(super) fn settle(
    ui: &Ui,
    doc: &OpenDoc,
    geometry: Option<&CanvasGeometry>,
    actions: &mut Vec<Action>,
) {
    let Some(geometry) = geometry else {
        return;
    };
    preview(ui, geometry);
    release(ui.ctx(), doc, geometry, actions);
}

/// The cursor over a guide, or over the ruler that yields one.
///
/// A resize cursor rather than a move cursor, and the pair of them rather than
/// one: a two-headed arrow across the guide says *this slides that way*, which
/// is the whole of what a guide drag does. `Grabbing` would say "pan", which
/// is what the middle button and the hand tool already mean on this canvas.
fn cursor(axis: GuideAxis) -> egui::CursorIcon {
    match axis {
        GuideAxis::Horizontal => egui::CursorIcon::ResizeVertical,
        GuideAxis::Vertical => egui::CursorIcon::ResizeHorizontal,
    }
}

/// The canvas's half of the gesture: **grab a guide to move it, or
/// double-click it to remove it.**
///
/// Called from inside the scroll area, **after every page widget has been
/// allocated** — which is the whole of why a guide drag cannot also marquee.
/// See the module header §3.
///
/// Registers nothing at all when the toggle is off or the document has no
/// guides, so the overwhelming majority of frames pay one boolean and one
/// `is_empty`.
pub(super) fn canvas_drag(
    ui: &mut Ui,
    doc: &OpenDoc,
    pages: &[PageView],
    actions: &mut Vec<Action>,
) {
    if !doc.view.guides || doc.guides.is_empty() {
        return;
    }
    let mut removed: Option<usize> = None;
    for view in pages {
        for (index, guide) in doc.guides.on_page(view.page) {
            let response = ui.interact(
                guide.band(view.map),
                Id::new((BAND_KEY, view.page, index)),
                Sense::click_and_drag(),
            );
            if response.hovered() {
                ui.ctx().set_cursor_icon(cursor(guide.axis));
            }
            if response.double_clicked() {
                removed = Some(index);
            } else if response.drag_started() {
                store(
                    ui.ctx(),
                    Some(Drag {
                        axis: guide.axis,
                        moving: Some((view.page, index)),
                    }),
                );
            }
        }
    }
    // Applied after the loop: removing inside it would renumber the indices
    // the remaining iterations are keyed on, which is the same renumbering
    // hazard `canvas::moving`'s header tabulates for the delete verbs.
    if let Some(index) = removed {
        let mut next = doc.guides.clone();
        next.remove(index);
        // An in-flight drag on the guide that has just gone would resolve
        // against an index that now names a different guide. Cancelled rather
        // than remapped: a double-click is a complete gesture and there is
        // nothing left to drag.
        store(ui.ctx(), None);
        actions.push(Action::SetGuides(next));
    }
}

/// Draw the guide being dragged, if one is.
///
/// Across the page it would land on, at full strength — or across the whole
/// viewport at [`DISCARD_ALPHA`] when it would land nowhere, which is how the
/// line says *release here and this does not happen*. See [`DISCARD_ALPHA`] on
/// why the difference is emphasis rather than a second colour.
fn preview(ui: &Ui, geometry: &CanvasGeometry) {
    let Some(drag) = load(ui.ctx()) else {
        return;
    };
    let Some(p) = ui.ctx().pointer_latest_pos() else {
        return;
    };
    let base = ui.visuals().selection.stroke.color;
    let painter = ui.painter().with_clip_rect(geometry.viewport);
    match pointer_page(ui.ctx(), geometry) {
        Some((page, map, p)) => {
            let guide = Guide {
                page,
                axis: drag.axis,
                at: drag.axis.of(map.to_page(p)),
            };
            painter.line_segment(
                guide.segment(map),
                Stroke::new(1.0, super::overlay::at_alpha(base, GUIDE_ALPHA)),
            );
        }
        None => {
            let stroke = Stroke::new(1.0, super::overlay::at_alpha(base, DISCARD_ALPHA));
            let vp = geometry.viewport;
            match drag.axis {
                GuideAxis::Horizontal => painter.hline(vp.x_range(), p.y, stroke),
                GuideAxis::Vertical => painter.vline(p.x, vp.y_range(), stroke),
            };
        }
    }
}

/// Draw every guide on every page the frame is showing.
///
/// Called from `interact`'s draw step, **above** the find wash and the
/// selection outlines: a guide is a line the operator placed and has to be
/// able to see while they align something to it, and a selection outline is a
/// box a few points across that a guide crossing it does not hide.
pub(super) fn draw(ui: &Ui, doc: &OpenDoc, pages: &[PageView], clip: Rect) {
    if !doc.view.guides || doc.guides.is_empty() {
        return;
    }
    let stroke = Stroke::new(
        1.0,
        super::overlay::at_alpha(ui.visuals().selection.stroke.color, GUIDE_ALPHA),
    );
    let painter = ui.painter().with_clip_rect(clip);
    for view in pages {
        for (_, guide) in doc.guides.on_page(view.page) {
            painter.line_segment(guide.segment(view.map), stroke);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Guides {
        let mut g = Guides::default();
        assert!(g.add(Guide {
            page: 0,
            axis: GuideAxis::Horizontal,
            at: 120.5,
        }));
        assert!(g.add(Guide {
            page: 0,
            axis: GuideAxis::Vertical,
            at: 64.0,
        }));
        assert!(g.add(Guide {
            page: 3,
            axis: GuideAxis::Horizontal,
            at: -12.25,
        }));
        g
    }

    /// ★ **Every guide survives a round trip through the on-disk spelling**,
    /// including a negative coordinate.
    ///
    /// Negative is not an edge case invented for the test: canvas space has
    /// its origin at the page's top-left, and a guide can legitimately sit
    /// above or left of the sheet — the operator dragged it into the bleed.
    #[test]
    fn every_guide_round_trips_through_its_on_disk_spelling() {
        let guides = sample();
        assert_eq!(Guides::decode(&guides.encode()), guides);
    }

    /// Every axis has a spelling and every spelling names an axis.
    ///
    /// Both directions, so a variant added with a colliding or missing id
    /// fails here rather than becoming a guide that cannot be saved.
    #[test]
    fn every_axis_round_trips_through_its_on_disk_spelling() {
        for axis in [GuideAxis::Horizontal, GuideAxis::Vertical] {
            assert_eq!(GuideAxis::from_id(axis.id()), Some(axis));
        }
        assert_eq!(GuideAxis::from_id("x"), None);
        assert_eq!(GuideAxis::from_id(""), None);
        assert_ne!(GuideAxis::Horizontal.id(), GuideAxis::Vertical.id());
    }

    /// ★ **A corrupt payload degrades into fewer guides, never into an
    /// error.**
    ///
    /// The posture the module header commits to, asserted token by token:
    /// each of these is a different way a line can be wrong, and every one of
    /// them must cost exactly the guide it describes.
    #[test]
    fn a_corrupt_payload_drops_only_the_guides_it_breaks() {
        let good = "0:h:10 1:v:20";
        let mixed = "0:h:10 nonsense 2:q:5 3:h: :: 4:v:zz 1:v:20 5:h:1:2";
        assert_eq!(Guides::decode(mixed), Guides::decode(good));
        assert!(Guides::decode("").is_empty());
        assert!(Guides::decode("   ").is_empty());
    }

    /// A non-finite coordinate is refused rather than stored.
    ///
    /// It cannot come from a drag — the pointer is finite — but it can come
    /// from a hand-edited file, and a NaN guide is a line that paints nothing
    /// and a band that catches nothing: present in the count, absent from the
    /// canvas.
    #[test]
    fn a_non_finite_guide_is_refused() {
        let mut g = Guides::default();
        for at in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(!g.add(Guide {
                page: 0,
                axis: GuideAxis::Horizontal,
                at,
            }));
        }
        assert!(g.is_empty());
        assert!(Guides::decode("0:h:NaN 0:v:inf").is_empty());
    }

    /// The per-document ceiling is enforced, and enforcing it does not corrupt
    /// what is already there.
    #[test]
    fn a_document_stops_accepting_guides_at_the_ceiling() {
        let mut g = Guides::default();
        for i in 0..MAX_PER_DOCUMENT {
            assert!(g.add(Guide {
                page: 0,
                axis: GuideAxis::Vertical,
                at: i as f32,
            }));
        }
        assert!(!g.add(Guide {
            page: 0,
            axis: GuideAxis::Vertical,
            at: -1.0,
        }));
        assert_eq!(g.len(), MAX_PER_DOCUMENT);
    }

    /// `on_page` selects one page's guides and reports the index the whole
    /// collection knows them by — which is what a drag names.
    #[test]
    fn on_page_reports_the_index_a_drag_names() {
        let guides = sample();
        let on_zero: Vec<_> = guides.on_page(0).collect();
        assert_eq!(on_zero.len(), 2);
        assert_eq!(on_zero[0].0, 0);
        assert_eq!(on_zero[1].0, 1);
        let on_three: Vec<_> = guides.on_page(3).collect();
        assert_eq!(on_three.len(), 1);
        assert_eq!(on_three[0].0, 2, "the index is into the whole collection");
        assert_eq!(guides.on_page(9).count(), 0);
    }

    /// ★ **A guide is stored against a page in canvas space, so it does not
    /// move when the view does.**
    ///
    /// The property `GUI_ROADMAP.md` Phase 1 names for the selection, applied
    /// to a guide: the stored value is identical at every zoom, and the
    /// *screen* line it produces tracks the page. A guide stored in screen
    /// coordinates would pass no part of this.
    #[test]
    fn a_guide_holds_still_on_the_page_at_every_zoom() {
        let guide = Guide {
            page: 0,
            axis: GuideAxis::Horizontal,
            at: 100.0,
        };
        let extent = (612.0_f32, 792.0_f32);
        for zoom in [0.25_f32, 1.0, 4.0] {
            let rect = Rect::from_min_size(
                pos2(37.0, 11.0),
                egui::vec2(extent.0 * zoom, extent.1 * zoom),
            );
            let map = PageMapping::new(rect, extent, zoom);
            let [a, b] = guide.segment(map);
            // The line spans the page and sits 100 canvas units down it.
            assert!((a.x - rect.min.x).abs() < 0.01 && (b.x - rect.max.x).abs() < 0.01);
            let expected = rect.min.y + 100.0 * zoom;
            assert!(
                (a.y - expected).abs() < 0.01,
                "at {zoom}× the guide drew at {} rather than {expected}",
                a.y
            );
            // …and reading the screen position back gives the stored value.
            assert!((map.to_page(a).y - guide.at).abs() < 0.01);
        }
    }

    /// ★ **The catch band is the same number of screen points wide at every
    /// zoom.**
    ///
    /// The law `canvas::mapping` exists to enforce, applied to a guide: a
    /// page-space catch radius would be un-grabbable at exactly the zoom an
    /// operator uses to see a whole sheet.
    #[test]
    fn the_catch_band_is_the_same_width_at_every_zoom() {
        let guide = Guide {
            page: 0,
            axis: GuideAxis::Vertical,
            at: 300.0,
        };
        let extent = (612.0_f32, 792.0_f32);
        for zoom in [0.1_f32, 0.5, 1.0, 3.0, 8.0] {
            let rect =
                Rect::from_min_size(Pos2::ZERO, egui::vec2(extent.0 * zoom, extent.1 * zoom));
            let band = guide.band(PageMapping::new(rect, extent, zoom));
            assert!(
                (band.width() - CATCH_PTS * 2.0).abs() < 0.01,
                "at {zoom}× the band is {} pt wide",
                band.width()
            );
            assert!(
                (band.height() - rect.height()).abs() < 0.01,
                "the band must not run past its own page"
            );
        }
    }

    /// ★ **A document's guides come back after a reopen, and a second
    /// document's do not leak into the first.**
    ///
    /// The whole of `PLANNED`'s *"need a per-document store to survive a
    /// reopen"*, driven through the real reader and writer against a real
    /// file.
    #[test]
    fn guides_survive_a_reopen_and_stay_with_their_own_document() {
        let dir = std::env::temp_dir().join(format!("pdfcer-guides-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("guides-roundtrip.txt");
        let _ = std::fs::remove_file(&file);

        let drawing = PathBuf::from("D:\\Drawings\\job 4471\\sheet set.pdf");
        let report = PathBuf::from("C:\\reports\\quarterly.pdf");

        remember_at(Some(&file), &drawing, &sample());
        assert_eq!(recall_at(Some(&file), &drawing), sample());
        // A document nobody has ruled up has no guides, and asking does not
        // hand it the other document's.
        assert!(recall_at(Some(&file), &report).is_empty());

        // A second document writes its own line without disturbing the first
        // — the read-modify-write property, and the one a naive "write my
        // line" implementation loses.
        let mut theirs = Guides::default();
        theirs.add(Guide {
            page: 0,
            axis: GuideAxis::Vertical,
            at: 306.0,
        });
        remember_at(Some(&file), &report, &theirs);
        assert_eq!(recall_at(Some(&file), &drawing), sample());
        assert_eq!(recall_at(Some(&file), &report), theirs);

        // Clearing the last guide forgets the document rather than leaving an
        // empty marker behind.
        remember_at(Some(&file), &report, &Guides::default());
        assert!(recall_at(Some(&file), &report).is_empty());
        let text = std::fs::read_to_string(&file).expect("the file is still there");
        assert!(
            !text.contains("quarterly.pdf"),
            "an emptied document must not keep a line: {text:?}"
        );
        assert!(
            text.contains("sheet set.pdf"),
            "the other document survived"
        );

        let _ = std::fs::remove_file(&file);
    }

    /// ★ **A path containing spaces round-trips**, which is why the payload is
    /// written first and the path is the whole remainder of the line.
    ///
    /// Not hypothetical: `D:\Dev\temp\pdfcer` sits under a user profile on this
    /// machine, and every Windows operator has `C:\Users\<name>\My Documents`.
    #[test]
    fn a_path_with_spaces_survives_the_format() {
        let pairs = parse("0:h:10 1:v:20\tC:\\Program Files\\a b\\c d.pdf\n");
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "0:h:10 1:v:20");
        assert_eq!(pairs[0].1, PathBuf::from("C:\\Program Files\\a b\\c d.pdf"));
    }

    /// A line with no tab, or with an empty path, is dropped rather than
    /// producing a guide set attached to nothing.
    #[test]
    fn a_malformed_line_is_dropped() {
        assert!(parse("no tab here\n").is_empty());
        assert!(parse("0:h:10\t\n").is_empty());
        assert!(parse("").is_empty());
        // …and a well-formed line among broken ones still reads.
        let pairs = parse("junk\n0:h:10\tC:\\a.pdf\n\t\n");
        assert_eq!(pairs.len(), 1);
    }

    /// Reading a store that does not exist answers "no guides" rather than
    /// failing — the same posture `remembered::recall` takes, and the reason
    /// a first run needs no special case.
    #[test]
    fn a_missing_store_answers_no_guides() {
        let missing = std::env::temp_dir().join("pdfcer-guides-does-not-exist-4471.txt");
        let _ = std::fs::remove_file(&missing);
        assert!(recall_at(Some(&missing), Path::new("C:\\a.pdf")).is_empty());
        assert!(recall_at(None, Path::new("C:\\a.pdf")).is_empty());
    }
}
