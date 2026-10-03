//! # `find::bar` — the Find overlay's controls, and the keys they own
//!
//! One compact box **floating over the top-right of the page**, which is
//! where Acrobat Reader, Chrome's PDF viewer and Edge's all put theirs:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/find/bar.md`.

use egui::{Align, Align2, Layout, Pos2, Rect, Vec2};

use crate::appaction::Action;
use crate::find::{FindOptions, FindRequest, FindState, Readout, Step};
use crate::opendoc::Status;
use crate::text::find as t;
use pdfcer_core::edit::WordBoundary;

// ---------------------------------------------------------------------------
// Geometry — see the "the width is fixed" section of the module docs
// ---------------------------------------------------------------------------

/// How wide the overlay's content row is, in egui points.
pub const BAR_WIDTH_PTS: f32 = 460.0;

/// The height of the single row every control is laid out inside.
pub const ROW_HEIGHT_PTS: f32 = 24.0;

/// The gap between the overlay and the canvas viewport's top-right corner.
const MARGIN_PTS: f32 = 12.0;

/// How wide the search field is.
const FIELD_WIDTH_PTS: f32 = 190.0;

/// How wide the position readout is.
const READOUT_WIDTH_PTS: f32 = 110.0;

// ---------------------------------------------------------------------------
// Named regions — see `crate::diag::ui_rect` for the contract and the naming
// rule. These names are matched literally by `tools/ui-verify`, so renaming
// one silently un-aims whatever check was measuring it.
// ---------------------------------------------------------------------------

/// The whole overlay, including its frame.
const REGION_BAR: &str = "find-bar"; // ui-text-exempt: trace region name, never displayed

/// The search field itself.
const REGION_FIELD: &str = "find-field"; // ui-text-exempt: trace region name, never displayed

/// The step buttons and the position readout.
const REGION_POSITION: &str = "find-position"; // ui-text-exempt: trace region name, never displayed

/// The options menu button.
const REGION_OPTIONS: &str = "find-options"; // ui-text-exempt: trace region name, never displayed

/// The OCR offer's second row, when it is drawn.
const REGION_OCR_OFFER: &str = "find-ocr-offer"; // ui-text-exempt: trace region name, never displayed

/// Trace slot for the bar's steady state, de-duplicated on the rendered line.
const FIND_SLOT: &str = "find-bar"; // ui-text-exempt: trace slot name, never displayed

// ---------------------------------------------------------------------------
// Widget ids
// ---------------------------------------------------------------------------

/// The overlay's `egui::Area` id.
const AREA_ID: &str = "pdfcer-find-bar"; // ui-text-exempt: widget id, never displayed

/// The search field's id.
const FIELD_ID: &str = "pdfcer-find-field"; // ui-text-exempt: widget id, never displayed

// ---------------------------------------------------------------------------
// The overlay
// ---------------------------------------------------------------------------

