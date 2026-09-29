//! # `panels::properties::text` — how the selected text LOOKS, and changing it
//!
//! `RIBBON_IA.md` §5.8's Font controls — `format.font`, `format.font_size` —
//! built where that section says to build them:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/text.md`.

use egui::Ui;
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::text::panels::properties as t;

/// The trace region, so a driven check can find this section on screen.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.text";
/// The Bold button's own region.
///
/// Published per control rather than leaving a driven check to divide
/// [`REGION`] by eye — `geometry`'s own note on this is the precedent, and the
/// reason is that a check computing a control's position from a section's
/// bounds is a check that passes on a build where the controls moved.
// ui-text-exempt: trace region name, never displayed
pub const BOLD_REGION: &str = "properties.text.bold";
/// The **Italic** button's own region, published since 2026-08-29.
///
/// It had none while both weight buttons carried one shared sentence. They now
/// carry per-axis sentences derived from two separate
/// `preview_style_resolution` probes — a page holding a real `Arial-Bold` and no
/// `Arial-Italic` gives different answers for the two — so a driven check that
/// could point at Bold and not at Italic could assert half a per-axis answer,
/// which is the exact case the two probes exist to keep apart.
// ui-text-exempt: trace region name, never displayed
pub const ITALIC_REGION: &str = "properties.text.italic";
/// The size spinner's own region.
// ui-text-exempt: trace region name, never displayed
pub const SIZE_REGION: &str = "properties.text.size";
/// The face chooser's own region.
// ui-text-exempt: trace region name, never displayed
pub const FACE_REGION: &str = "properties.text.face";
/// The *Font file…* button beside the face chooser.
// ui-text-exempt: trace region name, never displayed
pub const FONT_FILE_REGION: &str = "properties.text.font-file";
/// The render-mode chooser's own region.
// ui-text-exempt: trace region name, never displayed
pub const RENDER_REGION: &str = "properties.text.render";

/// What the selected text looks like now, re-read only when it can have
/// changed.
#[derive(Default)]
pub struct TextStyleDraft {
    /// `(page, first run, edit epoch)` the values below were read at.
    stamp: Option<(usize, usize, u64)>,
    /// The run's `/BaseFont`, subset tag and all, or `None` when the
    /// provenance carried no font resource.
    face: Option<String>,
    /// The `Tf` size in points.
    size: f64,
    /// The fill colour as sRGB bytes, or `None` when the run is painted in a
    /// space this control cannot round-trip.
    ///
    /// `None` is shown as *"not a plain colour"* rather than as black. A
    /// swatch that renders a CMYK or Separation fill as its nearest RGB and
    /// then writes that back on the next press would silently convert the
    /// operator's ink — the exact narrowing pdfcer refuses to do on their
    /// behalf elsewhere.
    colour: Option<[u8; 3]>,
    /// The size the operator is typing, kept separate from [`Self::size`] so a
    /// half-typed number does not become an edit.
    typed_size: f64,
    /// **Which faces on this page `set_font` would accept for this run**,
    /// and the string to pass for each — `Pass 142.1`, consumed 2026-08-27.
    ///
    /// Read behind the same stamp as everything else here, because it costs
    /// another extraction and answers a question that only changes when the
    /// selection or the document does.
    ///
    /// `Vec<(selector, label, ambiguous)>`, flattened at read time rather than
    /// holding the engine's `FontPreflight`: the chooser needs three strings
    /// per row and holding the whole preflight would mean this struct's
    /// lifetime being tied to a `pdfcer-core` type that will grow.
    ///
    /// **Empty is a real answer and is not the same as "not read".** A page
    /// whose every font refuses this run — which `format_family.pdf` was until
    /// `Pass 144.0` — has no face to offer, and the chooser says so rather than
    /// falling back to a list of entries that cannot work.
    faces: Vec<FaceChoice>,
    /// **What pressing Bold would actually do to this run** —
    /// `EditSession::preview_style_ladder`, consumed 2026-09-11.
    ///
    /// Behind the same stamp as everything else here, because it costs a third
    /// content-stream plan and answers a question that changes only when the
    /// selection or the document does. `None` when the probe could not be run
    /// at all, which is a state distinct from every rung [`StyleOutlook`]
    /// carries and is rendered as the old conditional hint — the sentence that
    /// was there before any of this landed, and which is still the honest thing
    /// to say when nothing is known.
    ///
    /// It was `preview_style_resolution` from 2026-08-29 to 2026-09-11 and
    /// previewed the **R90 gate**, which is a different question from *"what
    /// will this button do?"* the moment a rung exists that binds a face the
    /// page does not carry. [`StyleOutlook`]'s header has the whole account.
    bold_outlook: Option<StyleForecast>,
    /// The italic twin of [`Self::bold_outlook`], probed **separately**.
    ///
    /// Two probes and not one, and it is not symmetry for its own sake.
    /// `gate_synthesis` is all-or-nothing per *combined* request: a page holding
    /// a real `Arial-Bold` but no `Arial-BoldItalic` answers a Bold+Italic
    /// request with *"no real face — synthesize both"*, silently passing over
    /// the real Bold. The two buttons here are two separate single-axis
    /// requests (`crate::app::dispatch::format` builds each with the other flag
    /// false — *"buttons that apply, not switches that reflect"*), so each one
    /// needs the answer for its own axis and neither may borrow the other's.
    ///
    /// ⇒ `StyleResolution::is_mixed` is deliberately not consulted anywhere in
    /// this module: it can only fire for a combined request, and this shell
    /// never issues one.
    italic_outlook: Option<StyleForecast>,
    /// **Which runs the controls act on when the operator CLICKED the text
    /// instead of sweeping it** — `OPERATOR_REQUESTS.md` O198, and the reason
    /// this struct is shared rather than duplicated.
    ///
    /// Every field above describes *one run*, read once per stamp. This one
    /// describes *which runs there are at all*, and it is here rather than in
    /// `app::textoperand` because this draft is the one object the ribbon's
    /// Font band and this panel already hold a **single shared instance** of.
    /// A cache of its own would mean two 392 ms extractions on one click and
    /// two answers free to disagree about what the operator selected.
    ///
    /// — It is `pub(crate)` in effect through [`Self::operand`] rather than
    /// directly, because the stamp is the whole value of the thing: a reader
    /// that took the runs without going through the resolver would get an
    /// answer from before the last edit.
    objects: crate::app::textoperand::Cache,
}

