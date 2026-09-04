//! Catalogue-wide properties: exhaustiveness, key spelling, and the closed
//! set of shared assets.
//!
//! Split out of `catalog/mod.rs` under R2. These are properties of the
//! CONTRACT rather than of any one variant, which is why they live together
//! and why several of them assert a shape ("exactly one pair shares an
//! asset") rather than a count ("93 icons") — a count rots, and this project
//! has spent several corrections proving it.

use super::Icon;
use std::collections::HashSet;

/// ★ [`Icon::ALL`] must really be all of them.
///
/// Everything catalogue-wide — "every asset parses", "every asset
/// rasterizes to something visible", "redaction is the only filled one"
/// — iterates `ALL`. A variant left out of it is therefore not merely
/// untested: it is *silently* untested, and a broken asset behind it
/// ships green.
///
/// There is no reflection in Rust to count enum variants, so this checks
/// the two things that would actually go wrong: a duplicate entry (a
/// copy-paste that hid the variant it was meant to add) and a count that
/// no longer matches the number of distinct keys.
#[test]
fn all_is_exhaustive_and_free_of_duplicates() {
    let unique: HashSet<Icon> = Icon::ALL.iter().copied().collect();
    assert_eq!(
        unique.len(),
        Icon::ALL.len(),
        "Icon::ALL contains a duplicate variant"
    );
    // 47 until 2026-08-14, when the pass that filled the ribbon's
    // remaining text buttons added 25 — and 76 until later the same day,
    // when the three unblocked Phase 6 markup kinds added `shape-polyline`,
    // `shape-polygon` and `shape-ink`. If this fails, the fix is not to
    // edit the number: it is to check that the variant you added really is
    // in `ALL`, and only then to update this count.
    //
    // ★ This comment used to also say "and update the two prose figures
    // that quote it". That instruction was followed exactly once. On
    // 2026-08-21 the count here was 86 while both of those paragraphs
    // still said 82 — the drift the instruction existed to prevent,
    // committed by the instruction's own readers, twice.
    //
    // So the paragraphs no longer carry a number. `from_key` now says
    // "one comparison per catalogue entry" and `super::cache` says "one
    // entry per icon per weight", both of which are true at every size
    // the set will ever be. THIS assertion is the only figure left, and it
    // is in a test, where drift fails the build instead of misinforming a
    // reader. Prefer that shape for any future count.
    assert_eq!(
        Icon::ALL.len(),
        93,
        "the catalogue changed size: add the new variant to Icon::ALL and update this count"
    );
}

/// Every key is unique. Two icons answering to one key would make
/// [`Icon::from_key`] return whichever came first in `ALL`, which is a
/// silently-wrong glyph rather than a missing one — the worse failure.
#[test]
fn every_name_is_distinct() {
    let mut seen: HashSet<&str> = HashSet::new();
    for &icon in Icon::ALL {
        assert!(
            seen.insert(icon.name()),
            "duplicate icon key '{}'",
            icon.name()
        );
    }
}

/// ★ The key vocabulary has exactly one definition.
///
/// [`Icon::from_key`] is documented as the inverse of [`Icon::name`].
/// This is what keeps that true if `from_key` is ever rewritten as a
/// `match` or a map for speed.
#[test]
fn every_name_round_trips_through_from_key() {
    for &icon in Icon::ALL {
        assert_eq!(
            Icon::from_key(icon.name()),
            Some(icon),
            "'{}' did not round-trip",
            icon.name()
        );
    }
}

