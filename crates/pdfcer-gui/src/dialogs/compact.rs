//! # `dialogs::compact` — the window before a full rewrite
//!
//! `file.save_compacted`, wired 2026-08-28. `OPERATOR_REQUESTS.md` **O48**,
//! answered *"yes to all three"*.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/compact.md`.

use egui::Ui;
use pdfcer_core::space_audit::SpaceCategory;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::dialogs::compact_images::{self, Pictures};
use crate::dialogs::compact_target;
use crate::dialogs::compact_version::{self, Lowering};
use crate::dialogs::recompress::{Choices, Job, Recompressed};
use crate::text::compact as t;
use pdfcer_core::PdfVersion;
use pdfcer_core::optimize::{DiscardObject, DiscardOptions};

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "compact.body";
/// The button that goes on to the picker.
// ui-text-exempt: trace region name, never displayed
pub const REGION_SAVE: &str = "compact.commit";
/// The button that closes the window without writing.
// ui-text-exempt: trace region name, never displayed
pub const REGION_CANCEL: &str = "compact.cancel";
/// The switch that cuts embedded fonts down.
// ui-text-exempt: trace region name, never displayed
pub const REGION_FONTS: &str = "compact.fonts";
/// Each sentence saying what cutting the fonts did.
// ui-text-exempt: trace region name, never displayed
pub const REGION_FONTS_SAID: &str = "compact.fonts.said";
/// The switch that tidies page drawing instructions.
// ui-text-exempt: trace region name, never displayed
pub const REGION_CONTENT: &str = "compact.content";
/// The sentence saying what tidying them did.
// ui-text-exempt: trace region name, never displayed
pub const REGION_CONTENT_SAID: &str = "compact.content.said";
/// The rounding list; each entry is `{REGION_ROUND}.{index}` in [`ROUNDINGS`].
// ui-text-exempt: trace region name, never displayed
pub const REGION_ROUND: &str = "compact.content.round";
/// The Fast Web View switch.
// ui-text-exempt: trace region name, never displayed
pub const REGION_LINEARIZE: &str = "compact.linearize";
/// The rounding distances offered, in points, after Off.
const ROUNDINGS: [f64; 4] = [0.001, 0.01, 0.1, 0.5];
/// Height kept for the separator and the Save row below the scrolling choices.
const BUTTON_ROW: f32 = 40.0;
/// Height kept above it for the two-line picture-loss warning.
const LOSSY_LINE: f32 = 40.0;

/// The window, and the bytes it measured.
pub struct CompactDialog {
    /// The compacted file, already serialised. See the header.
    bytes: Vec<u8>,
    /// What the document occupies on disk now.
    ///
    /// Read from the **file**, not from the session's own byte count, because
    /// the number the operator compares against is the one Explorer shows them.
    ///
    /// `0` for a document that has never been saved — a blank one — and that
    /// falls out correctly rather than by accident: `size_change` compares
    /// `after >= before`, so a zero `before` takes the *"no smaller"* branch and
    /// says the honest thing. A file that does not exist yet cannot be shrunk,
    /// and the alternative — an `Option` and a fourth sentence — would be a
    /// branch for a case the arithmetic already answers.
    /// Read from the **file** each time the window opens, never cached: the
    /// operator may have saved since, and a stale `before` would quote a saving
    /// against a file that is no longer there.
    before: u64,
    /// How many digital signatures the copy will not keep.
    signatures: usize,
    /// The file on disk, audited for the breakdown's Now column.
    path: std::path::PathBuf,
    /// The breakdown, measured the first time its fold is opened; the inner
    /// `None` is a copy that would not parse back.
    space: Option<Option<Breakdown>>,
    /// What is done to the copy beyond compacting. Recompressing and removing
    /// what goes nowhere start on: both are lossless to what a reader can
    /// reach, and the result is kept only when smaller. Removing unused named
    /// places starts off, since a link from another file may name one.
    choices: Choices,
    recompress: Recompress,
    /// A loaded settings file, run in place of [`Self::choices`].
    profile: crate::dialogs::compact_profile::State,
    /// *Make compatible with*; `None` keeps the copy's version.
    version: Option<PdfVersion>,
    /// Lowering the copy to [`Self::version`], last of the work on it.
    lowering: Lowering,
    /// The operator's settings, for the recompressed rewrite's session and
    /// save options.
    settings: pdfcer_core::settings::Settings,
    save_requested: bool,
    close_requested: bool,
}