//
// It moved because the control did. `Pass 162.0` let pdfcer restyle text to a
// face the document does NOT contain, so a row in this list is now either a
// `Tf` rewrite or a `Tf` rewrite plus a new `/Font` object in the operator's
// file — and a chooser presenting those as one list would hide a write behind a
// menu. The two surfaces that draw it (this section and the ribbon's Format ▸
// Font group) were already two copies of one loop; they are now one.

pub(crate) mod style;

//
// `style` is `pub(crate)` rather than private, and the reason is a lint
// rather than a caller: [`TextStyleDraft::bold_outlook`] is `pub(crate)` and
// returns a `StyleForecast`, so a private module would make the return type
// less visible than the function that returns it — `private_interfaces`, which
// is denied by R4's `-D warnings`. Widening the module is the honest fix;
// narrowing the accessor would be hiding a real part of the draft's surface to
// satisfy a lint.
pub(crate) use style::StyleForecast;

use style::{bold_hint, italic_hint};

use super::face::FaceChoice;

impl TextStyleDraft {
    /// **Which runs a restyle would act on** — swept, or the single
    /// selected text object. `OPERATOR_REQUESTS.md` O198.
    pub(crate) fn operand(&mut self, doc: &OpenDoc) -> Option<crate::app::textoperand::Operand> {
        self.objects.resolve(doc)
    }

