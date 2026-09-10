//! Tests for [`super`] — the widget-box model that decides which form fields
//! this canvas will draw a box for, where those boxes land in canvas space, and
//! what the in-place editor looks like once one is opened.
//!
//! # Why this file exists separately from the module it tests
//!
//! Split out of `boxes.rs` on 2026-09-10 under **R2** (no source file over
//! 1,500 lines). The file had reached 1,501 lines — one over — after a
//! fixture-wide edit added `Page::resources_defaulted` to every hand-built
//! page in the crate, and the seam this project takes first is always the one
//! between a module and its tests: they have different readers, they change for
//! different reasons, and the code half of this module is under 700 lines and
//! is the part someone debugging a mis-placed widget box needs to hold in their
//! head.
//!
//! Nothing here changed in the move except indentation. `use super::*;` still
//! reaches every item, and every test kept its name so `cargo test <name>` and
//! every citation elsewhere in the repository still resolve.
//!
//! # What these tests are for
//!
//! The subject is `classify`, and it is a **refusal** function: it answers
//! *"can this field/widget pair be drawn and edited on the canvas?"* and, when
//! the answer is no, it says which of the `NotOnCanvas` reasons applies. The
//! fixtures are built by hand rather than loaded from a document precisely
//! because the interesting inputs are combinations no single real file carries
//! — a rich-text field that is also read-only, a widget with a degenerate
//! `/Rect`, a page rotation that swaps the axes. A fixture-driven suite would
//! test the documents that happen to be on disk; this one tests the predicate.

//! ## ★ `#![cfg(test)]` as well as the parent's `#[cfg(test)] mod tests;`
//!
//! Redundant to the compiler, load-bearing to three instruments, and the
//! reason this split was not free.
//!
//! `tools/gates/check-ui-strings.sh` stops scanning a file at a `#[cfg(test)]`
//! line, because assertion messages are prose read by whoever is staring at a
//! failing test and never by an operator. A whole-file test module has no such
//! line to stop at, so **the first version of this file made the gate report 39
//! of its assertion messages as operator-facing copy** — the same arrival
//! `canvas/selection/tests.rs` (28 hits) and `stamps/tests.rs` (19) each
//! recorded before it. The hazard is not the failing gate, it is the noise: a
//! report full of false positives trains people to skim it.
//!
//! `check-theme-colors.sh` recognises the same inner attribute from the AST,
//! and so does `app::settings`' `syn` call-site scan — which this split also
//! caught blind, because it could see `#[cfg(test)] mod tests { … }` and
//! `#![cfg(test)]` but not a parent's `#[cfg(test)] mod tests;`. All of them
//! state the same reason for keying on the attribute rather than the filename:
//! the property that earns the exemption is *"not in the shipped binary"*, and
//! a filename is a restatement of that which goes stale the moment another such
//! module is written.

#![cfg(test)]

use super::*;
use pdfcer_core::object::Dict;
use pdfcer_core::page_tree::Rect as PageRect;
use pdfcer_core::vartext::Quadding;

/// A terminal text field with one drawn widget, built by hand.
///
/// By hand rather than from a fixture because the predicate under test is
/// about combinations no single real document carries — a rich-text
/// read-only field on a rotated page is not a document anybody shipped.
fn text_field() -> Field {
    Field {
        id: ObjId::new(1, 0),
        fully_qualified_name: "Name".to_owned(),
        partial_name: None,
        alternate_name: None,
        mapping_name: None,
        rich_value: None,
        default_style: None,
        field_type: Some(FieldType::Text),
        button_kind: None,
        flags: FieldFlags(0),
        value: FieldValue::Absent,
        default_value: FieldValue::Absent,
        default_appearance: None,
        quadding: Quadding::Left,
        max_len: None,
        options: Vec::new(),
        top_index: 0,
        selected_indices: Vec::new(),
        widgets: vec![drawn_widget()],
        merged: false,
        has_additional_actions: false,
        shares_parent_name: false,
        parent: None,
    }
}

