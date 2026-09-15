//! # `shell::commands::reach::register` — the allow-list, and only the allow-list
//!
//! **The DATA half of [`super`]**, kept apart from the check that reads it.
//!
//! ## The seam is real, and it is the one the file kept re-discovering
//!
//! `reach.rs` does two things that change for entirely different reasons:
//!
//! | half | what it is | changes when |
//! |---|---|---|
//! | **this file** | the register — every registered command with no dispatch arm, and *why* | a command is wired, deferred, or its reason expires |
//! | `mod.rs` | the CHECK — a `syn` parse of the dispatcher's `match`, the guard evaluation, and the tests | the dispatcher's shape changes, or the check gets sharper |
//!
//! The second is machinery and is nearly static. The first is a **living
//! document** that grows a paragraph every time somebody explains why a
//! control is inert and shrinks by an entry every time somebody fixes one —
//! and every one of those paragraphs is prose, so it is the half that pushes
//! the line count.
//!
//! ⚠ **The pressure that produces is the reason for the split, and trimming a
//! reason to get back under R2's ceiling is the worst available response**,
//! because the reason is the entry's whole value. R2 says so in as many words:
//! *"when a file approaches the limit, that is the signal to find the seam,
//! not to raise the limit."*
//!
//! ## The counts live with the data
//!
//! `super::tests::the_p3_tension_is_counted` pins both figures and reads them
//! from here through the ordinary path, so the number quoted in `super`'s
//! header and the length of the list below move together or fail.

// ===========================================================================
// THE ALLOW-LIST
// ===========================================================================

/// **Registered, deliberately without a dispatch arm, and why.**
///
/// ★ This list is the deliverable, not the leftovers. Every entry is a control
/// an operator can press today that does nothing, and writing the reason down
/// is what forces somebody to say *why it is drawn at all*.
///
/// # How this differs from `super::super::manifest::PLANNED`
///
/// `PLANNED` names commands that are **not registered** — ids `RIBBON_IA.md`
/// mentions that this build does not have, so nothing draws them and nothing
/// can invoke them. These are the opposite and worse state: **registered, drawn
/// and inert.** The two lists are disjoint by construction, because everything
/// here is in the registry and nothing in `PLANNED` is; that is asserted by
/// [`tests::no_scaffolded_command_is_also_planned`].
///
/// # What an entry has to carry
///
/// The **reason**, not the name. Where a reason already exists in the codebase
/// — a doc comment at the registration, a table in `app::dispatch` — the entry
/// cites that rather than inventing a second wording, because two wordings of
/// one decision drift apart and neither one announces that it has.
///
/// # ★ Several of these should probably not be drawn yet, and this list is
/// where that becomes visible
///
/// `RIBBON_IA.md` P3 says an unavailable capability renders **nothing**, and a
/// control that is drawn, enabled, pressable and inert breaches it more
/// severely than a greyed one does — a greyed control at least explains itself
/// on hover. Entries whose honest answer is *"this should not be on the ribbon
/// yet"* are marked **★ P3** in their reason. Removing them is a taxonomy
/// decision and is the operator's, not this module's; what this module can do
/// is make the count impossible to lose track of, which
/// [`tests::the_p3_tension_is_counted`] does.
///
/// # Ordering
///
/// Registry order, so this list and [`super::catalog::all`] read side by side.
/// [`tests::no_scaffolded_entry_is_stale`] asserts every entry is registered,
/// is genuinely unreachable, and carries a reason rather than a restatement of
/// its own id.
///
/// # ★★★ THE FOUR WAYS AN ENTRY HERE GOES FALSE, NONE OF WHICH A TEST SEES
///
/// `no_scaffolded_entry_is_stale` can prove an entry is registered, has no
/// dispatch arm, and that its reason is long enough. **It cannot prove the
/// reason is TRUE**, and every one of these has been a shipped entry:
///
/// 1. **The blocker cleared.** The engine grew the verb, or this shell did,
///    and nothing re-reads a paragraph in a list.
/// 2. **A citation of a citation.** The entry quotes a `FEATURES.md` row and
///    the ROW is the thing that was stale. One layer of indirection is enough
///    to make a false claim read as a sourced one.
/// 3. **A missing HOST rather than a missing capability.** *"The list needs
///    the pane it lives in"* goes stale the moment any other host will do —
///    silently, because nothing changed to make it stale. ⚠ **A blocker naming
///    a missing host is always the weak kind; name the missing capability or
///    do not write the entry.**
/// 4. **A reason REWRITTEN after its predecessor cleared.** The replacement
///    gets none of the scrutiny the original had and lands in a list nothing
///    re-reads. One such rewrite had the destination of a merge backwards
///    against a taxonomy stated two files away.
///
/// ⇒ The standing rule this bought, and it is cheap: **when you touch this
/// list for any purpose, re-derive the reason of the entry beside the one you
/// came for.** A scheduled audit of eleven entries found six wrong; an
/// opportunistic reader finds them one at a time, which is often enough.
///
/// ⇒ And the real replacement is not prose at all: `tools/ui-verify` presses
/// every registered id and fails on any `command-unimplemented` trace
/// line. **No paragraph can satisfy that.**
///
/// # ★★ An entry is DELETED, never reworded
///
/// When the work lands, the entry goes. Rewording it preserves the shape of a
/// justification for a control that now works, and a reader cannot tell a
/// live reason from a preserved one.
///
/// ★ A related trap: an entry that says *"no recorded reason anywhere"* is
/// **not** a neutral placeholder, and must not be written. It reads as
/// *"somebody looked and found nothing"*, which is indistinguishable from
/// *"somebody deferred this deliberately and forgot to say why"* — and the
/// first invites a re-derivation while the second discourages one. ⚠ An entry
/// with no reason is a reason to unregister the command, not to list it.
pub(crate) const SCAFFOLDED: &[(&str, &str)] = &[
    // Empty. Every id that stood here has been wired, unregistered, or
    // deleted; the reasons that were transferable are in this constant's doc
    // comment, and the rest were the entries themselves.
];

