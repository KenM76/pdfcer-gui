//! # `dialogs::page_crop` — **a page box of the chosen sheets**
//!
//! `pages.crop`, Pages ▸ Transform ▸ Crop…, and `pages.crop_to_region`, which
//! opens it seeded from the marquee region. The operator picks the box — crop,
//! bleed, trim or art ([`boxes`]) — and four margins, measured in from the
//! edges **as the operator sees the sheet** (after `/Rotate`), are applied to
//! each page in scope against that page's own sheet and rotation, as one
//! `EditSession::set_page_boxes` undo step. A crop with all four at zero
//! becomes `CropBoxEdit::Reset`, so the whole sheet shows again without a crop
//! box being written; the other three boxes write the sheet at zero, and their
//! Remove button resets them, so they default to the crop box. Enter applies;
//! Escape cancels.
//!
//! Design: `docs/modules/pdfcer-gui/dialogs/page_crop.md`.

use egui::Ui;
use pdfcer_core::edit::{CropBoxEdit, PageBox, PageBoxEdit};
use pdfcer_core::page_tree::Rect;
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::app::state::OpenDoc;
use crate::text::page_crop as t;

mod boxes;

/// The dialog body's published region, for `ui-verify`.
const REGION_BODY: &str = "page-crop.body";
/// One margin box; the suffix is `left`, `right`, `top` or `bottom`.
const REGION_MARGIN_PREFIX: &str = "page-crop.margin.";
/// The Show whole sheet button.
const REGION_WHOLE: &str = "page-crop.whole";
/// The commit button.
const REGION_APPLY: &str = "page-crop.apply";
/// One scope radio; the suffix is [`Scope::key`].
const REGION_SCOPE_PREFIX: &str = "page-crop.scope.";
/// The range field.
const REGION_RANGE: &str = "page-crop.range";
/// The margin kept by Remove white margins.
const REGION_WHITE_KEEP: &str = "page-crop.white.keep";
/// The Remove white margins button.
const REGION_WHITE: &str = "page-crop.white";

/// The on-screen edges, in the order the window lists them.
const EDGES: [Edge; 4] = [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom];

/// An edge of the sheet as the operator sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl Edge {
    /// The label beside its box.
    const fn label(self) -> &'static str {
        match self {
            Self::Left => t::left(),
            Self::Right => t::right(),
            Self::Top => t::top(),
            Self::Bottom => t::bottom(),
        }
    }

    /// The machine name, for regions and traces.
    const fn key(self) -> &'static str {
        // ui-text-exempt: region suffixes, never displayed.
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }

    /// Which **page-space** edge this screen edge is on a sheet turned
    /// `rotate` degrees clockwise: an index into `[left, right, bottom, top]`
    /// of the unrotated page.
    const fn pdf_index(self, rotate: u16) -> usize {
        // Turning a page 90° clockwise carries its left edge to the top, its
        // top to the right, its right to the bottom and its bottom to the left.
        let (l, r, b, t) = (0, 1, 2, 3);
        match (rotate, self) {
            (90, Self::Left) | (180, Self::Top) | (0, Self::Bottom) => b,
            (90, Self::Right) | (180, Self::Bottom) | (0, Self::Top) => t,
            (90, Self::Top) | (180, Self::Right) | (0, Self::Left) => l,
            (90, Self::Bottom) | (180, Self::Left) | (0, Self::Right) => r,
            (_, Self::Left) => t,
            (_, Self::Right) => b,
            (_, Self::Top) => r,
            (_, Self::Bottom) => l,
        }
    }
}

/// Which pages a crop covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    /// The pages the command was raised on: the rail's pick, the current
    /// page, or the region's page.
    Operands,
    All,
    /// Page numbers 1, 3, 5….
    Odd,
    /// Page numbers 2, 4, 6….
    Even,
    /// The pages typed in [`PageCropDialog::range`].
    Range,
}

const SCOPES: [Scope; 5] = [
    Scope::Operands,
    Scope::All,
    Scope::Odd,
    Scope::Even,
    Scope::Range,
];

