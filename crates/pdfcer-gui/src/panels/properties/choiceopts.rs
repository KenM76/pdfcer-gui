//! # `panels::properties::choiceopts` — a choice field's `/Opt` list
//!
//! `FORMS_PARITY.md` §8.1 row 1, and §7 names it as **one** sibling row rather
//! than seven features: *add · remove · reorder · rename display ·
//! export ≠ display · sort · default choice*. Plus the three `/Ff` flags
//! Acrobat's Options tab draws beside them and this shell could not set — bit
//! 19 `Edit`, bit 23 `DoNotSpellCheck`, bit 27 `CommitOnSelChange`.
//!
//! ## `/Opt` is one property, and that decides the shape of [`section`]
//!
//! `FieldEdit::with_options` **replaces the whole list** — Table 230 makes
//! `/Opt`'s order significant, so a per-entry merge has no defined meaning.
//! Every operation here therefore sends the entire list, and `fieldedit`'s
//! one-press-one-undo-entry rule is satisfied rather than bent.
//!
//! The corollary is that this module emits **at most one `FieldEdit` per
//! frame**, built from the typed draft. Pressing a button steals focus from a
//! half-typed box, so the box's commit and the button's operation arrive in the
//! same frame; two pushes would be two undo entries for one press, and the
//! second — built from the pre-rename list — would silently take the rename
//! back.
//!
//! ## Sort is one act, and both sides of the boundary now perform it
//!
//! `NewChoiceField::sort` sorts `/Opt` and sets bit 20. `FieldEdit::with_sort`
//! sets bit 20 alone, and `edit_field` sorts only when the same edit supplies
//! a replacement list — a caller that hands over the whole list has no
//! pre-existing order to destroy. Bit 20 set on its own still reorders nothing
//! and is disclosed as `sort_claim_unmet`.
//!
//! This panel always sends the whole list, so it is in the sorting case. It
//! reorders anyway, because the operator must see the order that will be
//! written before it is written, and it reorders through
//! `pdfcer_core::edit::sort_choice_options` rather than through a comparator
//! of its own: one exported ordering cannot disagree with the gate that checks
//! it. `FieldEditOutcome::options_sorted` is the tripwire for the case where
//! it does anyway — see [`crate::app::actions::forms`].
//!
//! ## The duplicate refusal, and which half of it is the shell's
//!
//! Both `add_choice_field` and `edit_field` refuse a repeated export by name
//! (`EditError::ChoiceOptionDuplicate`): the fill verb resolves to the first
//! match, so the second entry would be unselectable for ever.
//!
//! [`refuse_duplicate`] survives that as the **worded** half, not as a second
//! authority. An engine refusal reaching `vector_edit` is shown as
//! `Declined::EditRefused` — *"That change was refused"* — which names neither
//! the rule nor the value, and the operator's list is long enough that finding
//! the repeat unaided is the whole difficulty. The shell therefore asks first
//! and says which value repeated.
//!
//! It asks with `pdfcer_core::edit::duplicate_choice_export`, the same
//! predicate `edit_field` refuses on. `G027` was filed because that predicate
//! was private while the ordering beside it was public, which left this panel
//! spelling the rule itself and agreeing by construction rather than by
//! contract; the engine exported it, so the only thing spelled twice now is
//! the sentence, which is the part that has to be in the operator's
//! language.
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas. Removing an option the field is set to leaves
//! the field showing that answer; `edit_field` returns `value_no_longer_fits`
//! and the status row says so. Re-pointing the selection would be inventing an
//! answer the operator did not give.

use egui::Ui;
use pdfcer_core::edit::FieldEdit;
use pdfcer_core::forms::{Field, FieldFlags};

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::panels::PanelsState;
use crate::text::panels::choiceopts as t;

/// The section's rect, for `ui-verify`. Plain [`crate::diag::ui_rect`] —
/// visible-gated for a control a check clicks, plain for a section a check asks
/// a yes/no question about.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.choice_opts";
/// The new-option box's own region.
// ui-text-exempt: trace region name, never displayed
pub const ADD_REGION: &str = "properties.choice_opts.add";
/// The Add button's own region.
// ui-text-exempt: trace region name, never displayed
pub const ADD_BUTTON_REGION: &str = "properties.choice_opts.add_button";
/// The Keep-sorted checkbox's own region.
// ui-text-exempt: trace region name, never displayed
pub const SORT_REGION: &str = "properties.choice_opts.sort";
/// The Default-choice chooser's own region.
// ui-text-exempt: trace region name, never displayed
pub const DEFAULT_REGION: &str = "properties.choice_opts.default";
/// The Allow-typing checkbox's own region.
// ui-text-exempt: trace region name, never displayed
pub const EDITABLE_REGION: &str = "properties.choice_opts.editable";
/// The Check-spelling checkbox's own region.
// ui-text-exempt: trace region name, never displayed
pub const SPELL_REGION: &str = "properties.choice_opts.spell_check";
/// The Apply-immediately checkbox's own region.
// ui-text-exempt: trace region name, never displayed
pub const COMMIT_REGION: &str = "properties.choice_opts.commit_now";

