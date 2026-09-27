//! # `canvas::notepopup` — reading a comment where the comment is
//!
//! The window that opens when an operator clicks a note on the page, and the
//! tooltip that appears when they hover one. **The canvas half of the review
//! surface**, and the half that was missing.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/notepopup/mod.md`.

/// Turning a document into notes and their windows — testable without a `Ui`.
pub use pdfcer_gui_base::notepopupmodel as model;

/// Which pop-ups are showing, and who decided.
pub use pdfcer_gui_base::notepopupopen as open;

/// **Everything in the window that CHANGES something** — the note
/// editor's controls, *Delete comment*, and the `/Open` write-back.
mod controls;

use egui::{Pos2, Rect};
use pdfcer_core::object::ObjId;

use crate::app::actions::Action;
use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::panels::comments::note::NoteDraft;

use self::controls::{controls, save_draft};
use crate::text::annotpopup as t;
use crate::text::panels::comments as tp;

use self::model::NoteView;

/// **The region an open pop-up publishes**, for the first open pop-up and only
/// that one.
pub const REGION_POPUP: &str = "notepopup.window"; // ui-text-exempt: trace region name, never displayed
/// The region an open pop-up's close control publishes.
pub const REGION_CLOSE: &str = "notepopup.close"; // ui-text-exempt: trace region name, never displayed
/// The region the *Add note* / *Edit note* control publishes.
pub const REGION_EDIT: &str = "notepopup.edit"; // ui-text-exempt: trace region name, never displayed
/// The region the open editor's text box publishes.
pub const REGION_BOX: &str = "notepopup.box"; // ui-text-exempt: trace region name, never displayed
/// The region the open editor's *Save note* publishes.
pub const REGION_SAVE: &str = "notepopup.save"; // ui-text-exempt: trace region name, never displayed
/// The region *Delete comment* publishes.
pub const REGION_DELETE: &str = "notepopup.delete"; // ui-text-exempt: trace region name, never displayed
/// The region the *Open by default* control publishes — the one control in
/// this window whose effect is **in the file** and invisible on screen.
pub const REGION_OPEN_DEFAULT: &str = "notepopup.open_default"; // ui-text-exempt: trace region name, never displayed

/// The pop-up's width, in **screen points**.
const POPUP_WIDTH: f32 = 260.0;

/// The tallest a pop-up's body may grow before it scrolls, in screen points.
const POPUP_MAX_BODY: f32 = 220.0;

/// How far a pop-up sits from its note when the file gives no `/Popup` rect.
const POPUP_GAP: f32 = 8.0;

/// **Draw every open pop-up, and the hover tooltip.**
pub fn show(ctx: &egui::Context, doc: &OpenDoc, caps: Capabilities, actions: &mut Vec<Action>) {
    // This frame's map and viewport, published by `canvas::present` before it
    // called `interact`. `None` on a frame with no canvas at all — no
    // document, or a render failure that replaced the strip with a sentence —
    // in which case there is nothing to hang a pop-up off and nothing to draw.
    let Some(frame) = crate::canvas::zoom::last_frame(ctx) else {
        return;
    };
    let page_index = doc.view.page_index;
    let Some(page) = doc.pages.get(page_index) else {
        return;
    };
    // Read the SESSION, not the file on disk — the same rule
    // `crate::panels::comments` states in its header. An operator who has just
    // typed a note must see it in the window they typed it in, without saving.
    let view = doc.session.view();
    let notes = model::notes_on(&view, page);
    if notes.is_empty() {
        return;
    }

    let overrides = open::load(ctx, &doc.path);
    let ce_dimensions = crate::panels::comments::model::ce_dimension_annots(&doc.session);
    let mut drawn = 0_usize;
    let mut from_file = 0_usize;
    // The draft is loaded ONCE and stored back once, rather than read and
    // written per pop-up. Two pop-ups cannot be edited at the same time — the
    // draft names one annotation — and a load-per-window would let the second
    // one see the state the first had already written this frame.
    let mut draft = load_draft(ctx, &doc.path);
    draft.sync(doc.edit_epoch);

    for note in &notes {
        if !overrides.is_open(note.id, note.authored_open) {
            continue;
        }
        if note.authored_open && !overrides.touched(note.id) {
            from_file += 1;
        }
        let published = drawn == 0;
        drawn += 1;
        popup(
            ctx,
            &Ctx {
                doc,
                caps,
                page_index,
                map: &frame.map,
                clip: frame.viewport_rect,
                is_ce_dimension: ce_dimensions.contains(&note.id),
                published,
            },
            note,
            &mut draft,
            actions,
        );
    }
    store_draft(ctx, &doc.path, &draft);

    let tipped = tooltip(ctx, &notes, &overrides, &frame.map, frame.viewport_rect);

    trace(&notes, drawn, from_file, tipped);
}

