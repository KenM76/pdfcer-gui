//! # `app::actions::reface` — committing a text edit whose new characters the
//! run's font lacks, with those characters set in another face
//!
//! [`commit`] commits the tokenised line through the ordinary plan, then for
//! each segment, last first, moves its token to the face (`format_text`) and
//! writes the characters over it (`edit_text`). Every step lands in one undo
//! entry (`coalesce_last`); a step that fails rolls the session back to the
//! checkpoint taken before the first, so the page, Undo and Redo are either
//! fully edited or as they were.

use pdfcer_core::edit::{Checkpoint, CommandKind, EditSession};
use pdfcer_core::text_edit::{EditRequest, FontSelector, FormatOptions, FormatRequest};
use pdfcer_gui_base::editmodel::reface::{Reface, Tokens};
use pdfcer_gui_base::text::reface as t;

use super::funnel::vector_edit;
use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;
use crate::canvas::textedit::Plan;

/// Commit `replacement` with `reface`'s characters in its face, when the run
/// still refuses any of them and the line can be tokenised. Answers whether
/// it took the commit; `false` leaves it to the ordinary path.
pub(super) fn try_commit(
    doc: &mut OpenDoc,
    page: usize,
    run: usize,
    original: &str,
    replacement: &str,
    reface: &Reface,
) -> bool {
    let chars = crate::canvas::textedit::repertoire::refused_now(doc, page, run, &reface.chars);
    if chars.is_empty() {
        return false;
    }
    let reface = Reface {
        chars,
        ..reface.clone()
    };
    let tokens = page_lines(doc, page).and_then(|lines| {
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        pdfcer_gui_base::editmodel::reface::tokenise_any(replacement, &reface, &refs)
    });
    let Some(tokens) = tokens else {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("text-edit-reface-declined page={page} run={run}"));
        return false;
    };
    let plan = crate::canvas::textedit::plan(doc, page, run, original, &tokens.text);
    vector_edit(doc, "edit-text", page, 1, |session| {
        commit(session, &plan, &tokens, &reface, page)
    });
    let reads = page_lines(doc, page).is_some_and(|l| l.concat().contains(replacement));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "text-edit-reface-readback page={page} run={run} reads={}",
            u8::from(reads)
        )
    });
    true
}

/// A re-faced commit that was not made, with the engine's reason.
pub struct Stopped {
    why: t::Stopped,
    face: String,
    detail: String,
    /// The oldest undo entries the rollback could not keep.
    lost: usize,
}

impl std::fmt::Display for Stopped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&t::stopped(self.why, &self.face, &self.detail))?;
        if self.lost > 0 {
            write!(f, " {}", t::history_lost(self.lost))?;
        }
        Ok(())
    }
}

/// Run the whole gesture on `session`; answers the notes to show.
pub(super) fn commit(
    session: &mut EditSession,
    plan: &Plan,
    tokens: &Tokens,
    reface: &Reface,
    page: usize,
) -> Result<Vec<String>, Stopped> {
    let stop = |why, detail: String, lost| Stopped {
        why,
        face: reface.label.clone(),
        detail,
        lost,
    };
    let before = session.checkpoint();
    let mut notes = match plan
        .attempt("commit", |r| session.edit_text(r, &plan.options))
        .0
    {
        Ok(report) => report.disclosures,
        Err(e) => return Err(stop(t::Stopped::Line, e.to_string(), 0)),
    };
    let mut steps = 1_usize;
    for (token, segment) in tokens.segments.iter().rev() {
        let format = FormatRequest::new(page, token).font(FontSelector::new(&reface.face));
        if let Err(e) = session.format_text(&format, &FormatOptions::default()) {
            let lost = abandon(session, before, steps);
            return Err(stop(t::Stopped::Face, e.to_string(), lost));
        }
        steps += 1;
        let write = EditRequest::find_replace(page, token, segment);
        match session.edit_text(&write, &plan.options) {
            Ok(report) => notes.extend(report.disclosures),
            Err(e) => {
                let lost = abandon(session, before, steps);
                return Err(stop(t::Stopped::Chars, e.to_string(), lost));
            }
        }
        steps += 1;
    }
    trace(page, tokens, steps);
    if !session.coalesce_last(steps, CommandKind::EditText) {
        notes.push(t::undo_split(steps));
    }
    notes.push(t::set_in(&reface.chars, &reface.label));
    let mut seen = std::collections::HashSet::new();
    notes.retain(|n| seen.insert(n.clone()));
    Ok(notes)
}

/// Return the session to `before`, document and both stacks; answers how many
/// of the oldest undo entries could not be kept. A gesture longer than the
/// undo bound cannot be rolled back, and is undone step by step instead, which
/// leaves its steps on Redo.
fn abandon(session: &mut EditSession, before: Checkpoint, steps: usize) -> usize {
    match session.rollback(before) {
        Ok(rolled) => rolled.history_lost,
        Err(e) => {
            // ui-text-exempt: diagnostic trace, never displayed.
            crate::diag::trace(|| format!("text-edit-reface-rollback-refused error={e}"));
            for _ in 0..steps {
                let _ = session.undo();
            }
            0
        }
    }
}

fn trace(page: usize, tokens: &Tokens, steps: usize) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "text-edit-reface-committed page={page} segments={} steps={steps}",
            tokens.segments.len()
        )
    });
}

/// Every line of `page` as the session prints it now, for the token floor.
fn page_lines(doc: &OpenDoc, page: usize) -> Option<Vec<String>> {
    let view = doc.session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).ok()?;
    let text = pdfcer_core::text_extract::extract_page_view(
        &view,
        pages.get(page)?,
        page,
        &doc.settings.extract_options(),
    )
    .ok()?;
    Some(text.runs.into_iter().map(|r| r.text).collect())
}

#[cfg(test)]
mod tests {
    use pdfcer_core::edit::EditSession;
    use pdfcer_core::text_edit::{EditOptions, EditRequest};

    /// A refused gesture leaves Redo holding what it held before the gesture,
    /// not the gesture's own steps.
    #[test]
    fn an_abandoned_gesture_leaves_redo_as_it_was() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/augment-subset.pdf");
        let document = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let mut session = EditSession::new(document);
        let edit = |s: &mut EditSession, from: &str, to: &str| {
            s.edit_text(
                &EditRequest::find_replace(0, from, to),
                &EditOptions::default(),
            )
            .expect("the edit lands");
        };
        edit(&mut session, "ABC", "CAB");
        assert!(session.undo().is_some());
        assert_eq!(session.redo_depth(), 1);
        let before = session.checkpoint();
        edit(&mut session, "ABC", "BCA");
        edit(&mut session, "BCA", "CBA");
        assert_eq!(super::abandon(&mut session, before, 2), 0);
        assert_eq!(
            (session.undo_depth(), session.redo_depth()),
            (0, 1),
            "the abandoned steps are on Redo, or the entry before them is gone"
        );
    }
}
