//! # `app::actions::prepress` — a prepress window's Apply, as one undo step
//!
//! Contract: runs [`PrepressRun`]'s verb through the edit funnel, each of the
//! four engine verbs committing one undo entry of its own, then parks a
//! [`PrepressNote`] keyed by the run so the window that asked shows the
//! engine's account. A refusal leaves a note too, saying why nothing changed;
//! a session another thread still holds leaves none, because nothing ran.
//!
//! Traces `prepress-applied verb= run= pages= changed= ms=` plus the verb's
//! own counts, or `prepress-refused-run verb= run= why=`.

use pdfcer_core::edit::{EditError, EditSession};
use pdfcer_core::prepress::{
    ConvertColoursReport, ConvertScope, OutputIntentChange, PdfxConversionReport,
};
use pdfcer_core::transparency::{FlattenMode, FlattenTransparencyReport};
use pdfcer_emf::RecordingFlattener;
use pdfcer_gui_base::prepress::{PrepressNote, PrepressRun, PrepressVerb};
use pdfcer_render::convert_colours::IcccerConverter;

use crate::app::state::OpenDoc;
use crate::text::prepress as t;

/// What a verb reported: the note's text and the trace's counts.
struct Done {
    changed: bool,
    text: Vec<String>,
    counts: String,
}

pub(super) fn apply(doc: &mut OpenDoc, run: &PrepressRun) {
    let page = doc.view.page_index;
    let mut done: Option<Done> = None;
    let mut refused: Option<String> = None;
    let started = std::time::Instant::now();
    super::funnel::vector_edit(
        doc,
        "prepress",
        page,
        run.pages.len(),
        |session| match call(session, run) {
            Ok((d, disclosures)) => {
                done = Some(d);
                Ok(disclosures)
            }
            Err(e) => {
                refused = Some(e.to_string());
                Err(e)
            }
        },
    );
    let ms = started.elapsed().as_millis();
    let verb = run.verb.token();
    if let Some(why) = refused {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "prepress-refused-run verb={verb} run={} why={why:?}",
                run.run
            )
        });
        doc.prepress_note = Some(PrepressNote {
            run: run.run,
            refused: true,
            text: why,
        });
        return;
    }
    let Some(d) = done else {
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "prepress-applied verb={verb} run={} pages={} changed={} ms={ms} {}",
            run.run,
            run.pages.len(),
            u8::from(d.changed),
            d.counts
        )
    });
    doc.prepress_note = Some(PrepressNote {
        run: run.run,
        refused: false,
        text: d.text.join("\n"),
    });
}

/// The engine call, with the account it leaves and its disclosures.
fn call(session: &mut EditSession, run: &PrepressRun) -> Result<(Done, Vec<String>), EditError> {
    let flattener = RecordingFlattener::new();
    let converter = IcccerConverter::new();
    let (mut done, disclosures) = match &run.verb {
        PrepressVerb::Hairlines { min_width_pt } => {
            let r = session.fix_hairlines(&run.pages, *min_width_pt)?;
            (hairlines(&r, *min_width_pt), r.disclosures)
        }
        PrepressVerb::Flatten(options) => {
            let r = session.flatten_transparency(&run.pages, options, &flattener)?;
            (flatten(&r), r.disclosures)
        }
        PrepressVerb::ConvertColours(c) => {
            let scope = ConvertScope::Pages(run.pages.clone());
            let r = session.convert_colours(&scope, c, &converter)?;
            (colours(&r), r.disclosures)
        }
        PrepressVerb::Pdfx { level, conversion } => {
            let r = session.convert_to_pdfx(*level, conversion, &flattener, &converter)?;
            (pdfx(&r), r.disclosures)
        }
    };
    if !disclosures.is_empty() {
        done.text.push(String::new());
        done.text.push(t::notes_heading().to_owned());
        done.text.extend(disclosures.iter().cloned());
    }
    Ok((done, disclosures))
}