/// Draw the Find overlay, if it is open and there is a document to search.
///
/// `host` is the canvas viewport the overlay pins itself inside;
/// `replace_offered` is whether the mode may edit content, and so whether the
/// Replace toggle and row are drawn at all.
pub fn show(
    ui: &mut egui::Ui,
    state: &mut FindState,
    status: &Status,
    (host, replace_offered): (Rect, bool),
    actions: &mut Vec<Action>,
) {
    if !state.is_open() {
        return;
    }
    let Status::Open(doc) = status else {
        return;
    };
    let epoch = doc.edit_epoch;
    let ctx = ui.ctx().clone();

    let area = egui::Area::new(egui::Id::new(AREA_ID))
        // `Middle` rather than `Foreground`: the box floats over the page and
        // the docks, and is floated over in turn by its own options menu, by
        // tooltips and by a modal — all of which egui puts in higher orders.
        // Claiming `Foreground` here would put the Find bar over its own popup.
        .order(egui::Order::Middle)
        // Anchored by its RIGHT-top corner, and that is a fix rather than a
        // preference — found by driving the binary and reading the trace.
        //
        // With a LEFT-top pivot the position is `right − width`, and egui does
        // not know an `Area`'s width until it has laid it out once. So on the
        // frame Ctrl+F was first pressed the box appeared **108 points to the
        // left** of where it belonged and snapped into place on the next
        // frame: two `ui-rect name=find-bar` lines back to back, same size,
        // different origin. Visible as a flinch every time the bar opened.
        //
        // A right-top pivot makes the corner this design actually cares about
        // — the one MARGIN_PTS inside the canvas's top-right — the thing egui
        // is given, so it is exact from the first frame whatever the measured
        // width turns out to be. `default_width` closes the same gap for the
        // constraint below, which would otherwise solve against a width of
        // zero on that first frame.
        .pivot(Align2::RIGHT_TOP)
        .fixed_pos(anchor_right_top(host))
        .default_width(BAR_WIDTH_PTS)
        // A canvas viewport narrower than the box — reachable with both docks
        // open on a minimum-size window — must not push the close button off
        // the edge the operator is reaching for.
        .constrain_to(host);

    // The OCR offer's condition, evaluated HERE and nowhere else.
    //
    // Two questions, and the order is the whole affordability argument:
    //
    // 1. `readout == Empty` — a search has been committed and matched nothing.
    //    Free: it is a comparison against state the bar already holds.
    // 2. the page has no extractable text at all — a `PageTextCache` read
    //    (`OpenDoc::page_has_extractable_text`), which on a cache miss is one
    //    page extraction.
    //
    // Asking (2) only after (1) is what keeps this off the frame budget. The
    // bar draws on every frame it is open and this module's header records that
    // nothing here may search on a keystroke; a per-frame page extraction would
    // be the same defect one size smaller. By the time (1) holds, the operator
    // has just paid a WHOLE-DOCUMENT extraction for the search itself — so the
    // page extraction is strictly cheaper than the gesture that caused it, and
    // it is charged to that gesture rather than to the act of opening the bar.
    //
    // And (2) is not a refinement of (1). It is the *actual* trigger — the
    // operator's rule is that the offer means "this document is images", never
    // "this search had no matches". (1) is here because the offer is drawn in
    // the place the empty readout occupies and there is nowhere else on a
    // fixed-width bar for it to go; (2) is what makes it correct. A build that
    // dropped (2) would offer to OCR a text PDF every time somebody mistyped a
    // part number.
    let offer_ocr = offer_ocr(state.readout(epoch), || doc.page_has_extractable_text());
    // THE OTHER REASON A SEARCH FINDS NOTHING.
    //
    // `pdfcer-core` v0.11.0's note, in its own words: *"a zero-result search is
    // not proof the word is absent"*. Two situations produce an identical empty
    // result — the needle is not there, or the document's text was never
    // recoverable as Unicode so no needle could have matched. The second does
    // not look broken, because the text **renders perfectly**.
    //
    // Drawn only on an EMPTY readout: a search that found things has already
    // answered the operator's question, and a caveat under a successful result
    // is the nagging the operator objected to. See `find::Results::
    // unsearchable_fonts` for why this is owed at all under rule 4.
    let unsearchable = if matches!(state.readout(epoch), Readout::Empty) {
        state.unsearchable_fonts(epoch)
    } else {
        0
    };

    let response = area
        .show(&ctx, |ui| {
            // `Frame::popup` is the theme's own floating-surface frame — fill,
            // stroke, rounding and shadow all read from `Style`. A hand-built
            // frame here would be a second set of colours outside the theme
            // module, which is exactly what `check-theme-colors.sh` exists to
            // prevent.
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                body(ui, state, (epoch, replace_offered), actions);
                if replace_offered && state.replace_open() {
                    replace::row(ui, state, epoch, actions);
                }
                // FIRST of the second-row notes, and above the
                // unsearchable one deliberately: this is a statement about
                // what the OPERATOR typed, and the other two are statements
                // about the document. The one they can act on immediately
                // - by deleting a character - goes first.
                //
                // Unconditional on the readout, unlike its neighbour.
                // `unsearchable_note` draws only on an empty result because
                // a caveat under a successful search is the nagging he
                // objected to. This one is owed even when the search
                // SUCCEEDED: a trimmed query that finds things has still
                // silently searched for something other than what was
                // typed, and hits are exactly when the operator has no
                // reason to suspect it.
                if crate::find::query::has_edge_whitespace(state.query()) {
                    blanks_note(ui, state.trim_query());
                }
                if unsearchable > 0 {
                    unsearchable_note(ui, unsearchable);
                }
                if offer_ocr {
                    ocr_offer(ui, actions);
                }
            });
        })
        .response;

    crate::diag::ui_rect(REGION_BAR, response.rect);
}

