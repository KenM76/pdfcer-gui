//! # `canvas::fieldclip` — **cut, copy and paste a FORM FIELD**
//!
//! ## What this closes
//!
//! **Ken, 2026-08-29:** *"wire the request. ctrl v for paste as new. ctrl shift
//! v for paste as duplicate."* — `OPERATOR_REQUESTS.md` **O58**.
//!
//! Before this module there was **no path at all** from a selected form field
//! to `Ctrl+C`. Not a lossy one, not a refused one — none. The reason is two
//! deliberate decisions that were each correct and that together left a hole:
//!
//! 1. `canvas::selection::annot`'s exclusion table drops `/Widget` — *"the form
//!    field surface owns it — a click there focuses an editor, and two owners
//!    of one press is how a field becomes unfillable"*. A selected field
//!    therefore lives on `OpenDoc::selected_field`, not in `SelectionState`.
//! 2. [`crate::canvas::clipboard::copy`] reads `doc.selection` and nothing
//!    else.
//!
//! So `Ctrl+C` over a form field fell through to the *content* copy, which
//! looked at an empty content selection and refused with *"nothing is
//! selected"* — over an object with visible grips around it. That is
//! `DEFECTS.md` D4a's shape exactly: a refusal whose sentence describes a
//! different world than the one the operator is looking at.
//!
//! ## The two pastes
//!
//! Copying a field has two legitimate meanings and `pdfcer-core` refuses to
//! guess between them. The operator made the decision, and made **both**
//! answers reachable on two chords:
//!
//! | chord | [`PasteAs`] | engine policy | the value |
//! |---|---|---|---|
//! | `Ctrl+V` | [`PasteAs::NewField`] | `FieldPastePolicy::NewField` | its own |
//! | `Ctrl+Shift+V` | [`PasteAs::Duplicate`] | `FieldPastePolicy::AdditionalWidget` | shared — type in one, both fill |
//!
//! Each engine policy **refuses the other's situation by name**: a `NewField`
//! onto a taken name is `FieldNameTaken` and never a silent merge, and an
//! `AdditionalWidget` naming a field the document does not have is
//! `FieldNotFound` and never a silent creation. That matters because *the
//! difference between an independent field and a linked one is invisible on the
//! page* — it shows up only when somebody types in one and the other does not
//! follow.
//!
//! ## ★★★ This module was rewritten the day it was written, and the rewrite is
//! the interesting part
//!
//! The first version **re-authored**: it read the source field into a
//! `canvas::formfield::Draft` and pushed that back through `add_text_field` and
//! its four siblings. That worked, and it was lossy in eight measurable ways —
//! `/DA` (font, size, colour), `/Q`, `/DV`, `/AA`, `/MK` colours, `/BS` styles,
//! the flags no `New*Field` spec can express, and the baked `/AP`. Each was
//! *readable* on `forms::Field` and *writable* nowhere, so the shell carried a
//! hand-written table of what survived and disclosed it on the status row.
//!
//! `Pass 167.0` shipped `pdfcer_core::formclip` in answer to this project's own
//! request, **within the hour**, and every row of that table now travels. So:
//!
//! - The `Lost` enum is **deleted**, not deprecated. The engine's own words:
//!   *"delete the fidelity table — you should not be maintaining a hand-written
//!   map of which properties survive, because it rots silently every time we
//!   add an authoring key. The clip does not express properties, it carries
//!   them."*
//! - Two things travel that this shell **did not know to ask for**: the actual
//!   font its `/DA` names (installed into the destination's `/AcroForm /DR`,
//!   renamed if that name is taken there, with the `/DA` rewritten to match),
//!   and `/Ff` as an integer, which brings `DoNotSpellCheck`, `DoNotScroll`,
//!   `FileSelect`, `RichText` and `CommitOnSelChange` for free.
//! - **Signature fields are no longer refused.** An *unsigned* one copies and
//!   pastes normally, which — as the engine points out — hands this shell
//!   signature-field *authoring* it never had, because there is still no
//!   `add_signature_field`. A **signed** one is refused at the **copy**, by the
//!   engine, so the operator learns before spending a placement gesture.
//!
//! ⇒ The general lesson, and it has cost this project three days across three
//! separate capabilities: **a reply arriving is not a capability landing.** The
//! engine session works in parallel and answers within the hour; the failure
//! mode is a shell that files a request, ships a workaround, and never comes
//! back. This module came back the same afternoon and recovered eight
//! properties' worth of fidelity by doing so.
//!
//! ## ★★ Rule 4 — disclosure is the ENGINE's now, and that is a simplification
//!
//! A pasted field renders exactly as a saved-and-reopened one would. No badge,
//! no tint, no "this copy is incomplete" marker anywhere on the page, because
//! provisional styling is a second rendering path for the same content and two
//! paths drift.
//!
//! The half of rule 4 that binds is the off-canvas report, and
//! `FieldPasteOutcome::disclosures` is where it now lands — a `Vec<String>` the
//! engine builds, covering a dropped value, dropped actions, a carried
//! calculation and its `/CO` entry, a **renamed font resource**, an ignored
//! rectangle size, the tab-order position, a dropped structure-tree link and a
//! reused accessibility name. This shell surfaces it verbatim rather than
//! re-deriving any of it, which is the same *one fact, one wording* rule that
//! removed this module's own merge sentence a few hours earlier.
//!
//! ## ★ Radio groups travel whole, and that changes what the rectangle means
//!
//! `copy_field` on a radio field carries **every** widget in `/Kids` order with
//! its own rectangle and export value. On a `NewField` paste the group is
//! **translated** so widget 0's lower-left lands on the target point; every
//! widget keeps its size and its offset from widget 0, and **the target
//! rectangle's size is ignored**. The engine discloses that, and this module
//! does not try to be cleverer: a best-fit rescale of a radio group into a
//! rectangle is a guess that looks deliberate, and which button sits above
//! which is part of the group's meaning.
//!
//! On `AdditionalWidget` exactly **one** widget is placed even from a
//! multi-widget clip, because adding all N would give one field several views
//! with duplicate export values — radio buttons that select together.

