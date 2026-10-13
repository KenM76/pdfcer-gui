//! Carries a pasted raw annotation's geometry keys with its `/Rect`.
//!
//! `EditSession::paste_objects_on_layer` translates a carried raw annotation's
//! `/Rect` by the paste's offset and leaves `/QuadPoints`, `/L`, `/Vertices`,
//! `/CL` and `/InkList` at the source position. A pasted link then answers
//! clicks where the original was, in any viewer that honours `/QuadPoints`.
//! Engine request G186. Until it lands, [`carry`] translates those keys in the
//! clip before the paste, with the same key list and arithmetic as
//! `EditSession::move_annotation`. Delete this module when G186 is consumed.
//!
//! Only a pure translation is carried. The engine places any other matrix
//! translated by its bounds and discloses that itself.

use pdfcer_core::object::{Name, Object};
use pdfcer_core::vector::{ClipAnnotation, Matrix, ObjectClip};

/// The flat point arrays, in `move_annotation`'s order.
const FLAT_KEYS: [&[u8]; 4] = [b"L", b"Vertices", b"QuadPoints", b"CL"];

/// Translate every raw annotation's geometry keys in `clip` by `at`'s offset,
/// when `at` is a pure translation. Returns how many annotations changed.
pub(super) fn carry(clip: &mut ObjectClip, at: Matrix) -> usize {
    if !(at.a == 1.0 && at.b == 0.0 && at.c == 0.0 && at.d == 1.0) || (at.e == 0.0 && at.f == 0.0) {
        return 0;
    }
    clip.annotations
        .iter_mut()
        .filter_map(|annot| match annot {
            ClipAnnotation::Raw(raw) => Some(&mut raw.dict),
            _ => None,
        })
        .map(|dict| {
            let mut changed = false;
            for key in FLAT_KEYS {
                if let Some(Object::Array(items)) = dict.get(key)
                    && !items.is_empty()
                {
                    let moved = translate_flat(items, at.e, at.f);
                    dict.insert(Name::from(key), Object::Array(moved));
                    changed = true;
                }
            }
            if let Some(Object::Array(strokes)) = dict.get(b"InkList")
                && !strokes.is_empty()
            {
                let moved = strokes
                    .iter()
                    .map(|stroke| match stroke {
                        Object::Array(pts) => Object::Array(translate_flat(pts, at.e, at.f)),
                        other => other.clone(),
                    })
                    .collect();
                dict.insert(Name::from(b"InkList"), Object::Array(moved));
                changed = true;
            }
            changed
        })
        .filter(|&changed| changed)
        .count()
}

/// `items` as x,y pairs, each moved by `(dx, dy)`; a non-number is kept.
fn translate_flat(items: &[Object], dx: f64, dy: f64) -> Vec<Object> {
    items
        .iter()
        .enumerate()
        .map(|(i, o)| match o.as_number() {
            Some(v) => Object::Real(v + if i % 2 == 0 { dx } else { dy }),
            None => o.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_move_by_the_offset_and_non_numbers_stay() {
        let items = [Object::Integer(10), Object::Real(20.0), Object::Null];
        let moved = translate_flat(&items, 5.0, -3.0);
        assert_eq!(moved[0].as_number(), Some(15.0));
        assert_eq!(moved[1].as_number(), Some(17.0));
        assert!(matches!(moved[2], Object::Null));
    }
}
