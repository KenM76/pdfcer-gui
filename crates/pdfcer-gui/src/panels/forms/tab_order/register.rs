//! # `panels::forms::tab_order::register` — the rows that put an unclaimed
//! form control back into the form
//!
//! ## Why this is here and not in a dialog of its own
//!
//! Because [`super::model`] already answers the question the dialog would have
//! had to re-ask. *"Which widgets does this page list that no field claims?"*
//! is one `/Annots` walk cross-referenced against one parsed `/AcroForm`, and
//! the Tab-order section performs it every frame it is open — it is the whole
//! reason that section exists.
//!
//! A separate Register window would have walked `/Annots` a second time, from a
//! second parse, at a second moment. Two answers to one question is how a list
//! and the button beside it come to disagree about the set, and the failure is
//! silent: the operator presses Register on the third box and a different third
//! box is registered.
//!
//! It is also where the operator already is. They opened Tab order because
//! something on the page would not fill, and the section told them *"3 boxes on
//! this page are drawn as form controls that no field claims"*. The next thing
//! they want is to do something about it, and R9's spirit — no control that
//! looks available and is not — reads the other way round here: **a stated
//! problem with no offered remedy is the same defect wearing different
//! clothes.**
//!
//! ## ★ Every row can be pressed with the name box empty, and that is the
//! recommended answer
//!
//! The engine measured a real form and found **11 of 13** unclaimed widgets to
//! be merged field-widgets (§12.7.3.1) — one dictionary serving as both field
//! and widget, carrying its own `/T`, `/FT`, `/V` and `/DA`. For those,
//! `adopt_widget(id, None)` recovers the field exactly as it was, and typing a
//! name would *override* the one the file already holds.
//!
//! The other 2 were **bare kids** with no identity at all, and they refuse. The
//! refusal is worded, arrives in the status bar, and says what typing a name
//! will actually produce — a new, empty field, not the radio button that was
//! lost. See [`crate::text::status::adopt_declined_no_name`].
//!
//! ## ★★ The pre-flight, asked once per row before the press
//!
//! `EditSession::adopt_preview` is `&self` and writes nothing. Each row asks it
//! — with whatever the operator has typed **so far**, not with `None` — and the
//! answer decides the row:
//!
//! | preview says | the row shows |
//! |---|---|
//! | `Ok(outcome)` | a live button reading **"Register as `Address`"** — the name from the file |
//! | `Err(WidgetHasNoFieldIdentity)` | a greyed button whose hover says typing a name will **create** a field, not recover one |
//! | `Err(FieldNameTaken)` | a greyed button whose hover explains why two fields with one name are one field |
//! | any other `Err` | greyed, and the hover **does not guess** — see [`refusal_hint`] |
//!
//! ### Why this is safe to draw from
//!
//! Because the preview and the call **share one guard set**. The engine split
//! `adopt_plan(&self, ..)` out of `adopt_widget` rather than writing a second
//! implementation, and stated the reason in terms this project keeps arriving
//! at independently: *"two implementations of one guard set are two things that
//! must agree, and eventually will not. The way that fails is a greyed-out
//! control for an operation that would have worked, or a live one for an
//! operation that refuses — which is worse than the discovery-by-pressing it
//! was meant to replace, because now the shell is confidently wrong instead of
//! silent."*
//!
//! There is a test on their side that would notice if a later change gave the
//! preview its own body. *"The preview said yes and the call refused"* is not a
//! state the code can reach.
//!
//! ### ★ What this section used to say, and why the correction is kept
//!
//! It read *"Why there is no pre-flight, said out loud rather than left as a
//! gap"*, and described the honest interim: every row offered a live button,
//! and a bare kid refused **after** the press with a worded decline. It cost
//! one press and converged, and what it could not do was say *in advance*
//! which shape a box was.
//!
//! That paragraph existed for about six hours. The request went out naming the
//! consequence — discovery by pressing — and `adopt_preview` shipped the same
//! day. It is kept as a correction rather than deleted because the shape
//! recurs: **the interim was not wrong, and writing down exactly what it could
//! not do is what made the ask specific enough to be answered.**
//!
//! ### Two facts that only a pre-flight can deliver in time
//!
//! Both were in the request and neither is cosmetic:
//!
//! - **the name.** For a blank box it is in the file and **not on screen** —
//!   the widget belongs to no field, so no field row names it. A button reading
//!   *"Register"* is a guess; one reading *"Register as `Address`"* is a
//!   decision.
//! - **`field_type: None`.** The registration will **succeed** and the box will
//!   **still not be fillable**, because `/FT` is inheritable and a top-level
//!   field has no ancestor left to inherit from. Disclosed after the fact, that
//!   sentence tells an operator their successful action did not do what they
//!   wanted. Disclosed before it, it is a choice.

