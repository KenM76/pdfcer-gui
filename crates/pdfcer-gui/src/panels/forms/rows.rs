//! # `panels::forms::rows` — one field, one row
//!
//! The per-field half of the Forms panel: what a text field, a check box, a
//! radio group, a choice list and a rich-text field each look like, and which
//! [`FormEdit`] each of them can raise.
//!
//! The form-wide half — the disclosures, the recompute and reset sections, the
//! two whole-form buttons — is in [`super`], and the split is by *scope*
//! rather than by size: a control that acts on the whole form and a control
//! that acts on one field answer to different rules about placement, about
//! disclosure and about when they may be offered at all.
//!
//! ## Every unfillable row is DISABLED AND EXPLAINED, never hidden
//!
//! `RIBBON_IA.md` R83. An operator scrolling past a signature field should see
//! that pdfcer knows it is there; a row that vanishes teaches nothing, while a
//! disabled one with a sentence beside it teaches what would enable it.
//!
//! The reason is asked in a fixed order — see [`block_reason`] — because a
//! field can be blocked several ways at once and the most specific answer is
//! the useful one.
//!
//! ## ★ The salvaged appearance check could not fire, and the replacement
//! asks a question the model can answer
//!
//! Recorded because it is the single correction this module makes to code that
//! was otherwise carried across intact, and because the shape of the mistake
//! is one anybody could repeat.
//!
//! The old shell disclosed, **after** a check box was toggled, that the
//! document had no drawn appearance for the state just selected. The predicate
//! behind it was:
//!
//! ```text
//! let has_ap_for_target = |on: bool| {
//!     on || field.widgets.iter().any(|w| w.on_states.iter().any(|st| st == b"Off"))
//! };
//! ```
//!
//! `pdfcer_core::forms::Widget::on_states` is documented as *"the button
//! on-state names this widget's `/AP` `/N` subdictionary defines, **excluding
//! `Off`**"*. So the right-hand disjunct is **always false**, and the
//! predicate reduces to `on`: every *clear* reported "no appearance for that
//! state" and every *tick* reported none, whatever the document actually
//! contained. The disclosure fired on exactly the wrong half of the clicks.
//!
//! It cannot simply be repaired, because the fact it wanted is not in the
//! model: `on_states` excludes `Off` by construction, so nothing a shell can
//! read says whether `/AP` `/N` `/Off` exists. (That is a `pdfcer-core`
//! boundary finding, recorded in [`super::edit`]'s KNOWN GAPS alongside the
//! other two.)
//!
//! What **is** knowable is the other direction, and it turns out to matter
//! more: if `on_states` is empty, there is no ON state at all, and
//! `EditSession::set_button_state` refuses any name but `Off` that no widget
//! defines — `EditError::FieldStateUnknown`. So the box is drawn **disabled**
//! with [`crate::text::forms::form_field_no_on_state_note`] beside it, which
//! is R83 rather than a disclosure after the fact: the control that would
//! always error is not offered.
//!
//! ## Deliberately absent: field creation, deletion, renaming, widget moving
//!
//! The old shell's rows carried a Rename editor with an ancestor breadcrumb, a
//! per-widget Delete, a whole-field Delete and a grouping-node roster —
//! roughly half of its 1,600 lines. None of it is here. Those are `Edit ▸
//! Forms` **authoring** commands (`edit.form_create_field`,
//! `edit.form_manage_fields`), they answer to core's *structural*
//! certification gate rather than the fill gate, and a reader that fills a
//! form does not create fields in it. They land with the commands that name
//! them.

use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::app::state::OpenDoc;
use pdfcer_core::forms::{ButtonKind, Field, FieldFlags, FieldType, FieldValue};
use pdfcer_core::object::ObjId;

use crate::panels::forms::edit::FormEdit;
use crate::text::forms as t;

/// The prefix of the per-row region names for the fill list; the 0-based row
/// index is appended.
///
/// A cross-repo stability contract with `tools/ui-verify`: renaming it is
/// changing an API, not tidying a string.
// ui-text-exempt: a diagnostic region name, never displayed.
const REGION_ROW_PREFIX: &str = "forms.fill.row.";

