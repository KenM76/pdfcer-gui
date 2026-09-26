//! Tests for [`super`] — the widget-box model that decides which form fields
//! this canvas will draw a box for, where those boxes land in canvas space, and
//! what the in-place editor looks like once one is opened.
//!
//! # Why this file is separate from the module it tests
//!
//! **R2** caps a source file at 1,500 lines, and the seam this project takes
//! first is always the one between a module and its tests: they have different
//! readers and they change for different reasons. What stays in `boxes.rs` is
//! the part someone debugging a mis-placed widget box needs to hold in their
//! head.
//!
//! `use super::*;` reaches every item from here, so a test's **name** is the
//! only handle anything outside needs: `cargo test <name>` and every citation
//! elsewhere in the repository resolve against the name and not against the
//! file it sits in.
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
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/forms/boxes/tests.md`.

//! ## `#![cfg(test)]` as well as the parent's `#[cfg(test)] mod tests;`
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
fn drawn_widget() -> Widget {
    Widget {
        id: ObjId::new(2, 0),
        rect: None,
        appearance_state: None,
        on_states: Vec::new(),
        // `None` means the file states no `/MK /R`, which is what every
        // fixture in this shell wants: an unrotated widget.
        rotation: None,
        has_off_appearance: false,
        page: None,
        caption: None,
        // The appearance group — `/MK /BG`, `/MK /BC`, `/MK /BW` and the
        // annotation flags. `None` is the honest value for a synthetic
        // widget: it means "this file states nothing", which is exactly true
        // of one built in a test rather than read from a document.
        background: None,
        // /BG IS consumed — it tints the in-place editor, see
        // `editor_fill`. /BC is read and not yet drawn, and that is the only
        // half of this pair still outstanding.
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

/// **A field with no `/AP` is not offered on the page.**
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

/// **A rotated page withholds the EDITOR, not the click.**
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

/// **The canvas refuses exactly what the panel's `block_reason` refuses.**
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

/// **A radio widget carries its OWN on-state, not the field's first.**
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

/// **A widget with no `/P` entry is still placed** — the defect no
/// fixture in the corpus can catch.
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

/// **The hit test is containment, and it is exclusive between
/// neighbours.**
#[test]
fn two_adjacent_fields_never_claim_each_others_clicks() {
    let boxes = vec![
        WidgetBox {
            page: 0,
            id: ObjId::new(1, 0),
            field: "A".to_owned(),
            widget: 0,
            kind: BoxKind::Text {
                multiline: false,
                password: false,
                max_len: None,
                align: Quadding::Left,
            },
            rect: Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(60.0, 30.0)),
            fill: None,
        },
        WidgetBox {
            page: 0,
            id: ObjId::new(2, 0),
            field: "B".to_owned(),
            widget: 0,
            kind: BoxKind::Text {
                multiline: false,
                password: false,
                max_len: None,
                align: Quadding::Left,
            },
            rect: Rect::from_min_max(Pos2::new(61.0, 10.0), Pos2::new(110.0, 30.0)),
            fill: None,
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
        id: ObjId::new(3, 0),
        field: "Under".to_owned(),
        widget: 0,
        kind: BoxKind::Check {
            on_state: "Yes".to_owned(),
            on: false,
        },
        rect: Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0)),
        fill: None,
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

/// **A tiny field still gets a legible editor, and the box stays
/// centred on it.**
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

/// **A field's `/Q` reaches the box a click makes, and it reaches it
/// per field.**
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

/// **The three `/Q` codes map to the three ends of the box, and the
/// centre one is not an end.**
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

/// **Filling is offered in the select tool and in no other.**
#[test]
fn only_the_select_tool_fills_a_form() {
    use crate::canvas::markup::MarkupKind;

    assert!(offered_in(CanvasTool::Select));
    assert!(!offered_in(CanvasTool::Hand));
    for &kind in MarkupKind::ALL {
        assert!(!offered_in(CanvasTool::Markup(kind)), "{kind:?}");
    }
}

/// **A whole document's boxes, from a real form fixture.**
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

    // And the fixture's undrawn field really is absent from the list —
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

/// **The fixture O204’s driven check aims at really has three clickable text
/// fields** — `app::state::THREE_TEXT_FIELDS`.
#[test]
fn the_three_field_fixture_offers_three_clickable_text_boxes() {
    let doc = crate::app::state::open_local_fixture(crate::app::state::THREE_TEXT_FIELDS);
    let view = doc.session.view();
    let form = pdfcer_core::forms::parse_acroform(&view).expect("the fixture has a form");
    let pages = &doc.pages;
    let annots: Vec<Vec<(ObjId, [f64; 4])>> = (0..pages.len())
        .map(|page| doc.session.widget_rects(page))
        .collect();

    let placed = place(&form, pages, &annots);
    let text: Vec<&str> = placed
        .boxes
        .iter()
        .filter(|b| matches!(b.kind, BoxKind::Text { .. }))
        .map(|b| b.field.as_str())
        .collect();
    assert_eq!(
        text,
        vec!["FieldOne", "FieldTwo", "FieldThree"],
        "three drawn text fields, in the order the form declares them"
    );
    for b in &placed.boxes {
        assert_eq!(b.page, 0, "{} is not on the single page", b.field);
    }
}
/// **A generator, not a check: build a form with a DRAWN text field.**
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
/// green.
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
    // temp-path-exempt: an `#[ignore]`d fixture generator. The file is named
    // so a human can drive the binary against it afterwards, which a
    // pid-stamped name would defeat.
    let out = std::env::temp_dir().join("pdfcer-drawn-text-form.pdf");
    std::fs::write(&out, bytes).expect("the temp directory is writable");
    println!("wrote {}", out.display());

    // …and the same document turned a quarter-turn, because the ROTATED
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
    // temp-path-exempt: the same generator, its rotated half.
    let rotated = std::env::temp_dir().join("pdfcer-drawn-text-form-rotated.pdf");
    std::fs::write(&rotated, turned).expect("the temp directory is writable");
    println!("wrote {}", rotated.display());

    // Prove the point of the exercise: the field is DRAWN, so the canvas
    // will offer it. If this ever stops being true the generator is
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

/// **A kind that cannot be FILLED on the canvas can still be
/// SELECTED there**, which is the whole reason [`FieldTarget`] exists.
#[test]
fn a_choice_field_with_no_options_is_not_fillable_but_is_still_selectable() {
    let mut field = text_field();
    field.field_type = Some(FieldType::Choice);
    assert!(field.options.is_empty(), "the premise of this fixture");
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

/// **`/MK` `/BG`, every variant, and the two different `None`s.**
#[test]
fn a_background_is_read_and_stating_none_is_not_the_same_as_stating_nothing() {
    let mut w = drawn_widget();

    w.background = None;
    assert_eq!(
        editor_fill(&w),
        None,
        "a file that states no /MK /BG at all"
    );

    w.background = Some(MkColor::None);
    assert_eq!(
        editor_fill(&w),
        None,
        "an empty /BG array states transparent on purpose, which is still leave the theme box alone"
    );

    w.background = Some(MkColor::Gray(0.75));
    assert_eq!(editor_fill(&w), Some([0.75, 0.75, 0.75]));

    w.background = Some(MkColor::Rgb(1.0, 0.95, 0.6));
    assert_eq!(
        editor_fill(&w),
        Some([1.0, 0.95, 0.6]),
        "the pale-yellow form field this whole feature exists for"
    );
}

/// **A CMYK background goes through the ENGINE's calibrated conversion,
/// not an arithmetic one.**
#[test]
fn a_cmyk_background_uses_the_engines_own_table_and_not_one_minus_k() {
    let mut w = drawn_widget();

    w.background = Some(MkColor::Cmyk(0.0, 0.0, 0.0, 0.0));
    assert_eq!(
        editor_fill(&w),
        Some([1.0, 1.0, 1.0]),
        "no ink at all is paper white"
    );

    w.background = Some(MkColor::Cmyk(0.0, 0.0, 0.0, 1.0));
    let k100 = editor_fill(&w).expect("solid K is a colour");
    assert!(
        k100[0] > 0.10 && k100[0] < 0.18,
        "solid K ink alone is a warm near-black near 0.13, not the 0.0 that 1.0 - k gives: {k100:?}"
    );

    assert_eq!(
        editor_fill(&w),
        Some(pdfcer_core::color::cmyk_to_srgb(0.0, 0.0, 0.0, 1.0)),
        "the engine conversion verbatim, so this box and the raster under it can never disagree"
    );
}

/// **`/Ff` bit 18 reaches the box census, because it decides where the
/// options are drawn.**
#[test]
fn the_combo_flag_reaches_the_box_census() {
    for (flags, want_combo) in [(FieldFlags(1 << 17), true), (FieldFlags(0), false)] {
        let mut field = text_field();
        field.field_type = Some(FieldType::Choice);
        field.flags = flags;
        field.options = vec![
            pdfcer_core::forms::ChoiceOption {
                export: b"a".to_vec(),
                display: b"Alpha".to_vec(),
            },
            pdfcer_core::forms::ChoiceOption {
                export: b"b".to_vec(),
                display: b"Beta".to_vec(),
            },
        ];
        let widget = field.widgets[0].clone();
        let placed = place(&form_of(field), &[page(0)], &annots_listing(&widget));

        let Some(BoxKind::Choice { combo, .. }) = placed.boxes.first().map(|b| &b.kind) else {
            panic!("a choice field with options must be fillable on the canvas");
        };
        assert_eq!(*combo, want_combo, "flags {flags:?}");
    }
}

/// **`/Ff` bit 19 is only a capability when bit 18 is also set.**
#[test]
fn the_edit_flag_is_a_capability_only_alongside_the_combo_flag() {
    const COMBO: u32 = 1 << 17;
    const EDIT: u32 = 1 << 18;
    for (flags, want_editable) in [
        (FieldFlags(COMBO | EDIT), true),
        (FieldFlags(EDIT), false),
        (FieldFlags(COMBO), false),
    ] {
        let mut field = text_field();
        field.field_type = Some(FieldType::Choice);
        field.flags = flags;
        field.options = vec![pdfcer_core::forms::ChoiceOption {
            export: b"a".to_vec(),
            display: b"Alpha".to_vec(),
        }];
        let widget = field.widgets[0].clone();
        let placed = place(&form_of(field), &[page(0)], &annots_listing(&widget));

        let Some(BoxKind::Choice { editable, .. }) = placed.boxes.first().map(|b| &b.kind) else {
            panic!("a choice field with options must be fillable on the canvas");
        };
        assert_eq!(*editable, want_editable, "flags {flags:?}");
    }
}
