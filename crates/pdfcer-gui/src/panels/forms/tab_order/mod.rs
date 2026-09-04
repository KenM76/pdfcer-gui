//! # `panels::forms::tab_order` — the order this form is tabbed through, per
//! page, and the drag that changes it
//!
//! A **second list beside the fill list**, answering a different question. The
//! fill list ([`super::field_list`]) is in `/AcroForm` `/Fields` order because
//! that matches the printed form, and it is not changed by anything here. This
//! one is in each page's `/Annots` order, which is the order a viewer paints the
//! widgets in and — absent a `/Tabs` entry — the order it tabs through them in.
//!
//! [`model`] does the walking, the matching and the `/Tabs` reading, and carries
//! the whole argument: §1 why the order is `/Annots` order, §2 why it walks
//! `page_annotations` rather than `widget_rects`, §3 why `/P` is never
//! consulted, §4 the primary-source reading of `/Tabs` (including the finding
//! that it is **not** inheritable), §5 what is counted rather than listed. This
//! file is the drawing, the disclosures and the one action the view can raise.
//!
//! ## ★★★ It WAS read-only, and the history is the point
//!
//! This section used to be a prohibition. It said: no drag handles, no
//! `Sense::drag`, no up/down buttons, not even disabled ones — because
//! `pdfcer-core` had no verb that reordered a page's `/Annots`, and a handle
//! that cannot commit *"would teach the operator a gesture, let them perform
//! it, and then either do nothing or lie"*. It ended with a promise: **"when
//! the engine verb lands, the affordance arrives with it."**
//!
//! `EditSession::reorder_annotations` shipped on 2026-09-02, hours after the
//! request went out, and this is that promise being kept. The prohibition is
//! **superseded, not abandoned**, and it is left described rather than deleted
//! for two reasons: the reasoning is what should be applied to the *next* gap
//! (R9 has not changed), and a section that vanished would leave a reader
//! wondering whether anybody had thought about it.
//!
//! ## What the drag is, and what it deliberately is not
//!
//! [`drag`] holds the gesture and carries the argument in full. In outline:
//!
//! - **A row is a drag source; the marker is an insertion caret**, the same one
//!   `crate::panels::pages` draws for the page rail, because the operator named
//!   that panel as the reference — *"like we can with pages in the page
//!   preview"*.
//! - **Widgets move among widget slots and nothing else moves at all.** The
//!   list is form fields; the array also holds every `/Link` and markup
//!   annotation on the page, and `/Annots` order is *paint* order, so moving
//!   one would silently change what is drawn on top of what.
//! - **It does not write `/Tabs`.** Sourced, and counter-intuitive enough to
//!   restate: `/Tabs /A` is PDF 2.0 only, and PDF/UA-1 §7.18.3 *requires*
//!   `/Tabs /S`, so writing `/A` as a side effect of a drag would make a
//!   conforming file non-conforming with nobody asking. Acrobat's own manual
//!   tab order is an `/Annots` permutation with no `/Tabs` written.
//!
//! ★★ Which means the per-page `/Tabs` sentence this view has always shown is
//! now doing a second job. On a page whose `/Tabs` says `/S` or `/R` or `/C`,
//! a drag changes the array and **a conforming reader may still tab in the
//! order the file states**. The sentence is what stops that reading as a bug in
//! the drag. It stays above the rows, unconditional, on every page.
//!
//! ## Actions, not mutations — and it raises exactly one
//!
//! [`Action::GoToPage`], from a page heading's **Go to** control, exactly as
//! `crate::panels::bookmarks` and `crate::panels::comments` do. That is
//! navigation, not authoring: it moves the view, never the document. The body is
//! handed `&OpenDoc` — a **shared** reference, so this is a compile-time fact
//! and not a convention.
//!
//! ## Rule 4: this is disclosure, and it draws nothing on the page
//!
//! Not one pixel on the canvas. No numbered badge over each widget, no
//! highlight on the row under the pointer, no arrows between fields. The
//! one-line test is *would a screenshot of the editing canvas differ from a
//! screenshot of the same document saved and reopened?* — and the answer here
//! must stay "no".
//!
//! It is worth naming what rule 4 would *permit*, so nobody later reads its
//! absence as a prohibition: a hover highlight on the widget belonging to the
//! row under the pointer is the fourth clause's *"a snap indicator, a hover
//! highlight, a rubber-band, a selection handle — these are the cursor"*, and it
//! would be a genuinely good affordance here. It is not built for the same
//! reason `super`'s header gives for the fill rows: the panel→canvas channel
//! for *which row is hovered* does not exist in this build, and `crate::canvas`
//! is not this module's to extend.
//!
//! ## Where it sits, and why it is a section rather than a panel
//!
//! Inside the existing Forms panel, between the whole-form controls and the fill
//! list, as a collapsing section that is **closed by default**. Three reasons,
//! in order of weight:
//!
//! 1. **It needs nothing from the shell owner.** A new panel needs a `Panel`
//!    variant, a command id, a registry entry, a manifest reference, a RON
//!    regeneration and a mode-arrangement change — six files this work may not
//!    touch, any one of which missing produces the unreachable panel
//!    `crate::panels`' header is about.
//! 2. **It is about the same subject as the panel it is in.** An operator
//!    asking "what order does this form tab in?" is already looking at the form.
//! 3. **Closed by default** because it answers an occasional question, and
//!    because the panel's primary job is filling.
//!
//! ## The two layout rules, and which one applies
//!
//! 1. **Scrollbars must be visible.** `crate::panels::scroll_style` is applied
//!    by `crate::panels::Panel::show` before any body runs, and this section
//!    inherits it through the `Ui` it is handed.
//! 2. **A fixed-size child inside a scroll area needs the container's width
//!    stated.** ★ **There is no fixed-size child here**, so
//!    `crate::panels::content_width` is deliberately not called — stated rather
//!    than left to look like an omission, because skipping it silently is
//!    exactly how the Objects panel shipped clipped rows. Every child is a
//!    `Label`, which wraps to whatever width it is given; the only fixed-width
//!    child is the **Go to** button, at a couple of dozen points against a dock
//!    that opens at 320. `crate::panels::comments` records the same reasoning
//!    for the same shape.
//!
//! ### ★ The one layout thing this section does that no panel body does
//!
//! Its list is inside a scroll area with a **stated maximum height**
//! ([`MAX_LIST_HEIGHT`]). Every other list in this crate is the last thing in
//! its panel and takes whatever vertical space is left. This one is *not* last —
//! the fill list is below it — and the Forms panel's top level does not scroll,
//! so an unbounded list here would push the fill list off the bottom of a
//! non-scrolling container, where nothing would indicate it had gone. That is
//! the same class of defect as the invisible scrollbar: content clipped at a
//! container edge with nothing to say so.
//!
//! ## `PDFCER_DIAG` proves what this computed
//!
//! One `forms-tab-order` summary line plus one `forms-tab-page` line per page —
//! carrying the page's `/Tabs` state, where it was found, and every count — and
//! then one `forms-tab-row` line per row, capped. Written whenever the **panel**
//! draws, not only when the section is open, so the order is provable from a
//! trace without anyone having to click anything.
//!
//! That is `HANDOFF.md` §2 applied to a surface whose correctness is entirely
//! sequence and arithmetic: a screenshot of this list cannot tell you that the
//! numbering skipped a widget the file lists, that a `/Tabs` came from an
//! ancestor rather than the page, or that four annotations on the page are in
//! the tab sequence and not in the list. Every one of those is in the trace.

