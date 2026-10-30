//! # `dialogs::print::impose` — Multiple and Booklet over a planned job
//!
//! Like [`super::poster`], imposition transforms the planned [`Job`]: the job
//! is planned as an ordinary one, then its plans are replaced by one
//! [`PagePlan`] per printed side, parallel to [`State::out`], which says which
//! pages land where on that side. The stepper, the sheet count and the spool
//! walk sheets and need no second path; the commit and the preview look the
//! sheet up in [`State::out`] by plan position.
//!
//! # Contract
//!
//! - A sheet's plan has placement `1, 0, 0` and is never clipped: the sheet
//!   bitmap is the printable area, composed by
//!   [`pdfcer_gui_base::imposition::compose`]. Its `index` is the first page on
//!   it, which is what Auto orientation resolves from when the grid keeps the
//!   page's own orientation.
//! - The operator's page positions are not applied: a page's position is the
//!   layout's.
//! - While the orientation is Auto, the layout picks it ([`State::forced`]): a
//!   2- or 6-up sheet turns against the page, a 4-, 9- or 16-up sheet keeps it,
//!   a side-by-side booklet is landscape and a tall one portrait.
//! - A two-sided booklet prints flipped on the short edge; a one-sided pass
//!   prints one-sided. Any other duplex setting would print it out of order.
//! - A refusal from the engine leaves no job and is the Pages tab's caption.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/imposition.md`.

use egui::Ui;
use pdfcer_gui_base::entry;
use pdfcer_gui_base::imposition::{self as base, Cell, ImposedSheet, Summary};
use pdfcer_print::imposition::{Binding, BookletSpec, BookletSubset, NUpGrid, NUpSpec, PageOrder};

use super::PrintDialog;
use super::spooler::{
    Collate, DeviceSettings, Duplex, Job, JobSpec, Orientation, PageBitmap, PagePlan, Placement,
};
use crate::app::prefs::print_impose::{ImposeMode, ImposePrefs, NUP_COUNTS};
use crate::text::print::impose as ti;

/// The published region of the Multiple choice.
pub(super) const REGION_MULTIPLE: &str = "print.impose.multiple";
/// The published region of the Booklet choice.
pub(super) const REGION_BOOKLET: &str = "print.impose.booklet";
/// The published region of the pages-per-sheet choice.
pub(super) const REGION_COUNT: &str = "print.impose.count";

const ORDERS: [PageOrder; 4] = [
    PageOrder::Horizontal,
    PageOrder::HorizontalReversed,
    PageOrder::Vertical,
    PageOrder::VerticalReversed,
];
const BINDINGS: [Binding; 4] = [
    Binding::Left,
    Binding::Right,
    Binding::LeftTall,
    Binding::RightTall,
];
const SUBSETS: [BookletSubset; 3] = [
    BookletSubset::BothSides,
    BookletSubset::FrontOnly,
    BookletSubset::BackOnly,
];

/// The dialog's imposition state: the operator's answers and this frame's
/// layout.
pub(crate) struct State {
    /// The remembered answers.
    pub(super) prefs: ImposePrefs,
    /// Which booklet sides this pass prints. About one job; not remembered.
    subset: BookletSubset,
    /// The booklet's physical sheets to print, 1-based, inclusive. The engine
    /// clamps the end.
    sheets: (usize, usize),
    /// One entry per plan of this frame's job; empty when not imposing.
    pub(super) out: Vec<ImposedSheet>,
    /// What this frame's layout made.
    summary: Option<Summary>,
    /// The Pages tab's line about it.
    caption: Option<String>,
    /// Whether the first page sent is wider than tall, for [`Self::forced`].
    pub(super) first_landscape: bool,
    /// The preview's composed sheet; see `preview::imposed`.
    pub(super) preview: Option<(PreviewKey, egui::TextureHandle)>,
}

/// What a composed preview sheet was drawn from.
pub(super) type PreviewKey = (
    ImposedSheet,
    pdfcer_gui_base::printpreviewkey::PreviewKey,
    pdfcer_print::output_color::OutputColor,
);

impl State {
    /// Opened from the remembered answers. Poster mode wins a file that has
    /// both on, as the mode row would have left only one.
    pub(super) fn new(mut prefs: ImposePrefs, poster_on: bool) -> Self {
        if poster_on {
            prefs.mode = ImposeMode::Off;
        }
        Self {
            prefs,
            subset: BookletSubset::BothSides,
            sheets: (1, usize::MAX),
            out: Vec::new(),
            summary: None,
            caption: None,
            first_landscape: false,
            preview: None,
        }
    }