/// A widget with a drawn appearance.
///
/// # ★ `page: None` and `rect: None` are the point of this fixture
///
/// Both keys are Optional in the spec and **both are frequently absent in
/// real files** — and `pdfcer-core` additionally reads `/P` without
/// resolving through the graph, so a direct rather than indirect `/P` also
/// reads as absent. Every one of the ten form fixtures in
/// `D:\Dev\pdfcer\fixtures\synthetic\forms\` writes `/P` on every widget, so
/// a test built from a fixture cannot reach the case; the engine team hit
/// exactly that when a deliberate sabotage of their own `/P` handling
/// passed against their whole corpus.
///
/// So the fixture omits both, and every assertion in this module is
/// therefore also an assertion that neither is consulted. If someone
/// reintroduces a `/P` lookup or a `Widget::rect` read, these tests stop
/// passing here rather than stopping working in the field.
fn drawn_widget() -> Widget {
    Widget {
        id: ObjId::new(2, 0),
        rect: None,
        appearance_state: None,
        on_states: Vec::new(),
        // ★ `rotation` arrived with the engine's `rotate_widget` Pass on
        // 2026-08-30. `None` here means the file states none, which is what
        // every fixture in this shell wants: a widget with no `/MK /R`.
        rotation: None,
        has_off_appearance: false,
        page: None,
        caption: None,
        // `Pass 146.0`'s three, in a test fixture: the file states no
        // border and no unusual flags. `None` is the honest value for a
        // synthetic widget — it means "this file says nothing", which is
        // exactly true of one built in a test.
        // ★ `background` arrived with the engine's field-shading Pass on
        // 2026-09-04, alongside `border`. `None` is the honest value for a
        // synthetic widget for the same reason the two below it are: it
        // means "this file states no /MK /BG", which is exactly true of one
        // built in a test.
        background: None,
        // The widget states no /MK /BC. Arrived with the engine commit
        // fad0d2d (2026-09-07), which added /BC beside /BG and fixed a
        // read/write key mismatch between them. See ENGINE_BACKLOG.md:
        // neither colour is consumed by this shell yet.
        border_color: None,
        border: None,
        visibility: None,
        annot_flags: pdfcer_core::annot::AnnotFlags(0),
        has_normal_appearance: true,
        merged: false,
    }
}

/// The `/Rect` `EditSession::widget_rects` reports for [`drawn_widget`] —
/// near the TOP of a 612×792 page in PDF terms, i.e. a large Y.
const WIDGET_RECT: [f64; 4] = [100.0, 700.0, 300.0, 720.0];

/// One page's `/Annots` listing, as the engine's verb hands it over.
fn annots_listing(widget: &Widget) -> Vec<Vec<(ObjId, [f64; 4])>> {
    vec![vec![(widget.id, WIDGET_RECT)]]
}

/// A one-field form around `field`.
fn form_of(field: Field) -> AcroForm {
    AcroForm {
        fields: vec![field],
        groups: Vec::new(),
        need_appearances: false,
        sig_flags: 0,
        signatures_exist: false,
        append_only: false,
        calc_order_count: 0,
        calc_order: Vec::new(),
        has_default_resources: false,
        default_appearance: None,
        quadding: Quadding::Left,
        xfa: pdfcer_core::forms::XfaPresence::None,
        inline_field_roots: 0,
    }
}

fn page(rotate: u16) -> Page {
    Page {
        id: ObjId::new(9, 0),
        resources: Dict::new(),
        media_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
        crop_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
        rotate,
        contents: Vec::new(),
        contents_unresolved: 0,
        resources_defaulted: false,
        contents_flattened: 0,
    }
}

/// ★ **A field with no `/AP` is not offered on the page.**
///
/// The decision the module header §5.1 argues for, pinned. `demo-form.pdf`
/// carries this case, and the failure if it regressed is the worst kind:
/// an invisible click target over blank paper, which an operator can only
/// find by accident and cannot find again.
#[test]
fn an_undrawn_widget_is_not_offered_on_the_canvas() {
    let field = text_field();
    let mut widget = drawn_widget();
    widget.has_normal_appearance = false;
    assert_eq!(
        classify(&field, &widget, 0),
        Err(NotOnCanvas::NoAppearance),
        "a widget the page draws nothing for must not be clickable"
    );
    // …and the drawn twin IS offered, or the assertion above passes on a
    // build where nothing is ever offered.
    assert!(classify(&field, &drawn_widget(), 0).is_ok());
}

