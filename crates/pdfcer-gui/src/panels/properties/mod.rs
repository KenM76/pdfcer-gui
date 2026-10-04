//! # `panels::properties` — the detail of what is selected, and nothing else
//!
//! ## Every section here is scoped to a SELECTION
//!
//! The file's `/Info` fields and the seven facts about the file itself are
//! [`crate::panels::docprops`], a panel of its own, reached by
//! `file.document_properties` on File ▸ Document and mounted in all three
//! modes. **Every section in THIS panel is scoped to a selection**, and that is
//! the property to preserve: the one thing this panel is for is answering
//! *what is the thing I picked*, which is what makes it the detail half of the
//! Objects/Properties master–detail column `OPERATOR_REQUESTS.md` O123 asks
//! for.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/mod.md`.

/// **Whether the selected annotation can be deleted, and what would go with
/// it** — `EditSession::annotation_deletion_refusal` and
/// `annotation_deletion_preview`.
pub mod annotdelete;
/// The selected annotation's show, print and lock switches.
pub mod annotflags;
/// A push button's picture and its caption position, under the box's rows.
pub mod buttonicon;
/// A check box's mark, in place of the caption box.
pub mod checkmark;
/// A **choice field's `/Opt` list** and the three `/Ff` flags Acrobat groups
/// with it. Its own module under R2 and on the seam the code takes:
/// [`fieldedit`] draws a field's flags, this draws the list those flags
/// describe.
pub mod choiceopts;
/// The **selected ce dimension's** own properties — a contextual section
/// drawn above this panel's object form.
mod dimension;
/// **The Tool panel's disclosure block, re-homed** —
/// `OPERATOR_REQUESTS.md` O123.
mod disclose;
/// The **face chooser**, which is one control drawn on two surfaces — this
/// panel's [`text`] section and the ribbon's Format ▸ Font group in
/// [`crate::app::fontband`], which draw the one loop from the one place.
pub mod face;
/// The **form field** clicked on the page in Edit mode. `pub` for the same
/// reason [`geometry`] is: its rename box holds a draft in
/// `crate::panels::PanelsState`.
/// The editable half of a placed form field's properties —
/// `EditSession::edit_field`. See its header for the sentence it deletes and
/// for the field-vs-widget scope rule.
pub mod fieldedit;
/// A text field's scrolling, spell-check and file-select flags, and every
/// field's export name.
pub mod fieldextras;
/// A text or drop-down field's format, validate and calculate scripts.
pub mod fieldscripts;
pub mod formfield;
pub mod geometry;
// The document's own properties are `crate::panels::docprops`, a panel of
// its own, and never a section here: a block that draws with no reference to
// the selection is on screen whenever nothing else has anything to say. See
// that module's header for the argument, and [`body_sections`] for the rule
// this panel keeps because of it.
/// The selection's layer, and the Move to layer window.
pub mod layer;
/// Restyling a markup that is already on the page — colour, line width and
/// opacity, through `EditSession::set_markup_style`.
mod markup;
/// **One colour control, three honest states** — a swatch, an
/// indeterminate swatch, and nothing at all over an ink pdfcer will not
/// overwrite. Shared by [`paint`] and [`textobject`], because O89's two pieces
/// have to answer the same three questions the same way.
pub use pdfcer_gui_base::mkcolour;
/// The colour of a selected path. `OPERATOR_REQUESTS.md` O89's vector half, and
/// the colour of a whole **selection** of paths, with the indeterminate state
/// the product class already agrees on.
mod paint;
/// **The character an edit was refused for, and the face that can type it**
/// — `OPERATOR_REQUESTS.md` O141.
pub mod refusedchar;
/// One text run's width, typed in points (`G038`).
mod runwidth;