/// Everything one pop-up needs that is the same for all of them.
struct Ctx<'a> {
    doc: &'a OpenDoc,
    caps: Capabilities,
    page_index: usize,
    map: &'a PageMapping,
    clip: Rect,
    /// Whether this note is a **ce dimension** — rule 15. Decided once per
    /// frame in [`show`] against the `/PieceInfo` sidecar, never per pop-up:
    /// `ce_dimension_annots` walks the catalog and deserializes it, and asking
    /// it per window would make the surface O(windows × sidecar).
    is_ce_dimension: bool,
    /// Whether this pop-up is the one that publishes [`REGION_POPUP`].
    published: bool,
}

/// Draw one pop-up.
fn popup(
    ctx: &egui::Context,
    f: &Ctx<'_>,
    note: &NoteView,
    draft: &mut NoteDraft,
    actions: &mut Vec<Action>,
) {
    let origin = popup_origin(note, f.map, f.clip);
    let area = egui::Area::new(egui::Id::new(("pdfcer-note-popup", note.id))) // ui-text-exempt: internal widget id, never displayed
        // `Middle` is egui's own layer for windows, which puts this above
        // the page raster and below tooltips and menus. Not `Foreground`: a
        // context menu opened over a pop-up must still be on top of it, and
        // `Foreground` would invert that.
        .order(egui::Order::Middle)
        .fixed_pos(origin)
        // Constrained to the CANVAS viewport, not to the window. A pop-up
        // for a note near the right edge of a sheet would otherwise slide out
        // over the docked panels, where it would look like a panel with no
        // title. See `REGION_POPUP` on why this is also what makes the driven
        // check able to fail.
        .constrain_to(f.clip)
        .show(ctx, |ui| {
            ui.set_max_width(POPUP_WIDTH);
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(POPUP_WIDTH);
                body(ui, f, note, draft, actions);
            });
        });
    if f.published {
        crate::diag::ui_rect_visible(REGION_POPUP, area.response.rect, f.clip);
    }
}

/// The pop-up's **outer** width — the box `Area::constrain_to` has to fit —
/// as distinct from [`POPUP_WIDTH`], which is the width of its *contents*.
const POPUP_BOX_WIDTH: f32 = POPUP_WIDTH + 16.0;

/// **Where a pop-up's top-left corner goes**, in screen space.
fn popup_origin(note: &NoteView, map: &PageMapping, clip: Rect) -> Pos2 {
    let anchor = map.rect_to_screen(note.anchor);
    let preferred = note.popup.and_then(|p| p.rect).map_or_else(
        || Pos2::new(anchor.max.x + POPUP_GAP, anchor.min.y),
        |rect| map.rect_to_screen(rect).min,
    );
    let raw = clear_of_anchor(preferred, anchor, clip);
    // Grown by the width so a window whose origin is just off the right edge
    // is not slammed to the left edge. `constrain_to` does the real work.
    let room = clip.expand2(egui::vec2(POPUP_WIDTH, POPUP_MAX_BODY));
    Pos2::new(
        raw.x.clamp(room.min.x, room.max.x),
        raw.y.clamp(room.min.y, room.max.y),
    )
}