use pdfcer_core::formclip::{FieldClip, FieldPastePolicy, PasteTooltip};
use pdfcer_core::page_tree::Rect;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;

/// Which of the two pastes the operator asked for.
///
/// A two-variant enum rather than a `bool`, because `paste(ctx, doc, true)` at
/// the call site says nothing about which is which, and the two differ in what
/// they do to the operator's *form* rather than merely in where a copy lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteAs {
    /// `Ctrl+V` — a new, independent field carrying a fresh name.
    NewField,
    /// `Ctrl+Shift+V` — another widget of the field that was copied.
    Duplicate,
}

/// Why a field could not be copied, cut or pasted.
///
/// A sentence on the status row, never a silence — the same posture
/// [`crate::canvas::clipboard::Refusal`] takes, for the same D4a reason.
///
/// ★ Two variants went when `formclip` landed: `KindCannotBeAuthored` (a
/// signature field, now copyable) and `RadioNeedsItsOwnExportValue` (the engine
/// refuses the collision itself, with a better message).
/// [`EngineRefused`](Self::EngineRefused) carries both now, in the engine's
/// wording — which is the wording that is right, because it reports what the
/// operation *did* rather than what this shell *intended*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// No form field is selected.
    NothingSelected,
    /// The selection names a field the document no longer has.
    Vanished,
    /// The widget has no `/Rect`, so there is no box to land the paste against.
    NoGeometry,
    /// The clipboard holds no form field.
    NothingCopied,
    /// ★★ **The engine declined, in its own words.**
    ///
    /// A `String` rather than a mirror of `EditError`'s taxonomy, for the
    /// reason `canvas::clipboard::Refusal::EngineRefused`'s doc gives about the
    /// same choice: a shell that modelled the engine's internals a second time
    /// is decision 058's failure mode. The cases that actually arrive here —
    /// `SignedFieldNotCopyable`, `FieldNameTaken`, `FieldNotFound`,
    /// `RadioExportValueTaken`, and the encryption and certification guards —
    /// each already carry a sentence written by the party that knows why.
    EngineRefused(String),
}

