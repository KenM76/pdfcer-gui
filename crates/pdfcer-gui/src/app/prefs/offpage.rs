//! **Whether the canvas grows to show what sits off the sheet — remembered
//! separately for each ribbon mode.**
//!
//! This module owns one preference and the whole of its reasoning: the type,
//! the per-mode default, the file keys, the parser and the writer. It follows
//! [`printing`](super::printing)'s precedent — *"the parser and the writer are
//! two spellings of one vocabulary and must not live in different files"* —
//! rather than adding a fourth family of arms to `prefs::file`.
//!
//! # What the preference is about
//!
//! A CAD export frequently carries marks outside its own `/MediaBox`. When
//! pdfcer is allowed to show them, the canvas grows a band of pasteboard wide
//! enough to hold them, and that band is not free: it lengthens every scroll,
//! and in a continuous layout it opens a visible grey gap between one sheet
//! and the next exactly where the off-sheet material lives.
//!
//! So the operator's answer is not *"can pdfcer do this"* — it always can —
//! but *"is it worth the band right now"*, and that depends entirely on what
//! they are doing. Reading a drawing, it is not: the page is the subject and
//! the band is in the way. Reviewing or editing one, it is: a title block
//! dragged off the sheet is a defect you must be able to see and grab.
//!
//! # ★★★ Why the answer is stored per MODE rather than once
//!
//! The operator's request, 2026-09-11, verbatim:
//!
//! > *"by default, read doesn't show off page items, review and edit do show
//! > off page items. these settings can be changed by the user and their
//! > preference is remembered for each read review edit modes."*
//!
//! That is three preferences wearing one name, and the reason it has to be is
//! that **the modes disagree about what the document is for**. A single global
//! flag would force the operator to re-answer the question every time they
//! switched modes, which is the same defect as not remembering it at all —
//! worse, because it would look like the toggle was forgetting.
//!
//! The consequence worth stating plainly: **switching mode can change the page
//! layout**, because leaving Edit for Read removes the band. That is intended.
//! It is also why the re-seed happens at exactly one place (the mode-change
//! site in `crate::app::surfaces`), so the layout can never be one mode's
//! answer while the ribbon shows another's.
//!
//! # What is NOT stored here
//!
//! Nothing about the document. Two open drawings can disagree about off-page
//! display, because the flag lives in [`crate::viewer::ViewState`] alongside
//! zoom and the overlays; this module holds only the *opening* answer and the
//! operator's last word per mode. A per-document memory would be a different
//! feature and would fight this one — see the mode-change note above.

use std::collections::BTreeMap;

use super::printing::KeyOutcome;
use crate::app::actions::ViewChrome;
use crate::app::state::Status;

/// The prefix every key in this family shares: `off_page.<mode>`.
///
/// ★ It is a **prefix**, not three fixed keys, and that is deliberate. Ribbon
/// modes come from the shell manifest, which an operator may customize (see
/// `crate::shell::manifest`), so this family cannot be closed over the three
/// modes that ship. A build whose manifest grows a fourth mode remembers that
/// mode's answer with no change here and no new key list to keep in step.
// ui-text-exempt: a file KEY prefix, written into preferences.txt and parsed
// back out of it. Never displayed.
const PREFIX: &str = "off_page.";

/// The ribbon mode whose default is **off** — the only one.
///
/// Spelled as a constant rather than inline in [`default_for_mode`] because it
/// is the one place this module knows a mode's *name*, and a reader looking
/// for "where does Read get special-cased" should find exactly one answer.
// ui-text-exempt: a MODE id from the shell manifest, compared against, never
// displayed. The label an operator sees comes from `crate::text`.
const READING_MODE: &str = "read";

/// Off-page display, remembered per ribbon mode.
///
/// # Why a map of the answers GIVEN rather than three `bool` fields
///
/// Three fields would be smaller, and would also be a closed set: a manifest
/// with a fourth mode could not be remembered, and — more importantly — three
/// fields cannot distinguish *"the operator chose `false` for Read"* from
/// *"nobody has said anything about Read yet"*. Those are the same value and
/// different facts. Keeping only the answers actually given means:
///
/// * a fresh profile writes **no** `off_page.*` lines at all, so the file does
///   not fill up with keys nobody set;
/// * [`default_for_mode`] remains the single source of the shipped behaviour,
///   free to change in a later build without a stale copy sitting in every
///   operator's preferences file contradicting it;
/// * the round trip is exact — an empty map writes nothing and reads back
///   empty, which is what `every_preference_round_trips_through_the_file`
///   requires of every member of [`Prefs`](super::Prefs).
///
/// [`BTreeMap`] and not [`HashMap`](std::collections::HashMap) for the writer's
/// sake: the file must be byte-stable across runs or a diff of two preference
/// files is unreadable noise.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OffPagePrefs {
    /// Mode id → the operator's answer. Absent means *never answered*.
    answers: BTreeMap<String, bool>,
}