/// Move `preferred` off `anchor` if the window it implies would cover it.
fn clear_of_anchor(preferred: Pos2, anchor: Rect, clip: Rect) -> Pos2 {
    // **An anchor that is not on screen cannot be covered**, and the
    // invariant is about a visible one. A note scrolled out of the viewport, or
    // one at 300,000 % zoom whose rect maps into the millions, gets its
    // preferred origin unchanged — because the placement that matters for it is
    // the *direction* it lies in, which `a_note_far_off_screen_is_clamped_
    // towards_itself` asserts and which pinning the window into the viewport
    // would destroy. Two rules, and this one comes first because the other has
    // nothing to protect here.
    //
    // The intersection, not the anchor, is what the rest of this function
    // separates from: for a mark half off the left edge, the half an operator
    // can actually press on is the half inside the clip, and reserving room
    // beside the part they cannot see would push the window off the other side
    // for nothing.
    let anchor = anchor.intersect(clip);
    if !anchor.is_positive() {
        return preferred;
    }
    // A window's x-range, given its left edge. Two windows that do not
    // overlap horizontally cannot overlap at all, whatever their heights.
    let covers_x = |x: f32| x < anchor.max.x && x + POPUP_BOX_WIDTH > anchor.min.x;
    // …and it is not enough to clear the anchor: the window must also FIT,
    // because a window that does not is one `Area::constrain_to` will SLIDE —
    // and sliding is the whole defect. `beside` puts the origin to the right of
    // the note, which clears it by construction and then, on any note within a
    // pop-up's width of the right edge, gets pushed straight back on top of it.
    // Testing `covers_x` alone accepts exactly those placements — origin
    // x = 768 in a viewport ending at 772 — which is what
    // `a_note_against_the_right_edge_puts_its_window_on_the_left` forbids.
    let fits = |x: f32| x >= clip.min.x && x + POPUP_BOX_WIDTH <= clip.max.x;
    let usable = |x: f32| !covers_x(x) && fits(x);
    // 1 — the preferred origin, when it already clears the anchor and fits.
    if usable(preferred.x) {
        return preferred;
    }
    // 2 — flip to the LEFT of the note, right edge against its left edge.
    let left = anchor.min.x - POPUP_GAP - POPUP_BOX_WIDTH;
    if usable(left) {
        return Pos2::new(left, preferred.y);
    }
    // …and the right, which candidate 1 reaches only when the file supplied an
    // origin of its own or when the right-hand placement did not fit. Written
    // out rather than assumed away: with no `/Popup` rect `preferred` IS this
    // position, so the arm is a no-op there and a real choice otherwise.
    let right = anchor.max.x + POPUP_GAP;
    if usable(right) {
        return Pos2::new(right, preferred.y);
    }
    // 3 — no horizontal separation is available. Go under the note, or over
    // it, whichever side of the anchor has more room. `x` is pinned into the
    // viewport so the window is not immediately slid back by `constrain_to`.
    let x = preferred
        .x
        .min(clip.max.x - POPUP_BOX_WIDTH)
        .max(clip.min.x);
    let below = clip.max.y - anchor.max.y;
    let above = anchor.min.y - clip.min.y;
    if below >= above {
        Pos2::new(x, anchor.max.y + POPUP_GAP)
    } else {
        // The **top of the viewport**, not "the anchor's top minus the window's
        // height", and the difference is the header's rule: the height is
        // decided by the note's own words and must not feed back into the
        // note's position. Starting at the top uses every point of the room
        // there is; when the window is shorter than that room it clears the
        // anchor, and when it is longer nothing could have.
        Pos2::new(x, clip.min.y)
    }
}