/// What the clipboard is holding when a form field was copied.
///
/// Parked inside [`crate::canvas::clipboard::Clipped`] rather than under a key
/// of its own, so that **one clipboard holds one thing**: copying a markup
/// after copying a field replaces it, which is what every program in the class
/// does and what makes `Ctrl+V` mean one thing at a time.
///
/// # ★ Why the BYTES and not the live `FieldClip`
///
/// The same three reasons `Clipped::Content` carries bytes, and here the third
/// is decisive rather than merely convenient:
///
/// 1. `egui::Memory` wants `Clone + Send + Sync + 'static`; bytes are all four
///    without asking anything of the engine's type.
/// 2. `Clipped` derives `PartialEq`, which bytes give for free.
/// 3. **`FieldClip::to_bytes` is total.** A field clip is dictionaries and
///    streams, and the engine tests that a clip through bytes and one that
///    stayed in memory produce **byte-identical documents**. So this
///    representation loses nothing, and it is the same one a private OS
///    clipboard format will take.
///
/// ★ This used to add *"unlike `ObjectClip`, whose `to_bytes` drops its
/// annotations"*, and **that stopped being true on 2026-08-29** — clip format
/// version 2 carries them, and `annotations_survive_serialisation()` now
/// answers `true` for every clip. Corrected here rather than deleted, because
/// the contrast was the reason this field is bytes and a reader who finds the
/// claim elsewhere should know it expired rather than that it was wrong. It is
/// the third stale absence-claim about `pdfcer-core` this project has corrected
/// in a week: **an absence claim about a crate you do not build has a shelf
/// life**, and what catches it is reading the reply, not re-deriving the claim.
#[derive(Debug, Clone, PartialEq)]
pub struct ClippedField {
    /// `FieldClip::to_bytes` — magic `PDFCERFLD…`, versioned, count-guarded.
    pub bytes: Vec<u8>,
    /// The source field's fully-qualified name.
    ///
    /// Carried rather than re-read from the clip on every glance: it seeds the
    /// candidate name for a new-field paste, addresses the field for a
    /// duplicate, and names the field in the OS-clipboard marker.
    pub name: String,
    /// The 0-based page it came from.
    ///
    /// Carried for [`crate::canvas::clipboard::PASTE_OFFSET_PT`]'s rule: a
    /// paste onto the same page offsets so the copy is visible, a paste onto a
    /// different one lands in place because *where it was on sheet 1* is the
    /// whole point of copying it to sheet 12.
    pub page: usize,
    /// The source widget's `/Rect`, in PDF user space — where the paste is
    /// measured from.
    pub rect: Rect,
    /// How many widgets the clip carries.
    ///
    /// ★ `> 1` means a radio group, and it changes what a paste rectangle
    /// *means* — see the module header. Carried so a caller can ask without
    /// deserialising.
    pub widgets: usize,
    /// Whether the field brings a calculation or format script with it.
    ///
    /// ★★ The one thing worth knowing **before** the press, and the operator
    /// cannot see it: a field in a calculation chain looks identical to one
    /// that is not. The engine can say a script is coming and deliberately does
    /// **not** resolve the field names inside it — Acrobat is documented
    /// silently dropping a copied script that references a field the target
    /// lacks, discovered only on reopen, and naming the uncertainty beats
    /// half-analysing it.
    pub carries_actions: bool,
}

/// **Copy the selected form field**, writing it to the shared clipboard.
///
/// # Errors
///
/// Every [`Refusal`] except [`Refusal::NothingCopied`], which only a paste
/// raises.
pub fn copy(ctx: &egui::Context, doc: &OpenDoc) -> Result<ClippedField, Refusal> {
    let clipped = read_selected(doc)?;
    crate::canvas::clipboard::store(
        ctx,
        crate::canvas::clipboard::Clipped::FormField(Box::new(clipped.clone())),
    );
    // ★★★ AND THE OS CLIPBOARD, WITHOUT WHICH CTRL+V DOES NOT ARRIVE AT ALL.
    //
    // Not a courtesy to other applications: `egui-winit` pushes `Event::Paste`
    // **only if the OS clipboard holds non-empty text**, and swallows the
    // keystroke otherwise — so with nothing here, whether a field paste works
    // depends on what the operator last copied in Notepad.
    //
    // ⇒ Found by driving on the day this module was written, with the trap
    // already documented in the RAG and already handled one function away in
    // `clipboard::copy_content`. See `text::fieldclip::os_marker`.
    ctx.copy_text(crate::text::fieldclip::os_marker(&clipped.name));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "fieldclip-copy name={} page={} widgets={} actions={} bytes={}",
            clipped.name,
            clipped.page,
            clipped.widgets,
            clipped.carries_actions,
            clipped.bytes.len()
        )
    });
    Ok(clipped)
}