/// Turning a document into a per-page widget sequence — the classification,
/// testable without a `Ui`.
mod drag;
pub mod model;
/// The rows that register an unclaimed widget back into the form.
mod register;
pub mod tabs;

use pdfcer_core::forms::AcroForm;
use pdfcer_core::view::DocumentView;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::forms as t;

use self::model::{Listing, PageTabs};
use self::tabs::{TabsEntry, TabsMode};

/// The tallest the list may grow before it scrolls, in points.
///
/// See this module's header, "★ The one layout thing this section does". The
/// number is a judgement rather than a measurement: tall enough that a page of
/// eight or nine widgets is read without scrolling, short enough that the fill
/// list below stays on screen in a dock pane opened at its default height.
const MAX_LIST_HEIGHT: f32 = 260.0;

/// How many rows the trace will print before it stops.
///
/// `pdfcer_core::forms::MAX_FORM_FIELDS` is 500,000, so an uncapped per-row
/// census on a pathological form would bury every other line in the capture —
/// the same failure `crate::canvas::forms` caps its `form-box` census for. The
/// summary lines are never capped, so the *counts* stay provable even when the
/// enumeration stops.
/// The region the collapsing header publishes, so a driven check can open it.
const REGION_HEADER: &str = "forms.tab_order.header"; // ui-text-exempt: trace region name, never displayed

/// The trace slot for whether the section is expanded.
// ui-text-exempt: trace slot name, never displayed
const OPEN_SLOT: &str = "forms-tab-order-open";

const MAX_TRACED_ROWS: usize = 200;