/// The pop-up's contents: the title row, the byline, the note, the thread and
/// the controls.
fn body(
    ui: &mut egui::Ui,
    f: &Ctx<'_>,
    note: &NoteView,
    draft: &mut NoteDraft,
    actions: &mut Vec<Action>,
) {
    // ---- the title row ------------------------------------------------
    let heading = if f.is_ce_dimension {
        t::popup_ce_dimension_heading(&note.subtype)
    } else {
        t::popup_heading(&note.subtype)
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(heading));
        // Right-aligned close. `with_layout` rather than a spacer, because a
        // spacer sized from the heading's width would move when a subtype name
        // changed length.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close = ui
                .button(t::popup_close())
                .on_hover_text(t::popup_close_tooltip());
            crate::diag::ui_rect_visible(REGION_CLOSE, close.rect, f.clip);
            if close.clicked() {
                open::set(ui.ctx(), &f.doc.path, note.id, false);
                draft.close();
            }
        });
    });

    // ---- who and when -------------------------------------------------
    //
    // `comment_row_byline` is CALLED rather than copied, and that is the
    // point: it carries a settled ruling about `/M` being shown verbatim
    // (§12.5.2 makes it *"date or text string"* and requires a reader to
    // accept any format). Two surfaces showing one comment must not show two
    // different dates for it, and one function is the only way that cannot
    // happen.
    if let Some(byline) = tp::comment_row_byline(note.author.as_deref(), note.modified.as_deref()) {
        let resp = ui.label(egui::RichText::new(byline).small().weak());
        if note.modified.is_some() {
            resp.on_hover_text(tp::comment_row_modified_tooltip());
        }
    }
    ui.separator();

    // ---- the words -----------------------------------------------------
    let existing = note.contents.as_deref().unwrap_or_default();
    let editing = draft.editing(note.id, f.doc.edit_epoch);
    egui::ScrollArea::vertical()
        .id_salt(("pdfcer-note-popup-body", note.id)) // ui-text-exempt: internal widget id, never displayed
        .max_height(POPUP_MAX_BODY)
        .show(ui, |ui| {
            if editing {
                let response = ui.add(
                    // escape-disposition: commits — `save_draft` on the `lost_focus` plus
                    // Escape pair a dozen lines down. *Cancel* is the deliberate discard.
                    egui::TextEdit::multiline(draft.text_mut())
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                );
                crate::diag::ui_rect_visible(REGION_BOX, response.rect, f.clip);
                // Escape closes the editor, and it does so through egui rather
                // than by reading the keyboard: a `TextEdit` surrenders focus
                // on Escape, so `lost_focus()` plus the key is the idiomatic
                // test — and, importantly here, it asks nothing about whether
                // "the operator is typing". A canvas surface that read the raw
                // key would be a second claimant on a key the caret and the
                // tool arming both want, which is the class of bug
                // `tools/gates/check-typing-guard.sh` exists for.
                //
                // It SAVES rather than discards, for the reason
                // [`controls::save_draft`] carries in full; *Cancel* below is
                // the deliberate discard and is unchanged.
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    save_draft(f, note, draft, actions);
                }
                ui.label(egui::RichText::new(t::popup_note_hint()).small().weak());
            } else if existing.trim().is_empty() {
                let caption = if f.is_ce_dimension {
                    t::popup_ce_dimension_note()
                } else {
                    t::popup_no_note()
                };
                ui.label(egui::RichText::new(caption).small().weak());
            } else {
                ui.label(t::popup_body(existing));
                if f.is_ce_dimension {
                    ui.label(
                        egui::RichText::new(t::popup_ce_dimension_note())
                            .small()
                            .weak(),
                    );
                }
            }
            thread(ui, f, note);
        });

    // ---- the controls ---------------------------------------------------
    ui.separator();
    controls(ui, f, note, draft, existing, actions);
}