/// An unknown key resolves to nothing rather than to something plausible.
///
/// The whole missing-icon story downstream ([`super::super::paint`])
/// depends on this returning `None` instead of guessing at a nearest
/// match: a fuzzy resolver would draw the *wrong* glyph for a typo,
/// which is undetectable, where `None` is drawn as a visible mark and
/// traced.
#[test]
fn an_unknown_key_resolves_to_nothing() {
    assert_eq!(Icon::from_key("no-such-icon"), None);
    assert_eq!(Icon::from_key(""), None);
    // Case and separator variants are NOT accepted: the vocabulary is
    // kebab-case, exactly, and a near-miss should be reported rather
    // than silently repaired.
    assert_eq!(Icon::from_key("Open"), None);
    assert_eq!(Icon::from_key("fit_page"), None);
}

/// Keys are kebab-case with no surprises, because they appear verbatim
/// in command definitions that a human types by hand.
#[test]
fn keys_are_lowercase_kebab_case() {
    for &icon in Icon::ALL {
        let name = icon.name();
        assert!(!name.is_empty(), "{icon:?} has an empty key");
        assert!(
            name.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
            "icon key '{name}' is not lowercase kebab-case"
        );
    }
}

/// The roles that deliberately share one asset still share it, and nothing
/// else accidentally does.
///
/// Asset sharing is a real decision (one glyph, two places it appears,
/// never simultaneously) but an *accidental* share means two controls that
/// should be distinguishable are not — which reads to an operator as a
/// wiring bug in whichever control they clicked second.
///
/// # ★★ The second pair, added 2026-08-27, and why the test's shape is what
/// made it a decision
///
/// This asserted an **exact list** of one pair, so adding a second could
/// not be done quietly — which is the whole point of writing it this way
/// rather than as "sharing is allowed". The argument had to be made:
///
/// `import-form-data` shares `insert-pages`' upload arrow. Both are
/// *"something comes in from a file"*, they are on **different tabs**
/// (File ▸ Export and Pages) so they are never drawn together, and the only
/// alternative was no icon at all — which would leave one control in a
/// two-control group bare, and `super`'s own header records what that looks
/// like: *"47 named and 41 bare with no rule behind which was which, so a
/// band drew pictures and words side by side and the ribbon read as
/// half-finished because it was."*
///
/// ★ Drawing new art was never the option. `icons/assets/PROVENANCE.md`
/// declares that directory the **operator's own work**, which is what
/// exempts it from `check-shipped-assets`, and a machine-drawn SVG would
/// make that note false.
///
/// ★★ What was refused: keying it to `insert-pages` itself. A shared *key*
/// says *two controls about one thing*, and inserting pages and importing
/// form data have nothing in common but a direction — a pages-named key on
/// a form command is the near-miss reuse this catalog's refusal table
/// exists to prevent.
/// Every pair of icons permitted to share one asset, with the argument for
/// each in [`only_the_documented_assets_are_shared`]'s doc comment.
const SHARED_PAIRS: &[&[&str]] = &[
    &["font-folders", "open"],
    &["import-form-data", "insert-pages"],
];

#[test]
fn only_the_documented_assets_are_shared() {
    let mut by_source: std::collections::HashMap<&str, Vec<Icon>> =
        std::collections::HashMap::new();
    for &icon in Icon::ALL {
        by_source.entry(icon.source()).or_default().push(icon);
    }
    for (_, icons) in by_source {
        if icons.len() > 1 {
            let mut names: Vec<&str> = icons.iter().map(|i| i.name()).collect();
            names.sort_unstable();
            assert!(
                SHARED_PAIRS.contains(&names.as_slice()),
                "an unexpected pair of icons shares one asset: {names:?}. Sharing is \
                 permitted and is a DECISION — add the pair to `SHARED_PAIRS` with \
                 the argument for it, in this test's own doc comment"
            );
        }
    }
}

/// Every variant has non-empty art. A `source()` arm wired to the wrong
/// (or an empty) constant would otherwise only show up as a blank
/// button.
#[test]
fn every_icon_has_source_text() {
    for &icon in Icon::ALL {
        let src = icon.source();
        assert!(
            src.contains("<svg"),
            "icon '{}' has no <svg> root in its source",
            icon.name()
        );
    }
}