/// **The point the overlay's right-top corner is pinned to** — [`MARGIN_PTS`]
/// inside `host`'s own top-right corner.
#[must_use]
fn anchor_right_top(host: Rect) -> Pos2 {
    Pos2::new(
        (host.right() - MARGIN_PTS).max(host.left()),
        (host.top() + MARGIN_PTS).min(host.bottom()),
    )
}

/// Everything on the row.
///
fn body(
    ui: &mut egui::Ui,
    state: &mut FindState,
    (epoch, replace_offered): (u64, bool),
    actions: &mut Vec<Action>,
) {
    let row = Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS);
    ui.allocate_ui_with_layout(row, Layout::left_to_right(Align::Center), |ui| {
        // Claim the whole row even if the content uses less of it.
        // `allocate_ui_with_layout` advances its parent by the child's
        // *min_rect* — what the content actually used — so without these the
        // box would breathe as the readout changed, and a right-anchored box
        // that breathes moves its own search field.
        ui.set_min_size(row);
        ui.set_max_size(row);

        field(ui, state, epoch, actions);
        position(ui, state, epoch, actions);

        // The options menu and the close button, hard right, in that order
        // from the right edge inwards. A right-to-left layout over whatever is
        // left cannot get this wrong; measuring the row and placing a button
        // at `width − button` goes negative the moment the content is wider
        // than the row, which `egui-shell`'s dock notes record as the fragile
        // pattern.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .button(t::close())
                .on_hover_text(t::close_tooltip())
                .clicked()
            {
                state.close();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "find-closed by=button".to_owned()
                });
            }
            options(ui, state, actions);
            if replace_offered {
                replace::toggle(ui, state);
            }
        });
    });

    // `is_open()` rather than a literal `true`: a control drawn on this very
    // row may have closed the bar during the frame — the close button does,
    // and so does Escape — and a line reading `open=true` on the frame the bar
    // closed would be the last thing in the trace and would be false.
    crate::diag::trace_changed(FIND_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // `trim` and `edge_blanks` are PLAIN, not `{:?}`, and that
            // is deliberate: they are read by a harness, and a Debug tuple
            // in a field a machine parses has already produced one driven
            // check that reported the opposite of the truth while quoting
            // the truth in its own message. `OPERATOR_REQUESTS.md` O180.
            //
            // `edge_blanks` is the DISCLOSURE's own condition rather than
            // whether trimming changed anything, so a harness can assert
            // the row appears in BOTH settings - which is the half of this
            // fix that is easy to leave untested.
            "find-bar open={} query={:?} readout={:?} trim={} edge_blanks={}",
            state.is_open(),
            state.query(),
            state.readout(epoch),
            state.trim_query(),
            crate::find::query::has_edge_whitespace(state.query()),
        )
    });
}

// ---------------------------------------------------------------------------
// The field
// ---------------------------------------------------------------------------