impl CompactDialog {
    /// Serialise the compacted copy and open on the result.
    ///
    /// `None` when the engine refuses the rewrite — see [`open_for`], which
    /// turns that into a sentence rather than into silence.
    pub fn open(doc: &OpenDoc, start: compact_target::Start) -> Result<Self, String> {
        use crate::app::settings::SettingsExt;
        // The SAME `SaveOptions` the ordinary save uses. Two settings ride on
        // it — the cross-reference entry line ending and the trailing newline —
        // and both change the bytes of the file the operator receives. A
        // `::default()` here would produce a compacted copy that differed from
        // an ordinary one in ways nobody chose, on a command whose whole subject
        // is the bytes.
        let options = doc.settings.save_options();
        let (bytes, _report) = doc
            .session
            .to_full_bytes(&options)
            .map_err(|error| t::refused(&error.to_string()))?;
        let before = std::fs::metadata(&doc.path).map(|m| m.len()).unwrap_or(0);
        let (preset, budget) = start.choices(bytes.len() as u64);
        Ok(Self {
            bytes,
            before,
            signatures: doc.session.signature_census().signatures,
            path: doc.path.clone(),
            space: None,
            choices: Choices {
                recompress: true,
                broken: true,
                destinations: false,
                discard: DiscardOptions::none(),
                pictures: Pictures::OFF,
                classes: crate::dialogs::compact_classes::Classes::OFF,
                fonts: false,
                content: true,
                round: None,
                linearize: false,
                preset,
                budget,
            },
            recompress: Recompress::Off,
            profile: crate::dialogs::compact_profile::State::default(),
            version: None,
            lowering: Lowering::Off,
            settings: doc.settings.clone(),
            save_requested: false,
            close_requested: false,
        })
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        self.recompress_step(ctx);
        let (frame, ()) = crate::dialogs::host::Host::new(
            "compact-copy", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(520.0, 340.0),
            egui::vec2(380.0, 240.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.save_requested) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "compact-requested before={} after={} signatures={}",
                    self.before,
                    chosen(&self.bytes, &self.recompress, &self.lowering).len(),
                    self.signatures
                )
            });
            // The measured bytes travel with the action. They are the operand
            // — see the header — and rebuilding them in the apply arm would put
            // a second serialisation between what the window promised and what
            // the operator receives.
            actions.push(Action::Write(
                crate::app::actions::write::WriteAction::Compacted {
                    bytes: self.take_chosen(),
                    before: self.before,
                },
            ));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        // The choices scroll and the button row does not: an opened fold must
        // never push Save below the window's edge.
        let footer = BUTTON_ROW + if self.lossy() { LOSSY_LINE } else { 0.0 };
        let room = (ui.available_height() - footer).max(BUTTON_ROW);
        egui::ScrollArea::vertical()
            .max_height(room)
            .auto_shrink([false, true])
            .show(ui, |ui| self.choices_column(ui));
        self.button_row(ui);
    }

    /// Everything above the button row.
    fn choices_column(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);
        let after = chosen(&self.bytes, &self.recompress, &self.lowering).len() as u64;
        ui.label(t::size_change(self.before, after));
        ui.add_space(4.0);
        self.recompress_row(ui);
        ui.add_space(8.0);
        ui.small(t::revisions_line());
        // The signature warning LAST of the three and drawn full-size, not
        // `small`, when it applies. It is the only irreversible loss in the
        // window and the only one most documents do not have — so it is
        // conditional, and where it appears it is the thing the eye lands on.
        if self.signatures > 0 {
            ui.add_space(8.0);
            ui.label(t::signature_line(self.signatures));
        }

        ui.add_space(8.0);
        self.space_fold(ui);
    }

    /// Whether the bytes Save would write lost picture detail.
    fn lossy(&self) -> bool {
        matches!(
            &self.recompress,
            Recompress::Done(done) if done.bytes.is_some() && done.pictures.lossy
        )
    }

    /// The loss line, the separator and Save / Cancel, always in view. The
    /// loss line sits here, full-size, because it is the one change a reader
    /// of the copy can see and it must not scroll away from the button.
    fn button_row(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        if self.lossy() {
            let line = ui.label(crate::text::compact_images::lossy_line());
            crate::diag::ui_rect_visible(compact_images::REGION_LOSSY, line.rect, ui.clip_rect());
        }
        ui.separator();
        ui.horizontal(|ui| {
            // Never greyed. Every state this window can be in is one an
            // operator may legitimately proceed from — including "no smaller",
            // which is an accurate answer about a tidy file and not a reason to
            // refuse them a copy they asked for.
            let working = matches!(self.recompress, Recompress::Working(_))
                || matches!(self.lowering, Lowering::Working(_));
            let save = ui
                .add_enabled(!working, egui::Button::new(t::save_button()))
                .on_disabled_hover_text(t::save_waiting());
            crate::diag::ui_rect_visible(REGION_SAVE, save.rect, ui.clip_rect());
            if save.clicked() {
                self.save_requested = true;
            }
            let cancel = ui.button(t::cancel_button());
            crate::diag::ui_rect_visible(REGION_CANCEL, cancel.rect, ui.clip_rect());
            if cancel.clicked() {
                self.close_requested = true;
            }
        });
    }
}