/// What a row needs to know that is not on the [`Field`] itself.
///
/// A struct rather than three parameters, because the row functions below
/// would otherwise take seven arguments each and because the grouping says
/// what these are: **facts about the document that are identical for every
/// row**, computed once per frame by [`super::body`].
pub(super) struct RowContext<'a> {
    /// Page object id → 1-based page number.
    ///
    /// Built once per frame rather than per row: a 400-field form would
    /// otherwise do 400 linear scans of the page list.
    pub page_numbers: &'a HashMap<ObjId, usize>,
    /// Why filling is refused for the WHOLE document, if it is.
    ///
    /// Asked once, before any row is drawn, and applied to every row: a
    /// certification signature forbids filling the whole **document**, not one
    /// field, so per-row re-asking would repeat a signature census per field
    /// and still say the same thing (R83 — know before you offer).
    pub fill_refusal: Option<&'static str>,
    /// **The open document**, for the one row that has to ask it a question.
    ///
    /// ★★ `Option`, and the `None` is not defensive padding: this module's
    /// tests build a `RowContext` to exercise labelling and blocking rules
    /// without a document, and they must go on being able to. A row that needs
    /// the document simply is not drawn without one — which is R9's rule
    /// applied to a test harness rather than to an operator.
    ///
    /// ★ Only `panels::forms::button` uses it. Every other row is a pure
    /// function of the `Field` it was handed, and that is a property worth
    /// keeping: it is why this module's rules can be unit-tested at all.
    pub doc: Option<&'a OpenDoc>,
    /// **What the operator is typing into a field ON THE PAGE right now**, as
    /// `(fully-qualified name, draft)` — see
    /// [`crate::canvas::forms::live_draft`], which is the one place this is
    /// decided.
    ///
    /// ★★★ The 2026-09 review's row **A12c**: *"the Fill-form panel does not
    /// update while you type on the page."* Two draft stores, reconciled only
    /// by a commit, and a commit only on focus loss — so a row sat beside the
    /// field it describes showing the *previous* value for the whole of a
    /// typing gesture. [`text_row`] now prefers this over its own draft, which
    /// is not a synchronisation between two stores but the removal of one of
    /// them from the answer.
    ///
    /// ★★ Asked **once per frame**, in [`super::field_list`], exactly like
    /// [`Self::page_numbers`] and [`Self::fill_refusal`] and for the same
    /// reason: it is a fact about the document that is identical for every
    /// row, and a 400-field form would otherwise read and clone the canvas's
    /// focus 400 times per frame to learn one field's name.
    ///
    /// `None` whenever the page's editor does not hold the keyboard — which
    /// includes every frame the operator is typing in this panel instead.
    pub live_canvas_draft: Option<(String, String)>,
    /// Which button's action chooser is open, and what it is set to.
    ///
    /// Held by the panel across frames because a chooser the operator has
    /// opened must not close when the panel repaints — and the panel repaints
    /// on every frame.
    pub button_draft: &'a mut Option<(String, crate::canvas::formfield::action::ButtonDoes)>,
    /// Where a push button's action change is raised.
    ///
    /// ★ Separate from `out: &mut Vec<FormEdit>`, which carries **fills**.
    /// Setting an action is not a fill — it is a structural change to the
    /// field, in `FieldAction`'s vocabulary rather than `FormEdit`'s — and
    /// giving it a `FormEdit` variant would have put a document-structure verb
    /// in the enum whose whole subject is values.
    pub actions: &'a mut Vec<crate::app::actions::Action>,
}

/// Draw one field's row, and push whatever the operator asked for.
///
/// `index` is the field's position in `AcroForm::fields` and is used **only**
/// for widget id salts. It is not a substitute for the field's name: several
/// terminal fields may legitimately share a fully-qualified name
/// (`AcroForm::fields_named` exists for exactly that), and a fill applies to
/// all of them — so the *edit* is keyed by name while the *widget* is keyed by
/// position, and conflating the two would either collapse two rows into one
/// egui id or fill the wrong field.
pub(super) fn row(
    ui: &mut egui::Ui,
    field: &Field,
    index: usize,
    ctx: &mut RowContext<'_>,
    drafts: &mut BTreeMap<String, String>,
    out: &mut Vec<FormEdit>,
) {
    let fqn = &field.fully_qualified_name;
    let label = row_label(field, ctx);

    ui.label(&label)
        .on_hover_text(t::form_field_row_tooltip(fqn));

    // The document-wide refusal wins over the per-field one: an operator on a
    // certified document needs to know that is why, not that this particular
    // field also happens to be a push button.
    if let Some(note) = ctx.fill_refusal.or_else(|| block_reason(field)) {
        blocked_row(ui, field, note);
        // ★★★ …AND a push button gets its action row anyway — 2026-09-01.
        //
        // `block_reason` answers *"this field cannot be FILLED"*, and for a
        // push button that is permanently true and always was: it runs an
        // action rather than holding a value, which is exactly what the note
        // above says. Until today that was the whole of what this shell had to
        // say about one.
        //
        // ★★ Not fillable is not the same as not editable, and conflating them
        // is what kept this row from existing. The distinction is now drawn
        // where it belongs: the note explains why there is no value box, and
        // the section below offers the thing a push button actually has.
        //
        // ★ Gated on the DOCUMENT-wide refusal being absent. A certified or
        // encrypted document refuses `set_button_action` too, and offering a
        // Change control that the engine would decline is the affordance-for-
        // an-impossible-act shape R9 exists to prevent.
        if ctx.fill_refusal.is_none()
            && matches!(field.button_kind, Some(ButtonKind::Push))
            && let Some(doc) = ctx.doc
        {
            crate::panels::forms::button::row(ui, doc, field, ctx.button_draft, ctx.actions);
        }
        return;
    }

    match (field.field_type, field.button_kind) {
        (Some(FieldType::Text), _) if field.is_rich_text() => rich_text_row(ui, field, out),
        (Some(FieldType::Text), _) => text_row(
            ui,
            field,
            index,
            mirrored(ctx.live_canvas_draft.as_ref(), fqn),
            drafts,
            out,
        ),
        (Some(FieldType::Button), Some(ButtonKind::Check)) => check_row(ui, field, out),
        (Some(FieldType::Button), Some(ButtonKind::Radio)) => radio_row(ui, field, index, out),
        (Some(FieldType::Choice), _) => choice_row(ui, field, index, out),
        // A field with no resolved `/FT` at all. It has already been counted
        // and named above; there is nothing to offer, and inventing a control
        // for a field whose type the document never stated would be pdfcer
        // guessing what it is.
        _ => {}
    }
}

