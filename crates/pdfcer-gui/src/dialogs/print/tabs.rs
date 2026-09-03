//! # `dialogs::print::tabs` — the three groups of settings, and the range parser
//!
//! ## Why tabs at all
//!
//! Carried across from the old shell with its reasoning intact, because the
//! reasoning is the valuable part:
//!
//! > The dialog grew past what one column can hold, and the previous answer
//! > was a `CollapsingHeader` labelled "More options" holding orientation,
//! > duplex, copies, collation, reverse, subset, tray selection, annotation
//! > scope and the DPI override. That is not progressive disclosure, it is a
//! > drawer: a control's location told the operator nothing about what it
//! > did, everything below the fold was equally invisible, and the header's
//! > label promised no more than "there is additional stuff".
//! >
//! > Three tabs replace it, each named for the QUESTION it answers, so a
//! > control's location is itself a hint.
//!
//! ## ★ Where each control lives, and why it lives there
//!
//! The placements are not arbitrary and three of them are counter-intuitive
//! enough to be worth stating, because a later hand "tidying" them would undo
//! a decision that was reasoned:
//!
//! | control | tab | why not the other one |
//! |---|---|---|
//! | odd/even subset | **Pages & Layout** | It is a *selection* question — it narrows the same set the range radios narrow, and the two compose (`1-10` + Odd prints five sheets). Put under Copies it sits beside Reverse and reads as a delivery option, which is what an operator hand-feeding a duplex job would reasonably but wrongly assume. |
//! | orientation | **Pages & Layout** | It is a statement about how the page meets the sheet — the same question the sizing radios answer. |
//! | reverse | **Copies & Finishing** | It is a *delivery* question: it changes nothing about which pages print, only the order they land in the tray, and the reason to want it is a printer that stacks face-up. Same class as collation, so an operator fixing "my stack comes out backwards" looks in one place. |
//! | tray | **Copies & Finishing** | A request to the *driver* about hardware, like duplex — not arithmetic pdfcer performs. It is about which *stack* a sheet is pulled from, which is a feed question, where paper is a *shape* question. |
//! | paper | **Pages & Layout** | Same reasoning as orientation, and it arrived on 2026-08-18 for the same reason: it is a statement about the sheet the page meets. An operator whose A1 drawing came out on A4 and one whose landscape drawing came out portrait are looking for the same thing. |
//! | DPI | **Comments & Resolution** | Both halves of that tab are about the **pixels** rather than the paper: the scope decides what is in the bitmap and the resolution decides how much of it survives. |
//!
//! The **printer selector stays outside the tabs** — see
//! [`super::PrintDialog::options_column`] for why that one is not a setting
//! like the others.

use egui::Ui;

use crate::dialogs::print::PrintDialog;
use crate::dialogs::print::spooler::{
    Duplex, FormSourceSupport, JobResolution, Orientation, PageSubset, PaperChoice, ScaleMode,
};
use crate::text::print as t;

/// Which group of settings the dialog is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum PrintTab {
    /// Range, subset, sizing, orientation — which pages, and how each lands
    /// on the sheet.
    #[default]
    PagesLayout,
    /// Copies, collation, reverse order, duplex, tray — how many sheets come
    /// out and in what state.
    CopiesFinishing,
    /// Annotation scope and rendering resolution — what is painted onto each
    /// page, and how finely.
    CommentsResolution,
}

impl PrintTab {
    /// Every tab, in the order the strip draws them.
    ///
    /// An array rather than three literal calls at the draw site, so adding a
    /// tab is one edit and cannot leave the strip and the content branch
    /// disagreeing about how many there are.
    pub(super) const ALL: [Self; 3] = [
        Self::PagesLayout,
        Self::CopiesFinishing,
        Self::CommentsResolution,
    ];

    /// The tab's label.
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::PagesLayout => t::tab_pages_layout(),
            Self::CopiesFinishing => t::tab_copies_finishing(),
            Self::CommentsResolution => t::tab_comments_resolution(),
        }
    }

    /// The tab's hover text — the question it answers.
    pub(super) fn tooltip(self) -> &'static str {
        match self {
            Self::PagesLayout => t::tab_pages_layout_tooltip(),
            Self::CopiesFinishing => t::tab_copies_finishing_tooltip(),
            Self::CommentsResolution => t::tab_comments_resolution_tooltip(),
        }
    }
}