/// Per-row regions, for the first three rows only.
///
/// Three because a driven check needs two rows to prove a reorder and a third
/// to prove it is a swap rather than a rotation. A region per row would cost a
/// `format!` per row per frame, on a channel-off build too, since
/// [`crate::diag::ui_control`] takes `&str`. Rows past the third publish
/// nothing, which is a stated limit rather than a silent one.
const ROW_REGIONS: [[&str; 5]; 3] = [
    [
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row0.shown",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row0.sent",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row0.up",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row0.down",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row0.remove",
    ],
    [
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row1.shown",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row1.sent",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row1.up",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row1.down",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row1.remove",
    ],
    [
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row2.shown",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row2.sent",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row2.up",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row2.down",
        // ui-text-exempt: trace region name, never displayed
        "properties.choice_opts.row2.remove",
    ],
];

/// One `/Opt` entry as the panel holds it — decoded, so it is what the operator
/// reads and types.
///
/// A third type beside `forms::ChoiceOption` (raw bytes, the read side) and
/// `edit::ChoiceOption` (`String`s, the write side): the one a text box can be
/// bound to. [`ChoiceOptsDraft::read`] and [`to_engine`] are the only two
/// places the conversion is stated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OptionRow {
    /// What a reader displays — `/Opt`'s second element.
    pub display: String,
    /// What a submit sends — `/Opt`'s first element.
    pub export: String,
}

/// Reorder the panel's rows into the engine's own `/Opt` order.
///
/// Delegates to `pdfcer_core::edit::sort_choice_options`, which is exported so
/// that a shell cannot hold a second opinion about what "sorted" means: the
/// same comparator decides the order here, the order `add_choice_field` writes
/// at placement, and the order `edit_field` tests the bit-20 claim against.
///
/// The round trip through [`to_engine`] is the price of [`OptionRow`] being a
/// third type; the list is an `/Opt` array, so it is bounded by what a person
/// will read from a drop-down.
fn sort_by_display(list: &mut [OptionRow]) {
    let mut engine = to_engine(list);
    pdfcer_core::edit::sort_choice_options(&mut engine);
    for (row, opt) in list.iter_mut().zip(engine) {
        row.display = opt.display;
        row.export = opt.export;
    }
}

/// The panel's rows, as the engine's writer takes them.
///
/// `ChoiceOption::new(export, display)` — that argument order, and it is the one
/// thing here a reader cannot check by eye, because both halves are `String` and
/// swapping them compiles. The engine collapses an equal pair to a bare `/Opt`
/// string itself, so there is no need to choose `plain`.
fn to_engine(list: &[OptionRow]) -> Vec<pdfcer_core::edit::ChoiceOption> {
    list.iter()
        .map(|r| pdfcer_core::edit::ChoiceOption::new(&r.export, &r.display))
        .collect()
}

/// The typed half of the option editor.
///
/// A draft for the reason [`super::fieldedit::FieldPropsDraft`] has one and for
/// no other: these controls take typing. The checkboxes read `field.flags` each
/// frame, so a refused press leaves them where they were.
#[derive(Default)]
pub struct ChoiceOptsDraft {
    /// `(fully-qualified name, edit epoch)` the rows below were read at. The
    /// name so a second field does not inherit the first field's list; the
    /// epoch so an applied edit is re-read rather than shown from a stale copy.
    stamp: Option<(String, u64)>,
    /// The list as it is being typed.
    rows: Vec<OptionRow>,
    /// The list as the document holds it, so a commit can tell whether anything
    /// changed. Its own field rather than a re-read, because the commit happens
    /// on a frame in which the draft has been typed into and the document has
    /// not changed.
    stored: Vec<OptionRow>,
    /// The new option being typed. Cleared by [`Self::sync`] rather than on a
    /// successful add: the epoch moves only when an edit *applies*, so a
    /// refused add correctly leaves the text in the box.
    adding: String,
}

