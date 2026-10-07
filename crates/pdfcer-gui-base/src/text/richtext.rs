//! # `text::richtext` — naming the formatting a rich-text body holds
//!
//! Shared by the Forms panel (a field's `/RV`) and the Comments panel (an
//! annotation's `/RC`): both are §12.7.3.4 rich text, parsed by
//! `pdfcer_core::richtext::parse` into runs, and both owe the operator the
//! formatting pdfcer shows as plain text.

use pdfcer_core::richtext::{Align, Run};

/// The distinct formatting `runs` carry, emphasis first, then size, family
/// and colour, then alignment. Empty when no run sets anything.
#[must_use]
pub fn formatting(runs: &[Run]) -> Vec<String> {
    let mut emphasis: Vec<String> = Vec::new();
    let mut typography: Vec<String> = Vec::new();
    let mut layout: Vec<String> = Vec::new();
    let push = |bucket: &mut Vec<String>, s: String| {
        if !bucket.contains(&s) {
            bucket.push(s);
        }
    };

    for r in runs {
        let st = &r.style;
        if st.weight.is_some_and(|w| w >= 700) {
            push(&mut emphasis, "bold".to_owned());
        }
        if st.italic == Some(true) {
            push(&mut emphasis, "italic".to_owned());
        }
        if st.underline == Some(true) {
            push(&mut emphasis, "underlined".to_owned());
        }
        if st.strikethrough == Some(true) {
            push(&mut emphasis, "struck through".to_owned());
        }
        if let Some(v) = st.baseline_shift_pt {
            // Named by meaning: Table 225's positive-is-superscript is the
            // opposite of what CSS suggests.
            let s = if v > 0.0 { "superscript" } else { "subscript" };
            push(&mut emphasis, s.to_owned());
        }
        if let Some(sz) = st.size_pt {
            push(&mut typography, format!("{sz} pt"));
        }
        if let Some(f) = st.family.first() {
            push(&mut typography, f.clone());
        }
        if let Some([r, g, b]) = st.color {
            let byte = |v: f64| (v * 255.0).round().clamp(0.0, 255.0) as u8;
            push(
                &mut typography,
                format!("#{:02X}{:02X}{:02X}", byte(r), byte(g), byte(b)),
            );
        }
        if let Some(a) = st.align {
            // Left is the interface's own reading direction and distinguishes
            // nothing; the other two are choices someone made.
            match a {
                Align::Center => push(&mut layout, "centred".to_owned()),
                Align::Right => push(&mut layout, "right-aligned".to_owned()),
                Align::Left => {}
            }
        }
    }

    emphasis.extend(typography);
    emphasis.extend(layout);
    emphasis
}

/// `header`, then one line per run: its text (cut at 32 characters) and its
/// emphasis.
#[must_use]
pub fn breakdown(header: &str, runs: &[Run]) -> String {
    let mut s = String::from(header);
    for r in runs {
        let text: String = if r.text.chars().count() > 32 {
            let head: String = r.text.chars().take(32).collect();
            format!("{head}…")
        } else {
            r.text.clone()
        };
        let mut bits: Vec<&str> = Vec::new();
        if r.style.weight.is_some_and(|w| w >= 700) {
            bits.push("bold");
        }
        if r.style.italic == Some(true) {
            bits.push("italic");
        }
        if r.style.underline == Some(true) {
            bits.push("underlined");
        }
        if r.style.strikethrough == Some(true) {
            bits.push("struck through");
        }
        // "as the rest" rather than "plain": a run with no emphasis of its own
        // still carries the default size, family and colour.
        let how = if bits.is_empty() {
            "as the rest".to_owned()
        } else {
            bits.join(" + ")
        };
        s.push_str(&format!("\n  “{text}” — {how}"));
    }
    s
}