pub use pdfcer_gui_base::swatch;
/// A refused letter typed in a face from the font folders.
pub mod installedletter;
/// The **selected text's** face, size, weight and colour — O37's Font
/// controls, built panel-first as §5.8 says to.
pub mod text;
/// **The colour of the text the operator CLICKED** — `OPERATOR_REQUESTS.md`
/// O89, piece 1.
pub mod textobject;
/// **The armed tool's own settings** — the text pen's face, size and
/// colour, the circular measure's pick list, and the three resize switches.
pub mod tool;
/// The **box** a form field is drawn in — `EditSession::edit_widget`. Its own
/// file rather than four more rows in [`fieldedit`],
/// because the engine has two verbs and Acrobat's own scripting model has two
/// scopes; see its header.
pub mod widgetedit;
/// The offer to make a refused text edit another way.
pub mod workaround;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::panels::objects::summary::{self, ObjectSummary};
use crate::text::panels::objects as ot;
use crate::text::panels::properties as t;

/// What pdfcer can say about whether a named font's program is in the file.
///
/// Three answers, not two — see the module header on why the third is not a
/// failure to resolve but the honest result of a name join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontEmbedded {
    /// Exactly one font dictionary carries that name, and it has a program.
    Yes,
    /// Exactly one font dictionary carries that name, and it has none.
    No,
    /// The name matches no font dictionary, or more than one, and those need
    /// not agree.
    Unknown,
}

/// Whether the document embeds the program for the font named `base_font`.
#[must_use]
pub fn font_embedded(
    inventory: &pdfcer_core::fontinfo::FontInventory,
    base_font: &str,
) -> FontEmbedded {
    let mut matches = inventory
        .fonts
        .iter()
        .filter(|f| f.base_font.as_deref() == Some(base_font));
    let (Some(first), None) = (matches.next(), matches.next()) else {
        return FontEmbedded::Unknown;
    };
    if matches!(first.program, pdfcer_core::fontinfo::Program::Embedded(_)) {
        FontEmbedded::Yes
    } else {
        FontEmbedded::No
    }
}

/// Draw the Properties panel.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    // **ONE SCROLL AREA, ROUND EVERYTHING**, and its absence is a defect an
    // operator cannot work around.
    //
    // This body draws its selection-scoped sections straight into `ui`. With a
    // `ScrollArea` only on `object_section`'s read-only rows, nested deep
    // inside, **every section above it is laid out unscrolled**, and when the
    // panel's dock slot is shorter than they need the overflow is simply
    // clipped. Not below a fold: below the window, with no scrollbar and no
    // gesture that reaches it.
    //
    // What that costs, measured and photographed with a path object selected
    // in a 1100 x 800 window
    // (`evidence/` via `ui-verify geometry_fields_resize_a_shape`):
    //
    // > `Left`, `Bottom`, `Width` and `Height` on screen. **`Apply` not.** The
    // > typed-geometry feature complete, wired, tested and unusable, because
    // > the only control that commits it cannot be reached at any window size
    // > the dock will give this panel.
    //
    // It reads as *"the Width field was scrubbed and Apply committed
    // nothing"* — a dead button — and is not one: the button is never pressed.
    // The coordinates say so — `properties.geometry.apply` at y 776 in a
    // viewport ending at 762 — and readings of them still reach the wrong
    // conclusion. **A screenshot settles it in one look**, which is the
    // standing rule: a layout defect has exactly one oracle.
    //
    // Nothing nests a second `ScrollArea` inside this one: a scroll area
    // inside a scroll area steals the wheel from its parent depending on where
    // the pointer happens to be, which is a worse surface than the one being
    // fixed. `crate::panels::docprops` carries a wrapper of its own — one per
    // panel, never one inside another.
    egui::ScrollArea::vertical()
        .id_salt("properties-body")
        .auto_shrink([false, false])
        .show(ui, |ui| body_sections(ui, doc, state, actions));
}

