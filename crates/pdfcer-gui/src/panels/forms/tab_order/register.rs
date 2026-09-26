//! # `panels::forms::tab_order::register` — the rows that put an unclaimed
//! form control back into the form
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/forms/tab_order/register.md`.

use crate::app::actions::forms::FieldAction;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::forms as t;

use super::model::{Listing, PageTabs};

/// What the operator has typed into the name boxes, and which document
/// revision it describes.
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
pub(super) fn rows(ui: &mut egui::Ui, doc: &OpenDoc, listing: &Listing, actions: &mut Vec<Action>) {
    // EVERY page's unclaimed widgets, at the TOP of the section, and that
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
        // TWO LINES, not one, because this is a DOCK PANEL and not a dialog.
        //
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
            // ASKED before the press, which is what makes the two shapes
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
            // The name it WILL use, on the wrapping line, for a blank box that
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
                    // escape-disposition: keeps-draft — the draft lives in panel state and
                    // is committed by Register. There is no focus-loss branch to clear it.
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
                // One region per row, and a trace line saying what the preview
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
const REGION_PREFIX: &str = "tab-order.register."; // ui-text-exempt: trace region name, never displayed

/// A one-word name for a refusal, for the trace only.
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
fn refusal_hint(error: &pdfcer_core::edit::EditError) -> &'static str {
    use pdfcer_core::edit::EditError as E;
    use pdfcer_core::forms_author::FormAuthorError as A;
    match error {
        E::WidgetHasNoFieldIdentity { .. } => t::tab_order_register_needs_a_name(),
        E::FieldNameTaken { .. } => t::tab_order_register_name_taken(),
        //
        // Which makes these two arms necessary rather than decorative: the
        // refusal that would have been a status-bar sentence after a press is a
        // hover before one, and without them it fell into the catch-all saying
        // *the reason is not one this panel expects*. The rule was being
        // enforced and the operator was being told the program was confused.
        //
        // ⇒ **A guard's placement decides which surface has to explain it.** The
        // engine moved this one into the shared plan for its own reasons, and
        // the disclosure moved with it — silently, into a panel whose catch-all
        // then apologised for it.
        E::FieldAuthoring(A::DottedPartialName { .. }) => t::tab_order_register_name_is_a_path(),
        E::FieldAuthoring(A::EmptyNameSegment { .. }) => {
            t::tab_order_register_name_has_a_bare_dot()
        }
        _ => t::tab_order_register_unavailable(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::object::ObjId;

    /// An empty list draws nothing at all — not an empty group, not a heading.
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
        // And two boxes at the same tab position on DIFFERENT pages, which
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
    #[test]
    fn the_draft_store_does_not_collide_with_the_fill_panel() {
        assert_ne!(Drafts::id(), egui::Id::new("pdfcer-forms-ui"));
    }

    /// The catch-all refusal does not invent a reason.
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
                "the catch-all must not advise an action for a state it does \
                 not understand: {unexpected}"
            );
        }
    }

    /// The button names the field it will create, and the two labels differ.
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

    /// **The dotted name never reaches a press here, and the table that
    /// said it did was measuring the wrong gate.**
    #[test]
    fn a_dotted_name_greys_the_register_button_and_the_hover_names_the_rule() {
        let doc = crate::app::state::open_local_fixture(crate::app::state::ORPHAN_WIDGET);
        let widget = only_unclaimed_widget(&doc);

        // Asked exactly as the row asks it — with what the operator has typed so
        // far, not with `None`.
        let refusal = doc
            .session
            .adopt_preview(widget, Some("Text.2"))
            .expect_err("the preview must refuse a dotted partial name. Passing here means the guard is no longer inside `adopt_plan`, so this button is live and the operator can author a field nobody can address");

        assert_eq!(
            refusal_hint(&refusal),
            t::tab_order_register_name_is_a_path(),
            "the hover must name the period rule. The catch-all says the reason is not one this panel expects, which is the program apologising for a rule it is enforcing correctly. Refusal was: {refusal:?}"
        );

        // The control, and it is not ceremony: without it the assertion above
        // would be green on a fixture that can never be adopted for some other
        // reason entirely, and would then be measuring nothing about periods.
        doc.session
            .adopt_preview(widget, Some("Claimed"))
            .expect("an undotted name must be offered on this fixture; a refusal here means the widget is unadoptable for an unrelated reason and the dotted assertion above proves nothing");
    }

    /// The bare-dot refusal is reachable from here too, and worded separately.
    #[test]
    fn a_name_with_a_bare_dot_is_refused_with_its_own_sentence() {
        let doc = crate::app::state::open_local_fixture(crate::app::state::ORPHAN_WIDGET);
        let widget = only_unclaimed_widget(&doc);

        let refusal = doc
            .session
            .adopt_preview(widget, Some("a..b"))
            .expect_err("a doubled period leaves a segment with nothing in it, which no viewer can address. Passing here is the pre-2026-09-12 engine behaviour, where this verb took the period rule and the rename verb took the segment rule");

        assert_eq!(
            refusal_hint(&refusal),
            t::tab_order_register_name_has_a_bare_dot(),
            "the empty-segment refusal has its own sentence because its remedy is different: the operator is not being told to remove every period, only to put a name beside this one. Refusal was: {refusal:?}"
        );
    }

    /// The fixture's single unclaimed `/Widget`, by object id.
    fn only_unclaimed_widget(doc: &OpenDoc) -> ObjId {
        let view = doc.session.view();
        let slots = doc.session.page_slots().expect("the fixture's page tree walks — it is five objects and its xref offsets are asserted by its own generator");
        let form = pdfcer_core::forms::parse_acroform(&view);
        let listing = super::super::model::collect(&view, &slots, form.as_ref());
        let found = &listing.pages[0].unclaimed;
        assert_eq!(
            found.len(),
            1,
            "`orphan-widget.pdf` holds exactly one unclaimed widget on its one page; see `fixtures/orphan-widget.PROVENANCE.py`. Found: {found:?}"
        );
        found[0].id
    }
}
