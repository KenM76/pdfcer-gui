//! # `app::dispatch::textformat` — Bold, Italic, the two drawn lines, and
//! paragraph alignment
//!
//! `OPERATOR_REQUESTS.md` O271 and O273. Every id here acts on *the text the
//! operator means*, resolved in this order:
//!
//! 1. **Inside a draft on existing text**: the selected characters; else the
//!    word the caret is inside; else the caret itself, which records a pending
//!    style for what is typed next (`canvas::textedit::typing`). A draft with
//!    edits is committed first, and closed.
//! 2. **A live text sweep**: exactly the swept characters.
//! 3. **A selected text object**: its whole runs.
//!
//! Bold and Italic are toggles. Bold read as absent is added; synthetic bold
//! (fill-and-stroke rendering) is taken off by returning to fill-only; bold
//! that is the face itself is declined with the remedy, because the engine's
//! style ladder cannot yet remove an axis. Italic present is declined for the
//! same reason.
//!
//! Alignment acts on whole paragraphs: the caret's, or every one the sweep or
//! object touches.

use pdfcer_core::text_edit::{BlockAlignment, TextPosition};

use crate::app::actions::Action;
use crate::app::actions::text::{Decoration, TextAction};
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::{OpenDoc, Status};
use crate::app::status::decline;
use crate::canvas::textedit::weight::{self, Axis};
use crate::canvas::textedit::{self, Anchor, Draft, typing};
use crate::text::status::TextStyleRefusal;

/// Whether this file owns `id`.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(
        id,
        "format.bold"
            | "format.italic"
            | "format.underline"
            | "format.strikethrough"
            | "format.align_left"
            | "format.align_centre"
            | "format.align_right"
            | "format.align_justify"
    )
}

/// The alignment an `format.align_*` id names.
fn alignment_of(id: &str) -> Option<BlockAlignment> {
    // ui-text-exempt: registered command ids, never displayed.
    match id {
        "format.align_left" => Some(BlockAlignment::Left),
        "format.align_centre" => Some(BlockAlignment::Center),
        "format.align_right" => Some(BlockAlignment::Right),
        "format.align_justify" => Some(BlockAlignment::Justified),
        _ => None,
    }
}

/// The command id as a `'static` key for the pending-style record.
fn command_key(id: &str) -> &'static str {
    // ui-text-exempt: registered command ids, never displayed.
    match id {
        "format.bold" => "format.bold",
        "format.italic" => "format.italic",
        "format.underline" => "format.underline",
        _ => "format.strikethrough",
    }
}

/// What a press acts on.
enum Target {
    /// A character range, with the draft to close first if it came from one.
    Range {
        page: usize,
        from: TextPosition,
        to: TextPosition,
        expected: String,
        draft: bool,
    },
    /// The caret of a draft on run `run`, with no range: a pending style.
    Caret { draft: Draft, run: usize },
    /// Whole runs of a selected text object.
    Runs { page: usize, runs: Vec<usize> },
}

/// Route one of this file's commands.
pub(crate) fn dispatch(
    app: &mut crate::app::PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    actions: &mut Vec<Action>,
) {
    let Status::Open(doc) = &app.status else {
        return;
    };
    if let Some(alignment) = alignment_of(id) {
        align(ctx, doc, alignment, actions);
        return;
    }
    let target = match target(ctx, doc) {
        Ok(target) => target,
        Err(why) => return refuse(id, why),
    };
    let decoration = match id {
        "format.underline" => Some(Decoration::Underline),
        "format.strikethrough" => Some(Decoration::Strikethrough),
        _ => None,
    };
    let pending = match decoration {
        Some(kind) => typing::Pending::Decorate(kind),
        None => match toggle(id, read_weight(ctx, doc, &target)) {
            Ok(change) => typing::Pending::Style(change),
            Err(why) => return refuse(id, why),
        },
    };
    match target {
        Target::Range {
            page,
            from,
            to,
            expected,
            draft,
        } => {
            if draft {
                textedit::settle(ctx, actions);
            }
            actions.push(Action::Text(match pending {
                typing::Pending::Style(change) => TextAction::SpanStyle {
                    page,
                    from,
                    to,
                    expected,
                    change,
                },
                typing::Pending::Decorate(kind) => TextAction::Decorate {
                    page,
                    from,
                    to,
                    expected,
                    kind,
                },
            }));
        }
        Target::Caret { draft, .. } => {
            let set = typing::toggle(ctx, draft.caret, command_key(id), pending);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("text-typing-style id={id} at={} set={set}", draft.caret)
            });
        }
        Target::Runs { page, runs } => match pending {
            typing::Pending::Style(change) => {
                actions.push(Action::TextStyle { page, runs, change })
            }
            typing::Pending::Decorate(kind) => {
                if let Some((from, to, expected)) = whole_runs(doc, page, &runs) {
                    actions.push(Action::Text(TextAction::Decorate {
                        page,
                        from,
                        to,
                        expected,
                        kind,
                    }));
                } else {
                    refuse(id, TextStyleRefusal::NoRun);
                }
            }
        },
    }
}