impl Scope {
    /// The machine name, for regions and traces.
    const fn key(self) -> &'static str {
        // ui-text-exempt: region suffixes, never displayed.
        match self {
            Self::Operands => "operands",
            Self::All => "all",
            Self::Odd => "odd",
            Self::Even => "even",
            Self::Range => "range",
        }
    }
}

/// The crop window's state.
pub struct PageCropDialog {
    /// The pages the command was raised on, 0-based, ascending.
    operands: Vec<usize>,
    /// Every page's sheet (`/MediaBox`) and rotation, by page index.
    sheets: Vec<(Rect, u16)>,
    /// The box being set.
    which: PageBox,
    /// The first operand's effective crop, bleed, trim and art boxes, in
    /// [`boxes::BOXES`] order: what choosing a box seeds the margins from.
    seeds: [Rect; 4],
    /// The margins of the marquee region this window was opened on, for
    /// Back to the region; `None` when opened from the menu.
    region: Option<[i64; 4]>,
    /// Same margin on every edge: editing one sets all four.
    equal: bool,
    /// Set by the Remove button of a bleed, trim or art box.
    remove_requested: bool,
    scope: Scope,
    /// The typed range, in the print dialog's syntax (`1-3, 7`).
    range: String,
    /// Margins in whole millimetres, by on-screen edge, in [`EDGES`] order.
    margins_mm: [i64; 4],
    /// Paper Remove white margins keeps around the ink, whole millimetres.
    white_mm: i64,
    /// Set by the Remove white margins button.
    white_requested: bool,
    /// Set by the commit button or Enter, consumed after the window closure
    /// returns.
    apply_requested: bool,
    /// Set by Cancel.
    close_requested: bool,
}

impl PageCropDialog {
    /// Open the window over `pages`, seeded with the first one's crop.
    #[must_use]
    pub fn open(doc: &OpenDoc, pages: &[usize]) -> Option<Self> {
        let first = doc.pages.get(*pages.first()?)?;
        Self::seeded(doc, pages, first.crop_box)
    }

    /// The margins of `rect` on sheet `media` turned `rotate`, in whole
    /// millimetres by on-screen edge, in [`EDGES`] order; never negative.
    fn margins_of(media: Rect, rotate: u16, rect: Rect) -> [i64; 4] {
        let mm = crate::units::whole_mm_from_points;
        // Page space `[left, right, bottom, top]`, then by on-screen edge.
        let page = [
            mm(rect.llx - media.llx),
            mm(media.urx - rect.urx),
            mm(rect.lly - media.lly),
            mm(media.ury - rect.ury),
        ];
        EDGES.map(|e| page[e.pdf_index(rotate)].max(0))
    }

    /// Open the window over `page`, seeded with `rect` — a region in that
    /// page's user space — as the crop. `None` when the page is not there.
    #[must_use]
    pub fn open_at(doc: &OpenDoc, page: usize, rect: Rect) -> Option<Self> {
        let sheet = doc.pages.get(page)?.media_box;
        let inside = Rect::from_corners(
            rect.llx.max(sheet.llx),
            rect.lly.max(sheet.lly),
            rect.urx.min(sheet.urx),
            rect.ury.min(sheet.ury),
        );
        let mut dialog = Self::seeded(doc, &[page], inside)?;
        dialog.region = Some(dialog.margins_mm);
        Some(dialog)
    }