/// **Cut the selected form field** — copy it, then raise the delete.
///
/// The delete travels as an [`Action`] rather than being applied here, because
/// this function borrows `doc` immutably and because every other destructive
/// gesture in this shell goes through the queue. `DeleteWidget` rather than
/// `DeleteField` is deliberate: the operator pointed at **a box**, and on a
/// field with three boxes removing all three is not what they asked for. The
/// engine collapses the field when its last widget goes.
///
/// # Errors
///
/// As [`copy`].
pub fn cut(
    ctx: &egui::Context,
    doc: &OpenDoc,
    actions: &mut Vec<Action>,
) -> Result<ClippedField, Refusal> {
    let clipped = copy(ctx, doc)?;
    if let Some(selected) = doc.selected_field.as_ref() {
        actions.push(Action::Field(
            crate::app::actions::forms::FieldAction::DeleteWidget {
                field: selected.field.clone(),
                widget: selected.widget,
            },
        ));
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("fieldclip-cut name={}", clipped.name)
    });
    Ok(clipped)
}

/// **Paste the clipboard's form field onto `page`**, in one of the two senses.
///
/// # Where it lands
///
/// [`crate::canvas::clipboard::PASTE_OFFSET_PT`] down and to the right on the
/// **same** page, in place on a different one. The same rule the markup
/// clipboard uses and for the same two reasons, which pull in opposite
/// directions and are both right: a same-page paste that landed exactly on the
/// original is invisible, and a cross-page paste that offset would move the
/// copy away from the position that was the reason for copying it.
///
/// ★ A [`PasteAs::Duplicate`] onto the same page offsets too. Two widgets of one
/// field stacked exactly on each other is a form the operator cannot separate,
/// and the fact that they share a value does not make them one box.
///
/// # Errors
///
/// [`Refusal::NothingCopied`] when the clipboard holds no field. The engine's
/// own refusals arrive when the action drains, not here.
pub fn paste(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    mode: PasteAs,
    // ★ Where the pointer is, in PDF user space, or `None` when the canvas has
    // never drawn. `OPERATOR_REQUESTS.md` O73; resolved once by
    // `app::dispatch::clipboard::paste` so all three paste kinds land in the
    // same place for the same reason.
    target: Option<egui::Pos2>,
    actions: &mut Vec<Action>,
) -> Result<(), Refusal> {
    let Some(crate::canvas::clipboard::Clipped::FormField(clipped)) =
        crate::canvas::clipboard::read(ctx)
    else {
        return Err(Refusal::NothingCopied);
    };

    let rect = placed_rect(clipped.rect, clipped.page, page, target);
    let policy = match mode {
        PasteAs::NewField => FieldPastePolicy::NewField {
            name: unique_name(doc, &clipped.name),
            // ★★ `Carry` — reuse the source's `/TU`, which R105 accepts as an
            // explicit decision because it is the operator's own field rather
            // than an invented name. The engine refuses `Undecided` outright
            // and discloses the reuse, because two fields announcing themselves
            // identically to a screen reader is invisible to a sighted
            // operator.
            //
            // The alternative — opening a dialog on every Ctrl+V to ask — is
            // the interruption this whole gesture exists to avoid. Four boxes
            // down a column should be four keystrokes.
            tooltip: PasteTooltip::Carry,
            // ★★★ A new field starts EMPTY, and this is the one place the shell
            // overrides "reproduce what was copied".
            //
            // A value is content, not a property. Copying the title-block box
            // called `Drawn By` to make one called `Checked By`, and having the
            // second arrive pre-filled with the first person's name, is a form
            // that is wrong on paper the moment it is printed. The engine's own
            // `field_defaults` excludes `/V` for the same reason.
            //
            // ★ `/DV` travels **regardless** — the engine carries it whether or
            // not this is set, and it is right to: a default is the *reset
            // target*, not content, and dropping it makes Reset restore the
            // wrong thing silently.
            copy_value: false,
            // ★ Actions DO travel. A copied field in a calculation chain that
            // arrives inert is a defect nothing on the page reveals, which is
            // the worst kind. `carries_actions` is disclosed before the press
            // and the engine discloses the `/CO` registration after it.
            copy_actions: true,
        },
        // ★ Addressed by the SOURCE's name, which is what makes the two widgets
        // one field. The engine refuses with `FieldNotFound` when that name is
        // not in this document — a duplicate paste across documents is
        // meaningless and must not fall back to creating a field, because the
        // operator pressed a different key on purpose.
        PasteAs::Duplicate => FieldPastePolicy::AdditionalWidget {
            existing: clipped.name.clone(),
        },
    };

    actions.push(Action::Field(
        crate::app::actions::forms::FieldAction::Paste {
            page,
            rect,
            clip: clipped.bytes.clone(),
            policy: Box::new(policy),
        },
    ));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "fieldclip-paste mode={mode:?} page={page} from_page={} widgets={}",
            clipped.page, clipped.widgets
        )
    });
    Ok(())
}