    /// Whether Multiple or Booklet is on.
    pub(super) fn on(&self) -> bool {
        self.prefs.mode != ImposeMode::Off
    }

    /// The orientation and duplex the layout requires, applied over the
    /// operator's where theirs is Auto (orientation) or would misprint
    /// (a booklet's duplex).
    pub(super) fn forced(&self, mut device: DeviceSettings) -> DeviceSettings {
        let turn = |landscape: bool| {
            if landscape {
                Orientation::Landscape
            } else {
                Orientation::Portrait
            }
        };
        let orientation = match self.prefs.mode {
            ImposeMode::Off => None,
            ImposeMode::NUp if matches!(self.prefs.count, 4 | 9 | 16) => None,
            ImposeMode::NUp => Some(turn(!self.first_landscape)),
            ImposeMode::Booklet => {
                device.duplex = match self.subset {
                    BookletSubset::BothSides => Duplex::ShortEdge,
                    BookletSubset::FrontOnly | BookletSubset::BackOnly => Duplex::Simplex,
                };
                Some(turn(!self.prefs.binding.is_stacked()))
            }
        };
        if device.orientation == Orientation::Auto
            && let Some(o) = orientation
        {
            device.orientation = o;
        }
        device
    }

    fn nup_spec(&self) -> NUpSpec {
        NUpSpec {
            grid: NUpGrid::Count(self.prefs.count),
            order: self.prefs.order,
            border: self.prefs.border,
            auto_rotate: self.prefs.rotate,
        }
    }

    fn booklet_spec(&self) -> BookletSpec {
        BookletSpec {
            binding: self.prefs.binding,
            subset: self.subset,
            sheets: Some(self.sheets),
            auto_rotate: self.prefs.rotate,
        }
    }

    /// The Pages tab's lines for this layout.
    fn caption_for(&self, summary: Summary, sheets: usize) -> String {
        match summary {
            Summary::NUp {
                rows,
                columns,
                turned,
            } => ti::nup_caption(rows, columns, sheets, turned),
            Summary::Booklet {
                total_sheets,
                printed,
                blanks,
            } => {
                let mut s = ti::booklet_caption(printed, total_sheets, blanks);
                if let Some(flip) = ti::booklet_flip(self.subset) {
                    s.push(' ');
                    s.push_str(flip);
                }
                s
            }
        }
    }
}

/// Impose `job` when Multiple or Booklet is on. Off, the job passes through.
pub(super) fn apply(
    dialog: &mut PrintDialog,
    job: Option<Job>,
    page_sizes: &[(f64, f64)],
    spec: &JobSpec,
) -> Option<Job> {
    let state = &mut dialog.impose;
    state.out.clear();
    state.summary = None;
    state.caption = None;
    if !state.on() || dialog.poster.on {
        return job;
    }
    let mut job = job?;
    let sequence = base::sequence_once(spec);
    let printable = job.device.printable_pt;
    let (imposed, collate) = match state.prefs.mode {
        ImposeMode::Off => return Some(job),
        ImposeMode::NUp => (
            base::n_up(printable, &sequence, page_sizes, &state.nup_spec()),
            spec.collate,
        ),
        // A booklet is folded as a set: an uncollated stack cannot fold.
        ImposeMode::Booklet => (
            base::booklet(printable, &sequence, page_sizes, &state.booklet_spec()),
            Collate::Collated,
        ),
    };
    let imposed = match imposed {
        Ok(imposed) => imposed,
        Err(error) => {
            let reason = error.to_string();
            crate::diag::trace_changed("print-impose-refused", || {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("print-impose-refused reason={reason}")
            });
            state.caption = Some(ti::impose_refused(&reason));
            return None;
        }
    };
    state.out = base::with_copies(&imposed.sheets, spec.copies, collate);
    let fallback = sequence.first().copied().unwrap_or(0);
    let render_scale = f64::from(job.resolution.dpi) / 72.0;
    job.plans = state
        .out
        .iter()
        .map(|sheet| PagePlan {
            index: sheet.cells.first().map_or(fallback, |c| c.page),
            placement: Placement {
                scale: 1.0,
                offset_x_pt: 0.0,
                offset_y_pt: 0.0,
                clipped: false,
            },
            render_scale,
            tile: None,
        })
        .collect();
    state.summary = Some(imposed.summary);
    state.caption = Some(state.caption_for(imposed.summary, imposed.sheets.len()));
    Some(job)
}