impl ChoiceOptsDraft {
    /// Re-read from the document when the stamp has moved.
    fn sync(&mut self, rows: Vec<OptionRow>, fqn: &str, epoch: u64) {
        let stamp = (fqn.to_owned(), epoch);
        if self.stamp.as_ref() == Some(&stamp) {
            return;
        }
        self.stamp = Some(stamp);
        self.stored = rows;
        self.rows.clone_from(&self.stored);
        self.adding.clear();
    }

    /// Pull `/Opt` off a real field, decoded, and sync.
    fn read(&mut self, field: &Field, fqn: &str, epoch: u64) {
        let rows = field
            .options
            .iter()
            .map(|o| OptionRow {
                display: pdfcer_core::edit::decode_text_string(&o.display).text,
                export: pdfcer_core::edit::decode_text_string(&o.export).text,
            })
            .collect();
        self.sync(rows, fqn, epoch);
    }
}

/// What one frame's presses asked for, beyond whatever was typed.
///
/// One value rather than a set, because a frame has one press in it. The typed
/// boxes are not here — they are read out of the draft — so [`Op::None`] still
/// means the list may have changed.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Op {
    /// Nothing pressed.
    None,
    /// Append the new-option box's contents.
    Add,
    /// Take row `n` off the list.
    Remove(usize),
    /// Swap row `n` with the one above it.
    Up(usize),
    /// Swap row `n` with the one below it.
    Down(usize),
}

/// Draw a choice field's option list and the three flags that belong with it.
///
/// Called from `fieldedit::section`'s choice branch. `epoch` is the document's
/// edit epoch — the draft's staleness key, and the stamp `record_note` needs so
/// a refusal retires when the next edit lands.
pub fn section(
    ui: &mut Ui,
    field: &Field,
    fqn: &str,
    state: &mut PanelsState,
    epoch: u64,
    actions: &mut Vec<Action>,
) {
    let combo = field.flags.has(FieldFlags::COMBO);
    let editable = field.flags.has(FieldFlags::EDIT);
    let sorted = field.flags.has(FieldFlags::SORT);

    editable_row(ui, combo, editable, fqn, actions);
    // Only on an editable drop-down: a choice field that cannot be typed into
    // has nothing to spell-check, so the control would describe a capability
    // the field does not have. R9 — an unavailable capability renders nothing.
    if combo && editable {
        spell_check_row(ui, field, fqn, actions);
    }
    commit_now_row(ui, field, fqn, actions);

    ui.add_space(4.0);
    ui.label(t::heading());
    ui.add_space(2.0);

    let mut edit = FieldEdit::new();
    let mut touched = TOUCHED_LIST;
    let mut wrote = false;

    let draft = state.choice_opts_mut();
    draft.read(field, fqn, epoch);
    let ChoiceOptsDraft {
        rows,
        stored,
        adding,
        ..
    } = draft;

    let mut op = if rows.is_empty() {
        ui.label(egui::RichText::new(t::no_options_yet()).small().weak());
        Op::None
    } else {
        column_headers(ui);
        option_rows(ui, rows, sorted)
    };
    // An Add wins over a row press, because a row press in the same frame is
    // the focus theft the Add caused rather than a second gesture. It cannot
    // actually be both — `add_row` only reports a press on its own widgets —
    // but the order states which is authoritative.
    if add_row(ui, adding) && !adding.trim().is_empty() {
        op = Op::Add;
    }

    let mut list = rows.clone();
    match &op {
        Op::None => {}
        Op::Add => {
            let text = adding.trim().to_owned();
            list.push(OptionRow {
                display: text.clone(),
                export: text,
            });
        }
        Op::Remove(n) => {
            list.remove(*n);
        }
        Op::Up(n) => list.swap(n - 1, *n),
        Op::Down(n) => list.swap(*n, n + 1),
    }
    if sorted {
        // What "Keep sorted" promises: a list kept in order, not one that was
        // in order once. Applied to every write while the flag is on, which is
        // also what stops `edit_field` disclosing `sort_claim_unmet` over a
        // list this panel just authored.
        sort_by_display(&mut list);
    }
    if op != Op::None || list != *stored {
        if let Some(duplicate) = refuse_duplicate(&list) {
            crate::app::actions::record_note(epoch, t::note_duplicate_sent(&duplicate));
        } else {
            if list.is_empty() && !stored.is_empty() {
                crate::app::actions::record_note(epoch, t::note_list_now_empty().to_owned());
            }
            touched = touched_for(&op);
            edit = edit.with_options(to_engine(&list));
            wrote = true;
        }
    }

    // Drawn after the list because it describes the list's order, and a caveat
    // placed above the thing it qualifies is read before that thing exists.
    if let Some(on) = sort_row(ui, sorted) {
        edit = edit.with_sort(on);
        if on {
            // The flag and the reordering in ONE edit, on `comb_row`'s
            // precedent: bit 20 over an unsorted list makes the file assert
            // something untrue, so the two are one act the standard makes
            // indivisible rather than two this panel chose to combine.
            let mut now = if wrote { list.clone() } else { stored.clone() };
            sort_by_display(&mut now);
            edit = edit.with_options(to_engine(&now));
        }
        touched = TOUCHED_SORT;
        wrote = true;
    }

    let current = if wrote { list } else { stored.clone() };
    if let Some(choice) = default_row(ui, field, &current) {
        edit = match choice {
            Some(export) => edit.with_default_value(export),
            None => edit.clearing_default_value(),
        };
        touched = TOUCHED_DEFAULT;
        wrote = true;
    }

    if wrote {
        actions.push(
            FieldAction::EditProperties {
                field: fqn.to_owned(),
                edit,
                touched,
            }
            .into(),
        );
    }
    crate::diag::ui_rect(REGION, ui.min_rect());
}

