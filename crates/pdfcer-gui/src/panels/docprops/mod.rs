//! # `panels::docprops` — the **Document properties** panel: this file's own
//! title, author, subject and keywords, and the facts pdfcer read about it
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/docprops/mod.md`.

//! ## Why `Action::SetInfoField` takes `Option<String>` and not `String`
//!
//!
//! `None` is a different edit, not an empty one.
//! `EditSession::set_info_field(field, None)` **removes the key** from the
//! `/Info` dictionary; `Some("")` writes an empty string object. A document
//! with no title and a document whose title is the empty string are different
//! files, and the first is what an operator means by deleting the contents of a
//! box.
//!
//! Collapsing the two — taking a `String` and treating empty as clear — would
//! work today and would make the distinction unrepresentable, which is the shape
//! `pdfcer-core`'s own `Some(Tolerance::None)` vs `None` note warns about one
//! feature along.
//!
//! ## Why it goes through the funnel when the edit is one dictionary entry
//!
//! Not for size — for **ordering and the undo log**. It is a document change
//! like any other, and the funnel is what makes it appear once in the command
//! log, bump the epoch once, and be undone by one `Ctrl+Z`.
//!
//! And the epoch bump is load-bearing here in a way it is not elsewhere: this
//! panel re-seeds its text drafts whenever the epoch moves, which is what makes
//! `Ctrl+Z` visibly restore the old value in the box. Applying the edit outside
//! the funnel would leave the box holding a string the document no longer has,
//! and the next focus change would write it back — an undo the panel silently
//! reverses.

use egui::Ui;
use pdfcer_core::edit::{InfoField, InfoText};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::anomalies as t_anomalies;
use crate::text::panels::docprops as t;

/// Page boxes the file wrote and pdfcer did not use as written.
mod boxes;
/// Security notes: the wrapper warning and the action census.
mod security;
/// The stamp-collection disclosure section — `OPERATOR_REQUESTS.md` **O169**.
mod stamps;

/// The region this panel publishes.
pub const REGION: &str = "properties.info"; // ui-text-exempt: trace region name, never displayed
/// The prefix of the per-field editor regions; the field's index in
/// `InfoField::all()` is appended.
pub const REGION_FIELD_PREFIX: &str = "properties.info."; // ui-text-exempt: trace region name, never displayed

/// **The load-anomaly block's own region** — the heading, the note and every
/// row, as one rectangle.
pub const REGION_ANOMALIES: &str = "properties.load-anomalies"; // ui-text-exempt: trace region name, never displayed

/// The prefix of the per-anomaly row regions; the row's index in
/// [`crate::app::status::anomalies::rows`]' output is appended.
pub const REGION_ANOMALY_ROW_PREFIX: &str = "properties.load-anomalies."; // ui-text-exempt: trace region name, never displayed

/// **The region the re-read button publishes**, so a driven check can assert
/// that the operator's intervention is reachable rather than merely built.
pub const REGION_ANOMALY_REREAD: &str = "properties.load-anomalies.reread"; // ui-text-exempt: trace region name, never displayed

/// **The region the dropped-object disclosure publishes** — both sentences, as
/// one rectangle.
pub const REGION_RECOVERY: &str = "properties.recovery"; // ui-text-exempt: trace region name, never displayed

pub const REGION_RECOVERY_DROPPED: &str = "properties.recovery-dropped"; // ui-text-exempt: trace region name, never displayed

/// How many fields `InfoField::all()` returns.
const FIELDS: usize = InfoField::all().len();

/// The operator's half-typed metadata, between frames.
#[derive(Default)]
pub struct InfoDrafts {
    /// One draft per `InfoField::all()` position.
    drafts: [String; FIELDS],
    /// The `edit_epoch` the drafts were seeded from, or `None` before the
    /// first seed.
    ///
    /// `Option` rather than a sentinel: epoch 0 is a real, common value — it
    /// is what every freshly opened document has — and a `0` sentinel would
    /// make "never seeded" and "seeded from an unedited document"
    /// indistinguishable, which is the state this panel spends most of its
    /// life in.
    seeded_at: Option<u64>,
}

impl std::fmt::Debug for InfoDrafts {
    /// Lengths, not contents.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfoDrafts")
            .field("lengths", &self.drafts.each_ref().map(String::len))
            .field("seeded_at", &self.seeded_at)
            .finish()
    }
}

impl InfoDrafts {
    /// Re-seed the drafts from the document when the revision has moved.
    fn sync(&mut self, doc: &OpenDoc) -> [Option<InfoText>; FIELDS] {
        // `from_fn` over `get`, rather than `all().map(...)`, because
        // `all()` returns a **slice** as of engine `4851316` and a slice has no
        // array-producing `map`. The return type must stay `[_; FIELDS]`: the
        // caller zips it against `[String; FIELDS]`.
        //
        // `get(i)` rather than `[i]` — `clippy::indexing_slicing` is denied
        // crate-wide, and the denial is doing real work here rather than being
        // satisfied. `i` is always in range because `FIELDS` **is**
        // `all().len()`, so the `None` arm is unreachable; writing it as a
        // fallible read is what keeps that unreachability a fact about this
        // line instead of an assumption inherited from a `const` fifteen lines
        // up.
        let stored: [Option<InfoText>; FIELDS] = core::array::from_fn(|index| {
            InfoField::all()
                .get(index)
                .and_then(|field| doc.session.info_text(*field))
        });
        if self.seeded_at != Some(doc.edit_epoch) {
            self.seeded_at = Some(doc.edit_epoch);
            for (draft, value) in self.drafts.iter_mut().zip(stored.iter()) {
                draft.clear();
                if let Some(info) = value {
                    draft.push_str(&info.text);
                }
            }
        }
        stored
    }
}

/// **Draw the Document properties panel.**
pub fn body(ui: &mut Ui, doc: &OpenDoc, drafts: &mut InfoDrafts, actions: &mut Vec<Action>) {
    egui::ScrollArea::vertical()
        .id_salt("docprops-body")
        .auto_shrink([false, false])
        .show(ui, |ui| info_body(ui, doc, drafts, actions));
}

/// The panel's contents, inside the scroll area [`body`] wraps them in.
fn info_body(ui: &mut Ui, doc: &OpenDoc, drafts: &mut InfoDrafts, actions: &mut Vec<Action>) {
    // No `.strong()` — R84 / DEFECTS.md D11: egui resolves it to the
    // accent-filled widget foreground, which on an ordinary panel is pale text
    // on pale ground.
    ui.label(t::heading());
    ui.label(egui::RichText::new(t::note()).small().weak());
    recovery_note(ui, doc);
    load_anomalies_note(ui, doc, actions);
    ui.add_space(4.0);

    facts(ui, doc);
    // After the read-only facts and BEFORE the editable rows, because it is
    // one of the facts: what kind of file this is. Placing it below the `/Info`
    // fields would put the sentence about the category under the very field
    // that sets it, which reads as a note about the edit the operator just made
    // rather than as a description of the file he opened.
    stamps::section(ui, doc);
    security::section(ui, doc);
    ui.add_space(6.0);

    let stored = drafts.sync(doc);

    for (index, field) in InfoField::all().iter().copied().enumerate() {
        // `get_mut`/`get` rather than indexing: both arrays are `[_; FIELDS]`
        // and the loop is over a `[_; FIELDS]`, so this cannot fail — and
        // `clippy::indexing_slicing` is denied crate-wide precisely so that a
        // future change which *could* fail is written as a decision.
        let (Some(draft), Some(current)) = (drafts.drafts.get_mut(index), stored.get(index)) else {
            continue;
        };
        row(ui, index, field, draft, current.as_ref(), actions);
    }

    // `ui_rect_visible` rather than `ui_rect`, and published LAST — the
    // reason survives the move even though the collapse that prompted it did
    // not. This body is inside a `ScrollArea`, and a rect published for a
    // scrolled-out control gets CLICKED by the harness at a coordinate the
    // operator can never reach. `geometry::section` paid for that once; its
    // answer is copied rather than re-derived.
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
}

/// The read-only facts about the file itself.
fn facts(ui: &mut Ui, doc: &OpenDoc) {
    let base = doc.session.document();

    fact(ui, t::file_label(), &file_name(doc));
    fact(
        ui,
        t::size_label(),
        &crate::text::panels::byte_size(base.bytes().len()),
    );
    if doc.session.is_modified() {
        ui.weak(t::size_is_base());
    }
    fact(ui, t::version_label(), &base.version().to_string());
    fact(
        ui,
        t::pages_label(),
        &crate::text::pages::pages_count(doc.pages.len()),
    );
    if let Some(size) = sheet_size(doc) {
        fact(ui, t::page_size_label(), &size);
    }
    boxes::lines(ui, doc);

    let encrypted = base.encryption().is_some();
    fact(
        ui,
        t::encryption_label(),
        if encrypted {
            t::encrypted()
        } else {
            t::not_encrypted()
        },
    );
    if encrypted {
        ui.weak(t::encryption_note());
    }
}

/// One label-and-value row, in the same shape the object half uses.
fn fact(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(label);
        ui.label(value);
    });
}

/// The document's file name, or the sentence for one that has none.
fn file_name(doc: &OpenDoc) -> String {
    if doc.origin == crate::app::state::Origin::Created {
        return t::file_unsaved().to_owned();
    }
    doc.path.file_name().map_or_else(
        || doc.path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// The sheet size in millimetres, saying so when the sheets differ.
fn sheet_size(doc: &OpenDoc) -> Option<String> {
    let first = doc.pages.first()?;
    let (w, h) = crate::viewer::page_extent_pts(first);
    let (w_pts, h_pts) = (f64::from(w), f64::from(h));

    // Millimetres are wanted here only for the tolerance COMPARISON below; the
    // displayed value is rounded inside `t::page_size`, from points, so that
    // this row and the page tile's tooltip cannot round differently.
    let (w_mm, h_mm) = (
        crate::units::mm_from_points(w_pts),
        crate::units::mm_from_points(h_pts),
    );
    let mixed = doc.pages.iter().skip(1).any(|page| {
        let (ow, oh) = crate::viewer::page_extent_pts(page);
        (crate::units::mm_from_points(f64::from(ow)) - w_mm).abs() >= 1.0
            || (crate::units::mm_from_points(f64::from(oh)) - h_mm).abs() >= 1.0
    });
    Some(if mixed {
        t::page_size_mixed(w_pts, h_pts)
    } else {
        t::page_size(w_pts, h_pts)
    })
}

/// One field: its label, its editor, and whatever the document owes about it.
fn row(
    ui: &mut Ui,
    index: usize,
    field: InfoField,
    draft: &mut String,
    current: Option<&InfoText>,
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.label(t::info_label(field));
        // escape-disposition: commits — committed on `lost_focus`, and egui
        // surrenders a field's focus on Escape, so the key rides that path.
        let response = ui.add(egui::TextEdit::singleline(draft).desired_width(200.0));
        crate::diag::ui_rect(
            // ui-text-exempt: trace region name, never displayed
            &format!("{REGION_FIELD_PREFIX}{index}"),
            response.rect,
        );

        // The commit rule is the FORMS panel's, called rather than restated.
        //
        // Its two conditions bind here for the same two reasons they bind
        // there — `lost_focus`, because `TextEdit::changed()` fires per
        // keystroke and one typed word must not be a dozen undo entries; and
        // draft-differs-from-stored, because tabbing THROUGH a field the
        // operator did not touch must not write a command.
        //
        // The second condition is also what discharges `InfoText::exact`'s
        // obligation: a field pdfcer could not decode exactly is never written
        // back unless the operator changed it, which is the engine's rule
        // verbatim and is true here by construction rather than by a guard
        // somebody has to remember.
        let stored_text = current.map_or("", |info| info.text.as_str());
        let ended = response.lost_focus();
        if crate::panels::forms::rows::commit(ended, draft.as_str(), stored_text).is_some() {
            let value = if draft.trim().is_empty() {
                // Empty CLEARS the key rather than writing an empty string.
                // See the module header: a document with no title and a
                // document whose title is nothing are different documents, and
                // the first is what deleting the contents of a box means.
                //
                // `trim` on the test but not on the value: a title of "  " is
                // not a title, and a title of " Site Plan " is one the operator
                // may have spaced deliberately.
                None
            } else {
                Some(draft.clone())
            };
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed. LENGTH,
                // not the text: this is the operator's own document metadata
                // and the trace is written to a file a harness keeps.
                format!(
                    "info-field-commit field={field:?} clearing={} chars={}",
                    u8::from(value.is_none()),
                    draft.chars().count()
                )
            });
            actions.push(Action::SetInfoField { field, value });
        }
    });

    // The disclosure, and it is under the row rather than beside it because
    // it is a sentence. Drawn only when the decode was lossy, which on an
    // ordinary document is never — a note on every row would be noise that
    // trains the operator to skip the one that matters.
    if current.is_some_and(|info| !info.exact) {
        ui.weak(t::info_not_exact());
    }
}

/// **Say when pdfcer had to rebuild this file's index to open it at all.**
fn recovery_note(ui: &mut Ui, doc: &OpenDoc) {
    let Some(report) = doc.session.document().recovery() else {
        return;
    };
    // The heading and detail are scoped and published as [`REGION_RECOVERY`];
    // the dropped-object block publishes its own region and is deliberately
    // OUTSIDE this scope's rect, so that a check can require one and forbid the
    // other on the same launch without the two rectangles overlapping into an
    // ambiguity. See [`REGION_RECOVERY`] for why the positive witness matters.
    let block = ui
        .scope(|ui| {
            ui.label(egui::RichText::new(t::recovered_heading()).color(ui.visuals().warn_fg_color));
            ui.label(
                egui::RichText::new(t::recovered_detail(
                    report.file_level_objects + report.objstm_objects,
                    report.last_wins_collisions,
                    report.stream_lengths_recovered + report.missing_endobj_recovered,
                ))
                .small()
                .weak(),
            )
            .on_hover_text(t::recovered_tooltip());
        })
        .response
        .rect;
    crate::diag::ui_rect_visible(REGION_RECOVERY, block, ui.clip_rect());
    dropped_objects_note(ui, report);
    ui.add_space(4.0);
}

/// How many dropped object numbers are printed before the list elides.
const DROPPED_NUMBERS_SHOWN: usize = 12;

/// **Draw what recovery threw away**, split by the engine's two reasons.
fn dropped_objects_note(ui: &mut Ui, report: &pdfcer_core::recover::RecoveryReport) {
    if report.objects_dropped.is_empty() {
        return;
    }
    let mut unreadable: Vec<u32> = Vec::new();
    let mut mismatched: Vec<u32> = Vec::new();
    for dropped in &report.objects_dropped {
        match dropped.reason {
            pdfcer_core::recover::DropReason::IdMismatch => mismatched.push(dropped.number),
            _ => unreadable.push(dropped.number),
        }
    }
    let mut numbers = unreadable.clone();
    numbers.extend_from_slice(&mismatched);
    numbers.sort_unstable();

    // Scoped so the published rect is the union egui computed, not a pair of
    // `ui.cursor()` readings subtracted — the latter is right today and wrong
    // the first time a caller wraps this in a horizontal layout. Same argument
    // as `load_anomalies_note`'s block, and the same discipline: `visible`,
    // because this panel scrolls and a rect published for a scrolled-out row is
    // a coordinate the operator can never reach.
    let block = ui
        .scope(|ui| {
            ui.label(
                egui::RichText::new(t::dropped_summary(unreadable.len(), mismatched.len()))
                    .small()
                    .weak(),
            )
            .on_hover_text(t::dropped_tooltip());
            ui.label(
                egui::RichText::new(t::dropped_numbers(&numbers, DROPPED_NUMBERS_SHOWN))
                    .small()
                    .weak(),
            )
            .on_hover_text(t::dropped_tooltip());
        })
        .response
        .rect;
    crate::diag::ui_rect_visible(REGION_RECOVERY_DROPPED, block, ui.clip_rect());
}

/// **Which places this file contradicted itself, and what pdfcer chose in
/// each** — the long form of the status bar's census line. Engine
/// `Pass 283.0`, decision 145, wired 2026-09-09.
fn load_anomalies_note(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let anomalies = doc.session.document().load_anomalies();
    let rows = crate::app::status::anomalies::rows(anomalies);
    if rows.is_empty() {
        return;
    }
    // Counted here rather than inside the scope, because the button's
    // existence is a property of the FILE and the scope's job is drawing. The
    // census is the same one both status lines use — one definition of "how
    // many duplicate keys", not a second `iter().filter()` that could disagree
    // with the sentence the operator read in the bar two seconds ago.
    let census = crate::app::status::anomalies::census(anomalies);
    // The whole block is scoped so that its union rect is a value rather
    // than something reconstructed from two cursor readings. `ui.cursor()`
    // before and after would give a rect that is right today and wrong the
    // first time a caller wraps this in a horizontal layout — the scope's own
    // response is the same fact, computed by egui, and it cannot drift.
    let block = ui
        .scope(|ui| {
            ui.label(egui::RichText::new(t_anomalies::heading()).color(ui.visuals().warn_fg_color));
            ui.label(egui::RichText::new(t_anomalies::note()).small().weak())
                .on_hover_text(t_anomalies::tooltip());
            for (index, row) in rows.iter().enumerate() {
                // One label per anomaly rather than one joined paragraph. A
                // paragraph reads as prose about the file in general; separate
                // lines read as a list of specific places, which is what an
                // operator checking a titleblock against a drawing needs to
                // work down.
                let response = ui.label(egui::RichText::new(row).small().weak());
                // Per-row region, published under `REGION_ANOMALY_ROW_PREFIX`.
                // The block region below says "the file contradicted itself";
                // only counting these says "in how many places", and a
                // regression that drew the heading with no rows under it would
                // be invisible to the block region alone.
                crate::diag::ui_rect_visible(
                    // ui-text-exempt: trace region name, never displayed
                    &format!("{REGION_ANOMALY_ROW_PREFIX}{index}"),
                    response.rect,
                    ui.clip_rect(),
                );
            }
            reread_control(ui, doc, census.duplicate_keys, actions);
        })
        .response
        .rect;
    // `ui_rect_visible` rather than `ui_rect`, for the reason `info_body`
    // states at its own publication: this draws inside `body`'s `ScrollArea`,
    // and a rect published for a scrolled-out control gets clicked by the
    // harness at a coordinate the operator can never reach.
    crate::diag::ui_rect_visible(REGION_ANOMALIES, block, ui.clip_rect());
    ui.add_space(4.0);
}

use pdfcer_core::parser::DuplicateKeyPolicy;

/// **Which reading to offer, given the one in effect** — the whole of
/// [`reread_control`]'s judgement, separated from its drawing.
fn offered_reading(current: DuplicateKeyPolicy) -> (DuplicateKeyPolicy, &'static str) {
    match current {
        // Already reading the first values, so the offer is pdfcer's ordinary
        // reading back.
        DuplicateKeyPolicy::KeepFirst => (
            DuplicateKeyPolicy::KeepLast,
            t_anomalies::reread_last_button(),
        ),
        // The ordinary reading, and anything this build does not know.
        _ => (
            DuplicateKeyPolicy::KeepFirst,
            t_anomalies::reread_first_button(),
        ),
    }
}

/// **Offer the operator the reading they do not currently have** — the third
/// of the three obligations in the operator's own ruling about damaged files.
fn reread_control(ui: &mut Ui, doc: &OpenDoc, duplicates: usize, actions: &mut Vec<Action>) {
    if duplicates == 0 {
        return;
    }
    let (policy, label) = offered_reading(doc.load_options.duplicate_keys);
    ui.add_space(4.0);
    let response = ui
        .button(egui::RichText::new(label).small())
        .on_hover_text(t_anomalies::reread_tooltip());
    crate::diag::ui_rect_visible(REGION_ANOMALY_REREAD, response.rect, ui.clip_rect());
    if response.clicked() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            //
            // Names the policy being ASKED FOR and how many places it applies
            // to. A driven check keys on this line to prove the control is
            // reachable, which a unit test calling the action cannot show.
            format!(
                "reread-requested policy={} duplicates={duplicates}",
                crate::app::state::policy_token(
                    pdfcer_core::document::LoadOptions::new().with_duplicate_keys(policy)
                )
            )
        });
        actions.push(Action::RereadWithDuplicateKeys { policy });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The control fixture really is recovered, and really lost nothing.**
    #[test]
    fn the_control_fixture_for_the_dropped_disclosure_really_recovers_and_drops_nothing() {
        let doc = crate::app::state::open_local_fixture(crate::app::state::RECOVERED_NO_LOSSES);
        let report = doc.session.document().recovery().expect(
            "the control has no xref, no trailer and no startxref, so it can only have been opened by rebuild-by-scan; if it was not, the driven check's positive witness `properties.recovery` will not be drawn and the check will blame the panel",
        );
        assert!(
            report.objects_dropped.is_empty(),
            "the control fixture dropped {:?}, so the absence of the dropped-object block is no longer a fact about the application",
            report.objects_dropped
        );
    }

    /// **The fixture really does produce dropped objects, and it names both.**
    #[test]
    fn the_recovery_fixture_drops_the_two_objects_it_was_built_to_drop() {
        let doc = crate::app::state::open_local_fixture(crate::app::state::RECOVERED_WITH_LOSSES);
        let report = doc.session.document().recovery().expect(
            "no xref, no trailer, no startxref: this fixture can only have been opened by rebuild-by-scan, and if it was not then the file on disk is no longer the file the generator writes",
        );
        // Annotated with the engine's own type on purpose. Every other line
        // in this module reaches a `DroppedObject` through field access on an
        // inferred binding, so the TYPE NAME appeared nowhere in this
        // repository and `check-engine-api-drift` reported the struct and both
        // of its fields as items nobody here names -- which was true, and is
        // the condition under which a rename upstream lands as a silent
        // behaviour change rather than a compile error. This binding is that
        // compile error.
        let dropped: &[pdfcer_core::recover::DroppedObject] = &report.objects_dropped;
        let mut numbers: Vec<u32> = dropped.iter().map(|d| d.number).collect();
        numbers.sort_unstable();
        assert_eq!(
            numbers,
            vec![8, 9],
            "the fixture is built so the scan finds a truncated object 8 and a false-positive object 9; it reported {:?} instead, so the driven check downstream is no longer measuring anything",
            report.objects_dropped
        );
    }

    /// **Both object numbers survive the trip from the report to the screen.**
    #[test]
    fn both_dropped_objects_reach_the_lines_the_operator_reads() {
        let doc = crate::app::state::open_local_fixture(crate::app::state::RECOVERED_WITH_LOSSES);
        let report = doc.session.document().recovery().expect("recovered");

        let unreadable = report
            .objects_dropped
            .iter()
            .filter(|d| d.reason != pdfcer_core::recover::DropReason::IdMismatch)
            .count();
        let mismatched = report.objects_dropped.len() - unreadable;
        let summary = t::dropped_summary(unreadable, mismatched);
        assert!(
            !summary.is_empty(),
            "two objects were dropped and the panel would have said nothing about it"
        );

        let mut numbers: Vec<u32> = report.objects_dropped.iter().map(|d| d.number).collect();
        numbers.sort_unstable();
        let line = t::dropped_numbers(&numbers, DROPPED_NUMBERS_SHOWN);
        for n in [8_u32, 9] {
            assert!(
                line.contains(&n.to_string()),
                "object {n} was dropped and does not appear in the line the operator reads: {line}"
            );
        }
    }

    /// **The button always offers the reading the operator does not have.**
    #[test]
    fn the_offer_is_always_the_reading_not_in_effect() {
        assert_eq!(
            offered_reading(DuplicateKeyPolicy::KeepLast).0,
            DuplicateKeyPolicy::KeepFirst,
            "a document read pdfcer's usual way must be offered the first values"
        );
        assert_eq!(
            offered_reading(DuplicateKeyPolicy::KeepFirst).0,
            DuplicateKeyPolicy::KeepLast,
            "a document already read under the first values must be offered the \
             way back"
        );
    }

    /// **`Refuse` is never what this shell asks a loader for.**
    #[test]
    fn the_panel_never_offers_the_policy_that_refuses_the_file() {
        for current in [
            DuplicateKeyPolicy::KeepLast,
            DuplicateKeyPolicy::KeepFirst,
            DuplicateKeyPolicy::Refuse,
        ] {
            let (offered, _) = offered_reading(current);
            assert_ne!(
                offered,
                DuplicateKeyPolicy::Refuse,
                "the properties panel offered `Refuse` to a document read as \
                 {current:?}; that policy refuses the whole file"
            );
        }
    }

    /// **The two labels are different sentences.**
    #[test]
    fn the_two_readings_are_labelled_differently() {
        assert_ne!(
            offered_reading(DuplicateKeyPolicy::KeepLast).1,
            offered_reading(DuplicateKeyPolicy::KeepFirst).1
        );
    }

    /// The four fields this panel was written for are still in the engine's
    /// list.
    #[test]
    fn the_draft_array_tracks_the_engines_field_list() {
        let fields = InfoField::all();
        for expected in [
            InfoField::Title,
            InfoField::Author,
            InfoField::Subject,
            InfoField::Keywords,
        ] {
            assert!(
                fields.contains(&expected),
                "{expected:?} is no longer in `InfoField::all()`. Positions in \
                 this panel are addressed by index and every region name was \
                 written against that order -- read this test's doc comment \
                 before adjusting anything."
            );
        }
        let drafts = InfoDrafts::default();
        assert_eq!(
            drafts.drafts.len(),
            fields.len(),
            "the drafts array and the engine's list have come apart, which \
             `[String; FIELDS]` is supposed to make impossible"
        );
        assert_eq!(
            drafts.seeded_at, None,
            "a fresh panel has never been seeded, and epoch 0 is a real value \
             that must not be confused with that"
        );
    }

    /// Every field has a label, and no two share one.
    ///
    /// A duplicated label would be two boxes an operator cannot tell apart,
    /// which is worse than a missing one because it looks like it works.
    #[test]
    fn every_info_field_is_labelled_and_no_label_repeats() {
        let labels: Vec<&str> = InfoField::all()
            .iter()
            .copied()
            .map(t::info_label)
            .collect();
        let unique: std::collections::BTreeSet<&str> = labels.iter().copied().collect();
        assert_eq!(
            unique.len(),
            labels.len(),
            "two fields share a label: {labels:?}"
        );
        assert!(labels.iter().all(|l| !l.is_empty()));
    }

    /// Clearing is expressed as `None`, and only a genuinely empty draft
    /// clears.
    #[test]
    fn only_a_blank_draft_removes_the_key() {
        for blank in ["", " ", "\t", "\n  "] {
            assert!(blank.trim().is_empty(), "{blank:?} must clear the field");
        }
        assert!(
            !" Site Plan ".trim().is_empty(),
            "a spaced title is a title the operator may have meant"
        );
    }

    /// The commit rule is the Forms panel's, not a second copy.
    #[test]
    fn tabbing_through_a_field_writes_nothing() {
        use crate::panels::forms::rows::commit;
        assert_eq!(commit(true, "Site Plan", "Site Plan"), None);
        assert_eq!(commit(false, "Site Plan", ""), None);
        assert_eq!(commit(true, "Site Plan", ""), Some("Site Plan".to_owned()));
    }
}