/// The replies hanging off this comment, read from `/IRT`.
fn thread(ui: &mut egui::Ui, f: &Ctx<'_>, note: &NoteView) {
    let view = f.doc.session.view();
    let replies = model::replies_to(&view, &f.doc.pages, note.id);
    if replies.is_empty() {
        return;
    }
    ui.separator();
    ui.label(
        egui::RichText::new(t::popup_replies(replies.len()))
            .small()
            .weak(),
    );
    for reply in &replies {
        // Indented, because a thread is a hierarchy and the indent is what
        // says so without a glyph. `crate::panels::bookmarks` indents its
        // levels the same way and for the same reason.
        ui.indent(("pdfcer-note-reply", reply.id), |ui| {
            // ui-text-exempt: internal widget id, never displayed
            if let Some(byline) =
                tp::comment_row_byline(reply.author.as_deref(), reply.modified.as_deref())
            {
                ui.label(egui::RichText::new(byline).small().weak());
            }
            match reply.contents.as_deref().map(str::trim) {
                Some(text) if !text.is_empty() => {
                    ui.label(t::popup_body(text));
                }
                _ => {
                    ui.label(egui::RichText::new(t::popup_reply_no_note()).small().weak());
                }
            }
            // Rule 4: pdfcer is showing a string §12.5.6.2 tells a conforming
            // reader to ignore in favour of the group primary's. Another
            // reader legitimately shows something else, so this says so.
            if reply.group_member {
                ui.label(
                    egui::RichText::new(t::popup_reply_is_group_member())
                        .small()
                        .weak(),
                );
            }
        });
    }
}

/// **The hover tooltip** — the cheap half of the same affordance.
fn tooltip(
    ctx: &egui::Context,
    notes: &[NoteView],
    overrides: &open::Overrides,
    map: &PageMapping,
    clip: Rect,
) -> bool {
    let Some(screen) = ctx.pointer_latest_pos() else {
        return false;
    };
    if !clip.contains(screen) {
        return false;
    }
    // Never over a pop-up. `layer_id_at` is egui's own answer to *"what is
    // under the pointer"*, so this cannot disagree with what will actually
    // receive the click — a hand-rolled rectangle test over the open windows
    // could, and would be a second claimant on the same question.
    if ctx
        .layer_id_at(screen)
        .is_some_and(|layer| layer.order > egui::Order::Background)
    {
        return false;
    }
    #[allow(clippy::cast_possible_truncation)]
    let tolerance = map.tolerance() as f32;
    let Some(note) = model::under(notes, map.to_page(screen), tolerance) else {
        return false;
    };
    if overrides.is_open(note.id, note.authored_open) {
        return false;
    }

    let text = t::popup_tooltip(note.author.as_deref(), note.contents.as_deref());
    egui::Area::new(egui::Id::new("pdfcer-note-tooltip")) // ui-text-exempt: internal widget id, never displayed
        .order(egui::Order::Tooltip)
        // Below and right of the pointer, which is where every tooltip in the
        // platform sits — above it would be under the operator's own hand on a
        // pen display, and the class convention exists for that reason.
        .fixed_pos(screen + egui::vec2(TOOLTIP_OFFSET, TOOLTIP_OFFSET))
        .constrain_to(clip)
        .interactable(false)
        .show(ctx, |ui| {
            ui.set_max_width(POPUP_WIDTH);
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(POPUP_WIDTH);
                ui.label(text);
            });
        });
    true
}

/// How far a tooltip sits from the pointer, in screen points.
///
/// Sixteen — clear of a standard cursor bitmap in both axes, so the tooltip
/// never appears *under* the arrow that summoned it.
const TOOLTIP_OFFSET: f32 = 16.0;