/// The control name a refusal names when the operator changed the list itself.
// ui-text-exempt: a control name carried for a refusal message.
const TOUCHED_LIST: &str = "the option list";
/// The control name a refusal names when they pressed Keep sorted.
// ui-text-exempt: a control name carried for a refusal message.
const TOUCHED_SORT: &str = "keep sorted";
/// The control name a refusal names when they chose a default.
// ui-text-exempt: a control name carried for a refusal message.
const TOUCHED_DEFAULT: &str = "default choice";
/// The control name a refusal names when they added an option.
// ui-text-exempt: a control name carried for a refusal message.
const TOUCHED_ADD: &str = "add option";
/// The control name a refusal names when they removed one.
// ui-text-exempt: a control name carried for a refusal message.
const TOUCHED_REMOVE: &str = "remove option";
/// The control name a refusal names when they moved one.
// ui-text-exempt: a control name carried for a refusal message.
const TOUCHED_MOVE: &str = "move option";

/// Which control a refusal should name.
///
/// `fieldedit`'s header carries why this exists: the engine's refusals arrive
/// from a direction the request does not name, so the decline is shown against
/// the control the operator touched.
const fn touched_for(op: &Op) -> &'static str {
    match op {
        Op::None => TOUCHED_LIST,
        Op::Add => TOUCHED_ADD,
        Op::Remove(_) => TOUCHED_REMOVE,
        Op::Up(_) | Op::Down(_) => TOUCHED_MOVE,
    }
}

/// The first repeated **sent** value, if any.
///
/// Compared on `export`, not on `display`: it is the export a fill resolves
/// against, so two options reading *Ontario* and *Ontario (ON)* that both send
/// `ON` are the broken pair, while two that read the same and send `ON` and `QC`
/// are merely confusing. §12.7.4.4 permits the second; nothing can select the
/// second half of the first — and that paragraph is the engine's, not this
/// panel's: the rule is `pdfcer_core::edit::duplicate_choice_export`, which
/// `edit_field` refuses on and which this asks with.
///
/// This still asks first, because it is the WORDING that is the shell's half:
/// an `EditError` reaching `vector_edit` renders as *"That change was
/// refused"*, naming neither the rule nor the value, and finding the repeat
/// unaided in a thirty-row list is the whole difficulty. Asking with the
/// engine's own predicate is what makes that a translation rather than a
/// second opinion.
///
/// The list is converted through [`to_engine`] rather than compared in place,
/// so the values tested are byte-for-byte the ones `with_options` sends.
fn refuse_duplicate(list: &[OptionRow]) -> Option<String> {
    let engine = to_engine(list);
    pdfcer_core::edit::duplicate_choice_export(&engine).map(ToOwned::to_owned)
}

/// The two column headings, once, above the rows.
fn column_headers(ui: &mut Ui) {
    let width = box_width(ui);
    ui.horizontal_wrapped(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(width, ui.spacing().interact_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(egui::RichText::new(t::column_shown()).small().weak())
                    .on_hover_text(t::column_shown_hover());
            },
        );
        ui.label(egui::RichText::new(t::column_sent()).small().weak())
            .on_hover_text(t::column_sent_hover());
    });
}

/// One reorder or remove button's width.
const BUTTON_WIDTH: f32 = 22.0;