/// The characters, word, caret or runs a press acts on.
fn target(ctx: &egui::Context, doc: &OpenDoc) -> Result<Target, TextStyleRefusal> {
    if let Some(draft) = textedit::read(ctx) {
        let Anchor::Run { run, .. } = draft.anchor else {
            return Err(TextStyleRefusal::NewText);
        };
        let chosen = textedit::caret::range(draft.mark, draft.caret)
            .or_else(|| word_around(&draft.text, draft.caret));
        let Some((a, b)) = chosen else {
            return Ok(Target::Caret { draft, run });
        };
        let (ba, bb) = (byte_of(&draft.text, a), byte_of(&draft.text, b));
        return Ok(Target::Range {
            page: draft.page,
            from: TextPosition::new(run, ba),
            to: TextPosition::new(run, bb),
            expected: draft.text[ba..bb].to_owned(),
            draft: true,
        });
    }
    if let Some(selection) = doc
        .text_selection
        .as_ref()
        .filter(|s| s.live(doc.edit_epoch) && !s.is_empty())
    {
        let (from, to) =
            pdfcer_gui_base::textselection::ordered(selection.anchor(), selection.focus());
        return Ok(Target::Range {
            page: selection.page,
            from,
            to,
            expected: selection.text.clone(),
            draft: false,
        });
    }
    crate::app::textoperand::resolve(doc)
        .map(|operand| Target::Runs {
            page: operand.page,
            runs: operand.runs,
        })
        .ok_or(TextStyleRefusal::NoRun)
}

/// The word the caret is strictly inside, as character indices.
fn word_around(text: &str, caret: usize) -> Option<(usize, usize)> {
    let chars: Vec<char> = text.chars().collect();
    let word = |i: usize| chars.get(i).is_some_and(|c| c.is_alphanumeric());
    if caret == 0 || !word(caret - 1) || !word(caret) {
        return None;
    }
    let mut start = caret;
    while start > 0 && word(start - 1) {
        start -= 1;
    }
    let mut end = caret;
    while word(end) {
        end += 1;
    }
    Some((start, end))
}

/// The byte offset of character `chars` in `text`.
fn byte_of(text: &str, chars: usize) -> usize {
    text.char_indices()
        .nth(chars)
        .map_or(text.len(), |(b, _)| b)
}

/// The weight of the text a press acts on: the first letter of a range or of
/// the runs, the letter before a caret.
fn read_weight(ctx: &egui::Context, doc: &OpenDoc, target: &Target) -> Option<weight::Weight> {
    let _ = ctx;
    match target {
        Target::Range { page, from, .. } => weight::at(doc, *page, *from),
        Target::Caret { draft, run } => {
            let before = draft.caret.saturating_sub(1);
            weight::at(
                doc,
                draft.page,
                TextPosition::new(*run, byte_of(&draft.text, before)),
            )
        }
        Target::Runs { page, runs } => {
            let run = *runs.first()?;
            weight::at(doc, *page, TextPosition::new(run, 0))
        }
    }
}

/// What a Bold or Italic press changes, given what the text carries now.
fn toggle(id: &str, weight: Option<weight::Weight>) -> Result<StyleChange, TextStyleRefusal> {
    let (bold, italic) = weight.map_or((Axis::Absent, Axis::Absent), |w| (w.bold, w.italic));
    if id == "format.bold" {
        return match bold {
            Axis::Absent => Ok(StyleChange::Weight {
                bold: true,
                italic: false,
            }),
            Axis::Synthetic => Ok(StyleChange::RenderMode(0)),
            Axis::Face => Err(TextStyleRefusal::BoldIsFace),
        };
    }
    if italic.present() {
        return Err(TextStyleRefusal::ItalicStays);
    }
    Ok(StyleChange::Weight {
        bold: false,
        italic: true,
    })
}

/// The whole of `runs` as one character range, with the text it reads.
fn whole_runs(
    doc: &OpenDoc,
    page: usize,
    runs: &[usize],
) -> Option<(TextPosition, TextPosition, String)> {
    let (first, last) = (*runs.iter().min()?, *runs.iter().max()?);
    let text = doc.provenance_page_text(page)?;
    let end = text.runs.get(last)?.text.len();
    let ctx = crate::canvas::textsel::PageContext {
        text: &text,
        page: doc.pages.get(page)?,
        index: page,
        epoch: doc.edit_epoch,
    };
    let from = TextPosition::new(first, 0);
    let to = TextPosition::new(last, end);
    let read = crate::canvas::textsel::span(&ctx, from, to)?;
    Some((from, to, read.text))
}

/// Align the caret's paragraph, or every paragraph the selection touches.
fn align(ctx: &egui::Context, doc: &OpenDoc, alignment: BlockAlignment, actions: &mut Vec<Action>) {
    use crate::text::textedit::ReflowRefusal;
    let (page, runs, draft) = match textedit::read(ctx) {
        Some(Draft {
            anchor: Anchor::Run { run, .. },
            page,
            ..
        }) => (page, vec![run], true),
        Some(_) => return refuse("format.align", TextStyleRefusal::NewText),
        None => {
            let live = doc
                .text_selection
                .as_ref()
                .filter(|s| s.live(doc.edit_epoch) && !s.is_empty());
            match live {
                Some(selection) => (selection.page, selection.runs(doc.edit_epoch), false),
                None => match crate::app::textoperand::resolve(doc) {
                    Some(operand) => (operand.page, operand.runs, false),
                    None => {
                        decline::record_reflow(ReflowRefusal::NeedsCaret);
                        return;
                    }
                },
            }
        }
    };
    let blocks = textedit::reflow::blocks_of_runs(doc, page, &runs);
    if blocks.is_empty() {
        decline::record_reflow(ReflowRefusal::NoBlock);
        return;
    }
    if draft {
        textedit::settle(ctx, actions);
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("text-align-resolved page={page} alignment={alignment:?} blocks={blocks:?}")
    });
    actions.push(Action::Text(TextAction::Align {
        page,
        blocks,
        alignment,
    }));
}

/// Say why on the bar and in the trace; change nothing.
fn refuse(id: &str, why: TextStyleRefusal) {
    decline::record_text_style(why.clone());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("text-style-declined id={id} why={why:?}")
    });
}