/// Whether the clipboard's field brings a script, for the pre-press sentence.
///
/// `None` when the clipboard holds no field. Split out so a status surface can
/// ask without knowing the clip's shape.
#[must_use]
pub fn carries_actions(ctx: &egui::Context) -> Option<bool> {
    match crate::canvas::clipboard::read(ctx)? {
        crate::canvas::clipboard::Clipped::FormField(c) => Some(c.carries_actions),
        _ => None,
    }
}

/// Where the pasted box goes. See [`paste`]'s header for the two rules.
fn placed_rect(source: Rect, from_page: usize, to_page: usize, target: Option<egui::Pos2>) -> Rect {
    // ★★★ **The pointer wins where there is one** — `OPERATOR_REQUESTS.md`
    // O73. The field keeps its size and is **centred** on the cursor, which is
    // the same rule `canvas::clipboard` applies to a markup and to page
    // content: three paste kinds, one answer to *"where does it land"*.
    //
    // Size preserved rather than the rect being placed corner-first, because a
    // widget's extent is a property of the field the operator copied and
    // nothing about pointing at a spot asks for it to change.
    let (dx, dy) = match target {
        Some(t) => (
            f64::from(t.x) - (source.llx + source.urx) / 2.0,
            f64::from(t.y) - (source.lly + source.ury) / 2.0,
        ),
        // The two older rules, unchanged, as the fallback: a different page
        // lands in place so a field copied to sheet 12 is where it was on
        // sheet 1; the same page offsets so the copy is visible rather than
        // stacked invisibly on its original.
        None if from_page != to_page => return source,
        None => {
            let d = crate::canvas::clipboard::PASTE_OFFSET_PT;
            (d, -d)
        }
    };
    Rect {
        llx: source.llx + dx,
        lly: source.lly + dy,
        urx: source.urx + dx,
        ury: source.ury + dy,
    }
}