/// The width each of the two text boxes gets.
///
/// A measurement feeding a size, which is R128's shape — but it cannot loop:
/// `available_width()` here is the dock's decision, made before the body draws
/// and unaffected by anything drawn into it, so the derived width cannot widen
/// the container that produced it.
///
/// `MIN_BOX` is the floor anyway, and the rows are `horizontal_wrapped`, so a
/// dock squeezed below two boxes plus the buttons moves the buttons to a second
/// line. That matters because the Properties panel's `ScrollArea` is
/// `vertical()` only: anything past the right edge is not below a fold, it is
/// unreachable.
fn box_width(ui: &Ui) -> f32 {
    /// The narrowest a text box may get before the row wraps instead.
    const MIN_BOX: f32 = 40.0;
    let spacing = ui.spacing().item_spacing.x;
    let buttons = BUTTON_WIDTH * 3.0 + spacing * 2.0;
    let available = ui.available_width();
    (((available - buttons - spacing * 2.0) / 2.0).max(MIN_BOX)).min(available)
}

/// Every option, one row each, and what was pressed.
///
/// The two boxes are explicitly sized so the buttons land in the same column on
/// every row, which is what makes a column of remove buttons readable as a
/// column.
fn option_rows(ui: &mut Ui, rows: &mut [OptionRow], sorted: bool) -> Op {
    let width = box_width(ui);
    let last = rows.len().saturating_sub(1);
    let mut op = Op::None;
    for (n, row) in rows.iter_mut().enumerate() {
        let regions = ROW_REGIONS.get(n);
        ui.horizontal_wrapped(|ui| {
            let shown = ui.add(
                // escape-disposition: keeps-draft — typed straight into the row list
                // this panel holds; the list is committed as a whole.
                egui::TextEdit::singleline(&mut row.display)
                    .desired_width(width)
                    .id_salt(("choice-opt-shown", n)),
            );
            let sent = ui.add(
                // escape-disposition: keeps-draft — the export half of the same row in
                // the same panel-held list.
                egui::TextEdit::singleline(&mut row.export)
                    .desired_width(width)
                    .id_salt(("choice-opt-sent", n)),
            );
            if let Some(names) = regions {
                crate::diag::ui_control(names[0], &shown, ui.clip_rect());
                crate::diag::ui_control(names[1], &sent, ui.clip_rect());
            }
            // Greyed while the list is kept sorted, explained on hover — R9's
            // temporary unavailability, and genuinely temporary: one checkbox
            // below makes the order the operator's again.
            // Each arrow is also greyed at its own end of the list, rather
            // than swallowing the press: a button that takes a click and does
            // nothing is the same silence a greyed-but-live one is. The two
            // reasons need different sentences, because "the list is kept
            // sorted" is not why the top row cannot move up.
            let can_up = !sorted && n > 0;
            let can_down = !sorted && n < last;
            let up = enabled_button(ui, can_up, t::row_up());
            let down = enabled_button(ui, can_down, t::row_down());
            if let Some(names) = regions {
                crate::diag::ui_control(names[2], &up, ui.clip_rect());
                crate::diag::ui_control(names[3], &down, ui.clip_rect());
            }
            let up_why = if sorted {
                t::reorder_locked_hover()
            } else {
                t::already_first_hover()
            };
            if hover(up, can_up, t::row_up_hover(), up_why).clicked() {
                op = Op::Up(n);
            }
            let down_why = if sorted {
                t::reorder_locked_hover()
            } else {
                t::already_last_hover()
            };
            if hover(down, can_down, t::row_down_hover(), down_why).clicked() {
                op = Op::Down(n);
            }
            let remove = button(ui, t::row_remove());
            if let Some(names) = regions {
                crate::diag::ui_control(names[4], &remove, ui.clip_rect());
            }
            if remove.on_hover_text(t::row_remove_hover()).clicked() {
                op = Op::Remove(n);
            }
        });
    }
    op
}

/// One fixed-width row button, so the three line up down the list.
fn button(ui: &mut Ui, label: &str) -> egui::Response {
    ui.add_sized(
        egui::vec2(BUTTON_WIDTH, ui.spacing().interact_size.y),
        egui::Button::new(label),
    )
}

/// [`button`], allocated inside a disabled scope when it should not respond.
///
/// The scope and not a greyed fill: `egui_shell::ribbon::sizing` measured that
/// a response from an enabled `Ui` reports itself enabled however it is
/// painted, which both kills `on_disabled_hover_text` and lets the click
/// through. A greyed control that still fires is worse than a live one.
fn enabled_button(ui: &mut Ui, enabled: bool, label: &str) -> egui::Response {
    ui.add_enabled_ui(enabled, |ui| button(ui, label)).inner
}

