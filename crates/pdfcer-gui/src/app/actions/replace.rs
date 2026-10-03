//! # `app::actions::replace` — the Find bar's Replace and Replace all
//!
//! Each hit the bar found is rewritten inside the show operator that draws it,
//! and one press — however many hits it replaces — is one undo step. Hits that
//! cannot be rewritten in place are left alone and counted in the status
//! line, never silently dropped.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/replace.md`.

mod locate;

use pdfcer_core::edit::{CommandKind, EditSession};
use pdfcer_core::text_edit::{EditOptions, EditRequest};
use pdfcer_gui_base::find::{FindRequest, FindState};
use pdfcer_gui_base::text::replace::{self as t, ReplaceRefusal};

use self::locate::{Located, Rewrite, Skip, Wanted};
use super::funnel::vector_edit;
use crate::app::state::OpenDoc;

/// Replace the bar's current hit, or every hit, with the bar's replacement.
pub(super) fn apply(find: &mut FindState, doc: &mut OpenDoc, all: bool) {
    let Some(ask) = find.replace_ask(doc.edit_epoch, all) else {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| "text-replace-declined reason=no-current-results".to_owned());
        return;
    };
    if ask.wildcards {
        crate::app::status::decline::record_replace(ReplaceRefusal::Wildcards);
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| "text-replace-declined reason=wildcards".to_owned());
        return;
    }
    let wanted: Vec<Wanted> = ask
        .hits
        .iter()
        .map(|&(page, quad)| Wanted { page, quad })
        .collect();
    let located = locate::locate(doc, &ask.query, ask.case_sensitive, &ask.with, &wanted);
    let options = crate::canvas::textedit::installed::augmented(
        doc,
        pdfcer_gui_base::editmodel::disposition::typing(),
    );
    let found = wanted.len();
    let first_page = wanted.first().map_or(0, |w| w.page);
    let mut tally = Tally::default();
    vector_edit(
        doc,
        "text-replace",
        first_page,
        located.rewrites.len(),
        |session| run(session, &located, &options, found, &mut tally),
    );
    trace(all, found, &tally, &located);
    // The hits moved; search again and stay at the same position in the list,
    // which after a single replace is the hit that followed it.
    crate::find::apply(find, doc, FindRequest::Search);
    find.land_on(doc, ask.current);
}

/// What one press achieved.
#[derive(Debug, Default)]
struct Tally {
    replaced: usize,
    refused: usize,
}

/// Apply every rewrite, coalesce them into one undo entry, and word the result.
fn run(
    session: &mut EditSession,
    located: &Located,
    options: &EditOptions,
    found: usize,
    tally: &mut Tally,
) -> Result<Vec<String>, String> {
    let mut steps = 0_usize;
    let mut notes = Vec::new();
    for rewrite in &located.rewrites {
        match session.edit_text(&request(rewrite), options) {
            Ok(report) => {
                steps += 1;
                tally.replaced += rewrite.hits;
                notes.extend(report.disclosures);
            }
            Err(e) => {
                tally.refused += rewrite.hits;
                // ui-text-exempt: diagnostic trace, never displayed.
                crate::diag::trace(|| {
                    format!(
                        "text-replace-refused page={} span={} error={e}",
                        rewrite.page, rewrite.span.start
                    )
                });
            }
        }
    }
    if steps == 0 {
        crate::app::status::decline::record_replace(ReplaceRefusal::NothingRewritable { found });
        return Err(t::nothing_rewritable(found));
    }
    if steps > 1 && !session.coalesce_last(steps, CommandKind::EditText) {
        notes.push(pdfcer_gui_base::text::reface::undo_split(steps));
    }
    let left = found - tally.replaced;
    let mut summary = vec![t::summary(tally.replaced, found)];
    summary.extend(reasons(&located.skipped, tally.refused, left));
    summary.extend(notes);
    let mut seen = std::collections::HashSet::new();
    summary.retain(|n| seen.insert(n.clone()));
    Ok(summary)
}

/// The request for one rewrite: the operator's whole text, found under its
/// pin, so a pin an earlier rewrite had moved refuses instead of editing
/// whatever now sits at that offset.
fn request(rewrite: &Rewrite) -> EditRequest {
    let mut r =
        EditRequest::find_replace(rewrite.page, &rewrite.old, &rewrite.new).pinned(rewrite.span);
    r.target = rewrite.target;
    r
}

/// One sentence per reason a hit was left alone.
fn reasons(skipped: &[Skip], refused: usize, left: usize) -> Vec<String> {
    if left == 0 {
        return Vec::new();
    }
    let count = |k: Skip| skipped.iter().filter(|s| **s == k).count();
    [
        (
            count(Skip::NotOnOneLine),
            t::left_not_on_one_line as fn(usize) -> String,
        ),
        (count(Skip::CrossesStyles), t::left_crosses_styles),
        (count(Skip::SharedOperator), t::left_shared_operator),
        (refused, t::left_refused),
    ]
    .into_iter()
    .filter(|(n, _)| *n > 0)
    .map(|(n, say)| say(n))
    .collect()
}

fn trace(all: bool, found: usize, tally: &Tally, located: &Located) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "text-replace-applied which={} found={found} replaced={} skipped={} refused={} \
             rewrites={}",
            if all { "all" } else { "current" },
            tally.replaced,
            located.skipped.len(),
            tally.refused,
            located.rewrites.len(),
        )
    });
}
