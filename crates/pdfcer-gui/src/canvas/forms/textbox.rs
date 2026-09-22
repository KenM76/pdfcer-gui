//! # `canvas::forms::textbox` — the live text box laid over a widget rectangle
//!
//! One mechanism, two callers: [`super::editor`]'s `/Tx` path and
//! [`super::choosing`]'s editable combo box. Both lay an `egui::TextEdit` over
//! a field's own rectangle on the page, and both have to answer the same three
//! questions about how it is dressed — which font, which end of the box the
//! text is set against, and what colour the box is.
//!
//! ## Contract
//!
//! [`lay`] builds and places the editor and returns its `Response`. It owns
//! **appearance only**. Requesting focus, seating the caret, reading Escape,
//! deciding what a commit is and what to do with the typed string all stay
//! with the caller, because the two callers answer them differently: a text
//! field commits `FillText` on focus loss, an editable combo commits
//! `SetChoice` on Enter and has a popup list to close first.
//!
//! ## ★ Why the dressing is shared and the lifecycle is not
//!
//! The three appearance rules below are *properties of the document*, so two
//! independent statements of them is two places for a form to be drawn with
//! the wrong quadding or a field to flash white. The lifecycle rules are
//! *properties of the interaction*, and the two interactions genuinely differ.
//! Sharing the first and not the second is the seam, not a compromise.

use egui::{Id, Rect, Ui};
use pdfcer_core::vartext::Quadding;

use super::boxes::{editor_align, editor_font_size};

/// Everything [`lay`] needs that is not the draft itself.
pub(super) struct Spec<'a> {
    /// The editor's `egui` id — [`super::Focus::editor_id`] for both callers,
    /// so the caret state survives between frames.
    pub(super) id: Id,
    /// Where to put it, in screen space.
    pub(super) rect: Rect,
    /// `/Q`, resolved through the field tree by `pdfcer-core`.
    pub(super) align: Quadding,
    /// `/MK` `/BG` as sRGB, when the widget has one.
    pub(super) fill: Option<[f32; 3]>,
    /// `/Ff` `Multiline`. Always `false` for a combo box.
    pub(super) multiline: bool,
    /// `/Ff` `Password`. Always `false` for a combo box.
    pub(super) password: bool,
    /// The field's name, when this frame is the one that should report what
    /// the tint resolved to. `None` on every other frame — see [`lay`]'s ★★.
    pub(super) trace: Option<&'a str>,
}

