//! One paragraph's own answer to whether a click opens it whole, whatever the
//! page and the switch say. Held for the life of the window, never saved.
//!
//! A paragraph is known by its box's upper-left corner (PDF user space),
//! matched within [`SAME_PT`]: block indices renumber after any edit on the
//! page, and an edit inside the paragraph moves its lower and right edges but
//! not that corner.

use super::{Draft, opening_box, widen};
use crate::app::state::OpenDoc;
use crate::canvas::textedit::{Anchor, read, store};

/// Answers keyed by (document serial, page), each a corner and the answer.
const KEY: &str = "pdfcer.open-paragraphs.here"; // ui-text-exempt: a memory key, never displayed

/// Two corners within this many points name the same paragraph.
const SAME_PT: f64 = 2.0;

#[derive(Clone, Copy)]
struct Answer {
    serial: u64,
    page: usize,
    llx: f64,
    ury: f64,
    on: bool,
}

type Answers = Vec<Answer>;

fn answers(ctx: &egui::Context) -> Answers {
    ctx.data(|d| d.get_temp::<Answers>(egui::Id::new(KEY)))
        .unwrap_or_default()
}

fn same(a: &Answer, serial: u64, page: usize, llx: f64, ury: f64) -> bool {
    a.serial == serial
        && a.page == page
        && (a.llx - llx).abs() < SAME_PT
        && (a.ury - ury).abs() < SAME_PT
}

/// Does any paragraph on `page` carry its own answer?
#[must_use]
pub fn any(ctx: &egui::Context, serial: u64, page: usize) -> bool {
    answers(ctx)
        .iter()
        .any(|a| a.serial == serial && a.page == page)
}

/// The answer for the paragraph whose box is `bbox`, else `default`.
#[must_use]
pub fn opens(
    ctx: &egui::Context,
    serial: u64,
    page: usize,
    (llx, _, _, ury): (f64, f64, f64, f64),
    default: bool,
) -> bool {
    answers(ctx)
        .iter()
        .find(|a| same(a, serial, page, llx, ury))
        .map_or(default, |a| a.on)
}

fn set(ctx: &egui::Context, answer: Answer) {
    ctx.data_mut(|d| {
        let list = d.get_temp_mut_or_default::<Answers>(egui::Id::new(KEY));
        list.retain(|a| !same(a, answer.serial, answer.page, answer.llx, answer.ury));
        list.push(answer);
    });
}

/// Flip the answer for the paragraph the caret is in: an open paragraph goes
/// back to its clicked line when nothing is typed yet (the answer holds from
/// the next click either way); a line opens its whole paragraph now.
pub fn toggle(ctx: &egui::Context, doc: &OpenDoc) {
    let Some(mut draft) = read(ctx) else {
        trace_declined("no-draft");
        return;
    };
    let (page, serial) = (draft.page, doc.serial);
    match draft.anchor.clone() {
        Anchor::Block {
            llx, ury, original, ..
        } => {
            set(
                ctx,
                Answer {
                    serial,
                    page,
                    llx,
                    ury,
                    on: false,
                },
            );
            let demoted = draft.text == original && demote(&mut draft);
            store(ctx, draft);
            trace(false, page, llx, ury, demoted);
        }
        Anchor::Run { run, original } => {
            let (llx, _, _, ury) = match opening_box(doc, page, run, &original) {
                Ok(bbox) => bbox,
                Err(token) => return trace_declined(token),
            };
            set(
                ctx,
                Answer {
                    serial,
                    page,
                    llx,
                    ury,
                    on: true,
                },
            );
            let widened = widen(ctx, doc, &mut draft, true).is_ok();
            store(ctx, draft);
            trace(true, page, llx, ury, widened);
        }
        Anchor::Origin { .. } | Anchor::Box { .. } => trace_declined("not-existing-text"),
    }
}

/// `draft` (an unedited paragraph) re-anchored on the line it was opened
/// from, the caret kept on the same character where it lies in that line.
fn demote(draft: &mut Draft) -> bool {
    let Anchor::Block {
        run,
        original,
        places,
        ..
    } = &draft.anchor
    else {
        return false;
    };
    let mine = places.iter().filter(|p| p.pos.run == *run);
    let (Some(start), Some(end)) = (
        mine.clone().map(|p| p.at).min(),
        mine.map(|p| p.at + p.len).max(),
    ) else {
        return false;
    };
    let Some(text) = original.get(start..end).map(str::to_owned) else {
        return false;
    };
    let before = original[..start].chars().count();
    let len = text.chars().count();
    draft.caret = draft.caret.saturating_sub(before).min(len);
    draft.mark = None;
    draft.anchor = Anchor::Run {
        run: *run,
        original: text.clone(),
    };
    draft.text = text;
    true
}

fn trace(on: bool, page: usize, llx: f64, ury: f64, now: bool) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("open-paragraph-here enabled={on} page={page} corner={llx:.1},{ury:.1} now={now}")
    });
}

fn trace_declined(reason: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("open-paragraph-here-declined reason={reason}")
    });
}
