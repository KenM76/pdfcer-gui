//! # `entry::tests` — what the value-box reader accepts and refuses

use super::*;

const PT: Kind = Kind::Length(LengthUnit::Point);
const MM: Kind = Kind::Length(LengthUnit::Of(Unit::Millimeter));
const PCT: Kind = Kind::Number(&["%"]);

fn v(input: &str, current: f64, kind: Kind) -> f64 {
    match preprocess(input, current, kind) {
        Ok(Entry::Value(v)) => v,
        other => panic!("{input:?} read as {other:?}"),
    }
}

fn err(input: &str, kind: Kind) -> EntryError {
    preprocess(input, 0.0, kind).expect_err(input)
}

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn text_is_returned_verbatim() {
    let s = "  12mm + 3 = (x) ";
    assert_eq!(
        preprocess(s, 5.0, Kind::Text),
        Ok(Entry::Text(s.to_owned()))
    );
}

#[test]
fn a_bare_number_is_in_the_boxs_own_unit() {
    close(v("12.5", 0.0, PT), 12.5);
    close(v("12.5", 0.0, MM), 12.5);
}

// ORACLE, NOT A CONVERSION: each expected value is the definition of the
// unit (1 in = 72 pt = 25.4 mm), written out so the reader is checked against it.
#[test]
fn units_convert_into_the_boxs_unit() {
    close(v("1in", 0.0, PT), 72.0);
    close(v("1 \"", 0.0, PT), 72.0);
    close(v("25.4 mm", 0.0, PT), 72.0);
    close(v("2.54cm", 0.0, MM), 25.4);
    close(v("1 m", 0.0, MM), 1000.0);
    close(v("96px", 0.0, PT), 72.0);
    close(v("72 pt", 0.0, MM), 25.4); // ORACLE, NOT A CONVERSION: see above.
    close(v("1'", 0.0, PT), 864.0);
    close(v("1 Foot", 0.0, PT), 864.0);
    close(v("10 millimetres", 0.0, MM), 10.0);
}

#[test]
fn drawing_notation_reads_as_written() {
    close(v("55 5/8\"", 0.0, PT), 55.625 * 72.0);
    close(v("5/8\"", 0.0, PT), 0.625 * 72.0);
    close(v("4'-7 1/2\"", 0.0, PT), (48.0 + 7.5) * 72.0);
    close(v("4' 7 1/2\"", 0.0, PT), (48.0 + 7.5) * 72.0);
    close(v("4' 7", 0.0, PT), 55.0 * 72.0);
    close(v("1m 20cm", 0.0, MM), 1200.0);
    close(v("4\u{2032}-6\u{2033}", 0.0, PT), 54.0 * 72.0);
}

#[test]
fn arithmetic_follows_precedence() {
    close(v("12mm + 1/4\"", 0.0, MM), 12.0 + 6.35);
    close(v("2 + 3 * 4", 0.0, PT), 14.0);
    close(v("(2 + 3) * 4", 0.0, PT), 20.0);
    close(v("10mm / 2", 0.0, MM), 5.0);
    close(v("3 × 2", 0.0, PCT), 6.0);
    close(v("-10", 0.0, PT), -10.0);
    close(v("100-5", 0.0, PT), 95.0);
}

#[test]
fn a_leading_operator_applies_to_the_current_value() {
    close(v("+10px", 100.0, PT), 107.5);
    close(v("+ 10px", 100.0, PT), 107.5);
    close(v("+10", 100.0, MM), 110.0);
    close(v("- 5", 100.0, PT), 95.0);
    close(v("*2", 100.0, PT), 200.0);
    close(v("/4", 100.0, PCT), 25.0);
    close(v("+1mm", 10.0, MM), 11.0);
}

#[test]
fn a_hyphen_against_a_digit_is_a_negative_number_not_a_subtraction() {
    close(v("-10", 100.0, PT), -10.0);
}

#[test]
fn the_current_text_with_an_appended_operation_reads_too() {
    close(v("100+10px", 100.0, PT), 107.5);
}

#[test]
fn a_plain_number_box_refuses_units_but_takes_its_own_suffix() {
    close(v("50%", 0.0, PCT), 50.0);
    close(v("50 % + 5", 0.0, PCT), 55.0);
    assert_eq!(err("5mm", PCT), EntryError::UnitNotAllowed("mm".into()));
    assert_eq!(
        err("5mm", Kind::Count),
        EntryError::UnitNotAllowed("mm".into())
    );
}

#[test]
fn a_count_refuses_fractions_and_rounds_nothing() {
    close(v("3*4", 0.0, Kind::Count), 12.0);
    assert_eq!(err("7/2", Kind::Count), EntryError::NotWhole);
}

#[test]
fn unreadable_input_is_refused_not_guessed() {
    assert_eq!(err("", PT), EntryError::Empty);
    assert_eq!(err("   ", PT), EntryError::Empty);
    assert_eq!(
        err("12 furlongs", PT),
        EntryError::UnknownUnit("furlongs".into())
    );
    assert_eq!(err("2mm * 3mm", PT), EntryError::Mismatch);
    assert_eq!(err("2 / 3mm", PT), EntryError::Mismatch);
    assert_eq!(err("1 000", PT), EntryError::Unexpected(2));
    assert_eq!(err("12 +", PT), EntryError::Incomplete);
    assert_eq!(err("(12", PT), EntryError::Incomplete);
    assert_eq!(err("12 #", PT), EntryError::Unexpected(3));
    assert_eq!(err("1/0", PT), EntryError::NotFinite);
    assert_eq!(err("1 / 0", PT), EntryError::NotFinite);
}

/// One frame of a box holding `value`, with `events` delivered to it.
fn frame(ctx: &egui::Context, value: &mut f64, events: Vec<egui::Event>) -> egui::Id {
    let input = egui::RawInput {
        events,
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(400.0, 300.0),
        )),
        ..Default::default()
    };
    let mut id = egui::Id::NULL;
    let _ = ctx.run_ui(input, |ui| {
        let (widget, refusal) = drag_value(ui, value, PT);
        id = refusal.show(ui.add(widget)).id;
    });
    id
}

fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}

/// Type `text` the way `DragValue` sees typing: one reparse per keystroke, of
/// the text so far. Each prefix is typed into an emptied box, because a
/// headless `TextEdit` keeps its cursor at the start.
fn type_into(ctx: &egui::Context, id: egui::Id, value: &mut f64, text: &str) {
    ctx.memory_mut(|m| m.request_focus(id));
    frame(ctx, value, vec![]);
    for end in 1..=text.len() {
        ctx.data_mut(|d| d.insert_temp(id, String::new()));
        frame(ctx, value, vec![egui::Event::Text(text[..end].into())]);
    }
}

#[test]
fn relative_entry_applies_once_however_many_times_the_box_reparses() {
    let ctx = egui::Context::default();
    let mut value = 100.0;
    let id = frame(&ctx, &mut value, vec![]);

    type_into(&ctx, id, &mut value, "+10px");
    frame(
        &ctx,
        &mut value,
        vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    for _ in 0..3 {
        frame(&ctx, &mut value, vec![]);
    }
    close(value, 107.5);

    // The next edit is relative to the new value, not the first one.
    type_into(&ctx, id, &mut value, "*2");
    ctx.memory_mut(|m| m.surrender_focus(id));
    for _ in 0..3 {
        frame(&ctx, &mut value, vec![]);
    }
    close(value, 215.0);
}