impl OffPagePrefs {
    /// What this build ships for `mode`, before any operator answer.
    ///
    /// **Read is off; everything else is on**, including a mode this build has
    /// never heard of. The fallback direction is chosen so that an unfamiliar
    /// mode behaves like Edit rather than like Read: showing material that is
    /// there costs a band of pasteboard, whereas hiding it costs the operator
    /// the knowledge that it exists at all. Between a cosmetic cost and a
    /// silent omission, take the cosmetic one — the same posture
    /// `PageDisplay::default_for_mode` takes on its own unknown mode.
    #[must_use]
    pub fn default_for_mode(mode: &str) -> bool {
        mode != READING_MODE
    }

    /// The answer to use for `mode` — the operator's, or this build's.
    #[must_use]
    pub fn for_mode(&self, mode: &str) -> bool {
        self.answers
            .get(mode)
            .copied()
            .unwrap_or_else(|| Self::default_for_mode(mode))
    }

    /// Remember `on` as the answer for `mode`.
    ///
    /// ★ The answer is stored **even when it equals the shipped default**, and
    /// that is not redundancy. An operator who turns off-page display off in
    /// Review has expressed an intent about Review; if a later build changed
    /// its mind about Review's default, the operator's own answer must win
    /// over the new default rather than be indistinguishable from it.
    pub fn set(&mut self, mode: &str, on: bool) {
        self.answers.insert(mode.to_owned(), on);
    }
}

// ---------------------------------------------------------------------------
// The file format for this group — the parser and the writer, together
// ---------------------------------------------------------------------------

/// Read one `key = value` line into [`OffPagePrefs`], if it belongs here.
///
/// Returns [`KeyOutcome`] — borrowed from the print group rather than
/// re-declared, because the caller's dispatch chain needs one vocabulary and a
/// second three-variant enum meaning the same three things is how the two come
/// to disagree about what `NotMine` obliges the caller to do.
///
/// `value` arrives already trimmed, as `prefs::file` trims both halves before
/// it dispatches.
///
/// ⚠ **An empty mode (`off_page. = true`) is `BadValue`, not `NotMine`.** It is
/// unmistakably one of ours — it carries the prefix — and reporting it as an
/// unknown key would tell the operator to check the spelling of a key they
/// spelled correctly, which is the exact confusion [`KeyOutcome`]'s three
/// variants exist to prevent.
pub(super) fn parse_key(prefs: &mut OffPagePrefs, key: &str, value: &str) -> KeyOutcome {
    let Some(mode) = key.strip_prefix(PREFIX) else {
        return KeyOutcome::NotMine;
    };
    if mode.is_empty() {
        return KeyOutcome::BadValue;
    }
    match super::opening::bool_from_key(value) {
        Some(on) => {
            prefs.set(mode, on);
            KeyOutcome::Accepted
        }
        None => KeyOutcome::BadValue,
    }
}

/// Write this group's commented block into the file.
///
/// Called once by `Prefs::write_to_string`. The comment is written **always**,
/// even when no answer has been given, because the file is meant to be opened
/// in a text editor and a preference nobody can discover is a preference
/// nobody has. The key lines below it are written only for answers the
/// operator actually gave — see [`OffPagePrefs`] on why.
pub(super) fn write_block(prefs: &OffPagePrefs, out: &mut String) {
    out.push_str(
        "\n\
         # Off-page display, one answer per ribbon mode:\n\
         #   off_page.read = true | false\n\
         #   off_page.review = true | false\n\
         #   off_page.edit = true | false\n\
         #\n\
         # Some drawings carry marks outside the sheet itself -- a title block\n\
         # dragged off the page, a detail parked in the margin. When this is\n\
         # on, pdfcer draws them and lets you click them, and the canvas grows\n\
         # a band around the sheet wide enough to hold them. When it is off,\n\
         # the page is shown on its own and there is no band and no gap\n\
         # between pages.\n\
         #\n\
         # Unset means: off in Read, on in Review and Edit. A line appears\n\
         # here only for a mode where you have used View > Off-Page Content\n\
         # yourself, and that answer then wins for that mode from then on.\n",
    );
    for (mode, on) in &prefs.answers {
        // ui-text-exempt: a file KEY, written into preferences.txt and parsed
        // back out of it. Never displayed.
        out.push_str(PREFIX);
        out.push_str(mode);
        // ui-text-exempt: the file's key/value separator. Never displayed.
        out.push_str(" = ");
        out.push_str(super::opening::bool_key(*on));
        out.push('\n');
    }
}