/// **A click landed on the canvas — open or close a note's pop-up.**
pub fn clicked_on(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page_index: usize,
    point: Pos2,
    map: &PageMapping,
) -> Option<ObjId> {
    let page = doc.pages.get(page_index)?;
    let view = doc.session.view();
    let notes = model::notes_on(&view, page);
    #[allow(clippy::cast_possible_truncation)]
    let tolerance = map.tolerance() as f32;
    let note = model::under(&notes, point, tolerance)?;
    // **A mark with nothing to say does not open a window.** The press
    // falls through to selection, which is what the operator meant by clicking
    // a cloud that carries no comment. `model::under` answers for every
    // annotation that *can* carry a note; `has_something_to_read` is the one
    // that asks whether this one *does*. See its doc comment for why a sticky
    // note is exempt (an empty one is still a note, and is how you write in it)
    // and why a byline alone is not enough.
    //
    // ⚠ Asked HERE and not inside `model::under`, deliberately. `under` is also
    // the painter's answer to *"which note is the pointer over"*, and the same
    // list feeds the hover tooltip; narrowing it there would make a commentless
    // cloud stop reporting itself to the trace as well, and the harness would
    // lose the only evidence that the pointer was over anything at all.
    if !model::has_something_to_read(note) {
        return None;
    }
    // TOGGLE, not open. Clicking the icon again closes the window, which is
    // what every reader in the class does and what an operator who has just
    // opened one by accident will try.
    let overrides = open::load(ctx, &doc.path);
    let now = overrides.is_open(note.id, note.authored_open);
    open::set(ctx, &doc.path, note.id, !now);
    Some(note.id)
}

/// One `note-popup` line per frame that has something to say.
fn trace(notes: &[NoteView], open_count: usize, from_file: usize, tipped: bool) {
    if open_count == 0 && !tipped {
        return;
    }
    crate::diag::trace(|| {
        let with_note = notes
            .iter()
            .filter(|n| n.contents.as_deref().is_some_and(|c| !c.trim().is_empty()))
            .count();
        let authored_open = notes.iter().filter(|n| n.authored_open).count();
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "note-popup notes={} with_note={with_note} authored_open={authored_open} \
             open={open_count} from_file={from_file} tooltip={tipped}",
            notes.len(),
        )
    });
}

/// The egui id this document's canvas note draft is stored under.
fn draft_key(path: &std::path::Path) -> egui::Id {
    egui::Id::new(("pdfcer-note-popup-draft", path)) // ui-text-exempt: internal widget id, never displayed
}

/// Read the canvas pop-up's note draft.
fn load_draft(ctx: &egui::Context, path: &std::path::Path) -> NoteDraft {
    ctx.data(|d| d.get_temp::<NoteDraft>(draft_key(path)).unwrap_or_default())
}

/// Store the canvas pop-up's note draft.
fn store_draft(ctx: &egui::Context, path: &std::path::Path, draft: &NoteDraft) {
    ctx.data_mut(|d| d.insert_temp(draft_key(path), draft.clone()));
}

#[cfg(test)]
mod placement_tests {
    use super::*;

    /// A measured canvas viewport: the central panel with a dock on the
    /// right, `[[288 174] - [772 758]]`.
    const CLIP: Rect = Rect {
        min: Pos2::new(288.0, 174.0),
        max: Pos2::new(772.0, 758.0),
    };

    /// The rectangle the pop-up would occupy, given the origin under test.
    fn window(origin: Pos2, height: f32) -> Rect {
        Rect::from_min_size(origin, egui::vec2(POPUP_BOX_WIDTH, height))
    }

    /// **The overlap that swallows a drag, as an assertion.**
    #[test]
    fn the_popup_that_ate_the_drag_is_placed_clear_of_its_annotation() {
        let anchor = Rect::from_min_max(Pos2::new(464.0, 464.5), Pos2::new(551.7, 550.2));
        let preferred = Pos2::new(anchor.max.x + POPUP_GAP, anchor.min.y);
        let origin = clear_of_anchor(preferred, anchor, CLIP);
        // The measured window was 100 pt tall for a note with no words.
        let drawn = window(origin, 100.0);
        assert!(
            !drawn.intersects(anchor),
            "the pop-up at {drawn:?} still covers the annotation at {anchor:?} — this is the \
             state in which a markup could be rotated and could not be moved or resized"
        );
        // 290.5 pt above the note against 207.8 below it, so the rule the
        // header states — the side of the anchor with more room — puts the
        // window above. Asserted as the *rule*, not as "above": a viewport a
        // hundred points taller would legitimately answer the other way, and a
        // test that pinned the direction would fail on a resize rather than on
        // a regression.
        let (above, below) = (anchor.min.y - CLIP.min.y, CLIP.max.y - anchor.max.y);
        assert!(
            if below >= above {
                origin.y > anchor.max.y
            } else {
                origin.y == CLIP.min.y
            },
            "the window went to the side with LESS room: {above:.1} pt above the note, \
             {below:.1} pt below it, and the origin is {origin:?}"
        );
    }