/// A field name this document does not use, derived from `base`.
///
/// `Text1` → `Text2` → `Text3`, and `Drawn By` → `Drawn By2`. The spelling is
/// [`crate::text::fieldclip::candidate_name`]'s — a field name is
/// operator-facing text — and the *numbering* is [`split_trailing_number`]'s,
/// which is logic and belongs here.
///
/// ★★ The convention is Acrobat's, sourced rather than invented: its bulk
/// duplication auto-names copies `Date1`, `Date2`, `Date3`, and the separator is
/// load-bearing rather than cosmetic. `candidate_name`'s header carries both the
/// scripting rationale and the reason a **dot** is refused even though one
/// Acrobat account uses it.
///
/// ★ The name is generated here rather than by the engine, at the engine's own
/// insistence: *"an engine-invented name is a name nobody chose."* `paste_field`
/// refuses a taken name with `FieldNameTaken` and never auto-suffixes, so this
/// is the only place a candidate comes from.
///
/// Falls back to the base itself past a thousand tries, which then hits
/// `FieldNameTaken` and surfaces as a refusal. Unreachable in practice, and
/// written as a bounded loop because an unbounded loop over a document is a
/// hang.
fn unique_name(doc: &OpenDoc, base: &str) -> String {
    let view = doc.session.view();
    let Some(form) = pdfcer_core::forms::parse_acroform(&view) else {
        return base.to_owned();
    };
    let taken = |candidate: &str| form.fields_named(candidate).next().is_some();
    if !taken(base) {
        return base.to_owned();
    }
    let (stem, start) = split_trailing_number(base);
    // Bounded, not unbounded. `start` can be large if the operator numbered a
    // field `Rev2000`, so the ceiling is relative rather than absolute — a fixed
    // `2..1000` would give up immediately on a high-numbered base.
    for n in start..start.saturating_add(1000) {
        let candidate = crate::text::fieldclip::candidate_name(stem, n);
        if !taken(&candidate) {
            return candidate;
        }
    }
    base.to_owned()
}

/// Split a field name into its stem and the number to try first.
///
/// ★★ `Text1` → `("Text", 2)`, not `("Text1", 2)`. **Continuing an existing
/// number is the whole point**, and getting it wrong is what produced `Text1 2`.
///
/// This shell's own placement dialog names a new text field `Text1` — Acrobat's
/// convention, already numbered — so a base *with* a trailing number is the
/// ordinary case here, not the exotic one. A rule that only appended would
/// produce `Text12` from `Text1`, which reads as "field twelve" and sorts
/// nowhere near its source.
///
/// A base with no trailing number starts at **2**, because the source itself is
/// the unwritten 1: `Drawn By` and `Drawn By2` are a pair, `Drawn By1` beside a
/// bare `Drawn By` is not.
///
/// The digits are parsed as `u32` and a name whose trailing run does not fit —
/// `Rev99999999999` — falls back to treating the whole thing as the stem. That
/// is a name nobody has, and it is a branch rather than an `unwrap` because a
/// panic here would land on the operator's paste.
fn split_trailing_number(base: &str) -> (&str, u32) {
    let digits_start = base
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_digit())
        .map(|(i, _)| i)
        .last();
    match digits_start {
        // ★ `Some(0)` means the name is ALL digits — a field called `12`. The
        // stem is empty and the paste is named `13`, which is a legal field name
        // and a terrible one, but it is exactly what the operator's own scheme
        // implies. Left alone deliberately.
        Some(i) => match base[i..].parse::<u32>() {
            Ok(n) => (&base[..i], n.saturating_add(1)),
            Err(_) => (base, 2),
        },
        None => (base, 2),
    }
}