// ---------------------------------------------------------------------------
// The write side — what happens when the operator uses the toggle
// ---------------------------------------------------------------------------

/// **Remember the operator's answer to `View ▸ Off-Page Content`**, for the
/// ribbon mode they are in, and put it on disk.
///
/// Called from the single `Action::ToggleViewChrome` arm in
/// [`crate::app::actions`], for *every* chrome toggle, and it returns
/// immediately for the five that have no memory. That shape is deliberate and
/// it is worth defending, because the obvious alternative — an `if chrome ==
/// OffPage` at the call site — looks tidier and is worse:
///
/// * **The knowledge stays in one module.** That off-page is the one toggle
///   with a remembered answer, that the answer is keyed by mode, and that a
///   mode-less shell has nothing to key it by are all facts about *this*
///   preference. A caller that had to know the first of them would be a second
///   place to update when a second toggle grows a memory.
/// * **`apply.rs` has fourteen lines of R2 headroom** and an action arm is the
///   wrong place to spend them on a preference's business rules.
///
/// # What it writes, and why immediately
///
/// One `prefs.save()` per click, exactly as `view.smart_select` and the
/// find-zoom toggle do, and for the reason stated there: **one discrete
/// operator decision is one write**. Deferring to shutdown would lose the
/// answer to a crash or a power cut, and the operator's request was that the
/// preference be *remembered* — a memory that survives only an orderly exit is
/// not what anybody means by that.
///
/// A failed write is swallowed. Preferences are a convenience and a modal in
/// front of somebody who just flipped a view switch would be a worse defect
/// than the lost line; the same judgement every other `prefs.save()` call in
/// this crate makes.
///
/// # `mode` is an [`Option`] because the shell's is
///
/// [`crate::shell::ribbon`] answers `None` before a manifest has been applied.
/// There is no sensible key for "no mode", and inventing one (`""`, or
/// defaulting to `read`) would write an answer against a mode the operator was
/// never in — which the next real mode would then inherit. Doing nothing is
/// the honest response: the toggle still works for the session, it simply has
/// nowhere to be remembered.
pub fn remember(chrome: ViewChrome, on: bool, prefs: &mut super::Prefs, mode: Option<&str>) {
    if chrome != ViewChrome::OffPage {
        return;
    }
    let Some(mode) = mode else {
        return;
    };
    prefs.off_page.set(mode, on);
    let _ = prefs.save();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("off-page-remembered mode={mode} on={on}")
    });
}