    /// **Candidate 1: a preference that already clears the anchor is kept.**
    #[test]
    fn a_note_with_room_beside_it_keeps_the_placement_it_always_had() {
        let anchor = Rect::from_min_max(Pos2::new(300.0, 300.0), Pos2::new(360.0, 340.0));
        let preferred = Pos2::new(anchor.max.x + POPUP_GAP, anchor.min.y);
        assert_eq!(
            clear_of_anchor(preferred, anchor, CLIP),
            preferred,
            "the right-hand placement fits here, so nothing may be second-guessed"
        );
    }

    /// **Candidate 2: no room on the right, and the flip is to the LEFT —
    /// not a slide.**
    ///
    #[test]
    fn a_note_against_the_right_edge_puts_its_window_on_the_left() {
        let anchor = Rect::from_min_max(Pos2::new(700.0, 300.0), Pos2::new(760.0, 340.0));
        let preferred = Pos2::new(anchor.max.x + POPUP_GAP, anchor.min.y);
        let origin = clear_of_anchor(preferred, anchor, CLIP);
        assert!(
            origin.x + POPUP_BOX_WIDTH <= anchor.min.x,
            "the window at x={} is not clear of an anchor beginning at x={}",
            origin.x,
            anchor.min.x
        );
        assert!(
            origin.x >= CLIP.min.x,
            "…and it must still be inside the canvas viewport"
        );
        assert!(
            !window(origin, 400.0).intersects(anchor),
            "a horizontal separation has to hold for ANY height — that is why the first two \
             candidates are decided on x alone"
        );
    }

    /// **Candidate 3 the other way up: more room above than below.**
    #[test]
    fn a_wide_note_low_on_the_sheet_puts_its_window_above() {
        let anchor = Rect::from_min_max(Pos2::new(300.0, 700.0), Pos2::new(760.0, 740.0));
        let preferred = Pos2::new(anchor.max.x + POPUP_GAP, anchor.min.y);
        let origin = clear_of_anchor(preferred, anchor, CLIP);
        assert_eq!(
            origin.y,
            CLIP.min.y,
            "there are {} pt above this note and {} pt below it, so above is the side with room",
            anchor.min.y - CLIP.min.y,
            CLIP.max.y - anchor.max.y
        );
        assert!(
            origin.x >= CLIP.min.x && origin.x + POPUP_BOX_WIDTH <= CLIP.max.x,
            "the x is pinned into the viewport so `constrain_to` has nothing left to slide"
        );
    }

    /// **The file's `/Popup` rectangle is honoured — until it overlaps.**
    #[test]
    fn a_producers_popup_rectangle_is_kept_unless_it_covers_the_note() {
        let anchor = Rect::from_min_max(Pos2::new(400.0, 300.0), Pos2::new(440.0, 340.0));
        let beside = Pos2::new(460.0, 290.0);
        assert_eq!(
            clear_of_anchor(beside, anchor, CLIP),
            beside,
            "a `/Popup` rect that clears its note is the producer's statement and stands"
        );
        let over = Pos2::new(410.0, 305.0);
        let moved = clear_of_anchor(over, anchor, CLIP);
        assert_ne!(
            over, moved,
            "a `/Popup` rect laid over its own note is overruled"
        );
        assert!(
            !window(moved, 100.0).intersects(anchor),
            "…and the replacement clears it"
        );
    }