fn hairlines(r: &pdfcer_core::edit::HairlineFixReport, min: f64) -> Done {
    let mut text = vec![if r.changed == 0 {
        t::hairlines_none(min)
    } else {
        t::hairlines_widened(r.changed, r.per_page.len(), r.zero_width, min)
    }];
    text.extend(
        r.forms
            .iter()
            .map(|f| t::form_reach(&f.form.to_string(), f.invocations)),
    );
    text.extend(r.skipped.iter().map(|(p, why)| t::page_skipped(p + 1, why)));
    Done {
        changed: r.changed > 0,
        counts: format!(
            "widened={} zero={} forms={} skipped={}", // ui-text-exempt: a trace token list
            r.changed,
            r.zero_width,
            r.forms.len(),
            r.skipped.len()
        ),
        text,
    }
}

fn flatten(r: &FlattenTransparencyReport) -> Done {
    let mut text = Vec::new();
    let (mut flattened, mut raster) = (0, 0);
    for p in &r.pages {
        let n = p.page + 1;
        match p.mode {
            _ if p.found.is_empty() => text.push(t::flatten_page_clean(n)),
            None => text.push(t::flatten_page_left(n)),
            Some(mode) => {
                flattened += 1;
                raster += p.rasterised.len();
                text.push(t::flatten_page_done(
                    n,
                    mode == FlattenMode::Rebuilt,
                    p.vector_regions,
                    p.rasterised.len(),
                    p.text_runs_outlined,
                    p.strokes_outlined,
                ));
            }
        }
        if !p.spot_inks.is_empty() {
            text.push(t::flatten_spots(n, &p.spot_inks.join(", "))); // ui-text-exempt: a list separator
        }
        if !p.annotations_left.is_empty() {
            text.push(t::flatten_annots_left(n, p.annotations_left.len()));
        }
        text.extend(p.notes.iter().map(|note| t::flatten_page_note(n, note)));
    }
    Done {
        changed: r.changed,
        counts: format!("flattened={flattened} rasterised={raster}"), // ui-text-exempt: a trace token list
        text,
    }
}

fn colours(r: &ConvertColoursReport) -> Done {
    let mut text = vec![if r.changed.is_empty() {
        t::colours_none().to_owned()
    } else {
        t::colours_changed(r.changed.len())
    }];
    text.extend(r.by_source.iter().map(|s| {
        t::colours_source(
            s.source.name(),
            s.paints,
            s.images,
            s.lookups,
            s.shadings,
            s.out_of_gamut,
        )
    }));
    text.extend(r.unconverted.iter().map(|u| {
        let object = u.object.map_or_else(String::new, |o| o.to_string());
        t::colours_unconverted(&object, &u.reason)
    }));
    match &r.output_intent {
        OutputIntentChange::Added => text.push(t::intent_added().to_owned()),
        OutputIntentChange::Replaced { previous } => text.push(t::intent_replaced(previous)),
        _ => {}
    }
    Done {
        changed: !r.changed.is_empty(),
        counts: format!(
            "converted={} unconverted={} gamut={}", // ui-text-exempt: a trace token list
            r.changed.len(),
            r.unconverted.len(),
            r.out_of_gamut
        ),
        text,
    }
}

fn pdfx(r: &PdfxConversionReport) -> Done {
    let (before, after) = (r.before.as_str(), r.after.as_str());
    let mut text = vec![t::pdfx_result(
        r.level.claim(),
        t::verdict_word(before),
        t::verdict_word(after),
        r.claimed,
    )];
    if r.kept_partial {
        text.push(t::pdfx_kept_partial().to_owned());
    } else if !r.claimed && after != "conforms" {
        text.push(t::pdfx_reverted().to_owned());
    }
    text.extend(r.steps.iter().cloned());
    text.extend(
        r.validation
            .preflight
            .findings
            .iter()
            .map(|f| t::pdfx_finding(&f.rule_id, &f.message)),
    );
    Done {
        changed: r.claimed || r.kept_partial,
        counts: format!(
            "level={} before={before} after={after} claimed={} findings={}", // ui-text-exempt: a trace token list
            r.level.as_str(),
            u8::from(r.claimed),
            r.validation.preflight.findings.len()
        ),
        text,
    }
}