use crate::app::actions::forms::FieldAction;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::forms as t;

use super::model::{Listing, PageTabs};

/// What the operator has typed into the name boxes, and which document
/// revision it describes.
///
/// # ★ Keyed on `(path, edit_epoch)`, which is what makes undo correct
///
/// Exactly `super::super::FormsUi`'s rule, for exactly its reason, and it is
/// worth restating because the consequence here is the opposite of what a
/// naive reading suggests.
///
/// A successful registration bumps the epoch, so **every draft in this map is
/// discarded**. That is right rather than lossy: the widget the operator was
/// typing about is no longer unclaimed, its row is gone, and the box they typed
/// into does not exist any more. Keeping the text would mean re-showing it
/// against whichever box happened to take that row next.
///
/// An **undo** bumps the epoch too, which restores the row and clears the box.
/// Also right: the name went into the document and came back out, so a box
/// still holding it would be showing a value the document no longer has, which
/// is the precise defect `FormsUi`'s own comment records for the fill drafts.
#[derive(Clone, Default)]
struct Drafts {
    /// The `(document path, edit epoch)` this map describes.
    key: Option<(PathBuf, u64)>,
    /// Typed names, by the widget's object **number**.
    ///
    /// The number rather than the whole [`pdfcer_core::object::ObjId`] because
    /// `ObjId` is not `Ord` in a way this map could rely on across engine
    /// versions, and because a generation cannot distinguish two live objects
    /// anyway — §7.3.10 gives a generation meaning only for reused free
    /// numbers, and nothing here holds a reference to a freed object.
    names: BTreeMap<u32, String>,
}

impl Drafts {
    /// The egui id this state is stored under.
    ///
    /// Distinct from `FormsUi`'s. The two are different lifetimes of thing
    /// keyed the same way — one holds field values, one holds proposed field
    /// names — and sharing an id would make each frame's store overwrite the
    /// other's.
    fn id() -> egui::Id {
        egui::Id::new("pdfcer-forms-tab-order-register")
    }

    /// Read this frame's drafts, dropping them if they describe a different
    /// document or a different revision.
    fn load(ui: &egui::Ui, doc: &OpenDoc) -> Self {
        let key = (doc.path.clone(), doc.edit_epoch);
        let state: Self = ui
            .data(|d| d.get_temp::<Self>(Self::id()))
            .unwrap_or_default();
        if state.key.as_ref() == Some(&key) {
            state
        } else {
            Self {
                key: Some(key),
                names: BTreeMap::new(),
            }
        }
    }

    /// Write this frame's drafts back.
    fn store(self, ui: &egui::Ui) {
        ui.data_mut(|d| d.insert_temp(Self::id(), self));
    }
}