impl CompactDialog {
    /// The fold listing where the bytes go. Closed by default: auditing a
    /// dense file twice is not free, so it is paid only when asked for.
    fn space_fold(&mut self, ui: &mut Ui) {
        let fold = egui::CollapsingHeader::new(t::space_heading())
            .id_salt("compact-space") // ui-text-exempt: an id, never displayed
            .show(ui, |ui| {
                let (path, bytes) = (
                    &self.path,
                    chosen(&self.bytes, &self.recompress, &self.lowering),
                );
                let space = self.space.get_or_insert_with(|| {
                    let started = std::time::Instant::now();
                    let now = std::fs::read(path).ok();
                    let measured = breakdown(now.as_deref(), bytes);
                    crate::diag::trace(|| {
                        format!(
                            "compact-space rows={} ms={}", // ui-text-exempt: diagnostic trace
                            measured.as_ref().map_or(0, |b| b.rows.len()),
                            started.elapsed().as_millis()
                        )
                    });
                    measured
                });
                if let Some(space) = space {
                    space_table(ui, space);
                }
            });
        fold.header_response.on_hover_text(t::space_tooltip());
    }
}

/// Where recompressing the copy has got to.
enum Recompress {
    Off,
    Working(Job),
    Done(Box<Recompressed>),
    Failed(String),
}

/// The bytes Save writes: the lowered copy when lowering succeeded, else the
/// recompressed copy when it came out smaller, else the plain compacted one.
fn chosen<'a>(plain: &'a [u8], recompress: &'a Recompress, lowering: &'a Lowering) -> &'a [u8] {
    if let Some(lowered) = lowering.bytes() {
        return lowered;
    }
    match recompress {
        Recompress::Done(done) => done.bytes.as_deref().unwrap_or(plain),
        _ => plain,
    }
}

impl CompactDialog {
    /// Start the recompression when it is wanted, and take its result.
    fn recompress_step(&mut self, ctx: &egui::Context) {
        let profile = self.profile.settings();
        if (self.choices.any() || profile.is_some()) && matches!(self.recompress, Recompress::Off) {
            let job = Job::start(
                ctx,
                self.bytes.clone(),
                self.settings.clone(),
                self.choices,
                profile.cloned(),
            );
            self.recompress = Recompress::Working(job);
        }
        let landed = match &self.recompress {
            Recompress::Working(job) => job.poll(),
            _ => None,
        };
        if let Some(result) = landed {
            self.recompress = match result {
                Ok(done) => Recompress::Done(Box::new(done)),
                Err(detail) => Recompress::Failed(detail),
            };
            self.space = None;
        }
        self.lower_step(ctx);
    }

    /// Lower the copy once the work before it has finished, when a version
    /// is chosen, and take the result.
    fn lower_step(&mut self, ctx: &egui::Context) {
        if let Some(target) = self.version
            && matches!(self.lowering, Lowering::Off)
            && !matches!(self.recompress, Recompress::Working(_))
        {
            use crate::app::settings::SettingsExt;
            let copy = chosen(&self.bytes, &self.recompress, &Lowering::Off).to_vec();
            self.lowering = Lowering::start(ctx, copy, target, self.settings.save_options());
        }
        let working = matches!(self.lowering, Lowering::Working(_));
        self.lowering.step();
        if working && matches!(self.lowering, Lowering::Done(_)) {
            self.space = None;
        }
    }