/// Build the editor, dress it from the document, and place it.
///
/// # ★★★ `/Q`, and it is the one placement property this editor reads
///
/// [`super`]'s §3 refuses to make this box a facsimile of the rendered widget,
/// and the refusal is **arithmetic**: the overlay is a font substitution by
/// construction, so a box pretending to be the appearance stream would put the
/// caret where the glyph is not going to land, and would be wrong by more the
/// longer the string.
///
/// That argument is about glyph advances and it does not reach quadding, which
/// states which **end** of the box the run is anchored to and says the same
/// thing in any font. An editor that read `/Q` nowhere would type every field
/// left-aligned, and a centred or right-aligned form would re-lay itself out
/// the moment the value committed. The whole of the rule is
/// [`super::boxes::editor_align`].
///
/// # ★★★ `/MK` `/BG`, the second property, admitted by the same test
///
/// A live box painted `extreme_bg_color` — near-white under every light preset
/// — makes a pale-yellow or shaded field **turn grey the moment the operator
/// touches it** and turn back a gesture later. Nothing in the file has
/// changed; the only thing that changes is the colour of the thing being
/// looked at, which is what pdfcer's rule 4 forbids. The engine's own
/// `Widget::background` doc names this editor as its intended consumer.
///
/// ★★ **The fill and the ink arrive together, and that is not tidiness.** A
/// document-derived fill under a theme-chosen foreground is `DEFECTS.md` D2's
/// second shape — *a foreground assigned for a fill the text is not on* — and
/// it is how the old GUI shipped near-white headings on light grey.
/// `Theme::foreign_fill_pair` measures the pair and answers `None` when no
/// theme ink reads on that fill; `None` means **paint neither**, so an
/// unreadable field keeps the theme's own readable box rather than becoming a
/// tinted one the operator cannot read their own typing in.
///
/// What is deliberately NOT tinted: the focus ring. A ring is the *cursor*,
/// which rule 4 admits in full — it says where the keystrokes are going, not
/// what the document contains.
///
/// # ★★ The refusal is traced, because an operator cannot see one
///
/// Three outcomes reach this point and only two of them are visible. A field
/// with no `/BG` keeps the theme box, which is right and expected. A field
/// WITH a `/BG` that the theme has no readable ink for **also** keeps the
/// theme box — identical on screen, a different fact about the file — and
/// rule 4 is explicit that an inference the operator cannot see still owes an
/// off-canvas report. This is that report, in the place this shell puts
/// machine-readable ones.
///
/// [`Spec::trace`] is `Some` only on the frame the caller seats the caret, so
/// the line is one per opened editor rather than one per frame. Components are
/// printed as decimals rather than `Debug`-formatted, because a driven check
/// reads this line and a `{:?}` tuple is a shape that changes when the type
/// does.
pub(super) fn lay(ui: &mut Ui, draft: &mut String, spec: &Spec<'_>) -> egui::Response {
    let ctx = ui.ctx().clone();
    let font = egui::FontId::proportional(editor_font_size(spec.rect.height()));
    let halign = editor_align(spec.align);
    let tint = spec
        .fill
        .and_then(|srgb| egui_shell::theme::Theme::foreign_fill_pair(&ctx, srgb));

    let mut edit = if spec.multiline {
        // escape-disposition: commits — `canvas::forms`' own `Key::Escape` arm
        // runs ahead of the `lost_focus` branch and writes the draft.
        egui::TextEdit::multiline(draft)
    } else {
        // escape-disposition: commits — same arm in `canvas::forms`; a password
        // field is no less the operator's typing for being drawn as dots.
        egui::TextEdit::singleline(draft).password(spec.password)
    }
    .id(spec.id)
    .horizontal_align(halign)
    .font(egui::FontSelection::from(font));
    if let Some((fill, ink)) = tint {
        edit = edit.background_color(fill).text_color(ink);
    }

    if let Some(field) = spec.trace {
        crate::diag::trace(|| match (spec.fill, tint) {
            (None, _) => {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("form-editor-tint field={field} bg=absent")
            }
            (Some(srgb), Some((_, ink))) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "form-editor-tint field={field} bg={:.3},{:.3},{:.3} ink={},{},{}",
                srgb[0],
                srgb[1],
                srgb[2],
                ink.r(),
                ink.g(),
                ink.b()
            ),
            (Some(srgb), None) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "form-editor-tint field={field} bg={:.3},{:.3},{:.3} declined=unreadable",
                srgb[0], srgb[1], srgb[2]
            ),
        });
    }
    ui.put(spec.rect, edit)
}

/// Put the caret at the end of the draft, or select the whole of it.
///
/// # ★★ Two seatings, and the difference is measured rather than chosen
///
/// A `/Tx` field seats the caret at the **end**: the click that asked for the
/// editor was consumed by the page (see [`super`]'s §4), so there is no click
/// position to place a caret from, and selecting all would turn the operator's
/// next keystroke into a deletion of the field's contents.
///
/// An editable combo box seats it as a **select-all**, because that is what
/// the product class does and because the gesture means something different
/// there. Acrobat, photographed on `fixtures/all-field-kinds.pdf`: a click
/// anywhere in an editable combo's text area gives the field a white box, a
/// chevron drop button, and the existing value highlighted end to end. The
/// field's value came from a list, so replacing it wholesale is the common
/// act; in a text field the operator is usually amending what is there.
pub(super) fn seat(ctx: &egui::Context, id: Id, draft: &str, select_all: bool) {
    let end = egui::text::CCursor::new(draft.chars().count());
    let range = if select_all {
        egui::text::CCursorRange::two(egui::text::CCursor::new(0), end)
    } else {
        egui::text::CCursorRange::one(end)
    };
    if let Some(mut state) = egui::TextEdit::load_state(ctx, id) {
        state.cursor.set_char_range(Some(range));
        egui::TextEdit::store_state(ctx, id, state);
    }
}
