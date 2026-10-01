//! # `canvas::textedit::refused` — the keys the run's font would not take,
//! named beside the edit, with one click to a face that takes them
//!
//! [`note`] collects every refused key for the open draft; [`notice`] draws
//! them in a popup under the editor box and offers the nearest face that has
//! them all; [`resume`] puts the held keys back once that face has landed.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/refused.md`.

use pdfcer_gui_base::editmodel::nearface;

use super::{Anchor, Draft};
use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::panels::properties::face::{FaceChoice, FaceOrigin};
use crate::text::refusedkeys as t;

const KEY: &str = "textedit-refused-keys"; // ui-text-exempt: a memory key, never displayed.

/// The notice, published on the frames it draws.
pub const REGION: &str = "textedit.refused-keys"; // ui-text-exempt: trace region name, never displayed
/// Its one-click button.
pub const USE_REGION: &str = "textedit.refused-keys.use-face"; // ui-text-exempt: trace region name, never displayed

/// The draft as it stood, so a later frame can tell whether it has changed.
#[derive(Clone, PartialEq)]
struct Snapshot {
    text: String,
    caret: usize,
    mark: Option<usize>,
}

impl Snapshot {
    fn of(draft: &Draft) -> Self {
        Self {
            text: draft.text.clone(),
            caret: draft.caret,
            mark: draft.mark,
        }
    }
}

/// Where the offer stands.
#[derive(Clone, PartialEq)]
enum Stage {
    /// Naming the keys, with a face if one takes them all.
    Offer,
    /// The face change is raised; `epoch` and `frame` are when.
    Asked {
        face: String,
        epoch: u64,
        frame: u64,
    },
    /// The face landed and the held keys went back in.
    Retyped { face: String, keys: Vec<char> },
    /// The face landed, but the draft had moved on, so nothing was put back.
    TypeAgain { face: String },
    /// The document did not change: the restyle was refused.
    SwapRefused { face: String },
}

/// Everything the notice knows about the open draft's refused keys.
#[derive(Clone)]
struct Held {
    page: usize,
    run: usize,
    base_font: String,
    /// Every distinct refused key, in the order first refused.
    chars: Vec<char>,
    /// The refused keys typed since the draft last changed, in order.
    tail: String,
    /// The draft after the last refusal (or, in a closing stage, after the
    /// put-back); `None` when the tail cannot be placed.
    at: Option<Snapshot>,
    /// The faces that take every key in `chars`, read at this epoch.
    faces: Option<(u64, Vec<FaceChoice>)>,
    stage: Stage,
    /// The last trace line, so it is written on change only.
    traced: String,
}

fn read(ctx: &egui::Context) -> Option<Held> {
    ctx.data(|d| d.get_temp::<Held>(egui::Id::new(KEY)))
}

fn write(ctx: &egui::Context, held: Held) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), held));
}

/// Forget the notice; called when the draft closes.
pub(super) fn forget(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Held>(egui::Id::new(KEY)));
}

/// Record keys the run refused. `draft` is the draft after the keystroke;
/// `went_in` is whether any key of the same keystroke was inserted, in which
/// case the refused ones can no longer be placed where they were typed.
pub(super) fn note(
    ctx: &egui::Context,
    draft: &Draft,
    refused: &[char],
    base_font: &str,
    went_in: bool,
) {
    let Anchor::Run { run, .. } = &draft.anchor else {
        return;
    };
    let mut held = read(ctx)
        .filter(|h| h.page == draft.page && h.run == *run)
        .unwrap_or_else(|| Held {
            page: draft.page,
            run: *run,
            base_font: base_font.to_owned(),
            chars: Vec::new(),
            tail: String::new(),
            at: None,
            faces: None,
            stage: Stage::Offer,
            traced: String::new(),
        });
    if held.stage != Stage::Offer {
        held.chars.clear();
        held.tail.clear();
        held.at = None;
        held.stage = Stage::Offer;
    }
    held.base_font = base_font.to_owned();
    for c in refused {
        if !held.chars.contains(c) {
            held.chars.push(*c);
            held.faces = None;
        }
    }
    let now = Snapshot::of(draft);
    if went_in {
        held.tail.clear();
        held.at = None;
    } else {
        if held.at.as_ref() != Some(&now) {
            held.tail.clear();
        }
        held.tail.extend(refused);
        held.at = Some(now);
    }
    write(ctx, held);
}