/// The row's visible label.
///
/// Prefers `/TU` — what a screen reader announces for an interactive field —
/// so the operator reads the same string an assistive technology speaks rather
/// than two different names for one field. The raw name is always in the
/// tooltip, because `/TU` may be absent, may differ, and is not what a data
/// file matches on.
///
/// A blank-but-present `/TU` falls back to the name rather than producing an
/// unlabelled row: the file has technically supplied one, and honouring it
/// literally would leave the operator with a control identified by nothing.
fn row_label(field: &Field, ctx: &RowContext<'_>) -> String {
    let mut label = field
        .alternate_name
        .as_ref()
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| field.fully_qualified_name.clone());
    if let Some(n) = field
        .widgets
        .first()
        .and_then(|w| w.page)
        .and_then(|p| ctx.page_numbers.get(&p))
    {
        label.push_str(&t::form_field_page_suffix(*n));
    }
    if field.flags.has(FieldFlags::REQUIRED) {
        label.push_str(t::form_field_required_marker());
    }
    label
}

/// Why this field's value cannot be changed here, in order of specificity.
///
/// `None` means the row may offer a live control.
///
/// # The order is deliberate
///
/// A read-only signature field is both, and being read-only is the more
/// actionable fact — it is the flag an operator or another tool set, whereas
/// "this is a signature" is what the field permanently is. Reporting the less
/// specific reason first would be less useful.
///
/// # ★ The rich-text check is NOT here, and must not move here
///
/// A rich-text field gets a row of its own ([`rich_text_row`]) rather than a
/// refusal, because there **is** something an operator can do with it: convert
/// it, disclosed and deliberately. Folding it in here would replace an offer
/// with a shrug.
///
/// # ★ `FieldFlags::RICH_TEXT` shares its bit with `RADIOS_IN_UNISON`
///
/// Bit 26 is the only overloaded position in the whole `/Ff` family, and
/// `field.flags.has(FieldFlags::RICH_TEXT)` **compiles and is wrong on every
/// radio group**. Everything in this module asks `Field::is_rich_text()`,
/// which gates on the resolved `/FT` first. Stated here as well as at the use
/// site because this is the function someone extends when a new refusal is
/// added.
///
/// # ★ `pub(crate)`, because a second surface asks it rather than restating it
///
/// [`crate::canvas::forms::classify`] decides whether a field may be clicked
/// **on the page**, and the first thing it must decide is whether the field may
/// be filled at all. That is this question, and it now has exactly one answer
/// for both surfaces. Re-deriving it in `canvas/` would be a second statement
/// of one rule, whose failure is silent and specific: an operator clicking a
/// field on the page that the panel beside it says is read-only.
pub(crate) fn block_reason(field: &Field) -> Option<&'static str> {
    if field.flags.read_only() {
        return Some(t::form_field_readonly_tooltip());
    }
    match field.field_type {
        Some(FieldType::Signature) => Some(t::form_field_signature_note()),
        Some(FieldType::Button) => match field.button_kind {
            Some(ButtonKind::Push) => Some(t::form_field_pushbutton_note()),
            _ => None,
        },
        _ => None,
    }
}