    /// The window over `pages`, its margins measured from `crop` on the first
    /// one's sheet.
    fn seeded(doc: &OpenDoc, pages: &[usize], crop: Rect) -> Option<Self> {
        let first = doc.pages.get(*pages.first()?)?;
        let margins_mm = Self::margins_of(first.media_box, first.rotate, crop);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-crop-opened sheets={} rotate={} margins_mm={margins_mm:?}",
                pages.len(),
                first.rotate,
            )
        });
        Some(Self {
            operands: pages.to_vec(),
            sheets: doc.pages.iter().map(|p| (p.media_box, p.rotate)).collect(),
            which: PageBox::Crop,
            seeds: [
                first.crop_box,
                first.bleed_box,
                first.trim_box,
                first.art_box,
            ],
            region: None,
            equal: false,
            remove_requested: false,
            scope: Scope::Operands,
            range: String::new(),
            margins_mm,
            white_mm: 0,
            white_requested: false,
            apply_requested: false,
            close_requested: false,
        })
    }

    /// The pages in scope, 0-based and ascending; empty when the range names
    /// none.
    fn pages(&self) -> Vec<usize> {
        let count = self.sheets.len();
        match self.scope {
            Scope::Operands => self.operands.clone(),
            Scope::All => (0..count).collect(),
            Scope::Odd => (0..count).step_by(2).collect(),
            Scope::Even => (1..count).step_by(2).collect(),
            Scope::Range => {
                // `set_page_boxes` refuses a page named twice; `5, 1-5` is one
                // set of pages to the operator.
                let mut pages =
                    pdfcer_gui_base::pageselection::parse_page_range(&self.range, count)
                        .unwrap_or_default();
                pages.sort_unstable();
                pages.dedup();
                pages
            }
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "page-crop", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(440.0, 680.0),
            egui::vec2(340.0, 320.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.apply_requested = true;
            }
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.white_requested) {
            let pages = self.pages();
            if !pages.is_empty() {
                actions.push(Action::Page(PageAction::RemoveWhiteMargins {
                    pages,
                    margin_mm: self.white_mm,
                }));
                return false;
            }
        }

        if std::mem::take(&mut self.remove_requested) {
            let edits: Vec<PageBoxEdit> = self
                .pages()
                .into_iter()
                .map(|i| PageBoxEdit::new(i, self.which, CropBoxEdit::Reset))
                .collect();
            if !edits.is_empty() {
                self.trace_commit(&edits);
                actions.push(Action::Page(PageAction::SetPageBoxes { edits }));
                return false;
            }
        }

        if std::mem::take(&mut self.apply_requested)
            && let Some(edits) = self.edits()
        {
            self.trace_commit(&edits);
            actions.push(Action::Page(PageAction::SetPageBoxes { edits }));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The `page-crop-commit` trace line.
    fn trace_commit(&self, edits: &[PageBoxEdit]) {
        crate::diag::trace(|| {
            let set = edits
                .iter()
                .filter(|e| matches!(e.edit, CropBoxEdit::Set(_)))
                .count();
            let first = edits.first().and_then(|e| match e.edit {
                CropBoxEdit::Set(r) => Some(r),
                _ => None,
            });
            let r = first.unwrap_or(Rect::from_corners(0.0, 0.0, 0.0, 0.0));
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-crop-commit n={} scope={} box={} set={set} reset={} margins_mm={:?} \
                 llx={:.2} lly={:.2} urx={:.2} ury={:.2}",
                edits.len(),
                self.scope.key(),
                boxes::key(self.which),
                edits.len() - set,
                self.margins_mm,
                r.llx,
                r.lly,
                r.urx,
                r.ury,
            )
        });
    }

    /// The crop box the margins describe on a sheet `media` turned `rotate`,
    /// in page space, or `None` when they leave nothing.
    fn rect_on(&self, media: Rect, rotate: u16) -> Option<Rect> {
        let mut page = [0.0; 4];
        for (i, edge) in EDGES.iter().enumerate() {
            #[allow(
                clippy::cast_precision_loss,
                reason = "a millimetre count is exact in f64" // ui-text-exempt: lint justification
            )]
            let pt = crate::units::points_from_mm(self.margins_mm[i] as f64);
            page[edge.pdf_index(rotate)] = pt;
        }
        let r = Rect::from_corners(
            media.llx + page[0],
            media.lly + page[2],
            media.urx - page[1],
            media.ury - page[3],
        );
        (media.llx + page[0] < media.urx - page[1] - 1.0
            && media.lly + page[2] < media.ury - page[3] - 1.0)
            .then_some(r)
    }

    /// One edit per page in scope. `None` when the scope is empty or the
    /// margins leave some page nothing.
    fn edits(&self) -> Option<Vec<PageBoxEdit>> {
        let pages = self.pages();
        if pages.is_empty() {
            return None;
        }
        pages
            .into_iter()
            .map(|index| {
                let edit = if self.which == PageBox::Crop && self.margins_mm == [0; 4] {
                    CropBoxEdit::Reset
                } else {
                    let (media, rotate) = *self.sheets.get(index)?;
                    CropBoxEdit::Set(self.rect_on(media, rotate)?)
                };
                Some(PageBoxEdit::new(index, self.which, edit))
            })
            .collect()
    }

    /// The page-scope radios, the range field when Range is chosen, and the
    /// count of pages in scope.
    fn scope_group(&mut self, ui: &mut Ui) {
        ui.label(t::scope_heading());
        for scope in SCOPES {
            let label = match scope {
                Scope::Operands => t::scope_operands(self.operands.len()),
                Scope::All => t::scope_all().to_owned(),
                Scope::Odd => t::scope_odd().to_owned(),
                Scope::Even => t::scope_even().to_owned(),
                Scope::Range => t::scope_range().to_owned(),
            };
            let radio = ui.radio_value(&mut self.scope, scope, label);
            crate::diag::ui_rect(&format!("{REGION_SCOPE_PREFIX}{}", scope.key()), radio.rect);
        }
        if self.scope == Scope::Range {
            // escape-disposition: dialog-cancels
            let field =
                ui.add(egui::TextEdit::singleline(&mut self.range).hint_text(t::range_hint()));
            crate::diag::ui_rect(REGION_RANGE, field.rect);
        }
        ui.label(
            egui::RichText::new(t::scope_count(
                self.pages().len(),
                self.which == PageBox::Crop,
            ))
            .small()
            .weak(),
        );
    }

    /// The four margin boxes beside the picture of the page, and the buttons
    /// under them.
    fn margins_group(&mut self, ui: &mut Ui) {
        ui.label(t::margins_heading(self.which == PageBox::Crop));
        let before = self.margins_mm;
        ui.horizontal(|ui| {
            self.margin_grid(ui);
            self.picture(ui);
        });
        if self.equal
            && let Some(v) = (0..4)
                .find(|&i| before[i] != self.margins_mm[i])
                .map(|i| self.margins_mm[i])
        {
            self.margins_mm = [v; 4];
        }
        self.margin_buttons(ui);
    }

    /// The four margin boxes.
    fn margin_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("page-crop.grid") // ui-text-exempt: widget id
            .num_columns(2)
            .show(ui, |ui| {
                for (i, edge) in EDGES.iter().enumerate() {
                    ui.label(edge.label());
                    let (widget, refusal) = entry::drag_value(
                        ui,
                        &mut self.margins_mm[i],
                        entry::Kind::Length(entry::LengthUnit::Of(
                            pdfcer_core::dimension::Unit::Millimeter,
                        )),
                    );
                    let response = refusal.show(ui.add(widget.range(0..=5080)));
                    crate::diag::ui_rect(
                        &format!("{REGION_MARGIN_PREFIX}{}", edge.key()),
                        response.rect,
                    );
                    ui.end_row();
                }
            });
    }

    /// Remove white margins: the paper kept around the ink, and the button,
    /// present while the scope names a page.
    fn white_group(&mut self, ui: &mut Ui) {
        if self.pages().is_empty() {
            return;
        }
        ui.label(t::white_heading());
        ui.horizontal(|ui| {
            ui.label(t::white_keep());
            let (widget, refusal) = entry::drag_value(
                ui,
                &mut self.white_mm,
                entry::Kind::Length(entry::LengthUnit::Of(
                    pdfcer_core::dimension::Unit::Millimeter,
                )),
            );
            let keep = refusal.show(ui.add(widget.range(0..=500)));
            crate::diag::ui_rect(REGION_WHITE_KEEP, keep.rect);
            let button = ui.button(t::white_button());
            crate::diag::ui_rect(REGION_WHITE, button.rect);
            if button.on_hover_text(t::white_tooltip()).clicked() {
                self.white_requested = true;
            }
        });
    }

    /// The visible-area line for the first page in scope, or the refusal.
    fn summary(&self, ui: &mut Ui, valid: bool) {
        let pages = self.pages();
        let first = pages.first().and_then(|&i| self.sheets.get(i));
        if let (true, Some(&(media, rotate))) = (valid, first) {
            let shown = self.rect_on(media, rotate).unwrap_or(media);
            let (w, h) = if rotate % 180 == 90 {
                (shown.height(), shown.width())
            } else {
                (shown.width(), shown.height())
            };
            let mm = crate::units::whole_mm_from_points;
            ui.label(
                egui::RichText::new(t::box_summary(boxes::index(self.which), mm(w), mm(h)))
                    .small()
                    .weak(),
            );
            return;
        }
        let why = if pages.is_empty() {
            t::no_pages()
        } else {
            t::nothing_left(self.which == PageBox::Crop)
        };
        ui.label(
            egui::RichText::new(why)
                .small()
                .color(ui.visuals().error_fg_color),
        );
    }

    /// The controls and the two buttons.
    fn body(&mut self, ui: &mut Ui) {
        self.box_group(ui);
        ui.label(t::intro(boxes::index(self.which)));
        ui.add_space(8.0);
        self.scope_group(ui);
        ui.add_space(6.0);
        self.margins_group(ui);
        ui.add_space(6.0);
        let valid = self.edits().is_some();
        self.summary(ui, valid);
        ui.add_space(6.0);
        if self.which == PageBox::Crop {
            self.white_group(ui);
        }

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
            // Absent, not greyed, when nothing valid is asked: the refusal
            // line above already says why (R9).
            if valid {
                let apply = ui.button(t::apply(boxes::index(self.which)));
                crate::diag::ui_rect(REGION_APPLY, apply.rect);
                if apply
                    .on_hover_text(t::apply_tooltip(self.which == PageBox::Crop))
                    .clicked()
                {
                    self.apply_requested = true;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a4() -> Rect {
        Rect::from_corners(0.0, 0.0, 595.0, 842.0)
    }

    fn dialog(sheets: Vec<(Rect, u16)>, margins_mm: [i64; 4]) -> PageCropDialog {
        PageCropDialog {
            operands: vec![0],
            sheets,
            which: PageBox::Crop,
            seeds: [a4(); 4],
            region: None,
            equal: false,
            remove_requested: false,
            scope: Scope::Operands,
            range: String::new(),
            margins_mm,
            white_mm: 0,
            white_requested: false,
            apply_requested: false,
            close_requested: false,
        }
    }

    fn rect_of(edit: &PageBoxEdit) -> Rect {
        match edit.edit {
            CropBoxEdit::Set(r) => r,
            other => panic!("a crop, not {other:?}"),
        }
    }

    /// Every screen edge lands on a different page edge, at every rotation.
    #[test]
    fn each_rotation_maps_the_four_edges_onto_four_page_edges() {
        for rotate in [0, 90, 180, 270] {
            let mut seen: Vec<usize> = EDGES.iter().map(|e| e.pdf_index(rotate)).collect();
            seen.sort_unstable();
            assert_eq!(seen, vec![0, 1, 2, 3], "rotate={rotate}");
        }
        // A sheet turned a quarter clockwise shows its page-space left edge on top.
        assert_eq!(Edge::Top.pdf_index(90), 0);
        assert_eq!(Edge::Top.pdf_index(270), 1);
    }

    /// Zero margins ask for the whole sheet, not for a crop box equal to it.
    #[test]
    fn zero_margins_reset_rather_than_write_the_sheet() {
        let edits = dialog(vec![(a4(), 0)], [0; 4]).edits().unwrap();
        assert_eq!(edits[0].edit, CropBoxEdit::Reset);
    }

    /// A trim box at zero margins is the whole sheet, written; only the crop
    /// box resets at zero, and every edit names the chosen box.
    #[test]
    fn zero_margins_write_the_sheet_for_a_production_box() {
        let mut d = dialog(vec![(a4(), 0)], [0; 4]);
        d.which = PageBox::Trim;
        let edits = d.edits().unwrap();
        assert_eq!(edits[0].target, PageBox::Trim);
        assert_eq!(rect_of(&edits[0]).urx, 595.0);
    }

    /// Choosing a box seeds the margins from that box: 10 mm in from the left.
    #[test]
    fn margins_are_measured_from_the_chosen_box() {
        let trim = Rect::from_corners(28.35, 0.0, 595.0, 842.0);
        assert_eq!(PageCropDialog::margins_of(a4(), 0, trim), [10, 0, 0, 0]);
        // The page-space left edge is on top of a sheet turned a quarter.
        assert_eq!(PageCropDialog::margins_of(a4(), 90, trim), [0, 0, 10, 0]);
    }

    /// Margins become a rectangle measured in from the sheet's edges.
    #[test]
    fn margins_become_the_crop_rectangle() {
        // [left, right, top, bottom]: left 10 mm, top 20 mm.
        let edits = dialog(vec![(a4(), 0)], [10, 0, 20, 0]).edits().unwrap();
        let r = rect_of(&edits[0]);
        assert!((r.llx - 28.35).abs() < 0.01, "{r:?}");
        assert!((r.ury - (842.0 - 56.69)).abs() < 0.01, "{r:?}");
        assert!((r.urx - 595.0).abs() < 0.01, "{r:?}");
    }

    /// The same on-screen margin lands on each page's own page-space edge:
    /// the top of a sheet turned a quarter clockwise is its page-space left.
    #[test]
    fn each_page_is_cropped_against_its_own_rotation() {
        let mut d = dialog(vec![(a4(), 0), (a4(), 90)], [0, 0, 20, 0]);
        d.scope = Scope::All;
        let edits = d.edits().unwrap();
        assert!((rect_of(&edits[0]).ury - (842.0 - 56.69)).abs() < 0.01);
        assert!((rect_of(&edits[1]).llx - 56.69).abs() < 0.01);
    }

    /// Pages of different sizes each keep the margin from their own edges.
    #[test]
    fn each_page_is_cropped_against_its_own_sheet() {
        let a3 = Rect::from_corners(0.0, 0.0, 842.0, 1191.0);
        let mut d = dialog(vec![(a4(), 0), (a3, 0)], [0, 10, 0, 0]);
        d.scope = Scope::All;
        let edits = d.edits().unwrap();
        assert!((rect_of(&edits[0]).urx - (595.0 - 28.35)).abs() < 0.01);
        assert!((rect_of(&edits[1]).urx - (842.0 - 28.35)).abs() < 0.01);
    }

    /// Odd and even count page numbers from one; a range uses the print
    /// dialog's syntax.
    #[test]
    fn odd_even_and_range_pick_the_pages_named() {
        let mut d = dialog(vec![(a4(), 0); 5], [5; 4]);
        d.scope = Scope::Odd;
        assert_eq!(d.pages(), vec![0, 2, 4]);
        d.scope = Scope::Even;
        assert_eq!(d.pages(), vec![1, 3]);
        d.scope = Scope::Range;
        d.range = "2-3".to_owned();
        assert_eq!(d.pages(), vec![1, 2]);
        d.range = "nine".to_owned();
        assert!(d.edits().is_none(), "a range naming nothing offers no crop");
    }

    /// Margins that meet leave nothing, and nothing is offered.
    #[test]
    fn margins_that_cross_offer_no_crop() {
        assert!(dialog(vec![(a4(), 0)], [150, 150, 0, 0]).edits().is_none());
    }
}