/// Which pages a print job covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum PrintRange {
    /// Every page.
    #[default]
    All,
    /// The page on the canvas.
    Current,
    /// A typed range, parsed with [`parse_page_range`].
    Custom,
}

impl PrintRange {
    /// The zero-based document pages this range names, in document order.
    ///
    /// The subset filter, the reversal and the copy multiplication are **not**
    /// applied here: they are `pdfcer-print`'s, they have a defined order of
    /// operations (subset → reverse → copies) that is *"the only place a
    /// defect can hide"*, and restating any of it in the shell would be a
    /// second implementation of exactly the kind [`parse_page_range`]'s own
    /// docs argue against.
    ///
    /// An unparseable custom range yields an **empty** vector rather than a
    /// guess, which is what lets the dialog say so and withhold the commit
    /// button instead of printing a range nobody asked for.
    pub(super) fn indices(self, text: &str, page_count: usize, current: usize) -> Vec<usize> {
        match self {
            Self::All => (0..page_count).collect(),
            Self::Current => {
                if current < page_count {
                    vec![current]
                } else {
                    Vec::new()
                }
            }
            Self::Custom => parse_page_range(text, page_count).unwrap_or_default(),
        }
    }
}

/// Parse `3`, `1-4`, `5,1-2` into zero-based indices.
///
/// # Deliberately the same syntax the CLI accepts
///
/// Carried across verbatim, with its reasoning:
///
/// > Two range parsers would eventually disagree about something like
/// > `5,1-2` — whether it reorders, whether it deduplicates — and an operator
/// > moving between the GUI and a script would have no way to know which one
/// > they were talking to. The syntax is kept identical and the behaviour on
/// > malformed input is the same: an unparseable range yields NOTHING rather
/// > than a guess, so the Print button disables and says why instead of
/// > printing a range nobody asked for.
///
/// Note what "the same" *includes*, because two of these are surprising and
/// both are deliberate: the result **preserves the order typed** (so `5,1-2`
/// prints 5 first) and **does not deduplicate** (so `1,1` prints page 1
/// twice). Both fall out of treating the text as a *sequence* the operator
/// wrote rather than as a set, and both match the CLI.
///
/// Page numbers are one-based — the numbers printed on the paper — and any
/// number outside the document refuses the whole spec rather than being
/// clamped. Clamping would turn a typo into a job.
/// ★ `pub(crate)` since 2026-08-18, so `dialogs::insert_pages` shares it.
///
/// The argument above was made about the GUI and the CLI. It is the same
/// argument between two GUI surfaces and stronger: an operator who learned this
/// syntax on Print is entitled to it working on Insert, and a second parser
/// here would be the drift that paragraph exists to prevent, one layer in.
pub(crate) fn parse_page_range(spec: &str, count: usize) -> Option<Vec<usize>> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.split_once('-') {
            Some((a, b)) => {
                let a: usize = a.trim().parse().ok()?;
                let b: usize = b.trim().parse().ok()?;
                if a == 0 || b == 0 || a > b || b > count {
                    return None;
                }
                out.extend((a - 1)..b);
            }
            None => {
                let n: usize = part.parse().ok()?;
                if n == 0 || n > count {
                    return None;
                }
                out.push(n - 1);
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

// ---------------------------------------------------------------------------
// Tab 1 — Pages & Layout
// ---------------------------------------------------------------------------

/// Which pages, and how each one lands on the sheet.
pub(super) fn pages_layout(
    ui: &mut Ui,
    dialog: &mut PrintDialog,
    page_count: usize,
    sheet: Option<(f64, f64)>,
) {
    ui.label(t::pages_heading());
    ui.radio_value(&mut dialog.range, PrintRange::All, t::range_all(page_count));
    ui.radio_value(&mut dialog.range, PrintRange::Current, t::range_current());
    ui.horizontal(|ui| {
        ui.radio_value(&mut dialog.range, PrintRange::Custom, t::range_custom());
        if ui
            .add(egui::TextEdit::singleline(&mut dialog.range_text).desired_width(120.0))
            .on_hover_text(t::range_hint())
            .changed()
        {
            // Typing in the box means the operator wants that range; making
            // them also click the radio is the kind of second step that reads
            // as the software not listening.
            dialog.range = PrintRange::Custom;
        }
    });
    // Said only when it is true and only when it can be acted on: a range
    // that names nothing is a typo the operator can fix, and the commit
    // button is absent while it stands.
    if dialog.range == PrintRange::Custom
        && parse_page_range(&dialog.range_text, page_count).is_none()
        && !dialog.range_text.trim().is_empty()
    {
        ui.label(egui::RichText::new(t::range_unparsable()).color(ui.visuals().warn_fg_color));
    }

    ui.horizontal(|ui| {
        ui.label(t::subset_label());
        for (subset, label) in [
            (PageSubset::All, t::subset_all()),
            (PageSubset::Odd, t::subset_odd()),
            (PageSubset::Even, t::subset_even()),
        ] {
            if ui.radio(dialog.subset == subset, label).clicked() {
                dialog.subset = subset;
            }
        }
    })
    .response
    .on_hover_text(t::subset_tooltip());
    ui.add_space(8.0);

    ui.label(t::sizing_heading())
        .on_hover_text(t::sizing_tooltip());
    // Four modes, not three. `place_page`'s own test exists because
    // collapsing Fit and Shrink is the natural simplification and it silently
    // blows a business card up to fill a Letter sheet.
    //
    // Drawn with `radio` + `clicked` rather than `radio_value` because
    // `ScaleMode::Custom` carries an `f64`: a `radio_value` comparison would
    // make the Custom row deselect itself the moment the percentage changed.
    for (mode, label) in [
        (ScaleMode::Fit, t::scale_fit()),
        (ScaleMode::ActualSize, t::scale_actual()),
        (ScaleMode::ShrinkOversized, t::scale_shrink()),
    ] {
        if ui.radio(dialog.scale == mode, label).clicked() {
            dialog.scale = mode;
        }
    }
    let custom_selected = matches!(dialog.scale, ScaleMode::Custom(_));
    ui.horizontal(|ui| {
        if ui.radio(custom_selected, t::scale_custom()).clicked() {
            dialog.scale = ScaleMode::Custom(f64::from(dialog.custom_percent) / 100.0);
        }
        // Enabled only while Custom is chosen, and **greyed rather than
        // absent** — which is the correct side of the no-placeholders rule
        // here, because this really is *temporarily* unavailable: one click
        // on the radio beside it makes it live.
        // ★★★ …and it now SAYS so — O77's sweep. The comment above has
        // always argued that greying is correct here *because* one click on
        // the radio beside it makes the field live. R9 requires that argument
        // to reach the operator, and it never did: the control was greyed with
        // no hover explanation of any kind.
        let custom = ui.add_enabled(
            custom_selected,
            egui::DragValue::new(&mut dialog.custom_percent)
                .range(1..=1000)
                .suffix(t::percent_suffix()),
        );
        if !custom_selected {
            custom.on_disabled_hover_text(t::scale_custom_disabled());
        }
    });
    ui.add_space(8.0);

    ui.label(t::orientation_heading());
    for (orientation, label) in [
        (Orientation::Auto, t::orientation_auto()),
        (Orientation::Portrait, t::orientation_portrait()),
        (Orientation::Landscape, t::orientation_landscape()),
    ] {
        if ui
            .radio(dialog.device.orientation == orientation, label)
            .clicked()
        {
            dialog.device.orientation = orientation;
        }
    }

    // ★ WHICH PAPER — a control now, where a sentence used to be.
    //
    // The sentence it replaces said, correctly at the time, that paper came
    // from the printer's own Windows settings and *"pdfcer cannot change it"*.
    // `pdfcer-print` shipped `PaperSelection` on 2026-08-18 and that stopped
    // being true. See `crate::text::print::sheet_from_driver` for why an
    // expiring disclosure is a class of defect worth naming.
    //
    // It sits beside orientation because they are one decision: both are
    // statements about the sheet the page is going to meet, and an operator
    // whose A1 drawing came out on A4 looks in the same place as one whose
    // landscape drawing came out portrait.
    ui.add_space(8.0);
    ui.label(t::paper_heading());
    if dialog.forms.is_empty() {
        // R9: no combo with nothing in it. The driver enumerated no sheets,
        // which is a legal answer, and the honest response is a sentence.
        ui.label(egui::RichText::new(t::paper_not_listed()).small().weak());
    } else {
        let selected_text = match dialog.device.paper {
            PaperChoice::DeviceDefault => t::paper_device_default().to_owned(),
            // A form id with no matching entry falls back to the default
            // label rather than showing a bare number. It is reachable in
            // one way: the driver's own properties dialog can name a form
            // this device did not enumerate through `DC_PAPERS`.
            PaperChoice::Form(id) => dialog.forms.iter().find(|form| form.id == id).map_or_else(
                || t::paper_device_default().to_owned(),
                |form| t::paper_form(&form.name, form.size_pt),
            ),
        };
        let combo = egui::ComboBox::from_id_salt("print-paper")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                // ★ Every entry publishes its rect while the list is open.
                //
                // Not instrumentation for its own sake. An egui combo popup is
                // an `Area` laid out at paint time, so an out-of-process check
                // has no way to compute where an entry is — and without that
                // it can assert only that the control exists, which is exactly
                // the claim that was true of the tray checkbox while it did
                // nothing. See `REGION_PAPER_ITEM_PREFIX`.
                //
                // These regions vanish when the popup closes, and
                // `diag::end_ui_frame` emits `ui-rect-gone` for each — so the
                // trace says the list closed rather than leaving fossil rects
                // a reader would take for live layout.
                let default = ui.selectable_value(
                    &mut dialog.device.paper,
                    PaperChoice::DeviceDefault,
                    t::paper_device_default(),
                );
                crate::diag::ui_rect(
                    &format!("{}0", super::REGION_PAPER_ITEM_PREFIX),
                    default.rect,
                );
                for (index, form) in dialog.forms.iter().enumerate() {
                    let entry = ui.selectable_value(
                        &mut dialog.device.paper,
                        PaperChoice::Form(form.id),
                        t::paper_form(&form.name, form.size_pt),
                    );
                    crate::diag::ui_rect(
                        &format!("{}{}", super::REGION_PAPER_ITEM_PREFIX, index + 1),
                        entry.rect,
                    );
                }
            });
        crate::diag::ui_rect(super::REGION_PAPER, combo.response.rect);
    }

    // The disclosure under it, and WHICH disclosure depends on what was
    // chosen — because the two cases are genuinely different claims.
    //
    // `DeviceDefault` sends no paper request at all, so there is nothing a
    // driver could ignore; the line simply reports the sheet the job was
    // planned against. An explicit form IS a request, and `pdfcer-print`
    // measured two drivers silently ignoring one — so that line says so, and
    // names the only check available to anybody. See
    // `crate::text::print::paper_is_a_request`.
    //
    // Both name the sheet the PLAN used, taken from the turned geometry, so
    // the sentence describes the rectangle the preview above it is drawing
    // rather than a size read from somewhere else.
    ui.add_space(4.0);
    let line = match dialog.device.paper {
        PaperChoice::DeviceDefault => t::sheet_from_driver(sheet),
        PaperChoice::Form(_) => t::paper_is_a_request(sheet),
    };
    ui.label(egui::RichText::new(line).small().weak());
}