    /// Re-read from the document when the stamp has moved; otherwise keep what
    /// is on screen.
    ///
    /// Returns `true` when there is something to draw — i.e. the run resolved.
    pub(crate) fn sync(&mut self, doc: &OpenDoc, page: usize, run: usize) -> bool {
        let stamp = (page, run, doc.edit_epoch);
        if self.stamp == Some(stamp) {
            return self.face.is_some();
        }
        self.stamp = Some(stamp);
        let started = std::time::Instant::now();
        self.face = None;
        self.size = 0.0;
        self.colour = None;
        self.bold_outlook = None;
        self.italic_outlook = None;

        // The expensive call, made exactly here and nowhere else in this
        // module. See the module header on the 392 ms.
        let Some(read) = crate::canvas::textedit::pin::inspect(doc, page, run) else {
            // The unresolved run still gets a line. A failed `inspect` is the
            // EXPENSIVE failure — it walked the content stream and found no run
            // — and a panel that silently draws nothing after 400 ms is the
            // exact report ("the properties panel is blank and slow") that has
            // no evidence behind it unless the miss is traced as loudly as the
            // hit.
            let whole_ms = started.elapsed().as_millis();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("text-style-synced page={page} run={run} whole_ms={whole_ms} resolved=0")
            });
            return false;
        };
        self.size = f64::from(read.style.size);
        self.typed_size = self.size;
        // The join. `GlyphProvenance` records the RESOURCE KEY the content
        // stream used — `F1` — and an operator needs the `/BaseFont`. The
        // document's font inventory is the only place both appear, so this is
        // the one hop that turns a machine name into a human one.
        //
        // `None` when the key resolves to nothing, which is a real state on a
        // malformed page and is shown as such rather than as a blank combo.
        self.face = read.style.font_resource.as_ref().and_then(|key| {
            doc.font_inventory()
                .fonts
                .iter()
                .find(|record| record.resource_names.iter().any(|name| name == key))
                .and_then(|record| record.base_font.clone())
        });
        self.colour = read.style.fill.and_then(rgb_of);
        // The pre-flight, in the same stamped read as everything else. It
        // costs a second extraction, which is why it is here and not in the
        // chooser: a combo is drawn every frame it is open.
        //
        //
        // `accepted()` for the page half. A refused entry is deliberately
        // **not** offered greyed-with-a-reason, though the engine hands us the
        // reason: the refusals are per-character encoding facts (*"'o' has no
        // code in Times-Bold's encoding"*), and a list of twelve faces with nine
        // greyed rows each carrying a sentence about a character is a control an
        // operator cannot read. R9's absent-rather-than-greyed case: for THIS
        // run those faces are not a capability that is temporarily unavailable,
        // they are not applicable.
        self.faces = super::face::choices(
            // `None` — the run's OWN characters. This panel describes the run
            // as it stands, and there is no text about to be written; asking
            // about a candidate here would be asking about nothing.
            crate::canvas::textedit::pin::font_preflight(doc, page, &read, None).as_ref(),
        );
        // **What the two weight buttons would do**, asked here for the
        // reason everything else in this function is asked here: it costs a
        // content-stream read and a plan, and the answer changes only when the
        // stamp does. Twice — once per axis — because the two buttons issue two
        // separate single-axis requests; see `Self::italic_outlook`.
        //
        //
        // > AFTER `self.faces`, and the order is load-bearing rather than
        // > incidental: `Self::outlook` joins the preview's `selector` against
        // > the pre-flight's accepted list to tell `StyleOutlook::RealFace`
        // > from `StyleOutlook::FaceCannotCover`, so the pre-flight has to be
        // > in hand before the probes are read.
        //
        // That join was this shell telling two engine answers apart to
        // reconstruct a third. `preview_style_ladder` IS the third answer, so
        // `Self::forecast` reads nothing of `self` and could be called in any
        // order or from anywhere. It stays here because the cost belongs behind
        // the stamp, not because anything above it is a prerequisite.
        let forecast_started = std::time::Instant::now();
        self.bold_outlook = self.forecast(doc, page, &read, true);
        self.italic_outlook = self.forecast(doc, page, &read, false);
        let forecast_ms = forecast_started.elapsed().as_millis();

        // **The only instrument on this path, and it was added the day
        // the path got more expensive.**
        //
        // Before 2026-09-11 this module emitted nothing at all. It now makes
        // TWO `preview_style_ladder` calls per sync, each of which walks the
        // page's content stream — on top of the `inspect` this function's own
        // header records at 392 ms on the operator's site plan. The cost is
        // paid once per `(page, run, edit_epoch)` rather than per frame, so it
        // lands as a pause when text is SWEPT, which is exactly the shape of
        // report that arrives as *"selecting text got slow"* and has nothing
        // behind it to read.
        //
        // Measured rather than assumed, and reported even when it is
        // cheap: a number nobody logged until somebody complained is a number
        // with no baseline, and this project has already spent a session
        // reasoning about where time went instead of reading it
        // (`BENCHMARK.md`'s opening argument).
        //
        // `whole_ms` is the outer figure the operator would feel;
        // `forecast_ms` is this change's share of it. Both, because the ratio
        // is the actionable part — a slow sync whose forecast is 2 ms is not
        // this code's problem, and saying so takes one subtraction.
        let whole_ms = started.elapsed().as_millis();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "text-style-synced page={page} run={run} whole_ms={whole_ms} forecast_ms={forecast_ms} resolved=1 faces={} bold={} italic={}",
                self.faces.len(),
                u8::from(self.bold_outlook.is_some()),
                u8::from(self.italic_outlook.is_some()),
            )
        });
        self.face.is_some()
    }

    //
    // **One draft, two surfaces, and that is the whole reason these exist.**
    //
    // The ribbon's Font group and this panel's *This text* section show the
    // same four values and both need them read back from the document. The
    // read costs 392 ms on the operator's benchmark sheet — a text extraction
    // with provenance capture on — so a second draft would double it on every
    // selection change, on exactly the drawings this program is for.
    //
    // So `PanelsState` owns the one draft, `app::surfaces` borrows it for the
    // ribbon and `panels::properties` borrows it for the panel, and whichever
    // draws first in the frame pays for the read while the other gets a stamp
    // hit. These accessors are what let the ribbon read it without the fields
    // becoming public and without `app::fontband` learning how the stamp works.
    // -----------------------------------------------------------------------

    /// The run's `/BaseFont`, subset tag and all, or `None` when the
    /// provenance carried no font resource.
    #[must_use]
    pub(crate) fn face(&self) -> Option<&str> {
        self.face.as_deref()
    }

    /// The `Tf` size the document currently holds, in points.
    #[must_use]
    pub(crate) fn size(&self) -> f64 {
        self.size
    }

    /// The size the operator is typing.
    #[must_use]
    pub(crate) fn typed_size(&self) -> f64 {
        self.typed_size
    }

    /// The size the operator is typing, for a widget to write into.
    pub(crate) fn typed_size_mut(&mut self) -> &mut f64 {
        &mut self.typed_size
    }

    /// The fill colour as sRGB bytes, or `None` when the run is painted in a
    /// space this control cannot round-trip. See [`rgb_of`].
    #[must_use]
    pub(crate) fn colour(&self) -> Option<[u8; 3]> {
        self.colour
    }

    /// The faces `set_font` would accept for this run, in the order the engine
    /// listed the page's resources.
    #[must_use]
    pub(crate) fn faces(&self) -> &[FaceChoice] {
        &self.faces
    }

    /// What pressing **Bold** would do to this run, or `None` when the probe
    /// did not answer.
    #[must_use]
    pub(crate) fn bold_outlook(&self) -> Option<&StyleForecast> {
        self.bold_outlook.as_ref()
    }

    /// What pressing **Italic** would do to this run, or `None` when the probe
    /// did not answer.
    #[must_use]
    pub(crate) fn italic_outlook(&self) -> Option<&StyleForecast> {
        self.italic_outlook.as_ref()
    }
}

