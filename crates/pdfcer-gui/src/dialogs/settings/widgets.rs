//! # `dialogs::settings::widgets` — the three shapes every setting is made of
//!
//! Seven group modules draw thirteen settings, and every one of them is built
//! from the three functions here. That is deliberate: a settings window whose
//! entries are hand-laid-out drifts into thirteen slightly different layouts
//! within a year, and the reader notices the inconsistency before they notice
//! the content.
//!
//! ## ★ [`header`]'s signature is where obligation 2 and 3 are enforced
//!
//! `crate::dialogs::settings`' header names three things this window must show
//! that a conventional settings screen omits. Two of them are properties of
//! *every* setting, and rather than trusting each group module to remember
//! them, they are **required arguments**:
//!
//! ```text
//! header(ui, title, silence, radius)
//!               │       │       └── which way costs what
//!               │       └────────── what the standard leaves open
//!               └────────────────── what the setting is
//! ```
//!
//! A setting cannot be added without answering all three, because the code
//! does not compile otherwise. `crate::text::settings` mirrors the shape —
//! `*_title`, `*_silence`, `*_radius` for all thirteen — and its own tests
//! assert none of the answers is empty.
//!
//! Obligation 1 — *whether the default is a guess* — is **not** enforceable
//! this way: it belongs to one option rather than to the setting, and only
//! some options have anything to say. It is pinned by a test over the catalog
//! instead.

use egui::{RichText, Ui};

/// One collapsible subject group.
///
/// ## ★ Plain text, not `.strong()` — `DEFECTS.md` D11
///
/// Both this and [`header`] used `RichText::strong()` in their first draft, and
/// both were **near-invisible on screen**: pale grey on pale grey, while the
/// radio labels under them read normally. Found by capturing the running
/// window, which is the only oracle for this class of defect and is why the
/// check that opens this dialog exists.
///
/// The mechanism is `egui`'s, and D11 sets it out: there is **no separate role
/// for emphasised text** — `strong_text_color()` returns
/// `widgets.active.fg_stroke`, the foreground of the *accent-filled* state. In
/// any theme whose active state is accent-filled, which is all three of this
/// project's, `.strong()` on an ordinary panel resolves to a colour chosen to
/// sit on the accent. It also survives `override_text_color`.
///
/// D11 states the rule — *"do not use `RichText::strong()` in this
/// application"* — and prescribes the fix five other panels already took:
/// render as plain text, because *"the emphasis they were asking for was
/// invisible"*. The hierarchy that emphasis was reaching for is still there and
/// is carried by layout rather than by weight: a heading has a disclosure
/// triangle beside it, and a setting's title is the only line of the three that
/// is **not** `.small().weak()`.
///
/// The two legitimate uses in the workspace both take the colour back
/// explicitly on the next line — `egui-shell`'s ribbon and dock tab labels,
/// which are drawn on an accent fill and pair `.strong()` with
/// `.color(palette.on_accent)`. That pairing is what
/// `tools/gates/check-strong-text.sh` allows and a bare `.strong()` is what it
/// refuses, so this cannot be got wrong a third time by remembering.
///
/// ## The rest of this control
///
/// A `CollapsingHeader` rather than a `heading` size: this window is a list of
/// thirteen things and a true heading at each of seven would make it read as
/// seven documents. `default_open` is passed rather than remembered, because
/// which group is expanded is a statement about which symptom is most likely —
/// see the module header — and not a preference of the operator's to be
/// persisted.
/// `key` is the stable identifier the heading's rect is published under —
/// `settings.heading.<key>` — and it is deliberately **not** derived from the
/// caption. The caption is operator copy and may be reworded or translated; a
/// check aimed at a region named after it would then silently stop finding its
/// subject and report a heading that is not there rather than a heading that is
/// illegible. Those are different verdicts and only one of them is true.
pub fn group(
    ui: &mut Ui,
    key: &str,
    heading: &str,
    open_by_default: bool,
    body: impl FnOnce(&mut Ui),
) {
    group_focused(ui, key, heading, open_by_default, false, body);
}