/// Read `doc.selected_field` into a clip, without touching the clipboard.
///
/// ★ Almost all of what this used to do is now one `copy_field` call. The
/// previous version read a `forms::Field`, mapped its type to a
/// `FormFieldKind`, decoded four text strings, translated eleven flags and
/// assembled a `Draft` — about eighty lines, every one of them a chance to
/// disagree with the engine about what a field is. What remains is the two
/// facts the *shell* owns: which widget the operator clicked, and where it is.
fn read_selected(doc: &OpenDoc) -> Result<ClippedField, Refusal> {
    let selected = doc
        .selected_field
        .as_ref()
        .ok_or(Refusal::NothingSelected)?;

    // The rect comes from the widget the operator actually clicked, which the
    // clip does not know: a radio group's clip carries every widget, and the
    // paste is measured from where the pointer was.
    let view = doc.session.view();
    let form = pdfcer_core::forms::parse_acroform(&view).ok_or(Refusal::Vanished)?;
    let field = form
        .fields_named(&selected.field)
        .next()
        .ok_or(Refusal::Vanished)?;
    let rect = field
        .widgets
        .get(selected.widget)
        .ok_or(Refusal::Vanished)?
        .rect
        .ok_or(Refusal::NoGeometry)?;

    let clip: FieldClip = doc
        .session
        .copy_field(&selected.field)
        .map_err(|e| Refusal::EngineRefused(e.to_string()))?;

    Ok(ClippedField {
        bytes: clip.to_bytes(),
        name: clip.source_name().to_owned(),
        page: selected.page,
        rect,
        widgets: clip.widget_count(),
        carries_actions: clip.carries_actions(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ The naming convention, which was WRONG until 2026-08-29.
    ///
    /// It produced `Text1 2` from `Text1`: a space separator and no awareness
    /// that the base was already numbered. Both halves are fixed here and both
    /// are sourced from the Acrobat reference rather than chosen.
    #[test]
    fn a_numbered_base_continues_its_number_and_a_bare_one_starts_at_two() {
        assert_eq!(
            split_trailing_number("Text1"),
            ("Text", 2),
            "★ the placement dialog names a new text field `Text1`, so a numbered base is the ORDINARY case; appending gives `Text12`, which reads as field twelve"
        );
        assert_eq!(split_trailing_number("Text9"), ("Text", 10));
        assert_eq!(
            split_trailing_number("Drawn By"),
            ("Drawn By", 2),
            "a bare name starts at 2, because the source itself is the unwritten 1"
        );
        assert_eq!(
            split_trailing_number("Rev2000"),
            ("Rev", 2001),
            "a high number continues rather than restarting"
        );
    }

    /// The separator is load-bearing, and the DOT is refused.
    #[test]
    fn the_generated_name_carries_no_separator_and_never_a_dot() {
        let name = crate::text::fieldclip::candidate_name("Text", 2);
        assert_eq!(name, "Text2");
        assert!(
            !name.contains('.'),
            "★★★ `.` is the fully-qualified-name separator (12.7.3.2), so `Text.2` would be a CHILD field named `2` under a parent named `Text` — a hierarchy nobody asked for"
        );
        assert!(
            !name.contains(' '),
            "a space breaks the sourced scripting rationale: the suffix exists so a script can loop over fields sharing the non-number part of the name"
        );
    }

    /// The offset rule, both halves, because they disagree on purpose.
    #[test]
    fn same_page_offsets_and_cross_page_lands_in_place() {
        let src = Rect {
            llx: 100.0,
            lly: 200.0,
            urx: 260.0,
            ury: 220.0,
        };
        let same = placed_rect(src, 3, 3, None);
        assert!(
            (same.llx - 110.0).abs() < 1e-9 && (same.lly - 190.0).abs() < 1e-9,
            "a same-page paste must be displaced down and to the right so the copy is visible"
        );
        assert!(
            (same.urx - same.llx - (src.urx - src.llx)).abs() < 1e-9,
            "the displacement must not resize the box"
        );
        let cross = placed_rect(src, 3, 11, None);
        assert_eq!(
            cross, src,
            "a cross-page paste must land at the ORIGINAL coordinates -- that is the whole reason for copying a title-block field to another sheet"
        );
    }

    /// ★★★ **The pointer outranks both older rules, and keeps the size.**
    ///
    /// `OPERATOR_REQUESTS.md` O73. Asserted against BOTH fallback cases —
    /// same page and cross page — because the target arm has to win in each,
    /// and a fix that only reached one of them would look right in whichever
    /// case the author happened to try.
    #[test]
    fn a_paste_with_a_target_centres_the_field_on_it_and_keeps_its_size() {
        let src = Rect {
            llx: 100.0,
            lly: 200.0,
            urx: 260.0,
            ury: 220.0,
        };
        let target = egui::pos2(500.0, 700.0);
        for (from, to, label) in [(3usize, 3usize, "same page"), (3, 11, "cross page")] {
            let out = placed_rect(src, from, to, Some(target));
            let (cx, cy) = ((out.llx + out.urx) / 2.0, (out.lly + out.ury) / 2.0);
            assert!(
                (cx - 500.0).abs() < 1e-9 && (cy - 700.0).abs() < 1e-9,
                "{label}: the field must be CENTRED on the cursor, not cornered on it -- got ({cx}, {cy})"
            );
            assert!(
                (out.urx - out.llx - (src.urx - src.llx)).abs() < 1e-9
                    && (out.ury - out.lly - (src.ury - src.lly)).abs() < 1e-9,
                "{label}: pointing at a spot does not ask for the field to be resized"
            );
        }
    }
}