/// The held keys to type back in, on the frame the chosen face has landed
/// and only if the draft is exactly as it was when they were refused.
pub(super) fn resume(ctx: &egui::Context, doc: &OpenDoc, draft: &Draft) -> Option<String> {
    let mut held = read(ctx)?;
    let Stage::Asked { face, epoch, frame } = held.stage.clone() else {
        return None;
    };
    if doc.edit_epoch == epoch {
        // Actions apply after the frame that raised them; two frames on with
        // the revision unchanged, the restyle was refused.
        if ctx.cumulative_frame_nr() > frame + 1 {
            held.stage = Stage::SwapRefused { face };
            held.at = None;
            write(ctx, held);
        }
        return None;
    }
    let placeable = held.at.as_ref() == Some(&Snapshot::of(draft)) && !held.tail.is_empty();
    let back =
        placeable.then(|| super::repertoire::sieve(ctx, doc, held.page, held.run, &held.tail).kept);
    held.stage = match back.as_deref() {
        Some(kept) if !kept.is_empty() => Stage::Retyped {
            face,
            keys: kept.chars().collect(),
        },
        _ => Stage::TypeAgain { face },
    };
    held.tail.clear();
    held.at = None;
    write(ctx, held);
    back.filter(|k| !k.is_empty())
}

/// Draw the notice under the editor box, and raise the face change when the
/// operator takes the offer.
pub fn notice(ctx: &egui::Context, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let Some(draft) = super::read(ctx) else {
        forget(ctx);
        return;
    };
    let (Some(mut held), Some(layout)) = (read(ctx), super::hit::read(ctx)) else {
        return;
    };
    if !matches!(&draft.anchor, Anchor::Run { run, .. } if *run == held.run)
        || draft.page != held.page
    {
        return;
    }
    if !retire_closing(ctx, &mut held, &draft) {
        return;
    }
    if held.stage == Stage::Offer {
        sync_faces(doc, &mut held);
    }
    let best = best_face(&held).cloned();
    trace(&mut held, best.as_ref());
    let id = egui::Id::new(KEY);
    let size = ctx
        .memory(|m| m.area_rect(id))
        .map_or(egui::vec2(360.0, 60.0), |r| r.size());
    let gap = 6.0;
    let below = layout.body.left_bottom() + egui::vec2(0.0, gap);
    let pos = if below.y + size.y > ctx.content_rect().bottom() {
        layout.body.left_top() - egui::vec2(0.0, gap + size.y)
    } else {
        below
    };
    egui::Area::new(id)
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .constrain(true)
        .show(ctx, |ui| {
            let frame = egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(360.0);
                body(ui, doc, &mut held, best.as_ref(), actions);
            });
            crate::diag::ui_rect_visible(REGION, frame.response.rect, ui.clip_rect());
        });
    write(ctx, held);
}

/// End a closing stage once the draft changes after it; answer whether
/// anything is left to draw.
fn retire_closing(ctx: &egui::Context, held: &mut Held, draft: &Draft) -> bool {
    if matches!(held.stage, Stage::Offer | Stage::Asked { .. }) {
        return true;
    }
    let now = Snapshot::of(draft);
    match &held.at {
        None => {
            held.at = Some(now);
            true
        }
        Some(at) if *at == now => true,
        Some(_) => {
            forget(ctx);
            false
        }
    }
}