/// ★ **A rotated page withholds the EDITOR, not the click.**
///
/// Both halves, because the interesting content of the decision is the
/// asymmetry: a text field cannot be edited in place on a `/Rotate 90`
/// page (egui cannot rotate a `TextEdit`), while a check box has no text
/// direction and is offered exactly as it is anywhere else. A build that
/// refused both would be over-cautious in a way no operator could
/// understand.
#[test]
fn a_rotated_page_withholds_a_text_editor_but_not_a_button() {
    let field = text_field();
    for rotate in [90u16, 180, 270] {
        assert_eq!(
            classify(&field, &drawn_widget(), rotate),
            Err(NotOnCanvas::RotatedPage),
            "rotate={rotate}"
        );
    }
    assert!(classify(&field, &drawn_widget(), 0).is_ok());

    let mut check = text_field();
    check.field_type = Some(FieldType::Button);
    check.button_kind = Some(ButtonKind::Check);
    let mut widget = drawn_widget();
    widget.on_states = vec![b"Yes".to_vec()];
    for rotate in [0u16, 90, 180, 270] {
        assert!(
            classify(&check, &widget, rotate).is_ok(),
            "a check box has no text direction: rotate={rotate}"
        );
    }
}

/// ★ **The canvas refuses exactly what the panel's `block_reason` refuses.**
///
/// Asserted against the panel's own function rather than a re-derivation,
/// so the test cannot pass by agreeing with a third copy of the rule. The
/// silent failure it guards is two surfaces disagreeing about which fields
/// are fillable — an operator clicking a field on the page that the panel
/// says is read-only, or the reverse.
#[test]
fn the_canvas_declines_every_field_the_panel_blocks() {
    use pdfcer_core::forms::ButtonKind;

    let blocked = [
        Field {
            flags: FieldFlags(FieldFlags::READ_ONLY),
            ..text_field()
        },
        Field {
            field_type: Some(FieldType::Signature),
            ..text_field()
        },
        Field {
            field_type: Some(FieldType::Button),
            button_kind: Some(ButtonKind::Push),
            ..text_field()
        },
    ];
    for field in blocked {
        assert!(
            crate::panels::forms::rows::block_reason(&field).is_some(),
            "the fixture must be blocked for the assertion below to mean \
             anything"
        );
        assert_eq!(
            classify(&field, &drawn_widget(), 0),
            Err(NotOnCanvas::NotOffered),
        );
    }

    // Rich text is the case `block_reason` deliberately does NOT cover —
    // the panel offers a conversion. There is no conversion gesture on a
    // page, so the canvas declines it on its own account.
    let rich = Field {
        flags: FieldFlags(FieldFlags::RICH_TEXT),
        ..text_field()
    };
    assert!(rich.is_rich_text());
    assert!(crate::panels::forms::rows::block_reason(&rich).is_none());
    assert_eq!(
        classify(&rich, &drawn_widget(), 0),
        Err(NotOnCanvas::NotOffered),
    );
}

/// ★ **A radio widget carries its OWN on-state, not the field's first.**
///
/// The defect this prevents is a radio group in which every button selects
/// the first option: the field's `/V` would be set to the same name
/// whichever kid was clicked, and the group would look broken in a way
/// that reads as an engine fault rather than a shell one.
#[test]
fn each_radio_widget_selects_its_own_state() {
    let mut field = text_field();
    field.field_type = Some(FieldType::Button);
    field.button_kind = Some(ButtonKind::Radio);
    field.value = FieldValue::Name(b"Blue".to_vec());

    let mut red = drawn_widget();
    red.on_states = vec![b"Red".to_vec()];
    let mut blue = drawn_widget();
    blue.on_states = vec![b"Blue".to_vec()];

    assert_eq!(
        classify(&field, &red, 0),
        Ok(BoxKind::Radio {
            on_state: "Red".to_owned(),
            on: false
        })
    );
    assert_eq!(
        classify(&field, &blue, 0),
        Ok(BoxKind::Radio {
            on_state: "Blue".to_owned(),
            on: true
        })
    );
}