/// `on_hover_text` when live and `on_disabled_hover_text` when not.
///
/// Both, because each opens on only one of the two states, and the greyed state
/// is the one R9 requires an explanation for.
fn hover(response: egui::Response, enabled: bool, live: &str, disabled: &str) -> egui::Response {
    if enabled {
        response.on_hover_text(live)
    } else {
        response.on_disabled_hover_text(disabled)
    }
}

/// The new-option box and its button. Returns whether an add was asked for.
///
/// Enter in the box does what the button does, and the button's hover says so,
/// because a shortcut nobody is told about is not a shortcut.
fn add_row(ui: &mut Ui, adding: &mut String) -> bool {
    let width = box_width(ui);
    let mut pressed = false;
    ui.horizontal_wrapped(|ui| {
        let response = ui.add(
            // escape-disposition: keeps-draft — Enter adds the option; Escape
            // leaves what was typed sitting in the box for the next press.
            egui::TextEdit::singleline(adding)
                .desired_width(width)
                .hint_text(t::add_hint())
                .id_salt("choice-opt-add"),
        );
        crate::diag::ui_control(ADD_REGION, &response, ui.clip_rect());
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            pressed = true;
        }
        // Greyed on an empty box: "add nothing" is not an operation, and a
        // button that takes the press and does nothing is the silence this
        // module's header is about. The hover is the same sentence in both
        // states, because what the button does is the answer to why it cannot
        // be pressed.
        let can_add = !adding.trim().is_empty();
        ui.add_enabled_ui(can_add, |ui| {
            let add = ui.button(t::add_button());
            crate::diag::ui_control(ADD_BUTTON_REGION, &add, ui.clip_rect());
            if hover(add, can_add, t::add_button_hover(), t::add_button_hover()).clicked() {
                pressed = true;
            }
        });
    });
    pressed
}

/// `/Ff` bit 20, as the checkbox that does both halves. Returns the new state
/// when it moved.
fn sort_row(ui: &mut Ui, sorted: bool) -> Option<bool> {
    let mut on = sorted;
    let response = ui.checkbox(&mut on, t::flag_sort());
    crate::diag::ui_control(SORT_REGION, &response, ui.clip_rect());
    response
        .on_hover_text(t::flag_sort_hover())
        .changed()
        .then_some(on)
}

/// `/DV` for a choice field. Returns the chosen export value when the chooser
/// moved, with `None` meaning *remove the default*.
///
/// # `/DV` holds the EXPORT value, and getting that wrong is invisible
///
/// `set_choice_value` writes the export value to `/V`, and `edit_field`'s
/// `value_fit_complaint` checks a selection against `option.export`. `/DV` takes
/// the same type as `/V` (Table 228), so it holds an export too. A default
/// written as the display string would look right in this panel, look right in
/// the drop-down, and fail to match any option the day someone pressed Reset.
///
/// No draft: `FieldPropsDraft`'s rule is that a draft exists only for controls
/// that take typing, and a chooser is a press.
fn default_row(ui: &mut Ui, field: &Field, list: &[OptionRow]) -> Option<Option<String>> {
    let current = current_default(field);
    let mut chosen = current.clone();
    let shown = match &current {
        Some(export) => list
            .iter()
            .find(|r| r.export == *export)
            .map_or_else(|| export.clone(), |r| r.display.clone()),
        None => t::default_none().to_owned(),
    };
    let response = ui
        .horizontal(|ui| {
            ui.label(t::label_default())
                .on_hover_text(t::label_default_hover());
            egui::ComboBox::from_id_salt("properties-choice-default")
                .selected_text(shown)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut chosen, None, t::default_none());
                    for row in list {
                        ui.selectable_value(&mut chosen, Some(row.export.clone()), &row.display);
                    }
                });
        })
        .response;
    crate::diag::ui_control(DEFAULT_REGION, &response, ui.clip_rect());
    (chosen != current).then_some(chosen)
}

/// The field's current `/DV`, as an export value.
///
/// `FieldValue::Choice` rather than `display_text()`, which joins several
/// selections with `", "` — a sentence, not a value, and it would round-trip a
/// two-selection default into one option named *"A, B"*. Several defaults are a
/// real state on a multi-select list box; this chooser offers one, so it reads
/// the first and the rest are left alone rather than silently discarded, which
/// is why it reports a change only when the operator moves it.
fn current_default(field: &Field) -> Option<String> {
    match &field.default_value {
        pdfcer_core::forms::FieldValue::Choice(items) => items
            .first()
            .map(|raw| pdfcer_core::edit::decode_text_string(raw).text),
        _ => None,
    }
}