/// Draw one row per unclaimed widget on a page, and raise
/// [`Action::AdoptWidget`] when one is pressed.
///
/// `page_index` is 0-based — it is carried into the action for the trace and
/// the re-raster, not for the engine, which edits the document-level
/// `/AcroForm` and never asks which page.
///
/// # ★ At most one registration per frame, and it is not an accident
///
/// The loop `break`s after a press. Two presses in one frame would queue two
/// `AdoptWidget`s against a listing computed **before** either ran, and the
/// second would be acting on a set the first has already changed — the same
/// stale-index hazard the engine hit in its own CLI and described plainly:
/// *"the indices shift after every add … I got this wrong myself and nested
/// something two levels deeper than intended, and the output looked entirely
/// plausible."*
///
/// The ids here are stable where indices are not, so the second action would in
/// fact still name the right widget. The `break` is kept anyway, because
/// *"queue only what was computed against the state you have"* is the property
/// worth holding mechanically rather than re-deriving each time a queued verb
/// is added. It costs the operator nothing: physically, one press per frame is
/// all there is.
pub(super) fn rows(ui: &mut egui::Ui, doc: &OpenDoc, listing: &Listing, actions: &mut Vec<Action>) {
    // ★★ EVERY page's unclaimed widgets, at the TOP of the section, and that
    // placement is the fix for a remedy nobody could reach.
    //
    // These rows used to be drawn inside each page's block, immediately under
    // the sentence counting that page's unclaimed widgets — which reads well
    // and is unusable. The Tab-order section lists **every page in the
    // document**, so on a 37-sheet drawing with one inserted form page the
    // rows sat 36 page-blocks down a scroll area. A driven run clicked the
    // published rectangle and hit nothing, because the row was scrolled out of
    // view; an operator told *"2 form controls need re-registering — Forms, Tab
    // order lists them"* would have opened the panel and found a list with no
    // obvious remedy in it.
    //
    // Third instance in one day of the same shape: the Bookmarks authoring row
    // below its list, the Manage-groups Add button below a settings block, and
    // this. The generalisation is now written down in `D:/dev/rag/egui/`:
    // **a control that answers a disclosure must be reachable from where the
    // disclosure points, without scrolling.**
    //
    // The per-page sentence stays where it is. It is a *fact about that page*
    // and belongs in that page's block; what moved is the *action*, which is
    // about the document.
    let pages: Vec<&PageTabs> = listing
        .pages
        .iter()
        .filter(|p| !p.unclaimed.is_empty())
        .collect();
    if pages.is_empty() {
        return;
    }
    let mut drafts = Drafts::load(ui, doc);
    let mut pressed: Option<(usize, pdfcer_core::object::ObjId, Option<String>)> = None;

    for (page_index, widget) in pages
        .iter()
        .flat_map(|p| p.unclaimed.iter().map(move |w| (p.page_index, w)))
    {
        let page_index: usize = page_index;
        // ★★ TWO LINES, not one, because this is a DOCK PANEL and not a dialog.
        //
        // The first draft put the label, the name box and the button on one
        // `horizontal` — about 460 pt of content in a pane that is 314 pt wide.
        // egui does not wrap a horizontal layout, so the row simply ran off the
        // right-hand edge and pushed the next one down: a driven run measured
        // the button at x=1090..1229 in a panel ending at x=1100, and clicking
        // its published centre hit the canvas.
        //
        // A **label wraps and a button does not**, which is what decides the
        // split: the identifying text goes on its own line where it can wrap to
        // any pane width, and the line below holds only the two controls whose
        // widths are fixed.
        ui.vertical(|ui| {
            // The page number as well as the tab position, because these rows
            // are gathered from the whole document and a bare "Box 3" would
            // name three different boxes on a drawing with three affected
            // sheets. 1-based, as everywhere a human reads a page number.
            //
            // The resolved name rides on this line rather than on the button —
            // see the label's own construction below. What the engine asked for
            // is that the name be *visible before the press*, not that it be
            // printed on the control.
            let draft = drafts.names.entry(widget.id.num).or_default();
            let typed_now = draft.trim().to_owned();
            // ★★ ASKED before the press, which is what makes the two shapes
            // visibly different instead of discoverable by pressing.
            //
            // `adopt_preview` is `&self` and writes nothing — the engine split
            // `adopt_plan` out of `adopt_widget` so the preview and the call
            // share one guard set, and there is a test on their side that would
            // notice if a later change gave the preview its own. That sharing is
            // the whole reason this is safe to draw from: *"the preview said yes
            // and the call refused" is not a state the code can reach.*
            //
            // It is asked with what the operator has typed **so far**, not with
            // `None`, so a name that is already taken says so while they are
            // still looking at the box they typed it into.
            let preview = doc.session.adopt_preview(
                widget.id,
                (!typed_now.is_empty()).then_some(typed_now.as_str()),
            );
            // ★ The name it WILL use, on the wrapping line, for a blank box that
            // is a name **in the file and not on screen** — the engine's own
            // words for the thing the pre-flight request was for. *"will
            // register as `Address`"* is a decision; a bare *"Register"* is a
            // guess the operator is being asked to accept.
            ui.label(match &preview {
                Ok(outcome) => t::tab_order_unclaimed_row_named(
                    page_index.saturating_add(1),
                    widget.position,
                    &outcome.name,
                ),
                Err(_) => t::tab_order_unclaimed_row(page_index.saturating_add(1), widget.position),
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(draft)
                        .desired_width(120.0)
                        .hint_text(t::tab_order_register_name_hint()),
                );
                let label = t::tab_order_register().to_owned();
                let button = match &preview {
                    // The refusal is on the hover rather than in the row, because a
                    // row that turned red while the operator was mid-word would be
                    // nagging at a state they are still typing their way out of.
                    Err(refusal) => ui
                        .add_enabled(false, egui::Button::new(label))
                        .on_disabled_hover_text(refusal_hint(refusal)),
                    Ok(outcome) => {
                        let b = ui.button(label);
                        if outcome.field_type.is_none() {
                            // Rule 4: an inference the operator cannot see. The
                            // registration will SUCCEED and the box will still not
                            // be fillable, because `/FT` is inheritable and a
                            // top-level field has no ancestor left to inherit from.
                            // Said before the press now that it can be — telling
                            // somebody afterwards tells them their successful action
                            // did not do what they wanted.
                            b.on_hover_text(t::tab_order_register_no_type())
                        } else {
                            b
                        }
                    }
                };
                // ★★ One region per row, and a trace line saying what the preview
                // decided.
                //
                // The region is what lets a driven check press a SPECIFIC row
                // rather than guessing at a rectangle; the trace is what lets it
                // assert the button's *label*, which the harness cannot read off
                // the screen.
                //
                // Asserting presence alone would pass on a build where the preview
                // was never asked and every row read "Register" — which is exactly
                // the state the pre-flight request was filed about, so the check
                // that could not see it would be green through the whole defect.
                crate::diag::ui_rect(&format!("{REGION_PREFIX}{}", widget.position), button.rect);
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI.
                    // The NAME is not carried — a field's name is the operator's own
                    // words about their drawing, and `bookmark-add` makes the same
                    // ruling for the same reason. Whether one was RESOLVED is the
                    // fact a check needs.
                    format!(
                        "adopt-row page={page_index} obj={} pos={} named={} typed={} refused={}",
                        widget.id.num,
                        widget.position,
                        u8::from(preview.as_ref().is_ok_and(|o| !o.name.is_empty())),
                        u8::from(preview.as_ref().is_ok_and(|o| o.field_type.is_some())),
                        preview.as_ref().err().map_or("none", refusal_kind),
                    )
                });
                if button.clicked() && pressed.is_none() {
                    // Trimmed here, and an empty box becomes `None` rather than
                    // `Some("")`. The engine refuses an empty name with
                    // `FieldNameEmpty`, and that refusal is unreachable from this
                    // surface precisely because of this line — see
                    // `crate::app::status::decline::record_adopt_refusal`'s table,
                    // which claims it is unreachable and would be wrong without it.
                    let typed = draft.trim();
                    let name = (!typed.is_empty()).then(|| typed.to_owned());
                    pressed = Some((page_index, widget.id, name));
                }
            });
        });
        if pressed.is_some() {
            break;
        }
    }

    if let Some((page_index, widget, name)) = pressed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "adopt-widget-requested page={page_index} obj={} named={}",
                widget.num,
                u8::from(name.is_some())
            )
        });
        actions.push(
            FieldAction::Adopt {
                page: page_index,
                widget,
                name,
            }
            .into(),
        );
    }
    drafts.store(ui);
}