/// A button with no on-state anywhere is not offered, because
/// `set_button_state` would refuse every name but `Off`.
#[test]
fn a_button_with_no_on_state_is_not_offered() {
    let mut field = text_field();
    field.field_type = Some(FieldType::Button);
    field.button_kind = Some(ButtonKind::Check);
    assert_eq!(
        classify(&field, &drawn_widget(), 0),
        Err(NotOnCanvas::NotOffered)
    );
}

/// ★ **A widget with no `/P` entry is still placed** — the defect no
/// fixture in the corpus can catch.
///
/// `/P` is Optional (§12.5.2 Table 164) and frequently absent, and
/// `pdfcer-core` reads it without resolving through the graph, so a direct
/// `/P` reads as absent too. The obvious implementation of *"which page is
/// this widget on?"* — look up `Widget::page` — therefore returns **nothing
/// at all** on such a form: no error, no refusal, no trace, just a form on
/// which clicking a field silently does not work.
///
/// Every one of the ten form fixtures in
/// `D:\Dev\pdfcer\fixtures\synthetic\forms\` writes `/P` on every widget, so
/// a test opening a fixture cannot reach this. The engine team hit exactly
/// that: a deliberate sabotage of their own `/P` handling passed against
/// their whole corpus, and they had to build a form in memory that omits the
/// key. This is that form, in this shell — [`drawn_widget`] omits `/P`, and
/// the assertion is that the box is produced anyway.
///
/// It is `HANDOFF.md` §2's lesson in a new place: **a test that cannot
/// reach the case is satisfied by any implementation.**
#[test]
fn a_widget_with_no_p_entry_is_still_placed() {
    let field = text_field();
    assert!(
        field.widgets[0].page.is_none(),
        "the fixture must omit /P, or this test proves nothing"
    );

    let placed = place(
        &form_of(field),
        &[page(0)],
        &annots_listing(&drawn_widget()),
    );
    assert_eq!(
        placed.boxes.len(),
        1,
        "a widget with no /P must still be placed from its page's /Annots"
    );
    assert_eq!(placed.boxes[0].page, 0);
    assert_eq!(placed.routing, Routing::default());
}

/// A widget no page's `/Annots` lists has no place, and is counted as
/// unreachable rather than silently dropped.
#[test]
fn a_widget_no_page_lists_is_unreachable_and_counted() {
    let placed = place(&form_of(text_field()), &[page(0)], &[Vec::new()]);
    assert!(placed.boxes.is_empty());
    assert_eq!(
        placed.routing,
        Routing {
            undrawn: 0,
            unreachable: 1
        }
    );
}

/// ★ **The hit test is containment, and it is exclusive between
/// neighbours.**
///
/// The property the no-tolerance decision buys, asserted as the thing an
/// operator would notice: two fields one point apart — the ordinary shape
/// of a form table — resolve to exactly one answer each, and the gutter
/// between them resolves to neither. A six-point catch radius would make
/// all three of these ambiguous.
#[test]
fn two_adjacent_fields_never_claim_each_others_clicks() {
    let boxes = vec![
        WidgetBox {
            page: 0,
            field: "A".to_owned(),
            widget: 0,
            kind: BoxKind::Text {
                multiline: false,
                password: false,
                max_len: None,
                align: Quadding::Left,
            },
            rect: Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(60.0, 30.0)),
        },
        WidgetBox {
            page: 0,
            field: "B".to_owned(),
            widget: 0,
            kind: BoxKind::Text {
                multiline: false,
                password: false,
                max_len: None,
                align: Quadding::Left,
            },
            rect: Rect::from_min_max(Pos2::new(61.0, 10.0), Pos2::new(110.0, 30.0)),
        },
    ];

    assert_eq!(hit(&boxes, 0, Pos2::new(59.5, 20.0)).unwrap().field, "A");
    assert_eq!(hit(&boxes, 0, Pos2::new(61.5, 20.0)).unwrap().field, "B");
    assert!(
        hit(&boxes, 0, Pos2::new(60.5, 20.0)).is_none(),
        "the gutter between two fields belongs to neither"
    );
    // A different page never answers, however well the point fits.
    assert!(hit(&boxes, 1, Pos2::new(30.0, 20.0)).is_none());
}