/// Render one imposed sheet for the spool: each page at its cell's density,
/// composed onto the printable area.
pub(super) fn render_sheet(
    source: super::running::Source<'_>,
    sheet: &ImposedSheet,
    job: &Job,
    options: &pdfcer_render::RenderOptions,
    lines_pt: Option<f64>,
) -> Result<PageBitmap, String> {
    let px_per_pt = f64::from(job.resolution.dpi) / 72.0;
    let view = source.session.view();
    let mut pages = Vec::with_capacity(sheet.cells.len());
    for cell in &sheet.cells {
        let Some(page) = source.pages.get(cell.page) else {
            continue;
        };
        let scale = base::render_scale(&cell.fit, px_per_pt);
        let mut options = options.clone();
        super::lines::apply(&mut options, lines_pt, scale, cell.fit.scale);
        let rendered = pdfcer_render::render_page_with_view(&view, page, scale as f32, &options)
            .map_err(|e| e.to_string())?;
        pages.push((rendered.pixmap, *cell));
    }
    let printable = job.device.printable_pt;
    let borrowed: Vec<(&pdfcer_render::tiny_skia::Pixmap, Cell)> =
        pages.iter().map(|(p, c)| (p, *c)).collect();
    let composed = base::compose(printable, px_per_pt, &borrowed)
        .ok_or_else(|| ti::impose_sheet_too_large().to_owned())?;
    Ok(PageBitmap {
        width: composed.width(),
        height: composed.height(),
        rgba: composed.data().to_vec(),
        placement: Placement {
            scale: 1.0,
            offset_x_pt: 0.0,
            offset_y_pt: 0.0,
            clipped: false,
        },
        page_pt: printable,
    })
}

/// The trace fields for `print-plan`.
/// `impose_duplex=` is the duplex SENT (after [`State::forced`]): the
/// line's `duplex=` is the operator's choice, which a booklet overrides.
pub(super) fn trace_fields(state: &State, sent: Duplex) -> String {
    let mode = match state.prefs.mode {
        ImposeMode::Off => "off",
        ImposeMode::NUp => "nup",
        ImposeMode::Booklet => "booklet",
    };
    let (grid, turned, booklet) = match state.summary {
        Some(Summary::NUp {
            rows,
            columns,
            turned,
        }) => ((columns, rows), turned, (0, 0)),
        Some(Summary::Booklet {
            total_sheets,
            blanks,
            ..
        }) => ((0, 0), 0, (total_sheets, blanks)),
        None => ((0, 0), 0, (0, 0)),
    };
    format!(
        // ui-text-exempt: diagnostic trace fields
        "impose={mode} nup_grid={}x{} nup_turned={turned} imposed_sheets={} booklet_sheets={} booklet_blanks={} impose_duplex={}",
        grid.0,
        grid.1,
        state.out.len(),
        booklet.0,
        booklet.1,
        match sent {
            Duplex::Simplex => "simplex",
            Duplex::LongEdge => "long-edge",
            Duplex::ShortEdge => "short-edge",
        }
    )
}

/// Pick `value` from `options` in a labelled combo; returns the combo's rect.
fn choose<T: Copy + PartialEq>(
    ui: &mut Ui,
    label: &str,
    salt: &str,
    value: &mut T,
    options: &[T],
    name: impl Fn(T) -> String,
) -> egui::Rect {
    ui.horizontal_wrapped(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(salt)
            .selected_text(name(*value))
            .show_ui(ui, |ui| {
                for &o in options {
                    ui.selectable_value(value, o, name(o));
                }
            })
            .response
            .rect
    })
    .inner
}

/// The Multiple or Booklet controls, in place of the scale radios.
pub(super) fn controls(ui: &mut Ui, state: &mut State) {
    match state.prefs.mode {
        ImposeMode::Off => return,
        ImposeMode::NUp => nup_controls(ui, state),
        ImposeMode::Booklet => booklet_controls(ui, state),
    }
    if let Some(caption) = &state.caption {
        ui.label(egui::RichText::new(caption).small());
    }
}

fn nup_controls(ui: &mut Ui, state: &mut State) {
    let p = &mut state.prefs;
    let count = choose(
        ui,
        ti::pages_per_sheet(),
        "print-nup-count", // ui-text-exempt: an id salt, never displayed
        &mut p.count,
        &NUP_COUNTS,
        |n| n.to_string(),
    );
    crate::diag::ui_rect(REGION_COUNT, count);
    choose(
        ui,
        ti::page_order(),
        "print-nup-order", // ui-text-exempt: an id salt, never displayed
        &mut p.order,
        &ORDERS,
        |o| ti::order(o).to_owned(),
    );
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut p.border, ti::border());
        ui.checkbox(&mut p.rotate, ti::rotate())
            .on_hover_text(ti::rotate_tooltip());
    });
}