/// **The mirror defect: a literal arm that no token can reach, and why each is
/// tolerated.**
///
/// ★ **Empty, and that is the entry.** The list is kept rather than deleted
/// because an empty allow-list is still a gate: a dead arm cannot be added
/// quietly, it has to be written here with a reason, and
/// [`tests::the_p3_tension_is_counted`] pins the length at zero so shortening
/// or lengthening it is a visible act.
///
/// ## ★★ The shape it guards against, and how such an arm hides
///
/// An arm whose id is in NO registry is reachable by nothing:
/// `"view.zoom_in" => actions.push(Action::ZoomIn)` compiles, reads as working
/// code, and can never run, because dispatch begins at a registered command's
/// token and there is no token to begin at. Four such arms existed at once —
/// `view.zoom_in`, `view.zoom_out`, `view.next_page`, `view.prev_page` — with
/// no catalog entry, no manifest item, no [`crate::text::commands`] copy and
/// no `RIBBON_IA.md` row between them.
///
/// ⚠ **The check above cannot see them, and that is the point of this second
/// list.** It asks whether a registered id has an arm; this asks whether an arm
/// has a registered id. `app::dispatch`'s `format.delete` arm states the rule:
/// *"adding an arm for one would be an arm no token can ever reach — dead code
/// wearing a design pattern, which is what the no-placeholders invariant
/// forbids."*
///
/// Those four arms are gone, each verb having a live route that is not the
/// dispatcher:
///
/// | verb | keyboard | status bar |
/// |---|---|---|
/// | `Action::ZoomIn` | `app::keyboard`, `Ctrl` `+` | `status::zoom_group`'s `+` |
/// | `Action::ZoomOut` | `app::keyboard`, `Ctrl` `-` | `status::zoom_group`'s `−` |
/// | `Action::NextPage` | `app::keyboard`, `PageDown` | `status::page_box`'s `▶` |
/// | `Action::PrevPage` | `app::keyboard`, `PageUp` | `status::page_box`'s `◀` |
///
/// So deleting them removes **duplicate entrances, not behaviour**, and
/// `RIBBON_IA.md` §6 says the entrances that remain are the specified ones:
/// *"Find toggle, actual size, fit width, fit page, zoom −/%/+, page ◀ n/N ▶
/// … These are the controls a user touches constantly; they belong where they
/// never disappear behind a tab change."* Both pairs are status-bar verbs by
/// specification, and the arms were the redundant half.
///
/// ## The other answer, and what it would take
///
/// Registering the four instead is a **ribbon** decision and the operator's,
/// and it is recorded here so it can be made rather than inherited. It would be
/// four `catalog::all` entries in the `view.` token block, four
/// [`crate::text::commands`] pairs, a `RIBBON_IA.md` row each and a place to
/// draw them — View ▸ Zoom for the step pair, which currently draws Actual size,
/// Fit page, Fit width, Region and Selection and conspicuously not these two.
/// Page navigation would need a group that does not exist, against §6's
/// deliberate placement of it on the bar. The arms would then come back, and
/// each would be one line pushing the `Action` its two live routes already push.
///
/// ## The quieter failure, and why it is worth a list of its own
///
/// An inert control at least *looks* wrong when an operator presses it. A dead
/// arm reads as working code: it is maintained, reviewed and reasoned about by
/// everyone who passes it, and nothing in the suite touches it at all.
pub(crate) const UNREACHED_ARMS: &[(&str, &str)] = &[];