/// A widget drawn over another wins, because it is the one the operator
/// can see.
#[test]
fn a_widget_drawn_over_another_claims_the_click() {
    let under = WidgetBox {
        page: 0,
        field: "Under".to_owned(),
        widget: 0,
        kind: BoxKind::Check {
            on_state: "Yes".to_owned(),
            on: false,
        },
        rect: Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0)),
    };
    let over = WidgetBox {
        field: "Over".to_owned(),
        rect: Rect::from_min_max(Pos2::new(40.0, 40.0), Pos2::new(60.0, 60.0)),
        ..under.clone()
    };
    let boxes = vec![under, over];
    assert_eq!(hit(&boxes, 0, Pos2::new(50.0, 50.0)).unwrap().field, "Over");
    assert_eq!(
        hit(&boxes, 0, Pos2::new(10.0, 10.0)).unwrap().field,
        "Under"
    );
}

/// ★ **A tiny field still gets a legible editor, and the box stays
/// centred on it.**
///
/// A 12 pt field at 25 % zoom is three screen points tall. Without the
/// minimum the operator cannot read what they typed; without the centring,
/// growing it would slide the box off the field it belongs to and the
/// editor would appear to jump as the zoom changed.
#[test]
fn a_field_too_small_to_read_is_grown_about_its_own_centre() {
    let extent = (612.0_f32, 792.0);
    let map = PageMapping::new(
        Rect::from_min_size(Pos2::new(20.0, 20.0), egui::vec2(153.0, 198.0)),
        extent,
        0.25,
    );
    let canvas = Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(140.0, 112.0));
    let natural = map.rect_to_screen(canvas);
    assert!(
        natural.height() < MIN_EDITOR.y,
        "the fixture must be too small, or the test is vacuous: {natural:?}"
    );

    let grown = editor_rect(&map, canvas);
    assert!(grown.width() >= MIN_EDITOR.x && grown.height() >= MIN_EDITOR.y);
    assert!(
        (grown.center() - natural.center()).length() < 0.01,
        "growing must not move the box: {natural:?} -> {grown:?}"
    );

    // …and a field that is already big enough is not touched at all.
    let big = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(600.0, 300.0));
    assert_eq!(editor_rect(&map, big), map.rect_to_screen(big));
}

/// The editor's text size stays inside the legible band at every zoom,
/// and grows with the box in between.
#[test]
fn the_editor_text_size_is_clamped_at_both_ends() {
    assert_eq!(editor_font_size(1.0), EDITOR_TEXT_RANGE.0);
    assert_eq!(editor_font_size(10_000.0), EDITOR_TEXT_RANGE.1);
    let small = editor_font_size(20.0);
    let large = editor_font_size(30.0);
    assert!(small < large, "{small} !< {large}");
}

/// ★★★ **A field's `/Q` reaches the box a click makes, and it reaches it
/// per field.**
///
/// The half of the quadding fix that lives in [`classify`]. Before
/// 2026-09-04 the classification carried no alignment at all, so
/// [`super::editor`] had nothing to read and every field — left, centred
/// or right — was typed into left-aligned. The value is asserted for all
/// three codes rather than for one, because a `field.quadding` that was
/// hard-wired to `Left` would pass a single-value test perfectly.
#[test]
fn a_fields_quadding_reaches_the_box_a_click_makes() {
    for want in [Quadding::Left, Quadding::Center, Quadding::Right] {
        let field = Field {
            quadding: want,
            ..text_field()
        };
        let kind = classify(&field, &drawn_widget(), 0).expect("an ordinary text field");
        let BoxKind::Text { align, .. } = kind else {
            panic!("a text field classifies as text");
        };
        assert_eq!(align, want, "the box must carry the field's own /Q");
    }
}