/// Draw a field whose value may be read but not changed.
///
/// The value is shown in a **disabled text box** for a text field and as a
/// plain label otherwise, which is the old shell's shape and survives review:
/// a box that looks like every other box, greyed, says "this is the same kind
/// of thing and you may not type in it". A label would say "this is a
/// different kind of thing", which is not true.
///
/// A field holding nothing draws no value at all rather than an empty label,
/// so a read-only empty field is not indistinguishable from a rendering fault.
fn blocked_row(ui: &mut egui::Ui, field: &Field, note: &'static str) {
    ui.add_enabled_ui(false, |ui| {
        let mut shown = field.value.display_text();
        if matches!(field.field_type, Some(FieldType::Text)) {
            ui.add(egui::TextEdit::singleline(&mut shown).desired_width(f32::INFINITY));
        } else if !shown.is_empty() {
            ui.label(&shown);
        }
    });
    ui.label(egui::RichText::new(note).small().weak());
}

/// A `/Ff` `RichText` field: read-only, plus the one conformant way through.
///
/// # Why it is not editable in place
///
/// **Correctness, not fidelity.** pdfcer cannot author `/RV`, and appearance
/// generation for these fields is bound to `/RV` rather than `/V` (§12.7.3.4
/// and §12.7.3.3, both `shall`). Writing plain text and leaving `/RV` behind
/// would make conforming readers rebuild the appearance from the OLD text —
/// the document would display words nobody typed.
///
/// So the row shows the value read-only and offers a **disclosed downgrade**:
/// convert the field to a plain one. Deliberate, named and lossy, which is why
/// it is a button the operator presses rather than something that happens when
/// they start typing. `SALVAGE.md` requires the disclosure to travel with the
/// capability; the old shell carried it, and it is carried here.
///
/// # Why the `/RV` is parsed every frame
///
/// The alternative is a cache keyed by field name, which would then need
/// invalidating on every edit, undo and reload — a correctness problem in
/// exchange for parsing a few hundred bytes of XML on a panel that is already
/// re-laying-out its whole field list. Measure before trading one for the
/// other.
fn rich_text_row(ui: &mut egui::Ui, field: &Field, out: &mut Vec<FormEdit>) {
    let shown = field.value.display_text();
    let mut display = shown.clone();
    ui.add_enabled(
        false,
        egui::TextEdit::singleline(&mut display).desired_width(f32::INFINITY),
    );
    ui.label(
        egui::RichText::new(t::form_field_rich_text_note())
            .small()
            .weak(),
    );

    // WHAT would be lost, named, above the button that loses it. The note
    // above says a category ("this field holds formatted text"); this says the
    // document. Always rendered, never on hover: the plain value is visible in
    // the read-only box above and the formatting is not, so putting a gesture
    // in front of the one invisible fact defeats the disclosure.
    //
    // `(message, per-run tooltip)`. Only the parsed case has a breakdown to
    // hover for; the two failure cases have a message and nothing behind it.
    let summary: Option<(String, Option<String>)> = field.rich_value.as_ref().map(|rv| {
        let Ok(text) = String::from_utf8(rv.clone()) else {
            return (t::form_field_rich_text_not_utf8(), None);
        };
        let ds = field
            .default_style
            .as_ref()
            .map(|d| String::from_utf8_lossy(d).into_owned());
        match pdfcer_core::richtext::parse(&text, ds.as_deref()) {
            Ok(runs) => (
                t::form_field_rich_text_summary(&runs),
                Some(t::form_field_rich_text_runs_tooltip(&runs)),
            ),
            Err(e) => (t::form_field_rich_text_unreadable(&e.to_string()), None),
        }
    });
    // `None` is bit 26 set with no `/RV` at all — Table 228 makes that
    // malformed, and there is nothing to describe. The note above already
    // covers it, and inventing a summary would assert formatting the file does
    // not contain.
    if let Some((s, detail)) = summary {
        let l = ui.label(egui::RichText::new(s).small().weak());
        if let Some(d) = detail {
            l.on_hover_text(d);
        }
    }

    if ui
        .button(t::form_field_rich_text_convert())
        .on_hover_text(t::form_field_rich_text_convert_tooltip())
        .clicked()
    {
        out.push(FormEdit::ConvertRichTextToPlain {
            field: field.fully_qualified_name.clone(),
            value: shown,
        });
    }
}

/// **Is the page's live draft about THIS row?**
///
/// `live` is the whole frame's answer — one field, or none
/// ([`RowContext::live_canvas_draft`]) — and this is the per-row half of the
/// question: the name has to match, because a live draft for *Address* says
/// nothing whatever about the *Name* row it is being asked beside.
///
/// # ★ A pure function, for the reason [`commit`] is one
///
/// The rule it states — *the page wins for the field the page is typing into,
/// and for no other* — is one line of code and two ways to get it wrong, both
/// silent: mirror unconditionally and every text row in the form shows one
/// field's draft; mirror never and A12c is back. Neither is visible in a diff
/// and neither needs an `egui::Ui` to demonstrate, so it is tested rather than
/// looked at.
fn mirrored<'a>(live: Option<&'a (String, String)>, fqn: &str) -> Option<&'a str> {
    live.filter(|(name, _)| name == fqn)
        .map(|(_, draft)| draft.as_str())
}