/// The label, the search box, and the three keys the box owns.
fn field(ui: &mut egui::Ui, state: &mut FindState, epoch: u64, actions: &mut Vec<Action>) {
    let readout = state.readout(epoch);
    let rect = ui
        .scope(|ui| {
            ui.label(t::field_label());

            // Read the keys BEFORE the widget is built, so what is examined is
            // the frame's raw input rather than whatever survived the
            // `TextEdit` consuming it. egui's single-line `TextEdit` responds
            // to Enter and Escape by surrendering focus, which is why both are
            // recognised through `lost_focus()` below rather than through
            // `has_focus()` alone.
            let (enter, escape, shift) = ui.input(|i| {
                (
                    i.key_pressed(egui::Key::Enter),
                    i.key_pressed(egui::Key::Escape),
                    i.modifiers.shift,
                )
            });

            let focus_wanted = state.take_focus_request();
            let response = ui.add_sized(
                Vec2::new(FIELD_WIDTH_PTS, ROW_HEIGHT_PTS),
                // escape-disposition: not-content — a search query. Escape closes the
                // bar, which is what every find bar in the product class does.
                egui::TextEdit::singleline(state.query_mut())
                    .id(egui::Id::new(FIELD_ID))
                    .hint_text(t::field_label()),
            );
            let response = response.on_hover_text(t::field_tooltip());
            if focus_wanted {
                response.request_focus();
            }

            let had_focus = response.has_focus() || response.lost_focus();
            if had_focus && escape {
                // See the module docs: this bar takes Escape ONLY while the
                // field has focus, which is exactly the condition under which
                // `canvas::interact` has already declined it.
                state.close();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "find-closed by=escape".to_owned()
                });
                return;
            }
            if response.lost_focus() && enter {
                // Give the focus straight back, so a run of Enters walks the
                // hits instead of the first one dropping the operator out of
                // the box.
                response.request_focus();
                if let Some(request) = enter_intent(readout, shift) {
                    actions.push(Action::Find(request));
                }
            }
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_FIELD, rect);
}

/// **What Enter means**, as a pure function of the readout and the shift
/// key.
#[must_use]
fn enter_intent(readout: Readout, shift: bool) -> Option<FindRequest> {
    match readout {
        Readout::Idle | Readout::Stale => Some(FindRequest::Search),
        Readout::At { .. } => Some(FindRequest::Step(if shift {
            Step::Previous
        } else {
            Step::Next
        })),
        Readout::Empty => None,
    }
}

// ---------------------------------------------------------------------------
// The OCR offer
// ---------------------------------------------------------------------------

/// **Whether to offer OCR**, as a pure function of the readout and the page.
#[must_use]
fn offer_ocr(readout: Readout, page_has_text: impl FnOnce() -> bool) -> bool {
    matches!(readout, Readout::Empty) && !page_has_text()
}

/// The second row: what is true of the page, and the way out of it.
fn ocr_offer(ui: &mut egui::Ui, actions: &mut Vec<Action>) {
    ui.add_space(4.0);
    ui.separator();
    let rect = ui
        .scope(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.set_min_size(Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS));
                    // The muted role, because this is a statement about the
                    // document rather than a control. Not `.strong()`:
                    // `DEFECTS.md` D11 records that role as unusable in this
                    // theme.
                    let theme = egui_shell::theme::Theme::of(ui.ctx());
                    ui.label(
                        egui::RichText::new(crate::text::ocr::offer())
                            .color(theme.palette.text_muted),
                    );
                    if ui
                        .button(crate::text::ocr::offer_action())
                        .on_hover_text(crate::text::ocr::offer_tooltip())
                        .clicked()
                    {
                        // Raised as a COMMAND, not as a new action variant.
                        //
                        // The offer is a second route to `file.ocr` and must
                        // not become a second implementation of it: routing it
                        // through the command means the ribbon control and this
                        // button reach one dispatch arm, one dialog and one set
                        // of guards. `app/mod.rs`'s rule that dispatch arms
                        // route rather than compute is what makes that free.
                        actions.push(Action::Command(OCR_COMMAND.to_owned()));
                        crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed in the UI
                            "find-ocr-offer accepted=true".to_owned()
                        });
                    }
                },
            );
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_OCR_OFFER, rect);
}

/// The command the offer raises.
pub const OCR_COMMAND: &str = "file.ocr"; // ui-text-exempt: a command id, never displayed

// ---------------------------------------------------------------------------
// Stepping and the readout
// ---------------------------------------------------------------------------