/// ★★ **The three `/Q` codes map to the three ends of the box, and the
/// centre one is not an end.**
///
/// A silent transposition is the failure this guards: swapping `Center`
/// and `Right` compiles, draws a caret, passes every other test in this
/// file, and is visible only as a right-aligned form typed into centred.
/// Asserted as three separate, distinct answers so a mapping that
/// collapsed two codes into one cannot pass either.
#[test]
fn each_quadding_code_anchors_the_editor_at_its_own_end() {
    assert_eq!(editor_align(Quadding::Left), Align::LEFT);
    assert_eq!(editor_align(Quadding::Center), Align::Center);
    assert_eq!(editor_align(Quadding::Right), Align::RIGHT);
    // …and no two codes share an answer, which is what a transposition
    // would otherwise be free to do in one direction.
    assert_ne!(editor_align(Quadding::Left), editor_align(Quadding::Right));
    assert_ne!(editor_align(Quadding::Left), editor_align(Quadding::Center));
    assert_ne!(
        editor_align(Quadding::Center),
        editor_align(Quadding::Right)
    );
}

/// `/MaxLen` truncates by character, not by byte.
///
/// The byte version compiles, runs, and refuses an accented name three
/// letters early — while also being able to split a character in half.
#[test]
fn max_len_counts_characters_not_bytes() {
    assert_eq!(truncate("Ångström", Some(4)), "Ångs");
    assert_eq!(truncate("Ångström", None), "Ångström");
    assert_eq!(truncate("abc", Some(10)), "abc");
    assert_eq!(truncate("", Some(0)), "");
}

/// ★ **Filling is offered in the select tool and in no other.**
///
/// The whole of the "no `CanvasTool` variant" decision, expressed as the
/// one line it is. The markup rows matter most: a pen that also filled a
/// field would make one press mean two things, and the operator would
/// discover it by finding text in a box they were drawing a rectangle
/// over.
#[test]
fn only_the_select_tool_fills_a_form() {
    use crate::canvas::markup::MarkupKind;

    assert!(offered_in(CanvasTool::Select));
    assert!(!offered_in(CanvasTool::Hand));
    for &kind in MarkupKind::ALL {
        assert!(!offered_in(CanvasTool::Markup(kind)), "{kind:?}");
    }
}

/// ★ **A whole document's boxes, from a real form fixture.**
///
/// The end-to-end shape of the read path — parse, place, project — on the
/// document the panel's own disclosures were written against. It asserts
/// what a screenshot cannot: that boxes exist at all, that each one names a
/// field the form really has, and that each lands inside the page it
/// claims.
///
/// Note what it deliberately does **not** prove:
/// [`a_widget_with_no_p_entry_is_still_placed`] exists because this test
/// cannot reach the `/P`-absent case — every widget in this fixture carries
/// `/P`, so a `/P`-keyed implementation would pass here.
#[test]
fn a_real_form_produces_boxes_inside_its_own_pages() {
    let doc = crate::app::state::open_fixture("forms/demo-form.pdf");
    let view = doc.session.view();
    let form = pdfcer_core::forms::parse_acroform(&view).expect("the fixture has a form");
    let pages = &doc.pages;
    let annots: Vec<Vec<(ObjId, [f64; 4])>> = (0..pages.len())
        .map(|page| doc.session.widget_rects(page))
        .collect();

    let placed = place(&form, pages, &annots);
    let boxes = &placed.boxes;
    assert!(
        !boxes.is_empty(),
        "a fillable form produced no clickable boxes at all"
    );

    let names: std::collections::BTreeSet<&str> = form
        .fields
        .iter()
        .map(|f| f.fully_qualified_name.as_str())
        .collect();
    for b in boxes {
        assert!(
            names.contains(b.field.as_str()),
            "{} is not a field of this form",
            b.field
        );
        assert!(b.page < pages.len(), "{} is on page {}", b.field, b.page);
        let (w, h) = crate::viewer::page_extent_pts(&pages[b.page]);
        let page_rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(w, h));
        assert!(
            page_rect.expand(1.0).contains_rect(b.rect),
            "{}'s box {:?} is outside its own {w}x{h} page",
            b.field,
            b.rect
        );
    }

    // ★ And the fixture's undrawn field really is absent from the list —
    // the panel already discloses that this document has one, so this is
    // the same fact asserted from the other end.
    let undrawn: Vec<&str> = form
        .fields
        .iter()
        .filter(|f| !f.has_appearance())
        .map(|f| f.fully_qualified_name.as_str())
        .collect();
    for name in undrawn {
        assert!(
            !boxes.iter().any(|b| b.field == name),
            "{name} has no drawn appearance and must not be clickable"
        );
    }
}