/// An ordinary `/Tx` field: a draft the operator types into, committed on
/// focus loss.
///
/// `live` is what the operator is typing into this same field **on the page**,
/// if they are — see [`mirrored`] and [`RowContext::live_canvas_draft`].
fn text_row(
    ui: &mut egui::Ui,
    field: &Field,
    index: usize,
    live: Option<&str>,
    drafts: &mut BTreeMap<String, String>,
    out: &mut Vec<FormEdit>,
) {
    let fqn = &field.fully_qualified_name;
    let stored = field.value.display_text();
    let draft = drafts.entry(fqn.clone()).or_insert_with(|| stored.clone());

    // ★★★ **THE PAGE WINS WHILE THE PAGE HAS THE KEYBOARD** — the 2026-09
    // review's row A12c, *"the Fill-form panel does not update while you type
    // on the page."*
    //
    // Two draft stores existed and only a commit reconciled them, so this row
    // showed the value from before the operator started typing, for as long as
    // they kept typing — two boxes on screen at once, disagreeing about one
    // field, with no way for the operator to tell which was the truth.
    //
    // ★★ This is an ASSIGNMENT rather than a second store being kept in step.
    // The row's own draft is simply overwritten with the page's, which means
    // there is exactly one uncommitted value for this field at any instant and
    // no reconciliation to get wrong. It is safe because `live` is `Some` only
    // while the page's editor owns the keyboard (`canvas::forms::live_draft`
    // checks `egui`'s focus, and `egui` has one focused widget), so this
    // cannot run on a frame where the operator is typing into the box below.
    //
    // ★ Left where it is — after `or_insert_with`, before `/MaxLen` — on
    // purpose. Before the seeding it would be undone by it; after the
    // truncation it would smuggle past a limit both surfaces enforce. Here the
    // mirrored value goes through exactly the same character clamp a typed one
    // does, which is what keeps the two surfaces' `/MaxLen` behaviour one rule
    // rather than two.
    if let Some(live) = live
        && draft.as_str() != live
    {
        draft.clear();
        draft.push_str(live);
    }
    let multiline = field.flags.has(FieldFlags::MULTILINE);
    let password = field.flags.has(FieldFlags::PASSWORD);

    // `/MaxLen` truncates LIVE rather than at commit: a limit discovered only
    // when the value is written is a limit the operator finds out about by
    // losing text.
    //
    // Counted in CHARACTERS, not bytes. `/MaxLen` is a limit on the text
    // string's length, and truncating by byte index would both cut a
    // multi-byte character in half and refuse an accented name three letters
    // early.
    if let Some(max) = field.max_len
        && max > 0
    {
        let max = usize::try_from(max).unwrap_or(usize::MAX);
        if draft.chars().count() > max {
            *draft = draft.chars().take(max).collect();
        }
    }

    let response = if multiline {
        ui.add(
            egui::TextEdit::multiline(draft)
                .id_salt(("pdfcer-forms-text", index))
                .desired_width(f32::INFINITY)
                .desired_rows(2),
        )
    } else {
        ui.add(
            egui::TextEdit::singleline(draft)
                .id_salt(("pdfcer-forms-text", index))
                .password(password)
                .desired_width(f32::INFINITY),
        )
    };
    response.clone().on_hover_text(if password {
        t::form_field_password_tooltip()
    } else {
        t::form_field_commit_tooltip()
    });

    if let Some(max) = field.max_len
        && max > 0
    {
        ui.label(
            egui::RichText::new(t::form_field_length_caption(draft.chars().count(), max))
                .small()
                .weak(),
        );
    }

    // ★★★ **POINT THE SPOTLIGHT AT THIS FIELD** — `OPERATOR_REQUESTS.md` O98,
    // *"when I click on fields in it … it should highlight the field on the
    // canvas that is being filled."*
    //
    // ★★ On **focus**, not on click, and the difference is his own word
    // *"filled"*. A click that lands in the value box focuses it, so clicking
    // lights it up — but so does arriving by Tab, and so does still being there
    // three keystrokes later. A click-only trigger would put the spotlight out
    // the moment the operator started typing, which is exactly when they want
    // to know which box on the page they are typing into.
    //
    // ★ Written every frame the box has focus rather than once on the
    // transition: that is what keeps it alive with no timer and no teardown,
    // and `spotlight::set` is idempotent.
    if response.has_focus() {
        crate::panels::forms::spotlight::set(ui.ctx(), fqn);
    }

    // ★★ The value box's own rectangle, so a driven check can CLICK a row.
    //
    // Added 2026-09-02 with O98's check, and it had to be added before the
    // check could exist: this panel published no per-row region at all, so
    // there was nothing to aim a pointer at and the whole feature was
    // unverifiable by the only method that counts. That is the third time on
    // this project a feature has needed the instrument built before the
    // evidence could be gathered.
    //
    // ★ Keyed on the row INDEX rather than the field name. The name is the
    // right identity for the spotlight channel — it crosses a frame boundary,
    // and an index into a walk of the form is only valid for the revision it
    // was taken from — but it is the wrong identity for a region name, because
    // a fully-qualified name legitimately contains dots, spaces and any byte a
    // PDF string can hold, and a region name is parsed out of a trace line.
    //
    // `ui_rect_visible` rather than `ui_rect`: this panel is a scroll area, and
    // a row scrolled out of view still reports a rectangle. A control that
    // exists and cannot be reached is a defect this project has met twice.
    crate::diag::ui_rect_visible(
        &format!("{REGION_ROW_PREFIX}{index}"),
        response.rect,
        ui.clip_rect(),
    );

    if let Some(text) = commit(response.lost_focus(), draft.as_str(), &stored) {
        out.push(FormEdit::FillText {
            field: fqn.clone(),
            value: text,
        });
    }
}