/// The panel's sections, inside the scroll area [`body`] wraps them in.
fn body_sections(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    state: &mut PanelsState,
    actions: &mut Vec<Action>,
) {
    // **The disclosure block, FIRST** — `OPERATOR_REQUESTS.md` O123.
    //
    // Above everything, on the rule that a disclosure sits above the thing it
    // qualifies: a caveat below a list arrives after the operator has already
    // drawn a conclusion. Everything below this
    // line describes what is selected, and a refusal read after that
    // description arrives too late to explain it. See its header for why the
    // status bar could not be the home and this panel can.
    //
    // Its answer is deliberately NOT part of `something_drew`. That
    // predicate is O75's, and O75 is about whether a **selection**-scoped
    // section has spoken; a refusal from the last edit is not a description of
    // the current selection, and letting it collapse the document section
    // would make the panel change shape for a reason unconnected to what is
    // picked.
    let _drew_disclosure = disclose::section(ui, doc);
    // **The character an edit was refused for, and the way out of it** —
    // `OPERATOR_REQUESTS.md` O141.
    //
    // Directly under the disclosure block and above everything else, on the same
    // rule that puts the disclosure first: everything below describes what is
    // *selected*, and this describes what the operator's **last keystroke** ran
    // into. A refusal read after a description of a selection arrives too late to
    // explain it — and here it would arrive after the *This text* section's own
    // face chooser, which is the control this block exists to hand them, so the
    // operator would meet the answer before the question.
    //
    // NOT part of `something_drew`, for `disclose::section`'s reason stated one
    // notch more sharply: this block is scoped to an **edit**, not to a
    // selection, and by the time it draws the caret that raised it has already
    // been abandoned. Folding it in would let a refusal suppress *"nothing is
    // selected"* while genuinely nothing is.
    let _drew_refused_char = refusedchar::section(ui, doc, state.refused_char_mut(), actions);
    // The workaround offer: scoped to an edit like the block above, so it is
    // not part of `something_drew` either.
    let _drew_workaround = workaround::section(ui, doc, actions);
    // **The armed tool's settings, second** — the controls that were in
    // the Tool panel until O123 moved them here.
    //
    // Above the selection-scoped sections because an armed tool is the more
    // immediate subject: somebody who has just armed the text pen is about to
    // ask *what size*, not *what is that path's line width*. Also NOT part of
    // `something_drew`, and for a sharper reason than the disclosure block's —
    // `Block::ScaleSwitches` draws whenever Select is armed, which is most of
    // the time, so folding it in would collapse the document section for ever
    // and suppress *"nothing is selected"* for ever. That is O75 answered
    // backwards.
    let _drew_tool = tool::armed_section(ui);
    // The markup restyle section, first among the selection-scoped ones.
    //
    // Before the ce-dimension section and before the object one, because the
    // three are **mutually exclusive by construction** — `SelectionState` holds
    // an annotation or a content selection, never both, and `AnnotKind` splits
    // the annotation case in the type — so the order is about which the reader
    // meets in the source rather than which the operator sees. Markup is first
    // because it is the one that WRITES: the other two describe.
    let drew_markup = markup::section(ui, doc, actions);
    let drew_dimension = dimension::section(ui, doc, actions);
    // Directly under the two sections that restyle the selected annotation,
    // and above everything that describes a *content* object — because this is
    // about the same subject those two are about, and the panel's reading order
    // is "what you can change about this thing", then "what is true of it".
    //
    // It is the one section here whose subject is a control that lives
    // **somewhere else**: the Delete it explains is on the Format tab, on both
    // canvas menus and on the Delete key, and none of those can hold a sentence.
    // R9 sends a permanently-refused capability's explanation to the surface
    // that describes what is selected, and this is that surface.
    //
    // It draws for an annotation of ANY kind — a markup, a ce dimension, a
    // stamp — where `markup` and `dimension` each draw for one. Deletion is the
    // one verb they share, and `annotation_deletion_refusal` is a document-wide
    // question that does not care which `/Subtype` is selected.
    let drew_annot_flags = annotflags::section(ui, doc, actions);
    let drew_annot_delete = annotdelete::section(ui, doc, state.annot_delete_mut());
    // The geometry fields sit between the sections that WRITE and the
    // section that describes, because that is what they are: the only editable
    // thing about a selected *content* object, where the two above it are the
    // only editable things about a selected *annotation*. Reading the panel top
    // to bottom therefore goes "what you can change" then "what is true", which
    // is the order `RIBBON_IA.md` §5.6 asks a properties surface to use.
    // The form-field section, above the geometry fields and below the two
    // that restyle. It is a fourth claimant on this panel and it is mutually
    // exclusive with all three by construction: a form field is selected by
    // `doc.selected_field`, which the object and annotation selections neither
    // set nor read. See `app::state::SelectedField` for why they are separate.
    let drew_form_field = formfield::section(ui, doc, state, actions);
    // Before geometry, after markup. The restyle controls sit with the other
    // "change how this looks" rows and above the read-only facts, which is the
    // order `RIBBON_IA.md` §5.6 asks a properties surface to use.
    //
    // This section and `geometry` DO both draw for one selection
    // (`OPERATOR_REQUESTS.md` O198): a **clicked** text object resolves through
    // `app::textoperand` into runs, so this section draws for an object
    // selection and `geometry` draws for the same selection in the same frame.
    // They are not competing for one slot — they are two paragraphs about one
    // object, and "what you can change about how it looks" reads before "where
    // it is".
    let drew_text = text::section(ui, doc, state.text_style_mut(), actions);
    // **The clicked-text colour, directly under the swept-text editor** —
    // `OPERATOR_REQUESTS.md` O89, piece 1.
    //
    // The two are **mutually exclusive by construction**: `textobject::section`
    // returns false whenever a live text selection exists, which is exactly when
    // `text::section` draws. So this is a reading-order placement rather than a
    // precedence one, and the operator meets one Colour control, never two —
    // which matters here more than anywhere else in the panel, because two
    // colour swatches with different operands one above the other is a way to
    // recolour the wrong thing while looking straight at it.
    //
    // Above `geometry` for the same reason `text` is: "what you can change
    // about how this looks" comes before "where it is" and before "what is true
    // of it", which is the order `RIBBON_IA.md` §5.6 asks a properties surface
    // to use.
    let drew_text_object = textobject::section(ui, doc, state.text_object_mut(), actions);
    let drew_run_width = runwidth::section(ui, doc, actions);
    let drew_geometry = geometry::section(ui, doc, state.geometry_mut(), actions);
    let drew_paint = paint::section(ui, doc, actions);
    let drew_layer = layer::section(ui, doc, actions);
    // **BOUND** — `OPERATOR_REQUESTS.md` O75: *has anything in this panel
    // described the selection?*
    //
    // A block drawn with **no condition of any kind** is the only thing on
    // screen whenever nothing above it has anything to say, which reads as the
    // panel showing the wrong subject. The sections above DO read the
    // selection; this disjunction is what lets anything below them ask whether
    // they spoke.
    //
    // One consumer: `object_section` needs it to decide whether *"nothing
    // is selected"* is true. Each term below keeps its own note, because each
    // records a case where omitting it puts a wrong sentence on screen.
    let something_drew = drew_dimension
        || drew_layer
        || drew_markup
        || drew_annot_delete
        || drew_annot_flags
        || drew_geometry
        || drew_form_field
        || drew_text
        // Part of the predicate, and not for symmetry. A selected text
        // object makes the COLOUR row speak, and omitting this term would put
        // *"Pick a row in the Objects panel"* under a live colour control for a
        // selected label. The term is load-bearing whatever else the panel
        // ends with; what changes is only which wrong sentence it prevents.
        //
        // It is not the only thing standing between a clicked label and
        // that sentence: `drew_text` above is true in the same state, because
        // the face, size and weight rows draw for a clicked object too
        // (`OPERATOR_REQUESTS.md` O198). Keeping this one is still right — the
        // two
        // sections are independently removable, and a predicate that relies on
        // a sibling drawing is a predicate that breaks when the sibling is
        // gated on something new.
        || drew_text_object
        || drew_run_width
        // And so is the PAINT section, whose answer must not be discarded.
        //
        // Discarding it is harmless only while `paint::section` draws for a
        // single selected path, because `object_section` speaks for that same
        // selection. It stops being harmless because the section has a
        // **multi-object** state: a selection of eleven paths makes
        // `object_section` say nothing (it has one subject) and
        // `paint::section` say something, so with the answer discarded the
        // panel draws a live Fill and Line control under *"Pick a row in the
        // Objects panel"* — the O75 shape exactly.
        || drew_paint;
    // …and `object_section` is what USES that predicate, rather than
    // contributing to it. It is the last section, it is the only one that can
    // say *"nothing is selected"*, and it must say that only when nothing above
    // it has spoken — `annotdelete::section` returns false for an ordinary
    // unlocked annotation and `text::route` returns false for a non-text
    // object, so for a selected **image or path** (the commonest selection in a
    // CAD file) this is the only section that speaks at all.
    //
    // Its own return value is discarded, because nothing is drawn below it
    // to inform. **The binding above is still read**, by this call, which is
    // the whole of what O75 needs it for.
    let _drew_object = object_section(ui, doc, something_drew);
    // **THE STANDING PREFERENCES, LAST** — `OPERATOR_REQUESTS.md` O198.
    //
    // The three *When you resize something* switches are on screen whenever
    // the Select tool is armed, which is the resting state, so drawn at the top
    // of this function they are the entire visible height of an ordinary dock
    // slot: a driven click on a text object photographs the font editor half
    // clipped and the Colour swatch below the viewport altogether. See
    // `tool::Slot` for the measurement.
    //
    // The rule is O75's, restated: a section that draws with no reference to
    // the selection must not sit above the sections that describe it. `tool`
    // answers WHICH of its blocks that applies to, through `tool::slot_of`, so
    // the text pen and the measure pick list keep the top of the panel and only
    // the standing preference sits here.
    let _drew_preferences = tool::preferences_section(ui);
}