/// ★ **A generator, not a check: build a form with a DRAWN text field.**
///
/// ```text
/// cargo test -p pdfcer-gui a_drawn_text_field_fixture -- --ignored --nocapture
/// ```
///
/// # Why this has to exist
///
/// **Not one of the eleven form fixtures in
/// `D:\Dev\pdfcer\fixtures\synthetic\forms\` carries a text field with a
/// drawn appearance.** Measured by driving the binary over every one of
/// them and reading the `form-box` census: `demo-form` yields one check
/// box, `radio-choice-form` five radios, `radio-group-form` three radios,
/// and the other eight yield **nothing at all** — every text field in the
/// corpus is `/AP`-less, which is exactly the case §5.1 routes to the
/// panel.
///
/// So the corpus cannot exercise the in-place editor, which is the largest
/// thing this feature adds. That is a fact about the fixtures rather than
/// about the feature, and the honest response is to make the missing
/// document rather than to declare the path verified because the tests are
/// green — `HANDOFF.md` §2's whole subject.
///
/// It is `#[ignore]` and writes outside the source tree, following
/// `crate::shell::ron`'s generator precedent: a test that writes a file is
/// run deliberately, never as part of a sweep.
///
/// What it produces: `demo-form.pdf` with `Full name` filled. Filling is
/// what draws it — `fill_text_field` writes `/V` **and** regenerates every
/// widget's `/AP` — which is also the remedy
/// [`crate::text::forms::forms_canvas_undrawn_note`] names.
#[test]
#[ignore = "generator: writes a PDF for driving the binary; run deliberately"]
fn a_drawn_text_field_fixture() {
    use pdfcer_core::edit::EditSession;
    use pdfcer_core::writer::SaveOptions;

    let path = crate::panels::objects::test_support::engine_fixture("forms/demo-form.pdf");
    let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
    let mut session = EditSession::new(doc);
    // The first UNDRAWN text field, found rather than named: the fixture's
    // fully-qualified names are not the `/TU` labels the panel shows, and a
    // generator that hard-coded one would break the day the fixture is
    // regenerated.
    let target = {
        let view = session.view();
        let form = pdfcer_core::forms::parse_acroform(&view).expect("the fixture has a form");
        form.fields
            .iter()
            .find(|f| {
                f.field_type == Some(FieldType::Text) && !f.has_appearance() && !f.flags.read_only()
            })
            .map(|f| f.fully_qualified_name.clone())
            .expect("demo-form has an undrawn text field")
    };
    println!("filling {target}");
    session
        .fill_text_field(&target, "Ken Mantle")
        .expect("an undrawn text field can be filled");

    let (bytes, _) = session
        .to_incremental_bytes(&SaveOptions::identity())
        .expect("an incremental save of one fill");
    let out = std::env::temp_dir().join("pdfcer-drawn-text-form.pdf");
    std::fs::write(&out, bytes).expect("the temp directory is writable");
    println!("wrote {}", out.display());

    // ★ …and the same document turned a quarter-turn, because the ROTATED
    // decision has no fixture either. §5.2 says a rotated page withholds
    // the text EDITOR and keeps the button click, and that asymmetry is
    // only checkable by opening a rotated form and reading the `form-box`
    // census: the check box must still appear and the text field must not.
    session
        .set_page_rotation(0, 90)
        .expect("a page can be turned");
    let (turned, _) = session
        .to_incremental_bytes(&SaveOptions::identity())
        .expect("an incremental save of a rotation");
    let rotated = std::env::temp_dir().join("pdfcer-drawn-text-form-rotated.pdf");
    std::fs::write(&rotated, turned).expect("the temp directory is writable");
    println!("wrote {}", rotated.display());

    // Prove the point of the exercise: the field is now DRAWN, so the
    // canvas will offer it. If this ever stops being true the generator is
    // producing a document that does not test what it exists to test.
    let view = session.view();
    let form = pdfcer_core::forms::parse_acroform(&view).expect("the form survived");
    let field = form
        .fields
        .iter()
        .find(|f| f.fully_qualified_name == target)
        .expect("the field survived");
    assert!(
        field.has_appearance(),
        "filling must draw the field, or the generator produces nothing new"
    );
}