/// Draw the Tab order section.
///
/// Called from [`super::body`] with the `/AcroForm` it has already parsed and
/// the `DocumentView` it already holds — neither is re-derived here, because two
/// parses of one form per frame is a cost with no benefit and because a second
/// `parse_acroform` could in principle disagree with the first one the panel is
/// drawing from.
///
/// `actions` is pushed at most once, with [`Action::GoToPage`].
pub(super) fn section(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    view: &DocumentView<'_>,
    form: Option<&AcroForm>,
    actions: &mut Vec<Action>,
) {
    // `page_slots` rather than `doc.pages`, because a slot carries the
    // `ancestors` chain `model::page_tabs` reads and a `Page` does not.
    //
    // An error here is very close to unreachable — `doc.pages` came from the
    // same page tree in the same session, so the tree has already been walked
    // successfully once — and there is nothing honest to draw for it: this
    // section has no capability of its own to refuse, only a reading that could
    // not be made. So it renders nothing and says so in the trace, which is the
    // no-placeholders rule applied to a failure rather than to an absence.
    let Ok(slots) = doc.session.page_slots() else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "forms-tab-order pages=? error=page-tree-unreadable".to_owned()
        });
        return;
    };

    let listing = model::collect(view, &slots, form);
    trace(&listing);

    // ★★★ NOTHING TO ORDER, SO NO SECTION — R9, added 2026-09-03 on an outside
    // review.
    //
    // The reviewer's observation, on an ordinary CAD drawing: a *"Tab order"*
    // heading in the Fill-form panel that opens onto nothing. It did. This
    // header was drawn **unconditionally**, and the no-AcroForm path in
    // `panels::forms` calls this function with `form = None` — so on every
    // document with no form fields, which is nearly every drawing the operator
    // opens, the panel offered a disclosure triangle whose whole content was an
    // explainer about reordering an empty list.
    //
    // ★ R9 in its exact words: an unavailable capability renders **nothing**.
    // A heading is not exempt from that because it is cheap — a heading is a
    // claim that something is behind it, and this project's own no-placeholders
    // rule makes no distinction between an inert button and an inert expander.
    //
    // ★★ The irony worth recording: the sibling branch that reaches this
    // function already invokes R9 by name, three lines above the call, to
    // explain why it skips *everything else* on the no-fields path — and then
    // draws this. The rule was applied to the code the author was looking at
    // and not to the one function it delegated to. **A rule cited at a call
    // site is not enforced inside the callee.**
    //
    // The listing is empty rather than the form being `None`, deliberately: a
    // document can carry an `/AcroForm` whose fields are all unreachable, and
    // that is equally nothing to order. The condition is what the section would
    // SHOW, not what it was handed.
    // `total_rows` plus the orphan count, because both are things this section
    // would have had something to say about. Zero of each is the honest
    // "nothing to order" — and `fields_without_widgets` is deliberately part of
    // it: a document whose only form content is fields with no widgets has
    // nothing on any page to reorder, but it DOES have a disclosure this
    // section is the only place that makes.
    if listing.total_rows() == 0 && listing.fields_without_widgets == 0 {
        return;
    }

    let mut go: Option<usize> = None;
    // ★ The header publishes its own rectangle so a driven check can OPEN
    // the section. It ships closed on purpose — the section is a diagnostic
    // rather than the panel's main job — and a check that assumed it open would
    // report the whole feature missing on a correct build.
    let header = egui::CollapsingHeader::new(t::tab_order_heading())
        .id_salt("pdfcer-forms-tab-order")
        .default_open(false)
        .show(ui, |ui| {
            // ★ EVERY DISCLOSURE ABOVE THE LIST, without exception — the rule
            // four other panels follow, for one reason: an operator who reads a
            // short list and stops has drawn their conclusion by the time a
            // footnote would reach them.
            //
            // The order is by how much it changes what they should conclude:
            // what this list IS **and how to reorder it**, then how big it is,
            // then what is missing from it.
            //
            // ★ The first of those used to end "and that it changes nothing".
            // It now teaches the drag instead, which is the only thing that
            // can: there is no handle, no grip glyph and no button, so a
            // sentence is the entire discoverability surface for the gesture.
            // Pinned by
            // `tests::the_explainer_teaches_the_drag_and_no_longer_claims_to_be_read_only`.
            ui.label(t::tab_order_explainer());
            ui.label(t::tab_order_count(
                listing.pages.len(),
                listing.total_rows(),
            ));
            if listing.fields_without_widgets > 0 {
                ui.label(
                    egui::RichText::new(t::tab_order_fields_without_widgets(
                        listing.fields_without_widgets,
                    ))
                    .small()
                    .weak(),
                );
            }
            if listing.total_rows() == 0 {
                // Reachable on a document that HAS an `/AcroForm` full of
                // fields — none of whose widgets any page lists. Said out loud,
                // because an empty list under a populated fill list above reads
                // as a broken section.
                ui.label(t::tab_order_empty());
            }

            // ★★ The remedy, ABOVE the scroll area and above the list.
            //
            // Every unclaimed widget in the document, gathered here rather than
            // left inside the page block that counts it. The per-page sentence
            // is a fact about that page and stays there; the *action* is about
            // the document, and it has to be where the disclosure that sends
            // the operator here points.
            //
            // See `register::rows` for what the driven run found: on a 37-sheet
            // drawing with one inserted form page, rows drawn per page sat 36
            // page-blocks down a scroll area, and a click on the published
            // rectangle hit nothing at all.
            let unclaimed_total: usize = listing.pages.iter().map(|p| p.unclaimed.len()).sum();
            if unclaimed_total > 0 {
                ui.separator();
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    t::tab_order_unclaimed(unclaimed_total),
                );
                register::rows(ui, doc, &listing, actions);
            }
            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("pdfcer-forms-tab-order-rows")
                .max_height(MAX_LIST_HEIGHT)
                .show(ui, |ui| {
                    for page in &listing.pages {
                        // `push_id` per page, or two pages' **Go to** buttons
                        // share one egui id and the wrong one responds to a
                        // hover — the collision `crate::panels::comments` keys
                        // its rows against.
                        ui.push_id(page.page_index, |ui| {
                            page_block(ui, doc, page, &mut go, actions);
                        });
                        ui.separator();
                    }
                });
        });
    crate::diag::ui_rect(REGION_HEADER, header.header_response.rect);
    // ★★★ WHETHER THE SECTION IS OPEN, as its own trace line.
    //
    // Added 2026-09-02 after a driven check spent an afternoon unable to tell
    // three states apart, all of which look like "no rows on screen":
    //
    //   * the section is collapsed (it ships that way, deliberately);
    //   * it is open and its rows are clipped by a short docked pane;
    //   * it is open, on screen, and genuinely has no rows.
    //
    // A rectangle for the header answers none of them — the header is drawn in
    // all three. `openness` is `egui`'s own animated 0..=1, so `> 0.99` is
    // "fully open" rather than "mid-animation", which matters because a check
    // reads the trace one frame after clicking.
    crate::diag::trace_changed(OPEN_SLOT, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "forms-tab-order-open open={} rows_drawn={}",
            u8::from(header.openness > 0.99),
            if header.openness > 0.99 {
                listing.total_rows()
            } else {
                0
            }
        )
    });

    if let Some(page) = go {
        actions.push(Action::GoToPage(page));
    }
}