/// Should a text-field draft be written to the session yet?
///
/// Two conditions, and each prevents a distinct defect:
///
/// 1. **`ended`** — `lost_focus()`, not `changed()`. `TextEdit` reports
///    `changed()` on every keystroke and `EditSession::fill_text_field` pushes
///    one undo entry per call, so committing on `changed` would make one typed
///    word a dozen undo steps and a dozen appearance regenerations.
/// 2. **The draft differs from what the document already holds** — otherwise
///    tabbing THROUGH a field without typing writes a command whose only
///    effect is an undo entry the operator did not earn. This bites harder on
///    a form than anywhere else: tabbing through a form is how people read
///    one.
///
/// A pure function, so both conditions are tested without an egui context.
/// They are the sort of thing that stays correct for months and then gets
/// "simplified" into `if response.changed()`.
///
/// # ★ `pub(crate)`, because the canvas commits by the identical rule
///
/// [`crate::canvas::forms`] fills the same fields from the page, and both of
/// the conditions above bind there for exactly the reasons they bind here —
/// one word must not be a dozen undo entries, and moving the caret through a
/// field without typing must not write a command. So it **calls this** rather
/// than restating it, and the three tests below are the tests for both
/// surfaces.
pub(crate) fn commit(ended: bool, draft: &str, stored: &str) -> Option<String> {
    if !ended || draft == stored {
        return None;
    }
    Some(draft.to_owned())
}

/// A `/Btn` check box.
///
/// Immediate commit, no draft: a check box has one atomic change and no
/// intermediate state to protect, so the sixty-undo-entries argument that
/// governs the text rows does not apply.
///
/// See this module's header for why an on-state-less box is drawn disabled
/// rather than being offered and refused.
fn check_row(ui: &mut egui::Ui, field: &Field, out: &mut Vec<FormEdit>) {
    let on_state = check_on_state(field);
    let is_on = match (&field.value, on_state.as_ref()) {
        (FieldValue::Name(n), Some(want)) => n == want,
        _ => false,
    };

    let Some(on_state) = on_state else {
        // No ON state anywhere in the widgets: `set_button_state` would refuse
        // every name but `Off`, so there is no click to offer. Disabled and
        // explained (R83), never hidden — the operator must still be able to
        // read the field's state.
        let mut shown = is_on;
        ui.add_enabled(false, egui::Checkbox::new(&mut shown, ""))
            .on_disabled_hover_text(t::form_field_no_on_state_note());
        ui.label(
            egui::RichText::new(t::form_field_no_on_state_note())
                .small()
                .weak(),
        );
        return;
    };

    let mut checked = is_on;
    if ui.checkbox(&mut checked, "").changed() {
        // `Off` is the §12.7.4.2.3 name for the cleared state of every check
        // box, whatever its ON state happens to be called. Core accepts it
        // unconditionally, which is why clearing never needs the widgets to
        // declare it.
        let state = if checked {
            String::from_utf8_lossy(&on_state).into_owned()
        } else {
            "Off".to_owned()
        };
        out.push(FormEdit::SetButtonState {
            field: field.fully_qualified_name.clone(),
            state,
        });
    }
}

/// The on-state name a tick would select, or `None` when the document
/// declares none.
///
/// The **first** state any widget offers, in widget order. A check box has one
/// ON state by definition (§12.7.4.2.3), so a second would be a malformation;
/// taking the first rather than asserting there is exactly one means a
/// malformed file still gets a working control instead of a refusal.
///
/// Extracted so [`check_row`]'s two paths — the live control and the disabled
/// one — read the same answer, rather than one of them re-deriving it.
fn check_on_state(field: &Field) -> Option<Vec<u8>> {
    field
        .widgets
        .iter()
        .find_map(|w| w.on_states.first().cloned())
}