/// ★★★ **A kind that cannot be FILLED on the canvas can still be
/// SELECTED there**, which is the whole reason [`FieldTarget`] exists.
///
/// A drop-down (`/Ch`) is `NotOffered` — this shell has no canvas gesture
/// for one, and `classify` refuses it. Before selection existed, that
/// refusal removed it from the only list the canvas hit-tested, so a field
/// the operator could plainly see was not clickable at all.
///
/// The two assertions are deliberately opposite, because a test that only
/// checked the target would pass against a change that made every widget
/// fillable — which is a different bug with the same symptom on this test.
#[test]
fn a_choice_field_is_not_fillable_on_the_canvas_but_is_selectable() {
    let mut field = text_field();
    field.field_type = Some(FieldType::Choice);
    let widget = field.widgets[0].clone();
    let placed = place(&form_of(field), &[page(0)], &annots_listing(&widget));

    assert!(
        placed.boxes.is_empty(),
        "a drop-down has no canvas fill gesture and must not offer one"
    );
    assert_eq!(
        placed.targets.len(),
        1,
        "…and must still be selectable, or its properties are unreachable \
         from the page it is drawn on"
    );
}

/// **A widget with no appearance is selectable too.**
///
/// `NoAppearance` routes a field to the panel for FILLING because the page
/// draws nothing there — but pdfcer authors widgets, and a widget it made
/// and then failed to draw is exactly the one an operator needs to reach in
/// order to delete it. The rectangle is real even when the appearance is
/// not.
#[test]
fn an_undrawn_widget_is_still_selectable() {
    let mut field = text_field();
    field.widgets[0].has_normal_appearance = false;
    let widget = field.widgets[0].clone();
    let placed = place(&form_of(field), &[page(0)], &annots_listing(&widget));

    assert!(
        placed.boxes.is_empty(),
        "nothing is drawn there to type into"
    );
    assert_eq!(placed.routing.undrawn, 1, "and the panel is told why");
    assert_eq!(
        placed.targets.len(),
        1,
        "but it occupies a rectangle, so it can be selected and deleted"
    );
}

/// **A widget no page lists is selectable from nowhere**, because it has no
/// rectangle to click. The one exclusion that is not a policy.
#[test]
fn an_unplaced_widget_is_not_selectable() {
    let field = text_field();
    let placed = place(&form_of(field), &[page(0)], &[vec![]]);
    assert!(placed.targets.is_empty());
    assert_eq!(placed.routing.unreachable, 1);
}

/// **The hit test takes the widget drawn on top**, the same rule the fill
/// hit test follows, because it is the same question: which one can the
/// operator see?
#[test]
fn the_selection_hit_test_prefers_the_widget_drawn_last() {
    let under = FieldTarget {
        page: 0,
        field: "Under".to_owned(),
        widget: 0,
        rect: Rect::from_min_size(Pos2::new(0.0, 0.0), egui::vec2(100.0, 100.0)),
    };
    let over = FieldTarget {
        field: "Over".to_owned(),
        ..under.clone()
    };
    let targets = vec![under, over];
    assert_eq!(
        hit_target(&targets, 0, Pos2::new(50.0, 50.0)).map(|t| t.field.as_str()),
        Some("Over")
    );
    assert!(
        hit_target(&targets, 1, Pos2::new(50.0, 50.0)).is_none(),
        "wrong page"
    );
    assert!(
        hit_target(&targets, 0, Pos2::new(150.0, 50.0)).is_none(),
        "outside"
    );
}