/// `/Ff` bit 19 — typing an answer that is not in the list.
///
/// # Live in three of the four states, and the fourth is the interesting one
///
/// Table 230 makes bit 19 legal only alongside bit 18, and `edit_field` checks
/// it against the **resulting** field:
///
/// | drop-down | typing | control | why |
/// |---|---|---|---|
/// | on | either | live | both directions are legal |
/// | off | off | greyed | turning it on would be refused |
/// | off | on | live | the file already breaks Table 230 |
///
/// The last row is why this is not `add_enabled_ui(combo, …)`. A file can arrive
/// with `Edit` set and `Combo` clear — pdfcer reads what is there — and greying
/// the control there would leave the operator looking at a nonconforming field
/// with no way to fix it. Clearing the flag *is* the fix, and `edit_field`
/// accepts it because the post-state conforms.
fn editable_row(ui: &mut Ui, combo: bool, editable: bool, fqn: &str, actions: &mut Vec<Action>) {
    let live = combo || editable;
    let mut on = editable;
    let mut changed = false;
    ui.add_enabled_ui(live, |ui| {
        let response = ui.checkbox(&mut on, t::flag_editable());
        crate::diag::ui_control(EDITABLE_REGION, &response, ui.clip_rect());
        let tip = if combo {
            t::flag_editable_hover()
        } else {
            t::flag_editable_without_combo_hover()
        };
        changed = hover(response, live, tip, t::flag_editable_needs_combo_hover()).changed();
    });
    if changed {
        actions.push(
            FieldAction::EditProperties {
                field: fqn.to_owned(),
                edit: FieldEdit::new().with_editable(on),
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "allow typing",
            }
            .into(),
        );
    }
}

/// `/Ff` bit 23, drawn as its own inverse.
///
/// The flag is `DoNotSpellCheck` and the checkbox says *Check spelling*, so
/// `checked` is `!flag` and the write is `!checked`. Worth the inversion — it is
/// what every application says — and confined to these lines so there is one
/// place to read it.
fn spell_check_row(ui: &mut Ui, field: &Field, fqn: &str, actions: &mut Vec<Action>) {
    let mut on = !field.flags.has(FieldFlags::DO_NOT_SPELL_CHECK);
    let response = ui.checkbox(&mut on, t::flag_spell_check());
    crate::diag::ui_control(SPELL_REGION, &response, ui.clip_rect());
    if response
        .on_hover_text(t::flag_spell_check_hover())
        .changed()
    {
        actions.push(
            FieldAction::EditProperties {
                field: fqn.to_owned(),
                edit: FieldEdit::new().with_no_spell_check(!on),
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "check spelling",
            }
            .into(),
        );
    }
}