/// **Put the mode's answer into every open document's view state.**
///
/// Called from the one place a ribbon mode change is observed
/// (`crate::app::surfaces`, beside `on_mode_capabilities_changed`). One site,
/// stated as a requirement rather than an accident: a second would let the
/// canvas show one mode's answer while the ribbon shows another's, and that
/// state is indistinguishable from the toggle being broken.
///
/// # The consequence, stated plainly because it is intended
///
/// **Leaving Edit for Read can change the page layout.** Read's shipped answer
/// is off, so the band of pasteboard that held off-sheet material goes, and
/// with it the gap it opened between one sheet and the next. That is the
/// operator's request — *"when not showing the stuff that is off page there
/// shouldn't be a gap between pages where the stuff is"* — arriving at the
/// moment he asked for it. It is not a surprise to be softened; a mode change
/// is a deliberate gesture and this is the thing it was asked to do.
///
/// # Why EVERY document and not just the active one
///
/// The answer is a property of the mode, not of the document. A parked tab
/// that kept the outgoing mode's layout would spring to the incoming mode's
/// the instant it was activated, with no gesture in between to explain it —
/// the operator would have watched a document rearrange itself for nothing.
/// Writing them all now costs one bool per open file and makes activation
/// inert, which is what the operator already believes it is.
///
/// ★ `parked` is taken as a slice rather than the app, so this function cannot
/// reach anything else and the caller's three field borrows stay disjoint.
pub fn apply_mode(prefs: &super::Prefs, mode: &str, status: &mut Status, parked: &mut [Status]) {
    let on = prefs.off_page.for_mode(mode);
    for status in std::iter::once(status).chain(parked.iter_mut()) {
        if let Status::Open(doc) = status {
            doc.view.off_page = on;
        }
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("off-page-mode mode={mode} on={on}")
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The operator's request, restated as an assertion.
    ///
    /// Read off, Review and Edit on. This is the whole of requirement (c) and
    /// it is asserted against the *names the manifest uses*, because those ids
    /// are what reaches [`OffPagePrefs::default_for_mode`] at runtime.
    #[test]
    fn read_ships_off_and_the_working_modes_ship_on() {
        assert!(!OffPagePrefs::default_for_mode("read"));
        assert!(OffPagePrefs::default_for_mode("review"));
        assert!(OffPagePrefs::default_for_mode("edit"));
    }

    /// A mode this build has never heard of behaves like Edit, not like Read.
    ///
    /// See [`OffPagePrefs::default_for_mode`] for the argument: a cosmetic
    /// cost beats a silent omission.
    #[test]
    fn an_unknown_mode_shows_off_page_content() {
        assert!(OffPagePrefs::default_for_mode("redline-2027"));
    }

    /// The three modes are three independent memories.
    ///
    /// The failure this catches is the one a single global flag would have:
    /// answering in one mode changing the answer in another.
    #[test]
    fn an_answer_in_one_mode_does_not_move_another() {
        let mut prefs = OffPagePrefs::default();
        prefs.set("edit", false);
        assert!(!prefs.for_mode("edit"));
        assert!(prefs.for_mode("review"), "review followed edit");
        assert!(!prefs.for_mode("read"), "read followed edit");
    }

    /// An answer equal to the shipped default is still stored.
    ///
    /// See [`OffPagePrefs::set`]: *chose the default* and *never answered* are
    /// the same value and different facts, and only the stored one survives a
    /// later build changing its mind.
    #[test]
    fn choosing_the_default_is_still_an_answer() {
        let mut prefs = OffPagePrefs::default();
        prefs.set("read", false);
        assert!(
            prefs
                .write_to_string_fragment()
                .contains("off_page.read = false"),
            "an explicit answer was not written"
        );
    }

    /// A fresh profile writes no key lines at all — only the comment.
    #[test]
    fn an_unanswered_profile_writes_no_keys() {
        let text = OffPagePrefs::default().write_to_string_fragment();
        assert!(
            !text.contains(&format!("\n{PREFIX}")),
            "a fresh profile wrote a key line: {text}"
        );
    }

    /// Every answer survives the file, and nothing else is invented.
    #[test]
    fn answers_round_trip_through_the_block() {
        let mut original = OffPagePrefs::default();
        original.set("read", true);
        original.set("edit", false);

        let mut read_back = OffPagePrefs::default();
        for line in original.write_to_string_fragment().lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if line.starts_with('#') {
                continue;
            }
            assert_eq!(
                parse_key(&mut read_back, key.trim(), value.trim()),
                KeyOutcome::Accepted
            );
        }
        assert_eq!(read_back, original);
    }

    /// A prefixed key with an unreadable value is reported against ITS OWN key.
    ///
    /// `BadValue`, never `NotMine` — the distinction the caller turns into
    /// *"this value is wrong"* versus *"this key does not exist"*.
    #[test]
    fn a_bad_value_and_an_empty_mode_are_both_ours() {
        let mut prefs = OffPagePrefs::default();
        assert_eq!(
            parse_key(&mut prefs, "off_page.edit", "yes"),
            KeyOutcome::BadValue
        );
        assert_eq!(
            parse_key(&mut prefs, "off_page.", "true"),
            KeyOutcome::BadValue
        );
        assert_eq!(
            parse_key(&mut prefs, "show_rulers", "true"),
            KeyOutcome::NotMine
        );
        assert_eq!(
            prefs,
            OffPagePrefs::default(),
            "a refused line stored something"
        );
    }

    impl OffPagePrefs {
        /// This group's block on its own — a test helper, so the assertions
        /// above read the writer's real output rather than a re-implementation
        /// of it.
        fn write_to_string_fragment(&self) -> String {
            let mut out = String::new();
            write_block(self, &mut out);
            out
        }
    }
}