/// A `/Btn` radio group.
///
/// A radio GROUP is one field with several widgets, each carrying its own
/// on-state name; the field's `/V` is whichever name is selected, or `Off`. So
/// the control is one exclusive cluster over the **distinct** on-states, NOT a
/// check box per widget — which is exactly the shape
/// `set_button_state(fqn, on_state)` takes.
///
/// Duplicates are real and meaningful — two kids sharing an on-state name is
/// what `RadiosInUnison` describes — but they are ONE choice to the operator,
/// so they get one control, and core turns every kid with that name on
/// together for free.
fn radio_row(ui: &mut egui::Ui, field: &Field, index: usize, out: &mut Vec<FormEdit>) {
    let current = match &field.value {
        FieldValue::Name(n) => String::from_utf8_lossy(n).into_owned(),
        _ => String::new(),
    };

    let states = radio_states(field);
    if states.is_empty() {
        // No on-state anywhere means nothing selectable — said out loud rather
        // than drawn as an empty cluster that looks broken (R83).
        ui.label(
            egui::RichText::new(t::form_field_radio_no_states())
                .small()
                .weak(),
        );
        return;
    }

    ui.push_id(("pdfcer-forms-radio", index), |ui| {
        for state in &states {
            let selected = current == *state;
            if ui.radio(selected, state.as_str()).clicked() && !selected {
                out.push(FormEdit::SetButtonState {
                    field: field.fully_qualified_name.clone(),
                    state: state.clone(),
                });
            }
        }
        // Clearing is offered ONLY when the field permits it. `NoToggleToOff`
        // means exactly one button is always selected, so a Clear control
        // there would be an affordance for something the engine will refuse
        // (R83).
        if !field.flags.has(FieldFlags::NO_TOGGLE_TO_OFF)
            && ui
                .button(t::form_field_radio_clear())
                .on_hover_text(t::form_field_radio_clear_tooltip())
                .clicked()
        {
            out.push(FormEdit::SetButtonState {
                field: field.fully_qualified_name.clone(),
                state: "Off".to_owned(),
            });
        }
    });
}

/// The distinct on-state names a radio group offers, in widget order.
///
/// Order is the document's, never sorted: the widgets are laid out on the page
/// in an order the form's designer chose, and re-ordering the cluster would
/// make the panel disagree with what the operator is looking at.
///
/// A `Vec` with a linear `contains` rather than a set, deliberately: a radio
/// group has a handful of options, order must be preserved, and a set that
/// preserved insertion order would be a dependency for nothing.
fn radio_states(field: &Field) -> Vec<String> {
    let mut states: Vec<String> = Vec::new();
    for w in &field.widgets {
        for st in &w.on_states {
            let name = String::from_utf8_lossy(st).into_owned();
            if !states.contains(&name) {
                states.push(name);
            }
        }
    }
    states
}

/// A `/Ch` choice field — a list box or a combo box.
///
/// # Options are displayed in `/Opt` ORDER, never sorted
///
/// §12.7.4.4: a conforming reader SHALL display them in the order they occur
/// in `/Opt`. Re-sorting is a conformance violation, not a presentation
/// choice, and the `Sort` flag is an instruction to the **writer**.
///
/// # `/V` stores the EXPORT value and the operator must see the DISPLAY one
///
/// `/Opt` entries may be `[export display]` pairs, so rendering `/V` verbatim
/// shows an operator `MX` where the form says `Mexico`. The old shell caught
/// this with a screenshot of a fixture built with export deliberately unequal
/// to display; the mapping is carried here.
///
/// A `/V` that matches no option is a real state — set by another program, or
/// left behind when the option list changed — so it is shown as stored, with
/// [`crate::text::forms::form_field_choice_value_not_listed`] beside it.
/// Showing blank would claim the field is unanswered when it is not.
fn choice_row(ui: &mut egui::Ui, field: &Field, index: usize, out: &mut Vec<FormEdit>) {
    if field.options.is_empty() {
        ui.label(
            egui::RichText::new(t::form_field_choice_no_options())
                .small()
                .weak(),
        );
        return;
    }

    let selected_now = choice_selections(field);
    let options: Vec<(String, String)> = field
        .options
        .iter()
        .map(|o| {
            (
                pdfcer_core::edit::decode_text_string(&o.export).text,
                pdfcer_core::edit::decode_text_string(&o.display).text,
            )
        })
        .collect();

    if field.flags.has(FieldFlags::MULTI_SELECT) {
        // A check-box stack: several selections are the point, and a combo
        // cannot express "these three".
        let mut wanted = selected_now;
        let mut changed = false;
        ui.push_id(("pdfcer-forms-choice-multi", index), |ui| {
            for (export, display) in &options {
                // Matched on EXPORT or DISPLAY: `/V` may hold either in the
                // wild, and a strict match on one shows a filled field as
                // empty.
                let mut on = wanted.iter().any(|v| v == export || v == display);
                if ui.checkbox(&mut on, display).changed() {
                    changed = true;
                    if on {
                        wanted.push(export.clone());
                    } else {
                        wanted.retain(|v| v != export && v != display);
                    }
                }
            }
        });
        if changed {
            out.push(FormEdit::SetChoice {
                field: field.fully_qualified_name.clone(),
                values: wanted,
            });
        }
        return;
    }

    let current = selected_now.first().cloned().unwrap_or_default();
    let matched = options
        .iter()
        .find(|(export, display)| *export == current || *display == current);
    let shown = match &matched {
        Some((_, display)) => display.clone(),
        None => current.clone(),
    };

    egui::ComboBox::from_id_salt(("pdfcer-forms-choice", index))
        .selected_text(if current.is_empty() {
            t::form_field_choice_unset().to_owned()
        } else {
            shown
        })
        .show_ui(ui, |ui| {
            for (export, display) in &options {
                let selected = current == *export || current == *display;
                if ui.selectable_label(selected, display).clicked() && !selected {
                    out.push(FormEdit::SetChoice {
                        field: field.fully_qualified_name.clone(),
                        values: vec![export.clone()],
                    });
                }
            }
        });

    if !current.is_empty() && matched.is_none() {
        ui.label(
            egui::RichText::new(t::form_field_choice_value_not_listed())
                .small()
                .weak(),
        );
    }
}