    /// The switches, and what the work on the copy did. Any change starts
    /// the work again from the plain copy; the running job is dropped.
    fn recompress_row(&mut self, ui: &mut Ui) {
        let choices = self.choices;
        let mut toggled = self
            .profile
            .row(ui, || crate::dialogs::recompress::settings_for(choices));
        toggled |= if self.profile.settings().is_some() {
            self.fonts_switch(ui)
        } else {
            self.switches(ui)
        };
        if toggled {
            self.recompress = Recompress::Off;
            self.lowering = Lowering::Off;
            self.space = None;
        }
        if compact_version::row(ui, &mut self.version) {
            self.lowering = Lowering::Off;
            self.space = None;
        }
        match &self.recompress {
            Recompress::Off => {}
            Recompress::Working(_) => {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Spinner::new().color(egui_shell::Theme::of(ui.ctx()).palette.text),
                    );
                    ui.small(t::recompress_working());
                });
            }
            Recompress::Failed(detail) => {
                ui.small(t::recompress_failed(detail));
            }
            Recompress::Done(done) => done_lines(ui, done),
        }
        compact_version::lines(ui, &self.lowering);
    }

    /// The starting point, size target and switches; whether one changed.
    fn switches(&mut self, ui: &mut Ui) -> bool {
        let c = &mut self.choices;
        let plain = self.bytes.len() as u64;
        let aimed = compact_target::rows(ui, &mut c.preset, &mut c.budget, plain);
        let toggled = [
            (
                &mut c.recompress,
                t::recompress_toggle(),
                t::recompress_tooltip(),
            ),
            (&mut c.broken, t::broken_toggle(), t::broken_tooltip()),
            (
                &mut c.destinations,
                t::destinations_toggle(),
                t::destinations_tooltip(),
            ),
        ]
        .into_iter()
        .fold(false, |any, (on, label, tip)| {
            ui.checkbox(on, label).on_hover_text(tip).changed() || any
        });
        let toggled = self.fonts_switch(ui) || toggled;
        let c = &mut self.choices;
        let content = ui
            .checkbox(&mut c.content, t::content_toggle())
            .on_hover_text(t::content_tooltip());
        crate::diag::ui_rect_visible(REGION_CONTENT, content.rect, ui.clip_rect());
        let toggled = content.changed() || toggled;
        let toggled = round_row(ui, c.content, &mut c.round) || toggled;
        let web = ui
            .checkbox(&mut c.linearize, t::linearize_toggle())
            .on_hover_text(t::linearize_tooltip());
        crate::diag::ui_rect_visible(REGION_LINEARIZE, web.rect, ui.clip_rect());
        let toggled = web.changed() || toggled;
        let toggled = discard_switches(ui, &mut c.discard) || toggled;
        compact_images::switches(ui, &mut c.pictures, &mut c.classes) || toggled || aimed
    }

    /// The fonts switch, which a settings file cannot carry.
    fn fonts_switch(&mut self, ui: &mut Ui) -> bool {
        let fonts = ui
            .checkbox(&mut self.choices.fonts, t::fonts_toggle())
            .on_hover_text(t::fonts_tooltip());
        crate::diag::ui_rect_visible(REGION_FONTS, fonts.rect, ui.clip_rect());
        fonts.changed()
    }

    /// The bytes [`chosen`] names, moved out for the write.
    fn take_chosen(&mut self) -> Vec<u8> {
        if let Some(lowered) = self.lowering.take_bytes() {
            return lowered;
        }
        match std::mem::replace(&mut self.recompress, Recompress::Off) {
            Recompress::Done(done) => match done.bytes {
                Some(bytes) => bytes,
                None => std::mem::take(&mut self.bytes),
            },
            _ => std::mem::take(&mut self.bytes),
        }
    }
}

/// One switch per class of document object the engine can discard and this
/// window offers, under a heading that starts closed and counts the switches
/// that are on, so a removal is never hidden; whether any changed.
fn discard_switches(ui: &mut Ui, discard: &mut DiscardOptions) -> bool {
    let on = DiscardObject::ALL
        .into_iter()
        .filter(|&c| t::discard_toggle(c).is_some() && discard.contains_object(c))
        .count();
    egui::CollapsingHeader::new(t::discard_heading(on))
        .id_salt("compact-discards")
        .show(ui, |ui| discard_list(ui, discard))
        .body_returned
        .unwrap_or(false)
}