/// The focused page object's read-only facts.
fn object_section(ui: &mut egui::Ui, doc: &OpenDoc, something_drew: bool) -> bool {
    // **THE CANVAS SELECTION**, and never a panel-local focus.
    //
    // A `PanelsState::focus` — written **only** by an Objects-panel row click
    // and read **only** here — leaves the canvas selection, which is what an
    // operator makes by clicking the page, something this panel has never heard
    // of: select an object on the canvas and the panel does not switch to the
    // editable stuff for it.
    //
    // Parallel notions of *"the thing I am working on"* — the armed tool, a
    // panel focus, the canvas selection — with no bridge between them and none
    // authoritative are the root of that. There is one notion here, written
    // from both ends: the Objects panel's row click raises
    // `Action::SelectObject`, and this reads the selection those two share.
    //
    // **The first object on the current page**, and the choice is stated
    // rather than incidental. A multi-selection has no single set of properties
    // to show; the *"3 objects selected (2 paths, 1 text)"* orientation line
    // belongs to the status bar and the Objects panel's header, which both take
    // `summary::census`. This panel has no such readout, and describing the
    // first object is better than describing nothing.
    //
    // Object-scoped rather than annotation-scoped: `object_indices_on` returns
    // page-content objects, which is exactly what this section describes. An
    // annotation selection is `markup::section`'s and a ce dimension is
    // `dimension::section`'s, both of which have already run above.
    let selected = doc
        .selection
        .object_indices_on(doc.view.page_index)
        .first()
        .copied();
    let Some(index) = selected else {
        // Silent when the section drew, because it is not true otherwise. A
        // ce dimension selected on the canvas with nothing focused in the
        // object tree is the ordinary state the instant an operator clicks one,
        // and *"nothing is selected"* under a section describing the thing that
        // is selected would be the panel contradicting itself.
        if !something_drew {
            ui.label(t::properties_nothing_focused());
        }
        return false;
    };
    // The description is taken out of the shared decomposition and OWNED
    // (it is one small record), so the `Ref` into the document's cache is
    // released before anything is drawn. Holding it across the whole body
    // would work too, but a short borrow is the honest shape: this panel
    // describes a snapshot of one object, not the page.
    let Some(described) = doc.page_objects().and_then(|provider| {
        provider
            .page_objects()
            .objects
            .get(index)
            .map(summary::describe_object)
    }) else {
        // The focused row named an object that is no longer there. Reachable
        // only if something clears the provider without clearing the focus,
        // which `PanelsState::sync` is written to make impossible — so this
        // is a guard, not a state with copy of its own. It reads as "nothing
        // is picked", which is the truth from the operator's side.
        ui.label(t::properties_nothing_focused());
        return false;
    };
    // Only a text object with a named font needs the inventory, and it is
    // the expensive one (it decodes every embedded font program), so it is
    // fetched only when there is a name to look up.
    let embedded = described
        .font
        .as_ref()
        .and_then(|f| f.base_font.as_deref())
        .map(|name| font_embedded(&doc.font_inventory(), name));

    // `.strong()` is unusable in this theme — see `DEFECTS.md` D11.
    ui.label(egui::RichText::new(t::properties_object_heading()));
    ui.label(
        egui::RichText::new(t::properties_read_only_note())
            .small()
            .weak(),
    );
    ui.separator();

    // No `ScrollArea` here — `body` wraps the whole panel in one, and nesting
    // a second inside it steals the wheel from the outer depending on where the
    // pointer sits.
    {
        {
            for (label, value) in property_rows(index, &described, embedded) {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(label));
                    ui.label(value);
                });
            }

            if !described.notes.is_empty() {
                ui.separator();
                ui.label(egui::RichText::new(t::properties_notes_heading()));
                for note in &described.notes {
                    // Ordinary weight, never warn-coloured: each of these is
                    // a fact about the FILE, and error styling would make it
                    // read as a pdfcer failure.
                    ui.label(ot::object_note(*note));
                }
            }
        }
    }

    // The region the panel's largest section has never published — O75.
    // `ui_rect_visible` rather than `ui_rect`, on `geometry::section`'s
    // recorded precedent: this panel is a `ScrollArea`, and a rect published
    // for a scrolled-out control gets CLICKED by the harness.
    crate::diag::ui_rect_visible(REGION_OBJECT, ui.min_rect(), ui.clip_rect());
    crate::diag::trace(|| {
        format!(
            "properties-panel object={index} kind={:?} notes={}",
            described.kind,
            described.notes.len()
        )
    });
    true
}