/// `⏴ ⏵  3 of 47`.
fn position(ui: &mut egui::Ui, state: &FindState, epoch: u64, actions: &mut Vec<Action>) {
    let readout = state.readout(epoch);
    let steppable = matches!(readout, Readout::At { .. });
    let rect = ui
        .scope(|ui| {
            if ui
                .add_enabled(steppable, egui::Button::new(t::previous()))
                .on_hover_text(t::previous_tooltip())
                .on_disabled_hover_text(t::step_unavailable_tooltip())
                .clicked()
            {
                actions.push(Action::Find(FindRequest::Step(Step::Previous)));
            }
            if ui
                .add_enabled(steppable, egui::Button::new(t::next()))
                .on_hover_text(t::next_tooltip())
                .on_disabled_hover_text(t::step_unavailable_tooltip())
                .clicked()
            {
                actions.push(Action::Find(FindRequest::Step(Step::Next)));
            }

            // A reserved slot, drawn even when it is empty, so that running a
            // search cannot move the box's own left edge. See the module docs.
            let (text, hover) = readout_text(readout);
            ui.allocate_ui_with_layout(
                Vec2::new(READOUT_WIDTH_PTS, ROW_HEIGHT_PTS),
                Layout::left_to_right(Align::Center),
                |ui| {
                    let label = ui.add(egui::Label::new(&text).truncate());
                    if !hover.is_empty() {
                        label.on_hover_text(hover);
                    }
                },
            );
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_POSITION, rect);
}

/// The readout's text and its hover text, or two empty strings for
/// [`Readout::Idle`].
#[must_use]
fn readout_text(readout: Readout) -> (String, &'static str) {
    match readout {
        // Deliberately blank rather than `0 of 0`. Nothing has been asked, so
        // there is nothing to answer.
        Readout::Idle => (String::new(), ""),
        Readout::Empty => (t::no_matches().to_owned(), t::no_matches_tooltip()),
        Readout::Stale => (t::stale().to_owned(), t::stale_tooltip()),
        Readout::At { current, total } => (t::position(current, total), t::position_tooltip()),
    }
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// The `Options` menu button.
fn options(ui: &mut egui::Ui, state: &mut FindState, actions: &mut Vec<Action>) {
    let mut options = state.options();
    let before = options;
    let mut zoom_on_jump = state.zoom_on_jump();
    let zoom_before = zoom_on_jump;

    let rect = ui
        .menu_button(t::options(), |ui| {
            options_menu(ui, &mut options, &mut zoom_on_jump);
        })
        .response
        .on_hover_text(t::options_tooltip())
        .rect;
    crate::diag::ui_rect(REGION_OPTIONS, rect);

    // The Zoom control is handled FIRST and SEPARATELY, and the separation is
    // the whole reason it is not a `FindOptions` field: the block below re-runs
    // the search, and this preference must not. See
    // `FindState::set_zoom_on_jump`.
    //
    // The live value is written here so the very next `Next` obeys it without
    // waiting for the apply phase; the action carries the *persistence*, which
    // is the half that needs `PdfcerApp`. Both, because either alone is a
    // defect — writing only the state loses the setting on restart, and
    // raising only the action makes the control lag by one frame.
    if zoom_on_jump != zoom_before {
        state.set_zoom_on_jump(zoom_on_jump);
        actions.push(Action::Pref(crate::subactions::PrefAction::FindZoom(
            zoom_on_jump,
        )));
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "find-zoom-option on={zoom_on_jump}"
            )
        });
    }

    if options == before {
        return;
    }
    // Asked BEFORE the new options are stored: afterwards the answer is
    // `false` by construction, because the stored results were computed under
    // the old options and would no longer match.
    let was_showing_an_answer = state.answered();
    state.set_options(options);
    if was_showing_an_answer && !state.query().is_empty() {
        actions.push(Action::Find(FindRequest::Search));
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "find-options case={} whole={} wildcards={} boundary={:?} research={was_showing_an_answer}",
            options.case_sensitive, options.whole_word, options.wildcards, options.word_boundary,
        )
    });
}