/// The notice's contents, by stage.
fn body(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    held: &mut Held,
    best: Option<&FaceChoice>,
    actions: &mut Vec<Action>,
) {
    let font = crate::panels::properties::text::shorten(&held.base_font).to_owned();
    match held.stage.clone() {
        Stage::Offer => {
            ui.label(t::named(&held.chars, &font));
            let Some(face) = best else {
                if held.faces.is_some() {
                    ui.label(egui::RichText::new(t::no_face(&held.chars)).small());
                }
                return;
            };
            if face.origin == FaceOrigin::PdfcerWouldAdd {
                ui.label(
                    egui::RichText::new(crate::text::panels::face::face_addable_disclosure())
                        .small(),
                );
            }
            let button = ui.button(t::use_face(&face.label, held.chars.len()));
            crate::diag::ui_rect_visible(USE_REGION, button.rect, ui.clip_rect());
            if button.clicked() {
                held.stage = Stage::Asked {
                    face: face.label.clone(),
                    epoch: doc.edit_epoch,
                    frame: ui.ctx().cumulative_frame_nr(),
                };
                actions.push(Action::TextStyle {
                    page: held.page,
                    runs: vec![held.run],
                    change: StyleChange::Face(face.selector.clone()),
                });
            }
        }
        Stage::Asked { face, .. } => {
            ui.label(t::switching(&face));
        }
        Stage::Retyped { face, keys } => {
            ui.label(t::retyped(&face, &keys));
        }
        Stage::TypeAgain { face } => {
            ui.label(t::swapped_type_again(&face, &held.chars));
        }
        Stage::SwapRefused { face } => {
            ui.label(t::swap_refused(&face));
        }
    }
}

/// Read the faces that take every refused key, once per revision and set.
fn sync_faces(doc: &OpenDoc, held: &mut Held) {
    if held
        .faces
        .as_ref()
        .is_some_and(|(e, _)| *e == doc.edit_epoch)
    {
        return;
    }
    let candidate: String = held.chars.iter().collect();
    let faces = super::pin::inspect(doc, held.page, held.run)
        .and_then(|read| super::pin::font_preflight(doc, held.page, &read, Some(&candidate)))
        .as_ref()
        .map_or_else(Vec::new, |p| {
            crate::panels::properties::face::choices(Some(p))
        });
    held.faces = Some((doc.edit_epoch, faces));
}

/// The face nearest the run's own among those that take every key.
fn best_face(held: &Held) -> Option<&FaceChoice> {
    let (_, faces) = held.faces.as_ref()?;
    let candidates: Vec<_> = faces
        .iter()
        .map(|f| nearface::Candidate {
            base_font: &f.label,
            on_page: f.origin == FaceOrigin::OnThisPage,
        })
        .collect();
    faces.get(nearface::nearest(&held.base_font, &candidates)?)
}

/// Trace the notice's state when it changes.
fn trace(held: &mut Held, best: Option<&FaceChoice>) {
    let stage = match &held.stage {
        Stage::Offer if held.faces.is_none() => "reading", // ui-text-exempt: a trace token, never displayed
        Stage::Offer if best.is_none() => "no-face", // ui-text-exempt: a trace token, never displayed
        Stage::Offer => "offer", // ui-text-exempt: a trace token, never displayed
        Stage::Asked { .. } => "asked", // ui-text-exempt: a trace token, never displayed
        Stage::Retyped { .. } => "retyped", // ui-text-exempt: a trace token, never displayed
        Stage::TypeAgain { .. } => "type-again", // ui-text-exempt: a trace token, never displayed
        Stage::SwapRefused { .. } => "swap-refused", // ui-text-exempt: a trace token, never displayed
    };
    let chars: Vec<String> = held
        .chars
        .iter()
        .map(|c| format!("U+{:04X}", u32::from(*c)))
        .collect();
    let line = format!(
        // ui-text-exempt: diagnostic trace, never displayed.
        "text-edit-refused-keys page={} run={} characters={} font={} faces={} face={} state={stage}",
        held.page,
        held.run,
        chars.join(","),
        held.base_font,
        held.faces.as_ref().map_or(0, |(_, f)| f.len()),
        best.map_or("none", |f| f.selector.as_str()),
    );
    if line != held.traced {
        crate::diag::trace(|| line.clone());
        held.traced = line;
    }
}