/// Draw the section, or nothing.
pub fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    draft: &mut TextStyleDraft,
    actions: &mut Vec<Action>,
) -> bool {
    // The staleness gate is inside the resolver, not here — a stale run
    // ordinal restyles the WRONG text, so the check lives with the data rather
    // than with each of its readers.
    //
    // `false`, not a sentence. Nothing text-shaped is selected, and a panel
    // that explained its own silence in that state would be explaining it on
    // most frames of most sessions.
    let Some(operand) = draft.operand(doc) else {
        return false;
    };
    let page = operand.page;
    let runs = operand.runs;
    let Some(&first) = runs.first() else {
        return false;
    };
    // Whether the Colour row below belongs to this section. See the header:
    // a clicked object's ink is classified by [`super::textobject`], which can
    // tell `Mixed` from `Agreed` from a spot ink and refuses a swatch over the
    // third. A sweep has no such problem — every run in it was swept
    // deliberately — so this section keeps its own row for that case.
    let owns_colour = operand.source == crate::app::textoperand::Source::Swept;

    if !draft.sync(doc, page, first) {
        // The selection is real and the run would not pin. Saying nothing here
        // would be the "control that is silently missing" defect: the operator
        // has text selected and the section they saw last time is gone. One
        // sentence, no controls.
        ui.heading(t::text_heading());
        ui.label(t::text_unreadable());
        crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
        return true;
    }

    ui.heading(t::text_heading());
    ui.label(t::text_covers(runs.len()));

    face_row(ui, doc, draft, page, &runs, actions);
    size_row(ui, draft, page, &runs, actions);
    weight_row(ui, draft, page, &runs, actions);
    render_row(ui, page, &runs, actions);
    if owns_colour {
        colour_row(ui, draft, page, &runs, actions);
    }

    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    if owns_colour {
        ui.separator();
    }
    true
}