    /// **The outer box is wider than the contents, and the gap depends on it.**
    const _: () = assert!(
        POPUP_BOX_WIDTH > POPUP_WIDTH,
        "the box a pop-up occupies is its contents plus `Frame::popup`'s margin and stroke"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The region names the sweep below checks.
    const REGIONS: &[&str] = &[
        REGION_POPUP,
        REGION_CLOSE,
        REGION_EDIT,
        REGION_BOX,
        REGION_SAVE,
        REGION_DELETE,
    ];

    /// **Every region name is unique and namespaced to this module.**
    #[test]
    fn the_region_names_are_unique_and_namespaced() {
        let mut seen = std::collections::BTreeSet::new();
        for name in REGIONS {
            assert!(
                name.starts_with("notepopup."),
                "`{name}` is not namespaced to this module, so it can collide \
                 with a region published somewhere else"
            );
            assert!(seen.insert(*name), "`{name}` is published twice");
        }
    }

    /// A note occupying `anchor` in canvas space, carrying `popup` as its
    /// companion rectangle.
    fn note(anchor: Rect, popup: Option<Rect>) -> NoteView {
        NoteView {
            id: ObjId::new(7, 0),
            subtype: "Text".to_owned(),
            contents: Some("words".to_owned()),
            author: None,
            modified: None,
            anchor,
            popup: popup.map(|rect| model::PopupBox {
                id: ObjId::new(8, 0),
                rect: Some(rect),
            }),
            authored_open: false,
            locked: false,
            in_reply_to: None,
        }
    }

    /// A canvas 800 × 600 points at 100 %, with the page drawn at the origin —
    /// the identity mapping, so a canvas coordinate is a screen coordinate and
    /// the arithmetic under test is visible rather than buried in a projection.
    fn identity_map() -> PageMapping {
        let image = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        PageMapping::new(image, (800.0, 600.0), 1.0)
    }

    fn viewport() -> Rect {
        Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))
    }

    /// **With no `/Popup` rect, the window goes BESIDE the note — never
    /// over it.**
    #[test]
    fn a_note_with_no_popup_rect_opens_beside_itself() {
        let anchor = Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(20.0, 20.0));
        let at = popup_origin(&note(anchor, None), &identity_map(), viewport());
        assert!(
            at.x > anchor.max.x,
            "the pop-up starts at x={} and the note ends at x={} — it would be \
             drawn over the mark that opened it",
            at.x,
            anchor.max.x
        );
        assert!(
            (at.y - anchor.min.y).abs() < 1.0,
            "top-aligned with the note, not floating above or below it: y={}",
            at.y
        );
    }

    /// **The file's own `/Popup` rectangle wins.**
    #[test]
    fn the_files_own_popup_rectangle_is_honoured() {
        let anchor = Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(20.0, 20.0));
        let placed = Rect::from_min_size(Pos2::new(400.0, 300.0), egui::vec2(150.0, 108.0));
        let at = popup_origin(&note(anchor, Some(placed)), &identity_map(), viewport());
        assert_eq!(
            at, placed.min,
            "the pop-up ignored the placement in the file"
        );
    }

    /// **A note far off the viewport does not put its window in the
    /// opposite corner.**
    #[test]
    fn a_note_far_off_screen_is_clamped_towards_itself() {
        let far = Rect::from_min_size(Pos2::new(9.0e6, -9.0e6), egui::vec2(20.0, 20.0));
        let at = popup_origin(&note(far, None), &identity_map(), viewport());
        assert!(at.x.is_finite() && at.y.is_finite(), "{at:?}");
        // Right of the viewport (the note is far right) and above it (the note
        // is far above) — the direction is preserved, which is the whole point.
        assert!(at.x >= viewport().max.x, "{at:?}");
        assert!(at.y <= viewport().min.y, "{at:?}");
        // …and bounded, rather than the raw millions that would defeat
        // `constrain_to`.
        assert!(at.x <= viewport().max.x + POPUP_WIDTH, "{at:?}");
    }
}