/// A choice field's current selections, as decoded display strings.
///
/// Accepts both shapes `/V` legally takes: an array for a `MultiSelect` field
/// and a bare string for a single-select one. A reader that handled only the
/// array form would show every ordinary combo box as unanswered.
fn choice_selections(field: &Field) -> Vec<String> {
    match &field.value {
        FieldValue::Choice(items) => items
            .iter()
            .map(|b| pdfcer_core::edit::decode_text_string(b).text)
            .collect(),
        FieldValue::Text(b) => vec![pdfcer_core::edit::decode_text_string(b).text],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **★ Tabbing through a field writes nothing.**
    ///
    /// The second half of [`commit`]'s condition, and the one that is easy to
    /// drop. Reading a form means tabbing through every field in it; if that
    /// wrote a command per field, an operator who merely *looked* at a
    /// forty-field form would have forty undo entries and a modified document
    /// they never edited.
    #[test]
    fn leaving_an_untouched_field_writes_nothing() {
        assert_eq!(commit(true, "Anna", "Anna"), None);
        // …and an empty field left empty is the same case, which is the one a
        // `draft.is_empty()` guard would get wrong in the other direction.
        assert_eq!(commit(true, "", ""), None);
    }

    /// **A keystroke is not a commit.**
    ///
    /// `TextEdit::changed()` fires per keystroke and `fill_text_field` pushes
    /// one undo entry per call, so committing on change would make one typed
    /// word a dozen undo steps — and a dozen appearance regenerations, each
    /// re-rasterizing the page.
    #[test]
    fn typing_does_not_commit_until_focus_leaves() {
        assert_eq!(commit(false, "Ann", "Anna"), None);
        assert_eq!(commit(true, "Ann", "Anna"), Some("Ann".to_owned()));
    }

    /// ★★★ **The page's draft reaches its own row, and reaches no other.**
    ///
    /// Row **A12c**. Both halves are asserted because both are silent
    /// failures: without the first the panel goes on showing the value from
    /// before the operator started typing on the page — two boxes disagreeing
    /// about one field — and without the second every text row in the form
    /// shows whatever is being typed into one of them, which is worse than the
    /// lag it replaced.
    #[test]
    fn the_pages_draft_reaches_its_own_row_and_no_other() {
        let live = ("Name".to_owned(), "Ann".to_owned());

        assert_eq!(mirrored(Some(&live), "Name"), Some("Ann"));
        assert_eq!(
            mirrored(Some(&live), "Address"),
            None,
            "another field's draft says nothing about this row"
        );
        // Nobody typing on the page: every row keeps its own draft.
        assert_eq!(mirrored(None, "Name"), None);
    }

    /// **Clearing a field is a real edit.**
    ///
    /// The case a `!draft.is_empty()` guard would silently swallow: emptying a
    /// field the document has a value for is exactly as much an edit as typing
    /// into an empty one, and it is the gesture an operator makes to correct a
    /// mistake.
    #[test]
    fn emptying_a_filled_field_is_committed() {
        assert_eq!(commit(true, "", "Anna"), Some(String::new()));
    }
}