/// [`group`], and whether this is the group the window was opened **for**.
///
/// ★★★ `focused` forces it open and scrolls the window to it, once, on the
/// frame the dialog is built. See [`super::Draft::focus`] for the argument: a
/// route that exists because of one setting must land on that setting, and
/// Tools ▸ Font folders was dropping the operator at the top of ten collapsed
/// headings.
///
/// ★★ `open(Some(true))` **only when focused**, never `Some(false)` otherwise.
/// `CollapsingHeader::open` overrides the operator's own click for as long as it
/// is passed, so a group forced open every frame is one they cannot collapse —
/// and the fix for a discoverability problem must not take away a control.
pub fn group_focused(
    ui: &mut Ui,
    key: &str,
    heading: &str,
    open_by_default: bool,
    focused: bool,
    body: impl FnOnce(&mut Ui),
) {
    let mut header = egui::CollapsingHeader::new(RichText::new(heading));
    if focused {
        header = header.open(Some(true));
    }
    let response = header.default_open(open_by_default).show(ui, body);
    if focused {
        // ★ The HEADER's response, so the window lands with the heading at the
        // top of the view rather than the body's last row. `Align::TOP` for the
        // same reason — the operator asked for this group and wants to read it
        // downward, not to arrive at its end.
        response
            .header_response
            .scroll_to_me(Some(egui::Align::TOP));
    }
    // ★ The HEADER's rect, not the whole collapsible's.
    //
    // `CollapsingHeaderResponse::header_response` is the row carrying the text;
    // the outer rect would include the expanded body, and a contrast check
    // measuring that would sample a hundred lines of prose and average the
    // heading away. D2 was a defect in one row of pixels, and it measured about
    // 1.1:1 — a figure only obtainable from the row itself.
    // ★ `ui_rect_visible`, not `ui_rect` — these headings live in a
    // `ScrollArea` and `egui` lays out the ones below the fold before clipping
    // them. Publishing a rect for a heading nobody can see makes a contrast
    // check measure whatever is genuinely at those coordinates, which on the
    // first live run of `settings_headings_legible` was the Pages panel and
    // the drawing behind the dialog — reported as three illegible headings in
    // a dialog whose visible headings measured 13.91:1. See
    // `crate::diag::ui_rect_visible`.
    crate::diag::ui_rect_visible(
        &format!("{}{key}", super::REGION_HEADING_PREFIX),
        response.header_response.rect,
        ui.clip_rect(),
    );
    ui.add_space(2.0);
}

/// One setting's three lines: what it is, what is open, and what it costs.
///
/// Always in this order, and the order is the argument the window makes. The
/// operator reads *what this is*, then *why they are being asked* — which is
/// the sentence that stops a pdfcer/Acrobat difference being read as a bug —
/// and then *what changing it will do to their file*, which is the one they
/// need before touching a radio rather than after.
///
/// `.small().weak()` for the second and third: they are context for the choice
/// rather than the choice, and at the same weight as the title they would make
/// every setting look like three settings.
///
/// ★ The title is **plain text**, not `.strong()` — see [`group`] for the
/// screenshot that found the difference and `DEFECTS.md` D11 for why no theme
/// this project ships can render `.strong()` legibly on a panel. Being the only
/// one of the three lines that is not small and weak is the whole of its
/// emphasis, and it is enough.
pub fn header(ui: &mut Ui, title: &str, silence: &str, radius: &str) {
    ui.label(RichText::new(title));
    ui.label(RichText::new(silence).small().weak());
    ui.label(RichText::new(radius).small().weak());
    ui.add_space(2.0);
}

/// One radio option, with an optional gloss under it.
///
/// # Why the note is an `Option`
///
/// A few labels are self-describing — *"Carriage return then newline"* needs
/// no gloss — and padding them out to match their neighbours would be noise.
/// The rule this window inherits about tooltips applies one layer down: text
/// that says nothing trains the reader to stop reading the text that does.
///
/// Exactly two of the thirteen settings' options pass `None`, and both are in
/// the *Saving files* group where the label names a byte sequence.
pub fn option<T: PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    value: T,
    label: &str,
    note: Option<&str>,
) {
    ui.radio_value(current, value, label);
    if let Some(note) = note
        && !note.is_empty()
    {
        ui.label(RichText::new(note).small().weak());
    }
}

/// One switch, with an optional gloss under it.
///
/// # ★ The fourth shape, and why a two-option radio group was refused
///
/// This module's header opens *"the three shapes every setting is made of"*,
/// and a fourth arriving needs a better reason than convenience. It has one: a
/// **switch is not a choice between named alternatives**.
///
/// [`option`] draws a radio, which is the right control when the operator is
/// picking one of several *named things* — `Nearest sample`, `Average the
/// area` — and the names carry the content of the choice. A visibility toggle
/// has no such names. Rendering it as a radio group would mean inventing the
/// pair *"Shown" / "Hidden"*, which says nothing the checkbox's own label does
/// not, and it would draw **six** controls for the three overlays where three
/// belong. Worse, three adjacent two-radio groups read as though the six were
/// somehow related — a reader scanning them has to work out that they are three
/// independent switches and not one six-way choice.
///
/// # Why the label is on the checkbox rather than in a [`header`]
///
/// Because these are the sub-parts of **one** setting rather than settings in
/// their own right. The Drawing-the-page group's overlay control has a single
/// header — one title, one silence line, one radius line — and three switches
/// under it, because the three interlock: a guide is dragged out of a ruler, so
/// switching guides on without rulers places nothing. Giving each its own
/// header would print that explanation three times, or once, in a place two of
/// the three readers would not look.
///
/// `note` is `Option` for the same reason it is on [`option`]: a label that
/// needs no gloss should not get a padded one, because text that says nothing
/// trains the reader to stop reading the text that does.
pub fn toggle(ui: &mut Ui, value: &mut bool, label: &str, note: Option<&str>) {
    ui.checkbox(value, label);
    if let Some(note) = note
        && !note.is_empty()
    {
        ui.label(RichText::new(note).small().weak());
    }
}

