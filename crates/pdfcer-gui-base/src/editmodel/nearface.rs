//! Which of several candidate faces is nearest a run's own: same family class
//! (serif, sans, monospace) first, then weight and slant, then a face the
//! page already carries over one pdfcer would add.
//!
//! Contract: [`nearest`] answers an index into `candidates`, or `None` when
//! the slice is empty. Ties go to the earlier candidate, so the caller's
//! order is the last word.

/// What a face name says about its design, read from the name alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Traits {
    class: Class,
    bold: bool,
    italic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Serif,
    Sans,
    Mono,
}

/// One candidate: its `/BaseFont` (any subset tag is ignored) and whether
/// the page already carries it.
#[derive(Debug, Clone, Copy)]
pub struct Candidate<'a> {
    pub base_font: &'a str,
    pub on_page: bool,
}

fn traits(base_font: &str) -> Traits {
    // A subset tag is six capitals and a plus; it says nothing about design.
    let name = base_font
        .split_once('+')
        .filter(|(tag, _)| tag.len() == 6)
        .map_or(base_font, |(_, rest)| rest)
        .to_ascii_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| name.contains(w));
    let class = if has(&["courier", "mono", "consol", "typewriter"]) {
        Class::Mono
    } else if has(&[
        "times", "roman", "serif", "georgia", "garamond", "cambria", "book", "palatino",
    ]) && !has(&["sans"])
    {
        Class::Serif
    } else {
        Class::Sans
    };
    Traits {
        class,
        bold: has(&["bold", "black", "heavy", "semibold", "demi"]),
        italic: has(&["italic", "oblique"]),
    }
}

/// The candidate nearest `base_font`. See the module contract.
#[must_use]
pub fn nearest(base_font: &str, candidates: &[Candidate<'_>]) -> Option<usize> {
    let want = traits(base_font);
    let score = |c: &Candidate<'_>| {
        let got = traits(c.base_font);
        u8::from(got.class == want.class) * 8
            + u8::from(got.bold == want.bold) * 4
            + u8::from(got.italic == want.italic) * 2
            + u8::from(c.on_page)
    };
    let mut best: Option<(usize, u8)> = None;
    for (i, c) in candidates.iter().enumerate() {
        let s = score(c);
        if best.is_none_or(|(_, b)| s > b) {
            best = Some((i, s));
        }
    }
    best.map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::{Candidate, nearest};

    fn add(base_font: &str) -> Candidate<'_> {
        Candidate {
            base_font,
            on_page: false,
        }
    }

    const STD: [&str; 6] = [
        "Courier",
        "Helvetica",
        "Helvetica-Bold",
        "Times-Roman",
        "Times-Bold",
        "Times-Italic",
    ];

    fn pick(run: &str) -> &'static str {
        let all: Vec<_> = STD.iter().map(|f| add(f)).collect();
        STD[nearest(run, &all).expect("non-empty")]
    }

    #[test]
    fn family_weight_and_slant_are_matched() {
        assert_eq!(pick("ABCDEF+ArialMT"), "Helvetica");
        assert_eq!(pick("ABCDEF+Arial-BoldMT"), "Helvetica-Bold");
        assert_eq!(pick("TimesNewRomanPS-ItalicMT"), "Times-Italic");
        assert_eq!(pick("ABCDEF+Georgia-Bold"), "Times-Bold");
        assert_eq!(pick("ConsolasMono"), "Courier");
        assert_eq!(pick("SUBSET+DemoFace"), "Helvetica");
    }

    #[test]
    fn a_face_on_the_page_wins_a_tie_and_empty_is_none() {
        let both = [
            add("Helvetica"),
            Candidate {
                base_font: "ArialMT",
                on_page: true,
            },
        ];
        assert_eq!(nearest("ABCDEF+ArialMT", &both), Some(1));
        assert_eq!(nearest("ArialMT", &[]), None);
    }
}