/// One page: its heading, what the file says about its tab order, its rows, and
/// what could not be listed.
///
/// A page with no widget on it is still drawn. Its `/Tabs` state is a fact about
/// the document, and a gap in the page numbering would read as a bug in the
/// view rather than as an empty page.
fn page_block(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    page: &PageTabs,
    go: &mut Option<usize>,
    actions: &mut Vec<Action>,
) {
    // 1-based **only here**, where a human reads it. The index travels 0-based
    // to `Action::GoToPage`; see
    // [`tests::the_page_index_travels_zero_based_and_prints_one_based`].
    let page_number = page.page_index + 1;

    // ★ A PLAIN LABEL, NOT `RichText::strong()`, and this is a measurement
    // rather than a preference.
    //
    // The first draft used `.strong()`, which is what every other heading in
    // `crate::panels` uses, and the running binary drew it **near-white on the
    // panel's light grey** — legible only if you knew it was there. `egui`'s
    // `RichText::strong()` resolves to `Visuals::strong_text_color()`, which is
    // `widgets.active.fg_stroke.color`; `egui_shell::theme` sets that to the
    // palette's `on_accent`, correctly, because the active state is the one
    // state whose background is the accent. The two decisions are each right
    // and their product is a near-invisible heading on any surface that is not
    // accent-filled.
    //
    // That is `DEFECTS.md` D2 in a new place — the same near-white-foreground
    // shape, arrived at from the other end — and it is **not this work's to
    // fix**: `egui-shell`'s theme is another territory, and the contrast gate
    // there checks foregrounds against the fills they are painted on, not
    // against the panel a `strong` label lands on. Reported rather than worked
    // around silently, because five other call sites in this crate have the
    // same line. Found only by driving the binary and looking at the pixels —
    // no test in this crate could have contradicted it.
    ui.label(t::tab_order_page_heading(page_number, page.rows.len()));
    // On its own line rather than beside the heading. In a horizontal row the
    // heading is a whole sentence and the button is fixed-width, so at any dock
    // width where the two do not both fit, `egui` clamps the button hard
    // against the text — measured at the default 320 pt pane, where they
    // already touch. A dock this narrow is the ordinary case, not the edge one.
    //
    // Guarded rather than assumed. `page_slots` and `doc.pages` are two walks
    // of one page tree and cannot normally disagree — but this index is the one
    // number that LEAVES this view, and a navigation to a page that is not
    // there would look like a document defect rather than an arithmetic one.
    if page.page_index < doc.pages.len()
        && ui
            .button(t::tab_order_goto())
            .on_hover_text(t::tab_order_goto_tooltip(page_number))
            .clicked()
    {
        *go = Some(page.page_index);
    }

    // ★ THE `/Tabs` SENTENCE, ALWAYS, ON EVERY PAGE.
    //
    // Not conditional, unlike almost every other disclosure in this panel. The
    // conditional rule — "draw it only when it is true, so the marker means
    // something when it appears" — is right for a *flag*, and wrong here: the
    // common case (no `/Tabs`) is itself the fact an operator most needs, and a
    // page that said nothing would be indistinguishable from a page whose
    // sentence had been forgotten.
    let (sentence, warn) = tabs_note(&page.tabs);
    if warn {
        ui.colored_label(ui.visuals().warn_fg_color, sentence);
    } else {
        ui.label(egui::RichText::new(sentence).small().weak());
    }

    if page.rows.is_empty() {
        ui.label(
            egui::RichText::new(t::tab_order_page_no_widgets())
                .small()
                .weak(),
        );
    }

    // ★★ Where a drag in flight would land, resolved DURING the row loop and
    // carried out of it — `crate::panels::pages`' shape, for its reason: a gap
    // is a rectangle, and a rectangle does not exist until the rows have been
    // laid out. Nothing outside this function is in a position to compute it.
    let mut drop: Option<drag::DropTarget> = None;
    let in_flight = drag::current(ui.ctx()).filter(|d| d.page_index == page.page_index);
    let pointer = ui.ctx().pointer_interact_pos();

    for (index, row) in page.rows.iter().enumerate() {
        // `/TU` when the field has a non-blank one, the fully-qualified name
        // otherwise — the fill rows' preference, so the operator reads the
        // string an assistive technology speaks. The raw name is one hover
        // away, through the SAME tooltip the fill rows use, so a name copied
        // out of either list is the same string.
        let label = row.label.as_deref().unwrap_or(row.field.as_str());
        // ★ No `dnd_drag_source`. egui 0.35's built-in drag-and-drop paints the
        // dragged widget under the cursor as a floating preview, which is the
        // wrong affordance for a *list reorder*: the operator is not carrying
        // the row somewhere, they are choosing a boundary, and the thing that
        // has to be legible is the boundary. The page rail made the same call,
        // and the operator's own words are "clear markers of where the field is
        // going to move to" — the marker is the feature; the ghost competes.
        //
        // ★★★ A PLAIN LABEL, THEN `ui.interact` OVER ITS RECT — and this is the
        // fix for a drag that did not start at all.
        //
        // The first version was `Label::new(..).sense(Sense::drag())`, which is
        // the obvious spelling and produced a row that sensed **nothing**: the
        // driven check pressed on it, dragged 39 pt down and no
        // `tab-order-drag-begin` was ever traced. Every unit test stayed green,
        // because the permutation arithmetic is reached only by a gesture.
        //
        // `crate::panels::pages` — the panel this is deliberately copying — has
        // always used `ui.interact(rect, id, Sense::click_and_drag())` with a
        // stable id, and that is the shape that works inside a `ScrollArea`.
        // Copying a panel's *look* and not its *mechanism* is how this got
        // written; the check is what caught it.
        //
        // ★ `click_and_drag`, not `drag`: a row is also a click target for a
        // tooltip, and a `Sense::drag()`-only widget swallows the press without
        // ever reporting the click.
        let placed = ui.add(egui::Label::new(t::tab_order_row(row.position, label)));
        let response = ui
            .interact(
                placed.rect,
                // ui-text-exempt: an egui id salt, never displayed.
                ui.id()
                    .with(("pdfcer-tab-order-row", page.page_index, index)),
                egui::Sense::click_and_drag(),
            )
            .on_hover_text(t::form_field_row_tooltip(&row.field));
        // ★★★ `ui_rect_visible`, NOT `ui_rect`, and getting this wrong cost an
        // afternoon of chasing a phantom.
        //
        // These rows live in a `ScrollArea` inside a docked panel, at the
        // bottom of a section that itself sits below the fill list. On a real
        // document they are frequently **off the bottom of the window** —
        // measured 2026-09-02: rows published at y = 1406 in a client area
        // 1369 points tall.
        //
        // Published unconditionally, that rectangle is a lie a driven check
        // cannot detect. The harness read it, converted it to a screen point 37
        // points below the window, and pressed there — so the row was never
        // hovered, no drag began, and the check reported *"the row does not
        // sense a drag"*: a confident, specific and entirely wrong defect report
        // about working code.
        //
        // `ui_rect_visible` intersects with the clip rectangle and publishes
        // NOTHING when the row is out of view, which turns that silent
        // mis-aim into an honest "the row is not on screen". This panel already
        // learnt the same lesson once — `register::rows` records a click on a
        // published rectangle hitting nothing at all, 36 page-blocks down a
        // scroll area — and the fill rows added on the same day got it right.
        crate::diag::ui_rect_visible(
            &format!("{}{}.{index}", drag::REGION_ROW_PREFIX, page.page_index),
            response.rect,
            ui.clip_rect(),
        );
        // ★ `drag_started_by(Primary)`, not `drag_started()` — the page rail's
        // reason, unchanged: egui's plain predicate is true for the middle
        // button too, and a right-press that wandered a few pixels before
        // releasing would start a reorder the operator meant as a context menu.
        if response.drag_started_by(egui::PointerButton::Primary) {
            drag::begin(
                ui.ctx(),
                drag::Drag {
                    page_index: page.page_index,
                    from: index,
                },
            );
        }
        // ★ The grab cursor is requested EVERY FRAME of the drag, not once at
        // `drag_started` — `crate::panels::pages` learnt this the hard way and
        // recorded it: egui resolves the cursor per frame from whatever asked
        // most recently, so a request made only at the start is overwritten by
        // the next widget the pointer passes over, which during a drag down a
        // list is every row below the one being carried.
        if in_flight.is_some() || response.hovered() {
            ui.ctx().set_cursor_icon(if in_flight.is_some() {
                egui::CursorIcon::Grabbing
            } else {
                egui::CursorIcon::Grab
            });
        }
        if let (Some(dragging), Some(pos)) = (in_flight, pointer) {
            drag::consider(response.rect, index, pos, dragging, &mut drop);
        }
        ui.label(
            egui::RichText::new(t::tab_order_row_where(
                page_number,
                row.widget + 1,
                row.widget_count,
            ))
            .small()
            .weak(),
        );
    }

    // The caret is painted AFTER the rows, so it lies over them rather than
    // under the row drawn next — an insertion mark on a boundary shares its
    // pixels with the rows either side of it.
    drag::paint(ui, drop.as_ref());
    // Settling reads raw input and so runs unconditionally, including on the
    // frame no gap was resolved: a drag released past the end of the list, or
    // outside the panel entirely, still has to END. See `drag::settle`.
    drag::settle(ui, page, drop.as_ref(), actions);

    // What this page could not list, each counted separately because each is a
    // different fact — see [`model`]'s §5. Conditional, because zero of any of
    // them says nothing.
    if !page.unclaimed.is_empty() {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t::tab_order_unclaimed(page.unclaimed.len()),
        );
    }
    if page.anonymous > 0 {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t::tab_order_anonymous(page.anonymous),
        );
    }
    if page.other_annots > 0 {
        ui.label(
            egui::RichText::new(t::tab_order_other_annots(page.other_annots))
                .small()
                .weak(),
        );
    }
}