// ---------------------------------------------------------------------------
// Tab 2 — Copies & Finishing
// ---------------------------------------------------------------------------

/// How many sheets come out, and in what state.
pub(super) fn copies_finishing(ui: &mut Ui, dialog: &mut PrintDialog) {
    ui.horizontal(|ui| {
        ui.label(t::copies_label());
        ui.add(egui::DragValue::new(&mut dialog.copies).range(1..=999));
    });
    ui.checkbox(&mut dialog.uncollated, t::uncollated());
    ui.checkbox(&mut dialog.reverse, t::reverse())
        .on_hover_text(t::reverse_tooltip());
    ui.add_space(8.0);

    // ★ R83: no duplex control for a device that cannot duplex. pdfcer does
    // NOT simulate it by reordering pages and asking the operator to reinsert
    // the stack — that workflow has a documented mis-assembly failure mode,
    // and offering it as though it were duplex would claim a capability the
    // hardware does not have.
    //
    // Absent rather than disabled: a greyed control implies something the
    // operator could turn on, and no setting in this dialog will ever make
    // this printer two-sided.
    //
    // Note what this means for the tab: on a simplex-only device this tab is
    // SHORTER, not emptier-looking. That is the intended reading — the tab
    // still holds copies, collation and reverse, so it never becomes a tab
    // with nothing in it.
    if dialog.features.supports_duplex {
        ui.label(t::duplex_heading());
        for (duplex, label) in [
            (Duplex::Simplex, t::duplex_off()),
            (Duplex::LongEdge, t::duplex_long()),
            (Duplex::ShortEdge, t::duplex_short()),
        ] {
            if ui.radio(dialog.device.duplex == duplex, label).clicked() {
                dialog.device.duplex = duplex;
            }
        }
        ui.add_space(8.0);
    }

    // ★ THE TRAY CHECKBOX, removed 2026-08-17 and restored 2026-08-18.
    //
    // It was removed because it did nothing:
    // `DeviceSettings::pick_tray_by_page_size` was a field `pdfcer-print`
    // declared and read nowhere, so the job spooled, the paper came out of
    // the default tray, and nothing reported that the request had been
    // dropped. That is the worst variety of inert control — it **succeeds** —
    // and it is indistinguishable from a driver that declined.
    //
    // The engine now honours it (`DMBIN_FORMSOURCE`, asserted only when this
    // box is ticked, so an unticked box cancels nothing the driver was doing
    // by itself). The reason for the removal no longer holds.
    //
    // ★ AND IT IS DRAWN IN ALL THREE CAPABILITY STATES, which inverts the
    // rule the duplex block above follows. `pdfcer-print` declined this
    // project's proposal to gate it like duplex, with a measurement: DC_BINS
    // on Microsoft Print to PDF returns nothing at all, while that same
    // device's `dmDefaultSource` is ALREADY `DMBIN_FORMSOURCE`. A bool would
    // have hidden the control on a device that was doing the thing by
    // default. R83 forbids offering what the hardware cannot honour; it does
    // not forbid offering what the driver merely declined to advertise.
    ui.add_space(8.0);
    ui.checkbox(&mut dialog.device.pick_tray_by_page_size, t::tray_by_size())
        .on_hover_text(t::tray_tooltip());
    if matches!(
        dialog.features.form_source,
        FormSourceSupport::NotListed | FormSourceSupport::Unknown
    ) {
        ui.label(egui::RichText::new(t::tray_not_advertised()).small().weak());
    }
}