/// The contents of the `Options` menu.
fn options_menu(ui: &mut egui::Ui, options: &mut FindOptions, zoom_on_jump: &mut bool) {
    ui.checkbox(&mut options.case_sensitive, t::match_case())
        .on_hover_text(t::match_case_tooltip());
    ui.checkbox(&mut options.whole_word, t::whole_word())
        .on_hover_text(t::whole_word_tooltip());

    if options.whole_word {
        ui.separator();
        ui.label(t::word_rule())
            .on_hover_text(t::word_rule_tooltip());
        for rule in FindOptions::WORD_RULES {
            ui.radio_value(&mut options.word_boundary, *rule, word_rule_label(*rule))
                .on_hover_text(word_rule_tooltip(*rule));
        }
        ui.separator();
    }

    ui.checkbox(&mut options.wildcards, t::wildcards())
        .on_hover_text(t::wildcards_tooltip());

    // Below a separator, and last. The three above answer *what counts as a
    // hit*; this one answers *what happens when you go to one*. They are two
    // subjects, and a menu that ran them together as four peers would read as
    // four ways of changing the search — which is exactly the misreading that
    // makes an operator expect this one to re-run it.
    ui.separator();
    ui.checkbox(zoom_on_jump, t::find_zoom())
        .on_hover_text(t::find_zoom_tooltip());
}

/// The label for one whole-word rule.
#[must_use]
pub(crate) fn word_rule_label(rule: WordBoundary) -> &'static str {
    match rule {
        WordBoundary::NonSpace => t::word_rule_non_space(),
        WordBoundary::NonSpaceOrDash => t::word_rule_non_space_or_dash(),
        // `Alphanumeric`, plus any variant a future `pdfcer-core` adds:
        // `WordBoundary` is `#[non_exhaustive]`, so a wildcard arm is required
        // and naming `Alphanumeric` beside it would be an unreachable pattern.
        // It falls back to the DEFAULT's label rather than to a blank, because
        // a new variant arriving from a core upgrade must not produce an empty
        // row in a menu. `super::tests::every_word_rule_the_chooser_offers_has_a_label`
        // is what keeps `FindOptions::WORD_RULES` honest.
        _ => t::word_rule_alphanumeric(),
    }
}

/// The hover text for one whole-word rule. See [`word_rule_label`].
#[must_use]
fn word_rule_tooltip(rule: WordBoundary) -> &'static str {
    match rule {
        WordBoundary::NonSpace => t::word_rule_non_space_tooltip(),
        WordBoundary::NonSpaceOrDash => t::word_rule_non_space_or_dash_tooltip(),
        _ => t::word_rule_alphanumeric_tooltip(),
    }
}

#[cfg(test)]
mod tests;

/// The Replace toggle and row.
mod replace;

/// **Say that part of this document could never have matched.**
fn unsearchable_note(ui: &mut egui::Ui, fonts: u64) {
    ui.allocate_ui_with_layout(
        Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_min_size(Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS));
            // Muted, for the reason `ocr_offer` gives: a statement about the
            // document, not a control. Not `.strong()` — `DEFECTS.md` D11
            // records that role as unusable in this theme.
            let theme = egui_shell::theme::Theme::of(ui.ctx());
            let text = if fonts == 1 {
                crate::text::find::unsearchable_one().to_owned()
            } else {
                crate::text::find::unsearchable_many(fonts)
            };
            ui.add(
                egui::Label::new(egui::RichText::new(text).color(theme.palette.text_muted))
                    .truncate(),
            )
            .on_hover_text(crate::text::find::unsearchable_tooltip());
        },
    );
}

/// The unsearchable note answers a DIFFERENT question from the OCR offer,
/// and both can be true at once.
fn blanks_note(ui: &mut egui::Ui, trimmed: bool) {
    ui.allocate_ui_with_layout(
        Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_min_size(Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS));
            let theme = egui_shell::theme::Theme::of(ui.ctx());
            let text = if trimmed {
                crate::text::find::blanks_trimmed()
            } else {
                crate::text::find::blanks_kept()
            };
            ui.add(
                egui::Label::new(egui::RichText::new(text).color(theme.palette.text_muted))
                    .truncate(),
            )
            .on_hover_text(crate::text::find::blanks_tooltip());
        },
    );
}

#[cfg(test)]
mod unsearchable_tests;