/// The sentence for one page's `/Tabs` state, and whether it is a warning.
///
/// Split out of [`page_block`] so the mapping can be tested without a `Ui`.
/// It is the single most consequential decision on screen: it is what tells the
/// operator whether the sequence they are reading **is** the tab order, and a
/// list that silently showed the wrong sequence would be worse than no list.
///
/// # Which states warn, and why exactly those
///
/// A warning here means *"what you are looking at is not the answer to the
/// question you are asking"*. That is true under `/R`, `/C` and `/S`, where the
/// order is **derived** — from geometry, or from the tag tree — rather than
/// stored, so the `/Annots` sequence on screen is a different sequence. It is
/// also true, more weakly, under a `/Tabs` name this build cannot interpret.
///
/// It is **not** true for an absent `/Tabs` (the `/Annots` order is what viewers
/// use), nor under `/A` or `/W` (the standard defines those *as* the `/Annots`
/// order), nor for an ancestor's `/Tabs` — which is disclosed as a fact about
/// the file but is not applied, because `/Tabs` is not an inheritable page
/// attribute. See [`model`]'s §4.
///
/// The `bool` rather than an enum is deliberate at this size: there are two
/// visual treatments in this crate for a line of disclosure (warn colour, or
/// small-and-weak), and a two-valued answer is the honest shape for a two-valued
/// question. It becomes an enum the day a third treatment exists.
fn tabs_note(tabs: &TabsEntry) -> (String, bool) {
    match tabs {
        TabsEntry::Absent => (t::tab_order_no_tabs_entry().to_owned(), false),
        // Disclosed, never applied. The mode's own sentence is deliberately NOT
        // appended: it would describe an order this page does not have, and two
        // sentences about two different orders is how an operator concludes the
        // wrong one is in force.
        TabsEntry::OnAncestor(mode) => (t::tab_order_tabs_on_ancestor(&mode_name(mode)), false),
        TabsEntry::OnPage(mode) => match mode {
            TabsMode::Row => (t::tab_order_tabs_row().to_owned(), true),
            TabsMode::Column => (t::tab_order_tabs_column().to_owned(), true),
            TabsMode::Structure => (t::tab_order_tabs_structure().to_owned(), true),
            TabsMode::AnnotsArray => (t::tab_order_tabs_annots_array().to_owned(), false),
            TabsMode::Widgets => (t::tab_order_tabs_widgets().to_owned(), false),
            TabsMode::Unrecognised(name) => (t::tab_order_tabs_unrecognised(name), true),
        },
    }
}