/// ★★ **A free-text setting, with the parse shown rather than enforced.**
///
/// The window's only non-radio, non-checkbox control, added 2026-08-26 for the
/// CMYK buffer ceiling. Everything else here is a choice between named options;
/// this one is a *quantity*, and a quantity the operator was explicitly given
/// the right to choose without a cap:
///
/// > *"can the size of the buffer be increased? Allow the user to set the size
/// > up to the maximum possible?"*
///
/// # Why it does not validate as you type, and does not refuse
///
/// `parse` is run on every frame and its result is **shown**, not imposed. A
/// field that rejected keystrokes would make `2` untypeable on the way to
/// `256mib`, and one that reverted on blur would silently discard what was
/// typed. So:
///
/// * **parses** → the value is written to the draft and the parsed form is
///   echoed back (`= 256 MiB`), which is how an operator learns that `0.25gb`
///   and `256mb` are the same number here;
/// * **does not parse** → the draft is left ALONE and the field says so. The
///   last good value stands, so Apply cannot commit a half-typed string.
///
/// ★ There is no upper bound and that is the operator's ruling, the same one
/// that governs the maximum zoom. A ceiling the machine cannot honour is not a
/// crash — the engine allocates fallibly and refuses down its ordinary disclosed
/// path — so this states the cost and does not prevent the choice.
///
/// `format` and `parse` are the CALLER's, and in the one use they are
/// `pdfcer_core::settings::format_byte_size` / `parse_byte_size` — the same pair
/// `settings.txt` itself uses. That is the point: the window and the file accept
/// and show identical strings, so an operator who reads one and types into the
/// other is never surprised. Writing a second parser here would have been the
/// obvious shortcut and the one thing guaranteed to drift.
pub fn text_value<T: Clone + PartialEq>(
    ui: &mut Ui,
    id: &str,
    value: &mut T,
    label: &str,
    note: Option<&str>,
    format: impl Fn(&T) -> String,
    parse: impl Fn(&str) -> Option<T>,
) {
    ui.label(label);
    // The buffer lives in `egui::Memory` keyed on this control's id, not in the
    // draft: the draft holds a parsed VALUE and this holds the operator's
    // keystrokes, and the two are legitimately different while a number is
    // half-typed. Seeded from the value the first time the control is drawn, so
    // reopening the window shows what is stored rather than an empty box.
    let id = egui::Id::new(id);
    let mut buffer = ui
        .ctx()
        .data_mut(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| format(value));
    let response = ui.add(egui::TextEdit::singleline(&mut buffer).desired_width(140.0));
    if response.changed()
        && let Some(parsed) = parse(&buffer)
    {
        *value = parsed;
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, buffer.clone()));

    match parse(&buffer) {
        Some(parsed) => {
            let echo = format(&parsed);
            // Echoed only when the operator's spelling and the canonical one
            // differ. `256 MiB` typed back as `256 MiB` is noise; `0.25gb`
            // answered with `256 MiB` is the whole reason this line exists.
            if echo != buffer {
                ui.label(
                    RichText::new(crate::text::settings::parsed_as(&echo))
                        .small()
                        .weak(),
                );
            }
        }
        None => {
            ui.label(
                RichText::new(crate::text::settings::unparsed_value_note())
                    .small()
                    // `notice` rather than `danger`: nothing is broken and
                    // nothing was lost — the stored value still stands and the
                    // operator is mid-keystroke. `danger` is for an act that
                    // destroys something.
                    .color(egui_shell::theme::Theme::of(ui.ctx()).palette.notice),
            );
        }
    }
    if let Some(note) = note
        && !note.is_empty()
    {
        ui.label(RichText::new(note).small().weak());
    }
}

/// A sentence the operator must see but that belongs to the **setting**, not to
/// any one of its options.
///
/// `.small()` and deliberately **not** `.weak()`, which is the whole point of
/// its existing separately from [`option`]'s note. There are exactly three of
/// these in the window and each is a disclosure rather than a description:
///
/// - the CMYK intent group's *"pdfcer's default deliberately differs from
///   Acrobat here"*, which the person reading that radio group is precisely the
///   person who needs;
/// - the replacement-text group's bound, which applies **whichever option is
///   chosen** and would be misread as an argument for one of them if it sat
///   inside a note;
/// - the unknown-theme sentence, which explains why none of the three radios is
///   selected.
///
/// Weak-grey is for context. A disclosure that pdfcer owes the operator is not
/// context, and greying it would be the quiet version of not saying it.
pub fn disclosure(ui: &mut Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(RichText::new(text).small());
}