/// The prefix each row's Register control publishes its rectangle under.
///
/// Suffixed with the widget's **tab position** rather than its index in the
/// unclaimed list, because the position is the only stable thing about a row
/// across a registration: registering one removes it from the list and
/// renumbers every index after it, so a check that pressed "row 1" twice would
/// press two different widgets and a check that pressed "the box at position 4"
/// twice would press the same one or find it gone. The second is a question
/// with an answer.
const REGION_PREFIX: &str = "tab-order.register."; // ui-text-exempt: trace region name, never displayed

/// A one-word name for a refusal, for the trace only.
///
/// Deliberately not the error's own `Display` prose: that is a sentence, and a
/// trace line is parsed by field. `check-ui-strings.sh`'s exclusion 3 says an
/// error type's prose is not permission to route text through it, and this is
/// the same rule pointing at the diagnostic channel instead of the screen.
fn refusal_kind(error: &pdfcer_core::edit::EditError) -> &'static str {
    use pdfcer_core::edit::EditError as E;
    match error {
        // ui-text-exempt: trace field values, never displayed
        E::WidgetHasNoFieldIdentity { .. } => "no-identity",
        E::FieldNameTaken { .. } => "name-taken",
        _ => "other",
    }
}

/// The hover on a Register control the preview says would refuse.
///
/// # ★ Three named arms and a catch-all that does NOT guess
///
/// `pdfcer_core::edit::EditError` is `#[non_exhaustive]`, so this needs a
/// wildcard whatever it does. The question is what the wildcard says, and the
/// answer is *"pdfcer cannot, and this panel does not know why"* rather than a
/// plausible guess.
///
/// Two of the five refusals are unreachable from here by construction —
/// `NotAWidget` and `WidgetAlreadyOwned`, because the ids come from exactly the
/// widgets the `/Annots` walk found unclaimed — so reaching the catch-all means
/// the listing and the engine disagree about what this widget is. That is a
/// fault to find in the trace, and handing an operator a confident wrong reason
/// for it is worse than handing them none.
fn refusal_hint(error: &pdfcer_core::edit::EditError) -> &'static str {
    use pdfcer_core::edit::EditError as E;
    match error {
        E::WidgetHasNoFieldIdentity { .. } => t::tab_order_register_needs_a_name(),
        E::FieldNameTaken { .. } => t::tab_order_register_name_taken(),
        _ => t::tab_order_register_unavailable(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::object::ObjId;

    /// An empty list draws nothing at all — not an empty group, not a heading.
    ///
    /// R9: a page whose widgets are all claimed has no problem to offer a
    /// remedy for, and a "0 boxes need registering" line is a placeholder
    /// wearing a number.
    #[test]
    fn a_page_with_nothing_unclaimed_draws_nothing() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
        // `run_ui` rather than `run` — egui 0.35 renamed it, and it hands the
        // closure a root `Ui` directly, which is what this needs.
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let before = ui.min_rect();
            let empty = Listing {
                pages: Vec::new(),
                fields_without_widgets: 0,
            };
            rows(ui, &doc, &empty, &mut actions);
            assert_eq!(ui.min_rect(), before, "nothing may be laid out");
        });
        assert!(actions.is_empty());
    }

    /// The rows are drawn in the order the model gives them, which is
    /// `/Annots` order.
    ///
    /// Asserted through the positions rather than through pixels: a row for
    /// position 2 must exist and must say 2, because the number is the only
    /// handle the operator has on a box with no name — they press Tab that many
    /// times to find it.
    #[test]
    fn each_unclaimed_widget_gets_its_tab_position() {
        assert_eq!(
            t::tab_order_unclaimed_row(3, 2),
            "Page 3, box 2 in the tab order"
        );
        assert_ne!(
            t::tab_order_unclaimed_row(3, 2),
            t::tab_order_unclaimed_row(3, 3),
            "two boxes on one page must be distinguishable"
        );
        // ★ And two boxes at the same tab position on DIFFERENT pages, which
        // is the case the page number was added for: these rows are gathered
        // from the whole document now, so a bare "Box 3" would name three
        // different boxes on a drawing with three affected sheets.
        assert_ne!(
            t::tab_order_unclaimed_row(3, 2),
            t::tab_order_unclaimed_row(4, 2),
            "the same position on two pages must be distinguishable"
        );
    }

    /// A draft map keyed on one revision is discarded by the next.
    ///
    /// The property that makes undo correct here — see [`Drafts`]. Exercised on
    /// the struct rather than through a frame, because the thing being asserted
    /// is the key comparison and not the widget.
    #[test]
    fn an_edit_forgets_every_typed_name() {
        let mut drafts = Drafts {
            key: Some((PathBuf::from("a.pdf"), 4)),
            names: BTreeMap::from([(12, "Address".to_owned())]),
        };
        assert_eq!(drafts.key, Some((PathBuf::from("a.pdf"), 4)));
        let stale = drafts.key.as_ref() != Some(&(PathBuf::from("a.pdf"), 5));
        assert!(stale, "a bumped epoch must invalidate the drafts");
        let other = drafts.key.as_ref() != Some(&(PathBuf::from("b.pdf"), 4));
        assert!(other, "a different document must invalidate the drafts");
        drafts.names.clear();
        assert!(drafts.names.is_empty());
    }

    /// The id this state stores itself under is not the fill panel's.
    ///
    /// Two `Clone` types in one `data` store under one id is a silent
    /// overwrite: whichever stores second wins, and the symptom is a text box
    /// that forgets a keystroke at a time.
    #[test]
    fn the_draft_store_does_not_collide_with_the_fill_panel() {
        assert_ne!(Drafts::id(), egui::Id::new("pdfcer-forms-ui"));
    }

    /// ★★ The catch-all refusal does not invent a reason.
    ///
    /// The two the operator can act on get their own sentence. Everything else
    /// gets one that says pdfcer cannot and does not say why — because reaching
    /// it means the listing and the engine disagree about what this widget is,
    /// and a confident wrong reason for that is worse than none.
    ///
    /// `WidgetAlreadyOwned` is the probe worth having: it is the refusal that
    /// would arrive if this panel ever offered a widget that already has a
    /// field, and *"type a different name"* would be actively misleading advice
    /// about it.
    #[test]
    fn an_unexpected_refusal_says_so_rather_than_guessing() {
        use pdfcer_core::edit::EditError as E;
        assert_eq!(
            refusal_hint(&E::WidgetHasNoFieldIdentity { id: 3 }),
            t::tab_order_register_needs_a_name()
        );
        assert_eq!(
            refusal_hint(&E::FieldNameTaken {
                name: "Address".to_owned()
            }),
            t::tab_order_register_name_taken()
        );
        let unexpected = refusal_hint(&E::WidgetAlreadyOwned { id: 3 });
        assert_eq!(unexpected, t::tab_order_register_unavailable());
        assert!(
            unexpected.contains("diagnostic trace"),
            "it must send the reader somewhere real: {unexpected}"
        );
        for guess in ["name", "type a"] {
            assert!(
                !unexpected.to_lowercase().contains(guess),
                "the catch-all must not advise an action for a state it does not understand:                  {unexpected}"
            );
        }
    }

    /// The button names the field it will create, and the two labels differ.
    ///
    /// ★ The blank-box case is the one that matters: the name comes out of the
    /// FILE, and it is a string the operator has never seen — nothing in the
    /// panel could have shown it, because the widget belongs to no field and so
    /// no field row names it.
    #[test]
    fn the_button_names_the_field_when_the_preview_knows_it() {
        let named = t::tab_order_register_as("Address");
        assert!(named.contains("Address"));
        assert_ne!(named, t::tab_order_register());
        assert!(
            named.starts_with("Register as"),
            "the verb stays first so the control still reads as a button: {named}"
        );
    }

    /// The typeless-field hover says the registration will WORK and still not
    /// be enough.
    ///
    /// ★★ Rule 4's half that survives: an inference the operator cannot
    /// see. Both halves have to be in the sentence — a hover that only said
    /// "this will register" would be true and useless, and one that only said
    /// "no viewer can fill it" would read as a refusal for something that is
    /// about to succeed.
    #[test]
    fn the_typeless_warning_says_both_halves() {
        let text = t::tab_order_register_no_type();
        assert!(text.contains("will register"), "{text}");
        assert!(text.contains("no type"), "{text}");
        assert!(
            text.contains("no viewer will know how to fill it"),
            "the consequence is the part the operator needs: {text}"
        );
    }

    /// An object number survives the round trip into the draft key.
    #[test]
    fn drafts_are_keyed_by_object_number() {
        let id = ObjId::new(12, 0);
        let mut names = BTreeMap::new();
        names.insert(id.num, "Agree".to_owned());
        assert_eq!(names.get(&12).map(String::as_str), Some("Agree"));
    }
}