/// `/Ff` bit 27 — the answer takes effect on the pick rather than on the exit.
fn commit_now_row(ui: &mut Ui, field: &Field, fqn: &str, actions: &mut Vec<Action>) {
    let mut on = field.flags.has(FieldFlags::COMMIT_ON_SEL_CHANGE);
    let response = ui.checkbox(&mut on, t::flag_commit_now());
    crate::diag::ui_control(COMMIT_REGION, &response, ui.clip_rect());
    if response.on_hover_text(t::flag_commit_now_hover()).changed() {
        actions.push(
            FieldAction::EditProperties {
                field: fqn.to_owned(),
                edit: FieldEdit::new().with_commit_on_sel_change(on),
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "apply choice immediately",
            }
            .into(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(display: &str, export: &str) -> OptionRow {
        OptionRow {
            display: display.to_owned(),
            export: export.to_owned(),
        }
    }

    /// The panel's sort must satisfy the test `edit_field` applies to the
    /// bit-20 claim, or the flag is set over a list the engine calls unsorted.
    ///
    /// [`sort_by_display`] delegates to the engine's exported sorter, so this
    /// can no longer drift by accident — it can still drift by someone
    /// re-hand-rolling the comparator, which is what it now guards. Asserted as
    /// the engine's own expression rather than as "it is sorted": `edit_field`
    /// computes `flags.has(SORT) && !effective.is_sorted()` over the
    /// **display** strings, so this asserts `is_sorted()` on exactly that
    /// projection. A case-insensitive or natural ordering would satisfy a human
    /// reading of "sorted" and fail this.
    #[test]
    fn the_panels_sort_satisfies_the_engines_own_sorted_test() {
        let mut list = vec![row("Quebec", "QC"), row("Alberta", "AB"), row("Ba", "BA")];
        sort_by_display(&mut list);
        let displays: Vec<String> = list.iter().map(|r| r.display.clone()).collect();
        assert!(
            displays.is_sorted(),
            "the engine tests bit 20 with is_sorted() over the display strings; \
             this ordering would leave it disclosing sort_claim_unmet: {displays:?}"
        );
        assert_eq!(
            displays,
            vec!["Alberta", "Ba", "Quebec"],
            "and it must be BYTE order, which is what add_choice_field's \
             a.display.cmp(&b.display) produces"
        );
    }

    /// A duplicate SENT value is refused; a duplicate SHOWN value is not.
    ///
    /// Both directions, because a gate that refused both would look correct and
    /// would block a legitimate list: two options that read the same and send
    /// different codes are permitted by §12.7.4.4. What nothing can select is
    /// the second half of a repeated export, which is what `add_choice_field`
    /// refuses.
    #[test]
    fn only_a_repeated_sent_value_is_refused() {
        assert_eq!(
            refuse_duplicate(&[row("Ontario", "ON"), row("Ontario (ON)", "ON")]),
            Some("ON".to_owned()),
            "a repeated export is unselectable - the fill verb resolves to the first match"
        );
        assert_eq!(
            refuse_duplicate(&[row("Other", "O1"), row("Other", "O2")]),
            None,
            "a repeated display is legal and useful; refusing it would block a real list"
        );
    }

    /// [`to_engine`] must not swap the pair.
    ///
    /// Both halves are `String`, so swapping them compiles and produces a file
    /// that opens, reads correctly in the drop-down, and submits the wrong
    /// data — the failure the engine's own note calls something that *"would
    /// silently break forms"*. The only thing between this crate and that is one
    /// line's argument order, so it is asserted rather than read.
    #[test]
    fn to_engine_keeps_shown_and_sent_on_the_right_halves() {
        let out = to_engine(&[row("Ontario", "ON")]);
        assert_eq!(out[0].display, "Ontario");
        assert_eq!(out[0].export, "ON");
    }

    /// The draft re-reads when either half of the stamp moves, and not
    /// otherwise: the name so a second field does not inherit the first's list,
    /// the epoch so an applied edit is shown rather than the pre-edit copy.
    #[test]
    fn the_draft_re_reads_on_a_new_name_or_a_new_epoch() {
        let mut draft = ChoiceOptsDraft::default();
        draft.sync(vec![row("A", "A")], "Province", 1);
        draft.rows.push(row("typed", "typed"));

        draft.sync(vec![row("A", "A")], "Province", 1);
        assert_eq!(draft.rows.len(), 2, "same stamp must not discard typing");

        draft.sync(vec![row("A", "A")], "Province", 2);
        assert_eq!(draft.rows.len(), 1, "a new epoch re-reads the document");

        draft.rows.push(row("typed", "typed"));
        draft.sync(vec![row("Z", "Z")], "Country", 2);
        assert_eq!(draft.rows, vec![row("Z", "Z")], "a new field re-reads too");
    }

    /// The new-option box clears when an edit applies and not before, so a
    /// refused add leaves the text where the operator can correct it.
    #[test]
    fn a_refused_add_keeps_its_text_and_an_applied_one_does_not() {
        let mut draft = ChoiceOptsDraft::default();
        draft.sync(vec![row("A", "A")], "Province", 1);
        draft.adding = "Bee".to_owned();

        draft.sync(vec![row("A", "A")], "Province", 1);
        assert_eq!(draft.adding, "Bee", "a refusal does not move the epoch");

        draft.sync(vec![row("A", "A"), row("Bee", "Bee")], "Province", 2);
        assert!(draft.adding.is_empty(), "an applied edit clears the box");
    }

    /// Every `Op` names a control, enforced by an exhaustive match with no
    /// wildcard: an eighth operation added without a name fails to compile here
    /// rather than shipping a refusal that says "the option list" when the
    /// operator pressed something else.
    #[test]
    fn every_operation_names_the_control_that_caused_it() {
        for (op, expected) in [
            (Op::None, TOUCHED_LIST),
            (Op::Add, TOUCHED_ADD),
            (Op::Remove(0), TOUCHED_REMOVE),
            (Op::Up(1), TOUCHED_MOVE),
            (Op::Down(0), TOUCHED_MOVE),
        ] {
            assert_eq!(touched_for(&op), expected, "{op:?}");
        }
    }
}