/// The face: what this page carries, and what pdfcer can add to the document.
fn face_row(
    ui: &mut Ui,
    doc: &OpenDoc,
    draft: &TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    let current = draft.face.clone().unwrap_or_default();
    let _ = doc;
    ui.horizontal(|ui| {
        ui.label(crate::text::panels::face::text_face_label());
        let mut chosen = None;
        let combo = egui::ComboBox::from_id_salt("properties-text-face")
            .selected_text(shorten(&current))
            .show_ui(ui, |ui| {
                chosen = super::face::popup_body(ui, FACE_REGION, draft.faces(), shorten(&current));
            });
        crate::diag::ui_rect_visible(FACE_REGION, combo.response.rect, ui.clip_rect());
        // The action is raised HERE, outside the popup closure, because
        // nothing mutates from a widget — `app::actions`' founding invariant.
        // The closure reports which row was pressed and this row turns that into
        // one `Action`, exactly as the ribbon's copy turns it into one parked
        // `StyleChange`.
        if let Some(selector) = chosen {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Face(selector),
            });
        }
        let file = ui
            .button(crate::text::panels::face::font_file_button())
            .on_hover_text(crate::text::panels::face::font_file_hover());
        crate::diag::ui_rect_visible(FONT_FILE_REGION, file.rect, ui.clip_rect());
        if file.clicked() {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::FaceFile(None),
            });
        }
    });
}

/// The size, in points.
fn size_row(
    ui: &mut Ui,
    draft: &mut TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.label(t::text_size_label());
        let (widget, refusal) = entry::drag_value(
            ui,
            &mut draft.typed_size,
            entry::Kind::Length(entry::LengthUnit::Point),
        );
        let response = refusal.show(ui.add(widget.speed(0.25).range(1.0..=1440.0)));
        crate::diag::ui_rect_visible(SIZE_REGION, response.rect, ui.clip_rect());
        if (response.drag_stopped() || response.lost_focus())
            && (draft.typed_size - draft.size).abs() > f64::EPSILON
        {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Size(draft.typed_size),
            });
        }
    });
}

/// Bold and Italic — buttons that apply, not switches that reflect.
fn weight_row(
    ui: &mut Ui,
    draft: &TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.label(t::text_weight_label());
        let bold = ui.button(t::text_bold());
        crate::diag::ui_rect_visible(BOLD_REGION, bold.rect, ui.clip_rect());
        if bold.on_hover_text(bold_hint(draft)).clicked() {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Weight {
                    bold: true,
                    italic: false,
                },
            });
        }
        let italic = ui.button(t::text_italic());
        crate::diag::ui_rect_visible(ITALIC_REGION, italic.rect, ui.clip_rect());
        if italic.on_hover_text(italic_hint(draft)).clicked() {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Weight {
                    bold: false,
                    italic: true,
                },
            });
        }
    });
}

/// The render mode (`Tr`) — a chooser that applies, not one that reflects:
/// nothing here reads the run's current mode back.
fn render_row(ui: &mut Ui, page: usize, runs: &[usize], actions: &mut Vec<Action>) {
    ui.horizontal(|ui| {
        ui.label(t::text_render_label());
        let mut chosen: Option<u8> = None;
        let combo = egui::ComboBox::from_id_salt("properties-text-render")
            .selected_text(t::text_render_choose())
            .show_ui(ui, |ui| {
                for mode in 0..=7_u8 {
                    let entry = ui.selectable_label(false, t::text_render_mode_name(mode));
                    // One region per entry, `properties.text.render.<mode>`,
                    // so a driven check can aim at the popup.
                    crate::diag::ui_rect_visible(
                        &format!("{RENDER_REGION}.{mode}"),
                        entry.rect,
                        ui.clip_rect(),
                    );
                    if entry.clicked() {
                        chosen = Some(mode);
                    }
                }
            });
        crate::diag::ui_rect_visible(RENDER_REGION, combo.response.rect, ui.clip_rect());
        let _ = combo.response.on_hover_text(t::text_render_hint());
        if let Some(mode) = chosen {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::RenderMode(mode),
            });
        }
    });
}