/// The `/Tabs` name as the file spells it.
///
/// Used only where the *name* is quoted back — the ancestor sentence and the
/// trace. For an unrecognised value this is the raw bytes as decoded, never a
/// substitute: a name pdfcer has never seen is a document fact, and printing
/// something else in its place would make the view claim the file said
/// something it did not.
fn mode_name(mode: &TabsMode) -> String {
    match mode {
        TabsMode::Row => "R".to_owned(),
        TabsMode::Column => "C".to_owned(),
        TabsMode::Structure => "S".to_owned(),
        TabsMode::AnnotsArray => "A".to_owned(),
        TabsMode::Widgets => "W".to_owned(),
        TabsMode::Unrecognised(name) => name.clone(),
    }
}

/// Where a `/Tabs` was found, for the trace.
///
/// Three values rather than two, because "on an ancestor" is a third state
/// this view reports and does not apply — see [`model`]'s §4.
fn tabs_field(tabs: &TabsEntry) -> String {
    match tabs {
        TabsEntry::Absent => "absent".to_owned(),
        TabsEntry::OnPage(m) => format!("page:{}", mode_name(m)),
        TabsEntry::OnAncestor(m) => format!("ancestor:{}", mode_name(m)),
    }
}

/// The trace: a summary, a line per page, then a capped line per row.
///
/// # Why this is more than a debug print
///
/// `HANDOFF.md` §2: *"Verify by driving the binary, not by a passing test"* —
/// and its eighth defect was found **only** by printing what the running
/// application had chosen, because *"2,450 hairlines and a wash are the same
/// picture"*. This view has that property in a sharper form than most: a
/// screenshot of a list of names in an order cannot tell you that the order is
/// the one the file lists, that a widget the file lists was skipped, or that a
/// `/Tabs` came from two levels up the page tree. Every one of those is text.
///
/// Every number here drives something on screen, which is the test for what
/// belongs: `tabs` decides the per-page sentence and its colour, `rows`,
/// `unclaimed`, `anonymous` and `other_annots` each decide a line, and
/// `no_widget_fields` decides the document-wide note. If a number here is wrong,
/// something on screen is wrong with it.
fn trace(listing: &Listing) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "forms-tab-order pages={} rows={} no_widget_fields={} declares_tabs={} derived_pages={}",
            listing.pages.len(),
            listing.total_rows(),
            listing.fields_without_widgets,
            listing.any_page_declares_tabs(),
            listing.pages_with_derived_order(),
        )
    });
    if !crate::diag::enabled() {
        return;
    }
    let mut traced_rows = 0usize;
    for page in &listing.pages {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "forms-tab-page page={} tabs={} sequence={:?} widgets={} listed={} \
                 unclaimed={} anonymous={} other_annots={}",
                page.page_index,
                tabs_field(&page.tabs),
                page.tabs.sequence(),
                page.widgets_seen(),
                page.rows.len(),
                page.unclaimed.len(),
                page.anonymous,
                page.other_annots,
            )
        });
        for row in &page.rows {
            if traced_rows >= MAX_TRACED_ROWS {
                return;
            }
            traced_rows += 1;
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "forms-tab-row page={} pos={} field={} widget={}/{} label={}",
                    page.page_index,
                    row.position,
                    row.field,
                    row.widget + 1,
                    row.widget_count,
                    row.label.as_deref().unwrap_or("-"),
                )
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use self::tabs::Sequence;
    use super::*;

    /// **★ Only the states where the sequence is NOT the tab order warn.**
    ///
    /// The single most consequential mapping in this view, and it is wrong in
    /// two opposite and equally bad ways. Warning on `/A` or `/W` would tell an
    /// operator the list is unreliable on the one kind of page where the file
    /// explicitly asks for exactly this order. *Not* warning on `/R`, `/C` or
    /// `/S` would let them read a sequence that is not the tab order and
    /// believe it is — which is the failure this whole view is designed around.
    #[test]
    fn the_derived_orders_warn_and_the_stored_ones_do_not() {
        for mode in [TabsMode::Row, TabsMode::Column, TabsMode::Structure] {
            let (sentence, warn) = tabs_note(&TabsEntry::OnPage(mode.clone()));
            assert!(warn, "{mode:?} is a DERIVED order and must warn");
            assert!(
                sentence.contains("NOT the tab order"),
                "{mode:?}: the sentence must say the list is not the tab \
                 order, in those words: {sentence}"
            );
            assert_eq!(mode.sequence(), Sequence::Derived);
        }

        for mode in [TabsMode::AnnotsArray, TabsMode::Widgets] {
            let (sentence, warn) = tabs_note(&TabsEntry::OnPage(mode.clone()));
            assert!(
                !warn,
                "{mode:?} IS the /Annots order — warning would be false: {sentence}"
            );
            assert!(
                !sentence.contains("NOT"),
                "{mode:?} must not deny being the tab order: {sentence}"
            );
            assert_eq!(mode.sequence(), Sequence::AnnotsOrder);
        }

        // An unrecognised name warns, but hedges rather than asserting.
        let odd = TabsMode::Unrecognised("Q".to_owned());
        let (sentence, warn) = tabs_note(&TabsEntry::OnPage(odd));
        assert!(warn);
        assert!(sentence.contains("/Tabs /Q"), "{sentence}");
        assert!(
            sentence.contains("may not be"),
            "a name nobody has defined must not be described with certainty: \
             {sentence}"
        );
    }

    /// **★ An absent `/Tabs` is reported as absent, with NO mode name.**
    ///
    /// The constraint this view is built around, asserted as a property of the
    /// string an operator actually reads rather than of the enum behind it.
    /// `D:\Dev\pdfcer`'s roadmap records what Acrobat's "Unspecified" tab-order
    /// state mechanically denotes as **unsourced after two attempts**, so any
    /// of these words on this page would be an assertion nobody can support.
    ///
    /// It also must not warn: with no `/Tabs`, the `/Annots` order is what
    /// viewers use, so the list *is* the answer and a warning would be false.
    #[test]
    fn an_absent_tabs_entry_is_named_absent_and_nothing_else() {
        let (sentence, warn) = tabs_note(&TabsEntry::Absent);
        assert!(!warn, "an absent /Tabs is not a warning: {sentence}");
        assert!(
            sentence.contains("no /Tabs entry"),
            "it must say what the file says: {sentence}"
        );
        let lower = sentence.to_lowercase();
        for invented in ["unspecified", "manual", "default order", "automatic"] {
            assert!(
                !lower.contains(invented),
                "an absent /Tabs was given the mode name “{invented}”, which \
                 nobody has been able to source: {sentence}"
            );
        }
    }

    /// **★ An ancestor's `/Tabs` is disclosed, named, and not applied.**
    ///
    /// Three assertions because the sentence has three jobs, and dropping any
    /// one of them produces a different wrong answer. It must say the page has
    /// none of its own (or it asserts an inheritance ISO 32000-2 Table 31
    /// denies); it must name the ancestor's value (or it hides a fact that
    /// changes what another viewer does); and it must not warn (because this
    /// build does not apply it, so the sequence on screen is still the
    /// `/Annots` order that an absent `/Tabs` implies).
    #[test]
    fn an_ancestor_tabs_is_disclosed_without_being_applied() {
        let (sentence, warn) = tabs_note(&TabsEntry::OnAncestor(TabsMode::Structure));
        assert!(
            !warn,
            "an unapplied ancestor value must not warn: {sentence}"
        );
        assert!(sentence.contains("no /Tabs entry of its own"), "{sentence}");
        assert!(sentence.contains("/Tabs /S"), "{sentence}");
        assert!(
            sentence.contains("inheritable"),
            "the sentence must say WHY it is not applied, or it reads as pdfcer \
             ignoring the file: {sentence}"
        );
        assert_eq!(
            TabsEntry::OnAncestor(TabsMode::Structure).sequence(),
            Sequence::AnnotsOrder
        );
    }

    /// **No two `/Tabs` sentences read alike.**
    ///
    /// Six states, six sentences. Two that read alike would send an operator
    /// looking for the wrong cause — and the pair most likely to be collapsed
    /// by someone tidying up is `/R` and `/C`, which differ by one word and
    /// describe genuinely different sequences.
    #[test]
    fn every_tabs_state_has_its_own_sentence() {
        let all = [
            tabs_note(&TabsEntry::Absent).0,
            tabs_note(&TabsEntry::OnAncestor(TabsMode::Row)).0,
            tabs_note(&TabsEntry::OnPage(TabsMode::Row)).0,
            tabs_note(&TabsEntry::OnPage(TabsMode::Column)).0,
            tabs_note(&TabsEntry::OnPage(TabsMode::Structure)).0,
            tabs_note(&TabsEntry::OnPage(TabsMode::AnnotsArray)).0,
            tabs_note(&TabsEntry::OnPage(TabsMode::Widgets)).0,
            tabs_note(&TabsEntry::OnPage(TabsMode::Unrecognised("Q".to_owned()))).0,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b, "two /Tabs states share a sentence");
            }
        }
    }

    /// **Every warning survives its glyph being stripped.**
    ///
    /// `RIBBON_IA.md` R84 — never a colour-class cue alone. `⚠` is exactly
    /// that, and it is doubly load-bearing here because these sentences are
    /// also drawn in the warn colour: a reader who sees neither the glyph nor
    /// the colour must still get the whole meaning from the words.
    #[test]
    fn a_warning_glyph_is_never_load_bearing() {
        let warnings = [
            t::tab_order_tabs_row().to_owned(),
            t::tab_order_tabs_column().to_owned(),
            t::tab_order_tabs_structure().to_owned(),
            t::tab_order_tabs_unrecognised("Q"),
            t::tab_order_unclaimed(2),
            t::tab_order_anonymous(1),
        ];
        for s in &warnings {
            // ★ The glyph has to BE there before it can be checked for being
            // load-bearing, and this half is the tripwire. Every one of these
            // is drawn in `warn_fg_color`; with no glyph the warning-ness is
            // carried by colour alone, which is exactly what R84 forbids — and
            // `trim_start_matches` on a string that never had a glyph is a
            // no-op, so the assertions below would sail over a colour-only
            // warning without noticing. Two of these sentences shipped without
            // a `⚠` on their first draft, and it was a screenshot rather than
            // this test that caught it.
            assert!(
                s.starts_with('⚠'),
                "a warn-coloured sentence carries its warning in the colour \
                 alone: {s}"
            );
            let stripped = s.trim_start_matches(['⚠', ' ']);
            assert!(
                stripped.len() > 40,
                "the sentence is carried by its glyph: {s}"
            );
            assert!(
                stripped.starts_with(|c: char| c.is_alphanumeric()),
                "stripping the glyph must leave a sentence, not a fragment: {s}"
            );
            assert!(
                stripped.trim_end().ends_with('.'),
                "a disclosure must be a complete sentence: {s}"
            );
        }
    }

    /// **★ The page index travels 0-based and prints 1-based.**
    ///
    /// The off-by-one that would otherwise be invisible.
    /// [`Action::GoToPage`] takes a 0-based index — the convention
    /// `crate::panels::bookmarks` and `crate::panels::comments` both pin from
    /// their own side — and every string a human reads takes the number one
    /// higher. Getting it backwards produces a view that navigates one page
    /// past every heading, which looks like a document defect.
    #[test]
    fn the_page_index_travels_zero_based_and_prints_one_based() {
        for index in [0usize, 1, 35] {
            let human = index + 1;
            assert_eq!(Action::GoToPage(index), Action::GoToPage(index));
            let heading = t::tab_order_page_heading(human, 3);
            assert!(heading.contains(&human.to_string()), "{heading}");
            let tip = t::tab_order_goto_tooltip(human);
            assert!(tip.contains(&human.to_string()), "{tip}");
            // …and the row's "where" line prints the same 1-based page, plus a
            // 1-based widget number over a 0-based index.
            let where_line = t::tab_order_row_where(human, 1, 3);
            assert!(
                where_line.contains(&format!("page {human}")),
                "{where_line}"
            );
            assert!(where_line.contains("widget 1 of 3"), "{where_line}");
        }
    }

    /// **★★ The explainer must TEACH THE GESTURE, and must not still claim
    /// the view is read-only.**
    ///
    /// This test replaces `no_string_in_this_view_offers_a_reorder`, which
    /// asserted that no string here mentioned dragging. That was the correct
    /// tripwire while `EditSession::reorder_annotations` did not exist — the
    /// failure mode then was somebody shipping a disabled drag handle "ready
    /// for when the verb lands", which is the placeholder R9 forbids. The verb
    /// shipped on 2026-09-02; the tripwire is now aimed at the wrong thing, and
    /// keeping it would have forbidden the feature the operator asked for.
    ///
    /// What replaces it is the property the old assertion was protecting all
    /// along, pointed the other way. **A drag with no visible handle is
    /// undiscoverable.** There is no button, no grip dots, no "Move up" — the
    /// only thing that can tell an operator this list is draggable is the
    /// sentence above it. So the sentence is load bearing, and two things about
    /// it are pinned:
    ///
    /// 1. It says how (`drag`) AND what will be shown (`line`). The operator
    ///    asked for "clear markers of where the field is going to move to"; a
    ///    caret nobody expects is not a clear marker.
    /// 2. It no longer claims the view does not change the order. That
    ///    sentence was true for the whole of this view's life and is now the
    ///    exact opposite of the truth — the class of stale copy that survives
    ///    longest, because nothing about it looks wrong.
    #[test]
    fn the_explainer_teaches_the_drag_and_no_longer_claims_to_be_read_only() {
        let explainer = t::tab_order_explainer().to_lowercase();
        assert!(explainer.contains("drag"), "{explainer}");
        assert!(explainer.contains("line"), "{explainer}");
        for stale in ["does not change", "read-only", "reports the order;"] {
            assert!(
                !explainer.contains(stale),
                "the view now reorders, so '{stale}' is stale: {explainer}"
            );
        }
    }

    /// **★ Still no labelled reorder control, and that is deliberate.**
    ///
    /// The gesture is the feature — the operator asked for drag and drop by
    /// name — and a "Move up / Move down" pair beside every row would double
    /// the height of a list that already runs to 200 rows on a real form. The
    /// absence is worth pinning because the obvious response to "the drag is
    /// hard to discover" is to add buttons, and the right response is to fix
    /// the sentence that teaches it.
    ///
    /// ★★ This is NOT a permanent prohibition and should not be read as one.
    /// A keyboard route to reordering is an accessibility gap this view has,
    /// and if it is filled the right way — a keymap command, not a pair of
    /// buttons per row — this test is what should be revisited, with its
    /// reasoning, rather than deleted quietly.
    #[test]
    fn no_row_carries_a_labelled_reorder_button() {
        for s in [
            t::tab_order_row(1, "Full name"),
            t::tab_order_row_where(1, 1, 2),
            t::tab_order_goto().to_owned(),
            t::tab_order_goto_tooltip(1),
        ] {
            let lower = s.to_lowercase();
            for verb in ["move up", "move down", "rearrange"] {
                assert!(
                    !lower.contains(verb),
                    "{verb} appeared in a row string: {s}"
                );
            }
        }
    }
}