/// The switches themselves; whether any changed.
fn discard_list(ui: &mut Ui, discard: &mut DiscardOptions) -> bool {
    let mut changed = false;
    for class in DiscardObject::ALL {
        let Some((label, tip)) = t::discard_toggle(class) else {
            continue;
        };
        let mut on = discard.contains_object(class);
        if ui.checkbox(&mut on, label).on_hover_text(tip).changed() {
            *discard = if on {
                discard.with_object(class)
            } else {
                discard.without_object(class)
            };
            changed = true;
        }
    }
    changed
}

fn space_table(ui: &mut Ui, space: &Breakdown) {
    egui::ScrollArea::vertical()
        .max_height(220.0)
        .show(ui, |ui| {
            egui::Grid::new("compact-space-grid") // ui-text-exempt: an id, never displayed
                .striped(true)
                .show(ui, |ui| {
                    ui.label(t::space_column_what());
                    ui.label(t::space_column_now());
                    ui.label(t::space_column_copy());
                    ui.end_row();
                    for &(category, now, copy) in &space.rows {
                        ui.label(t::space_category(category));
                        ui.label(now.map_or_else(|| t::space_no_file().to_owned(), t::space_bytes));
                        ui.label(t::space_bytes(copy));
                        ui.end_row();
                    }
                });
            for note in &space.notes {
                ui.small(note.as_str());
            }
        });
}
/// Where the bytes of the file and of the copy go, by category.
pub struct Breakdown {
    /// `(category, now, copy)`, largest first by the larger of the two;
    /// categories empty in both are left out. `now` is `None` for a document
    /// with no file.
    rows: Vec<(SpaceCategory, Option<u64>, u64)>,
    /// What either audit could not measure, in the engine's sentences.
    notes: Vec<String>,
}

/// Audit `now` (the file on disk, when there is one) and `copy`.
///
/// `None` when the copy does not parse back, which would be an engine defect:
/// the window then lists nothing rather than a breakdown of half the trade.
pub fn breakdown(now: Option<&[u8]>, copy: &[u8]) -> Option<Breakdown> {
    use pdfcer_core::document::Document;
    use pdfcer_core::space_audit::audit;
    let copy = audit(&Document::from_bytes(copy.to_vec()).ok()?);
    let now = now
        .and_then(|b| Document::from_bytes(b.to_vec()).ok())
        .map(|d| audit(&d));
    let mut rows: Vec<_> = copy
        .categories
        .iter()
        .map(|c| {
            (
                c.category,
                now.as_ref().map(|n| n.bytes_in(c.category)),
                c.bytes,
            )
        })
        .filter(|&(_, now, copy)| copy > 0 || now.is_some_and(|n| n > 0))
        .collect();
    rows.sort_by_key(|&(_, now, copy)| std::cmp::Reverse(now.unwrap_or(0).max(copy)));
    let mut notes = now.map(|n| n.notes).unwrap_or_default();
    notes.extend(copy.notes);
    notes.dedup();
    Some(Breakdown { rows, notes })
}

/// Build it for the current document, or answer why not.
pub fn open_for(
    status: &Status,
    start: compact_target::Start,
) -> Option<Result<CompactDialog, String>> {
    match status {
        Status::Open(doc) => Some(CompactDialog::open(doc, start)),
        _ => None,
    }
}

/// The rounding list, greyed while tidying is off; whether it changed.
fn round_row(
    ui: &mut Ui,
    content: bool,
    round: &mut Option<pdfcer_core::optimize::Tolerance>,
) -> bool {
    let before = *round;
    let choices = std::iter::once(None).chain(
        ROUNDINGS
            .iter()
            .map(|&pt| pdfcer_core::optimize::Tolerance::from_units(pt)),
    );
    ui.horizontal(|ui| {
        ui.add_space(ui.spacing().indent);
        ui.label(t::round_label()).on_hover_text(t::round_tooltip());
        ui.add_enabled_ui(content, |ui| {
            let list = egui::ComboBox::from_id_salt("compact-content-round") // ui-text-exempt: an id, never displayed
                .selected_text(t::round_choice(*round))
                .show_ui(ui, |ui| {
                    for (i, choice) in choices.enumerate() {
                        let item = ui.selectable_value(round, choice, t::round_choice(choice));
                        let region = format!("{REGION_ROUND}.{i}");
                        crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                    }
                });
            let shown = list
                .response
                .on_disabled_hover_text(t::round_needs_content());
            crate::diag::ui_rect_visible(REGION_ROUND, shown.rect, ui.clip_rect());
        });
    });
    *round != before
}