/// The fill colour.
fn colour_row(
    ui: &mut Ui,
    draft: &TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    //
    // The refusal below is a whole sentence. `ui.horizontal` does not wrap: it
    // lays out past the end of the available width and reports a `min_rect`
    // that wide. Measured on a 354 pt dock, the section's box came out 851.7 pt
    // wide and the operator saw
    //
    //     Colour  Set in CMYK or a spot colour — pdfcer will not offer to
    //
    // with the rest against the window edge. That is the disclosure failing
    // in the precise way Rule 4 exists to prevent: the control is withheld AND
    // the reason is unreadable, so the operator is left with a missing swatch
    // and no account of it.
    //
    // It also hid itself. `diag::ui_rect_visible` publishes a region only
    // when 60 % of it survives the clip, so `properties.text` was never
    // published at all, and
    // `tools/ui-verify/src/checks/restyle_text.rs` read that absence as the
    // section not drawing and named three entirely correct functions as
    // candidates. The `ui-rect-clipped` line added to `diag.rs` the same
    // afternoon is what turned the absence into the measurement above.
    //
    // Wrapped rather than given a wider minimum, for `dialogs::print::preview`'s
    // reason: a minimum would be a constant asserting how wide a sentence is,
    // which depends on the theme preset's font and on the wording, so it would
    // be right in one preset and wrong in another. A wrapped row is bounded by
    // its available width by construction.
    ui.horizontal_wrapped(|ui| {
        ui.label(t::text_colour_label());
        let Some(current) = draft.colour else {
            ui.label(t::text_colour_not_plain());
            return;
        };
        let mut rgb = current;
        if ui.color_edit_button_srgb(&mut rgb).changed() && rgb != current {
            let components = vec![
                f64::from(rgb[0]) / 255.0,
                f64::from(rgb[1]) / 255.0,
                f64::from(rgb[2]) / 255.0,
            ];
            if let Ok(fill) = pdfcer_core::text_edit::NewFill::new(
                pdfcer_core::text_edit::FillModel::Rgb,
                components,
            ) {
                actions.push(Action::TextStyle {
                    page,
                    runs: runs.to_vec(),
                    change: StyleChange::Fill(fill),
                });
            }
        }
    });
}

//
// It built the face chooser's list from `fontinfo::FontInventory`, filtered to
// the records naming this page, and its own doc comment named the flaw
// honestly: *"this is a superset that is usually exactly right, and when it is
// not, the press earns a named refusal rather than silence. A proper pre-flight
// is filed with the engine as `Pass 142.1`."*
//
// `Pass 142.1` shipped, the same night it was asked for.
// `canvas::textedit::pin::font_preflight` is the replacement and it closes
// **two** holes, not one: the entries that could not work (offered, pressed,
// refused), and — much worse — the page carrying two subsets of one
// `/BaseFont`, where a name match reached one of the twins arbitrarily and
// applied the wrong font with no refusal to show for it. The Fonts panel's own
// survey puts that at 87 % of embedding files.
//
// The filed request said the superset was "usually right". It was, and
// "usually right" about which font is applied to an operator's drawing is not
// a standard this program should have been holding itself to. The pre-flight is
// not an optimisation of it; it is the correct answer where that was a guess.

/// A `/BaseFont` without its §9.6.4 subset tag, for display only.
pub(crate) fn shorten(base_font: &str) -> &str {
    match base_font.split_once('+') {
        // A subset tag is exactly six uppercase letters (§9.6.4). Anything else
        // before a `+` is part of the name and is kept — `Foo+Bar` is a legal,
        // if unusual, font name.
        Some((tag, rest)) if tag.len() == 6 && tag.bytes().all(|b| b.is_ascii_uppercase()) => rest,
        _ => base_font,
    }
}

/// sRGB bytes for a fill colour, or `None` when the space cannot round-trip.
pub(super) fn rgb_of(colour: pdfcer_core::text_extract::TextColor) -> Option<[u8; 3]> {
    use pdfcer_core::text_extract::TextColor;
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    match colour {
        TextColor::Rgb(r, g, b) => Some([byte(r), byte(g), byte(b)]),
        // DeviceGray IS a subset of DeviceRGB with r == g == b, so this is a
        // faithful reading. The write-back is a widening, disclosed by the
        // engine, and the operator asked for a colour.
        TextColor::Gray(v) => Some([byte(v), byte(v), byte(v)]),
        TextColor::Cmyk(..) | TextColor::Other => None,
        // `TextColor` is `#[non_exhaustive]`: a space added later is unknown,
        // and unknown means do not guess.
        _ => None,
    }
}