fn booklet_controls(ui: &mut Ui, state: &mut State) {
    choose(
        ui,
        ti::subset_label(),
        "print-booklet-subset", // ui-text-exempt: an id salt, never displayed
        &mut state.subset,
        &SUBSETS,
        |s| ti::subset(s).to_owned(),
    );
    let total = match state.summary {
        Some(Summary::Booklet { total_sheets, .. }) => total_sheets.max(1),
        _ => 1,
    };
    let (mut from, mut to) = (state.sheets.0.min(total), state.sheets.1.min(total));
    ui.horizontal_wrapped(|ui| {
        ui.label(ti::sheets_from())
            .on_hover_text(ti::sheets_tooltip());
        let (widget, refusal) = entry::drag_value(ui, &mut from, entry::Kind::Count);
        refusal.show(ui.add(widget.range(1..=total)));
        ui.label(ti::sheets_to());
        let (widget, refusal) = entry::drag_value(ui, &mut to, entry::Kind::Count);
        refusal.show(ui.add(widget.range(from..=total)));
    });
    // An end at the last sheet stays "to the end", so a range widened by
    // choosing more pages keeps printing the whole booklet.
    state.sheets = (
        from,
        if to >= total {
            usize::MAX
        } else {
            to.max(from)
        },
    );
    choose(
        ui,
        ti::binding_label(),
        "print-booklet-binding", // ui-text-exempt: an id salt, never displayed
        &mut state.prefs.binding,
        &BINDINGS,
        |b| ti::binding(b).to_owned(),
    );
    ui.checkbox(&mut state.prefs.rotate, ti::rotate())
        .on_hover_text(ti::rotate_tooltip());
}

/// The Multiple and Booklet radios of the sizing row. `mode` is the
/// imposition the radio stands for.
pub(super) fn radio(ui: &mut Ui, dialog: &mut PrintDialog, mode: ImposeMode) {
    let (label, tip, region) = match mode {
        ImposeMode::NUp => (
            ti::sizing_mode_multiple(),
            ti::sizing_mode_multiple_tooltip(),
            REGION_MULTIPLE,
        ),
        ImposeMode::Booklet => (
            ti::sizing_mode_booklet(),
            ti::sizing_mode_booklet_tooltip(),
            REGION_BOOKLET,
        ),
        ImposeMode::Off => return,
    };
    let on = !dialog.poster.on && dialog.impose.prefs.mode == mode;
    let r = ui.radio(on, label).on_hover_text(tip);
    crate::diag::ui_rect(region, r.rect);
    if r.clicked() {
        dialog.poster.on = false;
        dialog.impose.prefs.mode = mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device() -> DeviceSettings {
        DeviceSettings::default()
    }

    #[test]
    fn the_layout_picks_the_orientation_only_when_auto() {
        let mut state = State::new(ImposePrefs::default(), false);
        state.prefs.mode = ImposeMode::NUp;
        state.prefs.count = 2;
        assert_eq!(state.forced(device()).orientation, Orientation::Landscape);
        state.first_landscape = true;
        assert_eq!(state.forced(device()).orientation, Orientation::Portrait);
        state.prefs.count = 4;
        assert_eq!(state.forced(device()).orientation, Orientation::Auto);
        let mut chosen = device();
        chosen.orientation = Orientation::Portrait;
        state.prefs.count = 2;
        state.first_landscape = false;
        assert_eq!(state.forced(chosen).orientation, Orientation::Portrait);
    }

    #[test]
    fn a_booklet_sets_its_sheet_and_duplex() {
        let mut state = State::new(ImposePrefs::default(), false);
        state.prefs.mode = ImposeMode::Booklet;
        let d = state.forced(device());
        assert_eq!(
            (d.orientation, d.duplex),
            (Orientation::Landscape, Duplex::ShortEdge)
        );
        state.prefs.binding = Binding::LeftTall;
        state.subset = BookletSubset::FrontOnly;
        let d = state.forced(device());
        assert_eq!(
            (d.orientation, d.duplex),
            (Orientation::Portrait, Duplex::Simplex)
        );
    }

    #[test]
    fn poster_wins_a_file_with_both_on() {
        let prefs = ImposePrefs {
            mode: ImposeMode::Booklet,
            ..ImposePrefs::default()
        };
        assert!(!State::new(prefs, true).on());
        assert!(State::new(prefs, false).on());
    }
}