// ---------------------------------------------------------------------------
// Tab 3 — Comments & Resolution
// ---------------------------------------------------------------------------

/// What is painted onto each page, and how finely.
///
/// Both halves are about the **pixels** rather than about the paper, which is
/// what makes them one tab: the annotation scope decides what is in the
/// bitmap and the resolution decides how much of it survives.
pub(super) fn comments_resolution(
    ui: &mut Ui,
    dialog: &mut PrintDialog,
    resolution: Option<JobResolution>,
) {
    ui.label(t::comments_heading());
    // ★ Four options, and there used to be one.
    //
    // Carried across with its history: `RenderOptions` once carried a single
    // `bool`, so the dialog offered a single honestly-labelled toggle rather
    // than the four-way selector — a control implying a capability that does
    // not exist is R83's failure even when the control itself works. The
    // renderer gained `AnnotationScope`, so the selector is backed and
    // offered.
    //
    // `AnnotationScope::ContentOnly` is deliberately not among them: it is
    // pdfcer's own fifth value, load-bearing for the round-trip raster oracle
    // rather than for an operator, and a print with neither form fields nor
    // links is not a thing this dialog has been asked for.
    for (scope, label) in [
        (
            pdfcer_render::AnnotationScope::Document,
            t::scope_document(),
        ),
        (
            pdfcer_render::AnnotationScope::DocumentAndMarkups,
            t::scope_markups(),
        ),
        (
            pdfcer_render::AnnotationScope::DocumentAndStamps,
            t::scope_stamps(),
        ),
        (
            pdfcer_render::AnnotationScope::FormFieldsOnly,
            t::scope_fields_only(),
        ),
    ] {
        if ui.radio(dialog.scope == scope, label).clicked() {
            dialog.scope = scope;
        }
    }
    ui.add_space(8.0);

    ui.label(t::resolution_heading());
    // Always true, so a static caption rather than a warning. A banner that
    // fires on every job trains an operator to stop reading banners — which
    // is how the *conditional* disclosure below it would come to be ignored
    // too.
    ui.label(egui::RichText::new(t::raster_note()).small().weak());
    // Conditional, because it is a per-job substitution: pdfcer picked a
    // resolution the operator did not. The spinner appears **with** the
    // sentence rather than always, so the control and the reason it exists
    // arrive together.
    if let Some(res) = resolution
        && res.capped
    {
        ui.add_space(4.0);
        ui.label(t::dpi_capped(res.dpi, res.device_dpi, res.uncapped_page_mb));
        ui.add(
            egui::DragValue::new(&mut dialog.max_dpi)
                .range(36..=2400)
                .suffix(t::dpi_suffix()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three forms the CLI accepts all parse, and to one-based-minus-one.
    #[test]
    fn the_three_range_forms_parse() {
        assert_eq!(parse_page_range("3", 10), Some(vec![2]));
        assert_eq!(parse_page_range("1-4", 10), Some(vec![0, 1, 2, 3]));
        assert_eq!(parse_page_range("5,1-2", 10), Some(vec![4, 0, 1]));
    }

    /// ★ The order typed is the order printed, and duplicates survive.
    ///
    /// Both are *behaviours*, not accidents, and both are shared with the
    /// CLI — which is the whole reason there is one parser. A future "tidy"
    /// that sorted or de-duplicated here would make the same text mean two
    /// different jobs depending on which surface the operator typed it into,
    /// and neither surface would say which.
    #[test]
    fn the_typed_order_is_preserved_and_duplicates_are_kept() {
        assert_eq!(parse_page_range("5,1-2", 10), Some(vec![4, 0, 1]));
        assert_eq!(parse_page_range("1,1", 10), Some(vec![0, 0]));
    }

    /// ★ Malformed input yields NOTHING, never a salvaged prefix.
    ///
    /// The property the whole "one parser" argument rests on: a range that
    /// cannot be read must not become a job. Each of these would be a
    /// plausible thing to "recover" from, and recovering would print pages
    /// nobody asked for.
    #[test]
    fn malformed_input_refuses_rather_than_recovering() {
        for spec in ["0", "4-2", "11", "1-11", "abc", "1-", "-3", "1,abc", ""] {
            assert_eq!(
                parse_page_range(spec, 10),
                None,
                "{spec:?} must refuse rather than yield a partial range"
            );
        }
    }

    /// Whitespace around numbers and separators is tolerated.
    ///
    /// Operators paste ranges. Refusing ` 1 - 4 ` would be a refusal about
    /// typography rather than about pages.
    #[test]
    fn surrounding_whitespace_is_tolerated() {
        assert_eq!(parse_page_range(" 1 - 3 , 5 ", 10), Some(vec![0, 1, 2, 4]));
    }

    /// A single-page document accepts `1` and refuses `2`.
    ///
    /// The boundary, asserted because an off-by-one here is a job that
    /// silently drops the only page.
    #[test]
    fn the_document_boundary_is_inclusive_at_one_page() {
        assert_eq!(parse_page_range("1", 1), Some(vec![0]));
        assert_eq!(parse_page_range("2", 1), None);
        assert_eq!(parse_page_range("1-1", 1), Some(vec![0]));
    }

    /// `All` covers the document; `Current` covers exactly one page.
    #[test]
    fn the_range_radios_select_what_they_say() {
        assert_eq!(PrintRange::All.indices("", 3, 1), vec![0, 1, 2]);
        assert_eq!(PrintRange::Current.indices("", 3, 1), vec![1]);
        assert_eq!(PrintRange::Custom.indices("2-3", 3, 0), vec![1, 2]);
    }

    /// ★ A current page past the end of the document selects nothing.
    ///
    /// Reachable rather than theoretical: the dialog holds the page index it
    /// opened on, and a document can be closed and a shorter one opened while
    /// it is up. Selecting *something* here would print a page that is not
    /// the one the radio names.
    #[test]
    fn a_stale_current_page_selects_nothing() {
        assert!(PrintRange::Current.indices("", 2, 7).is_empty());
    }

    /// An unparseable custom range selects nothing, so nothing can print.
    #[test]
    fn an_unparseable_custom_range_selects_nothing() {
        assert!(PrintRange::Custom.indices("banana", 9, 0).is_empty());
    }

    /// Every tab has its own label and its own question.
    ///
    /// The tabs earn their keep only if their names distinguish them; three
    /// tabs sharing a tooltip would be the drawer this design replaced,
    /// wearing a strip of buttons.
    #[test]
    fn the_three_tabs_are_distinguishable() {
        let labels: Vec<_> = PrintTab::ALL.iter().map(|t| t.label()).collect();
        let tips: Vec<_> = PrintTab::ALL.iter().map(|t| t.tooltip()).collect();
        for i in 0..PrintTab::ALL.len() {
            for j in (i + 1)..PrintTab::ALL.len() {
                assert_ne!(labels[i], labels[j]);
                assert_ne!(tips[i], tips[j]);
            }
        }
    }
}