/// The region [`object_section`] publishes when it has drawn an object's
/// properties — `OPERATOR_REQUESTS.md` O75.
const REGION_OBJECT: &str = "properties.object"; // ui-text-exempt: trace region name, never displayed

/// The panel's field list, as `(label, value)` pairs in display order.
#[must_use]
pub fn property_rows(
    index: usize,
    summary: &ObjectSummary,
    embedded: Option<FontEmbedded>,
) -> Vec<(&'static str, String)> {
    let mut rows: Vec<(&'static str, String)> = Vec::new();

    rows.push((
        t::field_type(),
        ot::object_kind_label(summary.kind).to_owned(),
    ));
    // The paint-order index: the handle every command-line verb takes, and
    // the reason it is a *field* rather than only part of the Objects row is
    // that an operator reading properties is exactly the operator about to
    // reach for `pdfcer`.
    rows.push((t::field_index(), t::value_index(index)));

    if let Some(paint) = summary.paint {
        rows.push((t::field_paint(), ot::paint_style_label(paint).to_owned()));
        // Stated in full here, unlike in the one-line row where only
        // even-odd is called out — see `winding_rule_label`'s own docs. A
        // field headed "Winding rule" that is blank nine times in ten reads
        // as a value pdfcer failed to read.
        if let Some(winding) = ot::winding_rule_label(paint) {
            rows.push((t::field_winding(), winding.to_owned()));
        }
    }
    // `None` for a path that paints nothing — its unused, default-black fill
    // colour appears nowhere on the page, so reporting it would be a
    // confidently wrong answer. Omitted rather than "not stated": the file
    // states a colour, it is simply not one a viewer shows.
    if let Some(colour) = summary.colour {
        rows.push((t::field_colour(), ot::rgb_hex(colour)));
    }
    if let Some(width) = summary.line_width {
        rows.push((t::field_line_width(), t::value_line_width(width)));
    }
    if let Some(nodes) = summary.nodes {
        rows.push((t::field_nodes(), nodes.to_string()));
    }
    if let Some(text) = summary.text.as_deref() {
        // A longer cap than the Objects row's: a panel field wraps, so it
        // can afford the whole preview the model kept. The quoting and the
        // control-character replacement are the same, which is the half that
        // must not diverge.
        rows.push((
            t::field_text(),
            ot::quoted_text(text, summary.text_truncated),
        ));
    }
    if let Some(font) = summary.font.as_ref() {
        rows.push((t::field_font(), ot::font_label(font)));
        rows.push((
            t::field_font_embedded(),
            match embedded {
                Some(FontEmbedded::Yes) => t::value_font_embedded_yes().to_owned(),
                Some(FontEmbedded::No) => t::value_font_embedded_no().to_owned(),
                // `None` reaches here when the font has no `/BaseFont` to
                // join on, which is the same situation as a name that
                // matches nothing: pdfcer has no dictionary to report about.
                Some(FontEmbedded::Unknown) | None => t::value_font_embedded_ambiguous().to_owned(),
            },
        ));
    }
    if let Some((w, h)) = summary.pixels {
        rows.push((t::field_pixels(), t::value_pixels(w, h)));
    }

    // Geometry LAST, and always present. An object with no finite bounds
    // still has a position field — it says the file does not state one,
    // which is the fact `ObjectNote::NoBounds` then explains at length.
    // Dropping the rows would leave the operator to notice an absence.
    match summary.size() {
        Some((w, h)) => {
            rows.push((
                t::field_position(),
                t::value_position(summary.bounds.min.x, summary.bounds.min.y),
            ));
            rows.push((t::field_size(), t::value_size(w, h)));
        }
        None => {
            rows.push((t::field_position(), t::value_not_stated().to_owned()));
            rows.push((t::field_size(), t::value_not_stated().to_owned()));
        }
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;
    use pdfcer_core::content::ContentStream;
    use pdfcer_core::vector::{Matrix, NoXObjects, decompose};

    fn describe(src: &[u8]) -> ObjectSummary {
        let cs = ContentStream::parse(src.to_vec()).expect("parse");
        let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
        assert_eq!(objects.objects.len(), 1);
        summary::describe_object(&objects.objects[0])
    }

    fn value<'a>(rows: &'a [(&'static str, String)], label: &str) -> Option<&'a str> {
        rows.iter()
            .find(|(l, _)| *l == label)
            .map(|(_, v)| v.as_str())
    }

    /// **The four facts `RIBBON_IA.md` §5.8 commissions this panel for are
    /// all present.**
    #[test]
    fn the_four_commissioned_facts_are_all_reported() {
        // Winding rule, node count, exact geometry — on a filled path.
        let path = describe(b"0 0 1 rg 10 10 80 80 re f*");
        let rows = property_rows(0, &path, None);
        assert_eq!(value(&rows, t::field_winding()), Some("Even-odd"));
        assert_eq!(value(&rows, t::field_nodes()), Some("4"));
        assert_eq!(value(&rows, t::field_position()), Some("10.0, 10.0 pt"));
        assert_eq!(value(&rows, t::field_size()), Some("80.0 × 80.0 pt"));

        // Embedded-font status — on text.
        let text = describe(b"BT /F1 12 Tf 40 40 Td (Hi) Tj ET");
        let rows = property_rows(0, &text, Some(FontEmbedded::No));
        assert_eq!(
            value(&rows, t::field_font_embedded()),
            Some(t::value_font_embedded_no())
        );
    }

    /// **A field is omitted when the object has no such property, and
    /// present-but-unstated when it has one the file does not give.**
    #[test]
    fn an_absent_property_is_omitted_and_an_unstated_one_says_so() {
        let path = describe(b"10 10 80 80 re f");
        let rows = property_rows(0, &path, None);
        assert_eq!(value(&rows, t::field_font()), None, "a path has no font");
        assert_eq!(value(&rows, t::field_text()), None);
        assert_eq!(value(&rows, t::field_pixels()), None);

        // Geometry is always present, even when there is none to report.
        let mut empty = path.clone();
        empty.bounds = pdfcer_core::vector::Bounds::EMPTY;
        let rows = property_rows(0, &empty, None);
        assert_eq!(
            value(&rows, t::field_position()),
            Some(t::value_not_stated())
        );
        assert_eq!(value(&rows, t::field_size()), Some(t::value_not_stated()));
    }

    /// **A path that paints nothing reports no colour, and does not invent
    /// one.**
    #[test]
    fn a_no_paint_path_reports_no_colour_at_all() {
        let clip = describe(b"10 10 80 80 re n");
        let rows = property_rows(0, &clip, None);
        assert_eq!(value(&rows, t::field_colour()), None);
        assert_eq!(
            value(&rows, t::field_paint()),
            Some("paints nothing (a clip or discarded path)")
        );
        // …and no winding rule either: an `n` path has no fill in effect, so
        // naming one would name a rule that decides nothing.
        assert_eq!(value(&rows, t::field_winding()), None);
    }

    /// **A stroke-only path reports the STROKE colour.**
    #[test]
    fn a_stroked_path_reports_the_colour_a_viewer_sees() {
        let stroked = describe(b"1 0 0 RG 0 0 1 rg 2 w 10 10 m 90 90 l S");
        let rows = property_rows(0, &stroked, None);
        assert_eq!(
            value(&rows, t::field_colour()),
            Some("#FF0000"),
            "the fill colour is never painted and must not be reported"
        );
        assert_eq!(value(&rows, t::field_line_width()), Some("2.00 pt"));
    }

    /// **An ambiguous font name is disclosed, never resolved.**
    #[test]
    fn the_embedded_font_join_declines_when_the_name_is_not_a_key() {
        let path = engine_fixture("text/subset-simple-embedded.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let inv = pdfcer_core::fontinfo::inventory(&doc.view());
        let name = inv
            .fonts
            .iter()
            .find_map(|f| f.base_font.clone())
            .expect("the fixture must declare a named font");

        // One dictionary answers to the name: a definite answer.
        let answer = font_embedded(&inv, &name);
        assert_ne!(
            answer,
            FontEmbedded::Unknown,
            "a name matching exactly one dictionary must resolve: {name}"
        );

        // A name nothing answers to is UNKNOWN, not "not embedded" — the
        // second would turn a gap in pdfcer's inventory into a claim about
        // the operator's document.
        assert_eq!(
            font_embedded(&inv, "NoSuchFaceInThisDocument"),
            FontEmbedded::Unknown
        );
    }

    /// The index is printed with its `#`, matching the Objects row and the
    /// CLI's `index=`.
    #[test]
    fn the_index_field_prints_the_paint_order_handle() {
        let path = describe(b"10 10 80 80 re f");
        let rows = property_rows(412, &path, None);
        assert_eq!(value(&rows, t::field_index()), Some("#412"));
    }

    /// Every row has a non-empty value.
    #[test]
    fn no_field_is_ever_rendered_blank() {
        for src in [
            &b"0 0 1 rg 10 10 80 80 re f"[..],
            &b"10 10 80 80 re n"[..],
            &b"1 0 0 RG 2 w 10 10 m 90 90 l S"[..],
            &b"BT /F1 12 Tf 40 40 Td (Hi) Tj ET"[..],
            &b"100 200 m 300 200 l S"[..],
            &b"q 100 0 0 50 10 10 cm BI /W 1 /H 1 /CS /G /BPC 8 ID \x00 EI Q"[..],
        ] {
            let s = describe(src);
            for (label, value) in property_rows(0, &s, Some(FontEmbedded::Unknown)) {
                assert!(!label.trim().is_empty());
                assert!(
                    !value.trim().is_empty(),
                    "the field `{label}` rendered blank for {}",
                    String::from_utf8_lossy(src)
                );
            }
        }
    }

    /// **A text object's disclosure list is never empty**, so the notes
    /// heading always has something under it when it is drawn.
    #[test]
    fn a_text_object_always_has_something_to_disclose() {
        let text = describe(b"BT /F1 12 Tf 40 40 Td (Hi) Tj ET");
        assert!(!text.notes.is_empty());
        for note in &text.notes {
            assert!(ot::object_note(*note).len() > 60);
        }
    }
}