/// What a finished recompression did, one small line each.
fn done_lines(ui: &mut Ui, done: &Recompressed) {
    if let Some(budget) = &done.budget {
        compact_target::done_lines(ui, budget);
    }
    // What a discarded result did is not in the copy, so only
    // the engine's notes are said then.
    if done.bytes.is_none() {
        ui.small(t::recompress_no_gain());
    } else {
        if done.not_smaller {
            ui.small(t::kept_not_smaller(done.linearized));
        }
        if done.changed > 0 {
            ui.small(t::recompressed(done.changed));
        }
        for line in done.pictures.lines() {
            ui.small(line);
        }
        crate::dialogs::compact_compare::show(ui, &done.compared);
        for line in done.cleaned.iter().chain(&done.restructured) {
            ui.small(line.as_str());
        }
        for line in &done.fonts {
            let said = ui.small(line.as_str());
            crate::diag::ui_rect_visible(REGION_FONTS_SAID, said.rect, ui.clip_rect());
        }
        if let Some(line) = &done.content {
            let said = ui.small(line.as_str());
            crate::diag::ui_rect_visible(REGION_CONTENT_SAID, said.rect, ui.clip_rect());
        }
        for line in &done.unrounded {
            ui.small(line.as_str());
        }
        for (reason, count) in &done.left {
            ui.small(t::left_line(reason, *count));
        }
    }
    for note in &done.notes {
        ui.small(note.as_str());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `hello.pdf` with an incremental update that replaces its content
    /// stream, so the replaced body is still in the file.
    fn updated() -> (Vec<u8>, Vec<u8>) {
        let path = crate::panels::objects::test_support::engine_fixture("hello.pdf");
        let clean = std::fs::read(path).expect("fixture");
        let mut file = clean.clone();
        let body = file.len() + 1;
        file.extend_from_slice(b"\n5 0 obj\n<< /Length 0 >>\nstream\n\nendstream\nendobj\n");
        let xref = file.len();
        file.extend_from_slice(
            format!(
                "xref\n0 1\n0000000000 65535 f\r\n5 1\n{body:010} 00000 n\r\n\
                 trailer\n<< /Size 6 /Root 1 0 R /Prev 526 >>\nstartxref\n{xref}\n%%EOF\n"
            )
            .as_bytes(),
        );
        (file, clean)
    }

    /// Each column adds up to its file, the replaced body shows as space the
    /// copy drops, and the rows run largest first with no empty ones.
    #[test]
    fn the_breakdown_accounts_for_every_byte_of_both_files() {
        let (now, copy) = updated();
        let space = breakdown(Some(&now), &copy).expect("both parse");
        let now_sum: u64 = space.rows.iter().filter_map(|r| r.1).sum();
        let copy_sum: u64 = space.rows.iter().map(|r| r.2).sum();
        assert_eq!(now_sum, now.len() as u64);
        assert_eq!(copy_sum, copy.len() as u64);
        let superseded = space.rows.iter().find(|r| r.0 == SpaceCategory::Superseded);
        assert!(
            matches!(superseded, Some(&(_, Some(n), 0)) if n > 0),
            "{:?}",
            space.rows
        );
        let sizes: Vec<u64> = space
            .rows
            .iter()
            .map(|r| r.1.unwrap_or(0).max(r.2))
            .collect();
        assert!(sizes.windows(2).all(|w| w[0] >= w[1]), "{sizes:?}");
        assert!(sizes.iter().all(|&n| n > 0));
    }

    /// A document with no file yet is broken down for the copy alone.
    #[test]
    fn an_unsaved_document_has_no_now_column() {
        let (_, copy) = updated();
        let space = breakdown(None, &copy).expect("parses");
        assert!(space.rows.iter().all(|r| r.1.is_none()));
        assert_eq!(
            space.rows.iter().map(|r| r.2).sum::<u64>(),
            copy.len() as u64
        );
    }
}
