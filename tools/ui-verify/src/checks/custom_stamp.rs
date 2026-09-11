//! `custom_stamp_reaches_the_page` — **one of the operator's OWN stamps is
//! picked out of the gallery with the pointer, dragged onto a drawing, and the
//! artwork arrives.**
//!
//! # The request this closes
//!
//! `OPERATOR_REQUESTS.md` **O172**, 2026-09-10:
//!
//! > *"add our own custom stamps and use them, preferrably exactly the same way
//! > acrobat does"*
//!
//! *Use them* is the half this file measures. Reading his stamps folder is
//! covered by unit tests over `stamps::library`; **placing** one is a chain
//! nine hops long, and the standing lesson of this project is that unit tests
//! cannot see the chain in front of the verb:
//!
//! | hop | its unit test | what that test cannot see |
//! |---|---|---|
//! | the folder is scanned when the dialog opens | `only_the_stamp_kind_scans_the_stamps_folder` | it is vacuous on a machine with no stamps |
//! | the gallery draws his half | `the_custom_half_appears_only_when_there_is_something_in_it` | whether it is reachable with a pointer |
//! | a click selects one | `exactly_one_of_the_two_galleries_holds_the_selection` | it calls `select_custom` directly |
//! | the selection reaches the action | `the_operators_own_stamp_reaches_the_commit_action` | it sets `accept_requested` by hand |
//! | `apply` forks on `custom: Some(..)` | the `apply` arm tests | they build the action by hand |
//! | the collection is reopened off disk | — | nothing in-process opens a real file |
//! | the engine imports the artwork | the engine's own tests | they never went through a dialog |
//! | the mark is turned upright on a rotated page | — | |
//! | four facts become sentences | `customstamp::tests` | they hand-build the outcome, because `PlacedArtwork` is `#[non_exhaustive]` |
//!
//! Every one of those passes on a build whose gallery is drawn below the
//! window's bottom edge, or whose radio never fires, or whose fork reads
//! `stamp` first and puts `Approved` on the sheet instead of his signature.
//!
//! # ★★★ The collection is PLANTED, and that is the whole reason this check
//! can fail
//!
//! The obvious version of this check reads the operator's own Acrobat stamps
//! folder and skips when it is empty. That version is worthless on any machine
//! but his — and, worse, it goes **vacuous on his own machine the day he
//! deletes a stamp**, silently, behind a SKIP nobody reads. A SKIP is not red.
//!
//! So this check builds a stamps folder of its own under `CheckContext::out_dir`
//! and points the application at it by overriding **`%APPDATA%`** for the
//! launched process. `stamps::folder::user_stamps_dir` resolves
//! `%APPDATA%\Adobe\Acrobat\<generation>\Stamps` and reads the variable from
//! the environment on every call, so a child process with a redirected
//! `APPDATA` sees exactly the collection this file put there and nothing else.
//!
//! ⚠ **What else moves when `APPDATA` moves**, measured rather than assumed:
//!
//! * `stamps::folder::user_stamps_dir` — the point of the exercise.
//! * `trust::…` — Acrobat's address book, which becomes empty. Nothing this
//!   check drives reads it.
//! * `pdfcer_core::settings::resolve_store`'s **fallback** only. Its first
//!   choice is `<directory of the running executable>/userdata`, which exists
//!   and is writable for every way this suite launches a binary, so the
//!   preferences, the layout and the recent-file list do **not** move. That is
//!   a property of the engine's resolver, and it is named here because if it
//!   ever changes this check silently starts running against a blank profile
//!   and its failures stop meaning what they say.
//!
//! ⇒ The redirect is also why this check leaves **nothing** in the operator's
//! own Acrobat folder. It never writes there and never reads there.
//!
//! # The fixture, and the two things it was already built to prove
//!
//! [`COLLECTION`] is this repository's `fixtures/stamp-collection.pdf`, whose
//! `.PROVENANCE.py` states its properties. Two are load-bearing here and
//! neither was chosen for this check — they were already true:
//!
//! 1. **The name-tree order is not the page order.** §7.9.6 sorts a name tree
//!    lexicographically, and `#SRIssued` (`#` is 0x23) sorts before
//!    `SRApproved` (`S` is 0x53) while its page is last. So the **first** entry
//!    in the gallery is *Issued*, on source page **2**. A build that enumerated
//!    pages instead of tree entries would put *Approved* there, look entirely
//!    plausible, and mislabel every stamp the operator owns. Phase E asserts
//!    the ordinal-0 stamp by name *and* by source page for exactly that reason.
//! 2. **Exactly one of the three is dynamic**, and it is that same first entry.
//!    So pressing ordinal 0 also drives the dynamic disclosure — the sentence
//!    saying the date baked into the artwork will not recompute — without
//!    needing a second placement.
//!
//! And one property this check adds: the collection's pages are **200 × 60 pt**
//! and the box dragged here is [`BOX_PT`] **square**. `place_page_artwork` maps
//! the form `/BBox` onto `/Rect` with independent horizontal and vertical
//! factors (§12.5.5), so the scales come out ≈1.1 and ≈3.7 and the engine's
//! `distorted` flag — `(scale_x - scale_y).abs() > 1e-6` — is true **by
//! construction rather than by luck**. The stretch disclosure is therefore
//! guaranteed to be owed, which is what lets phase G assert that a sentence is
//! on the status bar rather than merely hope one is.
//!
//! # R8b rule 4 — what is asserted, and what is deliberately NOT
//!
//! The placed stamp renders **exactly as a saved-and-reopened one would**: no
//! badge, no tint, no dashed outline, nothing on the canvas saying *"this was
//! stretched"*. This check therefore asserts the disclosure **off-canvas**, by
//! the presence of the `status-group:edit-disclosure` region, and asserts
//! nothing at all about the pixels of the mark. An edit that added a
//! provisional style to the canvas would leave every assertion here green, and
//! that is correct: it would be a defect for a different check to catch.
//!
//! # Rule 15
//!
//! [`BOX_PT`] is a **pdf dimension** — a length in the CAD-exported page's own
//! coordinate space, used to aim the pointer. Nothing here authors a ce
//! dimension.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | plant `fixtures/stamp-collection.pdf` into a scratch `%APPDATA%` | the file is on disk before launch |
//! | B | launch with `APPDATA` redirected, Review mode, Markup tab, arm **Stamp** | `markup-tool tool=TextAnnot(..)` |
//! | C | drag a square box | `text-annot-open`, and `custom-stamp-library categories=1 stamps=3` |
//! | D | read the gallery's custom half | `text-annot.custom-stamps` and `…custom-stamps.0` declared |
//! | E | click ordinal 0 | `custom-stamp-chosen name=Issued page=2 dynamic=true` |
//! | F | press **Add** | `custom-stamp-requested`, then `custom-stamp-placed distorted=true` |
//! | G | read the answer | the funnel line carries TWO joined sentences, and the status bar declares its disclosure region |
//! | H | arm **Stamp** again, drag a second box elsewhere | `stamp-gallery-opens restored=custom remembered="custom:Site Review/Issued"` |
//! | I | press **Add** without touching the gallery | a SECOND `custom-stamp-requested name=Issued` -- the memory reached the page |
//!
//! ★ **Phases H and I have been seen to fail.** On their first driven run
//! `REMEMBERED_TOKEN` carried the quotes the shell writes, `trace::parse_fields`
//! had already stripped them, and the check refused -- printing the line it
//! refused on, which is why the mistake cost thirty seconds. A phase that has
//! only ever been seen green is indistinguishable from one that cannot go red;
//! these two have a red to their name and it was the check's fault, not the
//! application's.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, frame_of, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use std::path::{Path, PathBuf};

/// The collection planted for this check, relative to the workspace root.
const COLLECTION: &str = "fixtures/stamp-collection.pdf";

/// The name the planted copy is given inside the scratch stamps folder.
///
/// ★ Deliberately **not** the fixture's own stem. `library::read_collection`
/// falls back to the file stem only when the document has no `/Info` `/Title`,
/// and this fixture has one — *Site Review*. Giving the copy an unrelated name
/// means the category can only have come from the title, so a build that
/// stopped reading `/Title` fails phase C instead of passing it by accident. A
/// copy called `Site Review.pdf` would pass either way.
const PLANTED_AS: &str = "collection-under-test.pdf";

/// How many stamps the fixture's name tree holds, all three placeable.
const STAMPS: usize = 3;

/// The **first** gallery entry — see the header's property 1. Tree order, not
/// page order.
const FIRST_LABEL: &str = "Issued";

/// …and its page in the collection: the **last** one. This pair is the whole
/// assertion about tree order.
const FIRST_SOURCE_PAGE: usize = 2;

/// The side, in **pdf** points, of the square dragged for the stamp.
///
/// Square, against a 200 × 60 pt source page, so the placement is distorted by
/// construction. Matched to `stamp_size`'s magnitude for its reason too: large
/// enough that the gesture machine reads a drag rather than rounding it to a
/// click.
const BOX_PT: f64 = 220.0;

/// The dialog's body.
const BODY: &str = "dialog:text-annot";
/// Add. `dialogs::textannot::REGION_ACCEPT`.
const ACCEPT: &str = "text-annot.accept";
/// The enclosing region of the gallery's custom half.
/// `dialogs::textannot::REGION_CUSTOM_STAMPS`.
const CUSTOM: &str = "text-annot.custom-stamps";
/// The first stamp's radio, keyed on its ordinal.
const CUSTOM_0: &str = "text-annot.custom-stamps.0";
/// The status bar's *"About your last edit"* group.
/// `app::status::REGION_EDIT_DISCLOSURE`.
const DISCLOSURE: &str = "status-group:edit-disclosure";

/// The ribbon control that arms the stamp tool. A **toggle** — see
/// [`arm_stamp`].
const STAMP_ITEM: &str = "ribbon.item.markup.stamp";
/// The trace line naming the armed markup tool. Emitted on change only.
const TOOL_EVENT: &str = "markup-tool";
/// The prefix every text-annotation tool's name carries.
const TOOL_TEXT_ANNOT: &str = "TextAnnot";

/// `custom-stamp-library categories= stamps= unreadable= unplaceable= folder=`
const LIBRARY_EVENT: &str = "custom-stamp-library";
/// `custom-stamp-chosen name= page= dynamic=`
const CHOSEN_EVENT: &str = "custom-stamp-chosen";
/// `custom-stamp-requested name= src-page= page= rotate= dynamic=`
const REQUESTED_EVENT: &str = "custom-stamp-requested";
/// `custom-stamp-placed id= form= scale-x= scale-y= distorted= …`
const PLACED_EVENT: &str = "custom-stamp-placed";
/// `custom-stamp-refused file= reason=` — the collection would not open.
const REFUSED_EVENT: &str = "custom-stamp-refused";
/// The funnel's own line. Its first token is the `vector_edit` label, which is
/// why the module's measurement line is `custom-stamp-placed` and not this.
const FUNNEL_EVENT: &str = "place-custom-stamp";

/// How many sentences this placement owes: the stretch, and the dynamic
/// promise. Both are guaranteed by the fixture's shape — see the header.
const OWED_DISCLOSURES: usize = 2;

/// How far, in **pdf** points, the SECOND box of phase H is dragged from the
/// first.
///
/// ★ It has to miss the first stamp's rectangle. A drag that STARTS inside an
/// annotation that already exists is a different gesture -- the canvas reads it
/// as grabbing that object -- so the dialog would never open and phase H would
/// report a memory failure that never happened. Borrowed, with its reason, from
/// `stamp_dialog_reopen`, which learned it the hard way.
const SECOND_OFFSET_PT: f64 = 300.0;

/// `stamp-gallery-opens restored= remembered=` -- what the window did with the
/// memory it was handed. See `dialogs::textannot::restored_kind`.
const GALLERY_EVENT: &str = "stamp-gallery-opens";

/// The word the second opening must report.
const RESTORED_CUSTOM: &str = "custom";

/// The token the second opening must report.
///
/// ★ **The space in it is the assertion.** The shell QUOTES this value
/// because the category name holds a space, and `trace::parse_fields` tracks
/// quoting and hands the value back with the quotes stripped -- measured on
/// 2026-09-10, when this constant was first written with the quotes left in and
/// the check failed against a line whose value was exactly right. So the quoting
/// is doing its job precisely when this constant does NOT mention it: an
/// unquoted emission would arrive here truncated at `custom:Site`.
const REMEMBERED_TOKEN: &str = "custom:Site Review/Issued";

/// How many times a click is repeated before it is called unheard.
const CLICK_TRIES: usize = 4;

/// The fixture, located from this crate rather than from the working directory.
///
/// `tools/ui-verify/` → up two → the workspace root, exactly as
/// `crate::checks::rotated_text` locates its own.
fn collection() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(COLLECTION)
}

/// See the module documentation.
pub struct CustomStampReachesThePage;

impl Check for CustomStampReachesThePage {
    fn name(&self) -> &'static str {
        "custom_stamp_reaches_the_page"
    }

    fn defect(&self) -> &'static str {
        "one of the operator's own Acrobat stamps cannot be got onto a drawing — the gallery's \
         custom half is missing or unreachable, the click selects nothing, the commit places a \
         standard stamp instead, or the artwork arrives with nothing said about the stretch it \
         was given"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// **Build the scratch `%APPDATA%` and put the collection in it.**
///
/// Returns the directory to hand the child process as `APPDATA`.
///
/// The generation folder is called `DC` because that is the one the operator's
/// own machine has, and because `folder::pick_generation`'s first rule — *a
/// folder that already holds `Stamps` wins outright* — makes the choice
/// unambiguous the moment this function creates the `Stamps` directory. There
/// is exactly one candidate, so the shortest-name heuristic never runs and this
/// check is not secretly a test of it.
///
/// # Errors
///
/// If the fixture is missing, or the scratch tree cannot be written.
fn plant(out_dir: &Path) -> Result<PathBuf> {
    let source = collection();
    if !source.is_file() {
        return Err(Error::new(format!(
            "{} is missing. Rebuild it with `python fixtures/stamp-collection.PROVENANCE.py`; \
             this check plants it as the operator's stamp collection and has nothing to drive \
             without it.",
            source.display()
        )));
    }
    let home = out_dir.join("custom-stamp-appdata");
    let stamps = home.join("Adobe").join("Acrobat").join("DC").join("Stamps");
    std::fs::create_dir_all(&stamps).map_err(|e| {
        Error::new(format!(
            "cannot create the scratch stamps folder at {}: {e}",
            stamps.display()
        ))
    })?;
    let planted = stamps.join(PLANTED_AS);
    std::fs::copy(&source, &planted).map_err(|e| {
        Error::new(format!(
            "cannot copy {} to {}: {e}",
            source.display(),
            planted.display()
        ))
    })?;
    Ok(home)
}

/// **The tool the shell says is armed right now, or `None` if it has never
/// said.** `markup-tool` is emitted on change.
fn armed_tool(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .last(TOOL_EVENT)
        .and_then(|l| l.get("tool"))
        .map(str::to_owned))
}

/// **Arm the stamp tool from the Markup tab — but only if it is not already
/// armed.**
///
/// `stamp_dialog_reopen::arm_stamp`'s body and its reasoning: the ribbon item
/// is a **toggle**, and a defensive second click on an already-armed tool puts
/// it down. That was a real driven failure on 2026-09-10, and its lesson — *a
/// driven failure is a claim about the check too* — is why this reads the state
/// before it acts.
///
/// ⚠ **Second copy, deliberately not folded yet.** `driving`'s own rule is that
/// a third copy is the point at which folding becomes worth doing *on its own
/// rather than in the change that happens to need it*. This is the second.
///
/// # Errors
///
/// If the ribbon item is not on the tab, or the click armed nothing.
fn arm_stamp(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    if armed_tool(session)?.is_some_and(|t| t.starts_with(TOOL_TEXT_ANNOT)) {
        return Ok(());
    }
    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, STAMP_ITEM).ok_or_else(|| {
        Error::new(format!(
            "no `{STAMP_ITEM}` region on the Markup tab. Items declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.markup."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(14);
    match armed_tool(session)? {
        Some(t) if t.starts_with(TOOL_TEXT_ANNOT) => Ok(()),
        Some(t) => Err(Error::new(format!(
            "clicking `{STAMP_ITEM}` left the tool at `{t}`, not `{TOOL_TEXT_ANNOT}(..)`. The \
             ribbon item is a toggle: if the tool was already armed, this click put it down."
        ))),
        None => Err(Error::new(format!(
            "clicking `{STAMP_ITEM}` traced no `{TOOL_EVENT}` line at all, so the control armed \
             nothing."
        ))),
    }
}

/// **Click a declared region until the application's own trace shows it was
/// heard**, re-reading the rect between attempts.
///
/// The same rule [`crate::checks::driving::press_until_traced`] encodes for
/// keys: *nothing measured after a click is evidence about the program until
/// the click is shown to have arrived.* A gallery radio that is never hit and a
/// gallery radio that is hit and does nothing are opposite findings, and one
/// message standing for both aims the fix at the wrong half.
///
/// ⚠ **This is the third copy of the click half** — `read_mode_chrome`'s
/// `press_until_invoked`, and this. By `driving`'s own rule the fold is now
/// due, **as its own change**. Filed rather than done here, because folding
/// three call sites inside a feature commit is how a feature commit becomes
/// unreviewable.
///
/// # Errors
///
/// If the trace cannot be read, or the pointer cannot be driven.
fn click_until_traced(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    region: &str,
    evidence: &str,
) -> Result<bool> {
    let count =
        |session: &Session| -> Result<usize> { Ok(session.trace()?.events(evidence).count()) };
    let before = count(session)?;
    for _ in 0..CLICK_TRIES {
        let trace = session.trace()?;
        let Some(rect) = declared(&trace, ui_rect, region) else {
            return Ok(false);
        };
        driver.click_at(frame_of(session, &trace, ui_rect, region)?.declared_center(rect))?;
        session.settle(16);
        if count(session)? > before {
            return Ok(true);
        }
    }
    Ok(false)
}

/// **How many sentences the funnel recorded**, read from the joined text.
///
/// # ⚠ `disclosures=` is NOT a count, and this check's first driven run
/// proved it - 2026-09-10
///
/// `actions::funnel` writes the sentences THEMSELVES, joined by ` | `, and
/// the literal word `none` when there are none:
///
/// ```text
/// place-custom-stamp page=0 n=1 epoch=1 disclosures=Stamp placed, and stretched: ... | The date and name on this stamp ...
/// ```
///
/// This check's first version read it with `TraceLine::get_usize`, which
/// returns `None` for text, and then wrote `unwrap_or(0)`. The result was a
/// FAIL saying *"the placement recorded 0 disclosures and owes 2"* printed
/// directly above the two sentences it claimed did not exist - the check
/// quoting the truth in its own failure message while reporting its opposite.
///
/// ★ The line is quoted in that failure, which is the only reason the
/// mistake took thirty seconds rather than an afternoon. A refusal that does
/// not print the evidence it refused on sends the reader to the application.
///
/// ⚠ One residual hazard, recorded rather than fixed: the field is
/// unquoted, so a disclosure sentence containing `word=` would be split by
/// `parse_fields` into a phantom key. No sentence does today. The remedy is to
/// quote the value at the `funnel` end, and it is not made here because the
/// field's current shape is read by four other checks and changing it inside a
/// feature commit is how a feature commit becomes unreviewable.
fn disclosures_said(trace: &crate::trace::Trace) -> usize {
    let Some(text) = trace.last(FUNNEL_EVENT).and_then(|l| l.get("disclosures")) else {
        return 0;
    };
    if text.trim() == "none" {
        return 0;
    }
    text.split(" | ").filter(|s| !s.trim().is_empty()).count()
}

/// Read one `key=` off the last line named `event`, as a `usize`.
fn number(trace: &crate::trace::Trace, event: &str, key: &str) -> Option<usize> {
    trace.last(event)?.get_usize(key)
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a ribbon control, drags on the \
             canvas and presses two things in a dialog. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. This check needs somewhere on the page to drag, and a guessed one \
             can land off the sheet — which is symptom-identical to a drag that never \
             registered.",
        )
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    // --- A: plant the collection -------------------------------------------
    let home = plant(&ctx.out_dir)?;
    report.note(format!(
        "planted {COLLECTION} as {PLANTED_AS} under a scratch %APPDATA% at {} — the operator's \
         own Acrobat folder is neither read nor written",
        home.display()
    ));

    let mut spec = LaunchSpec::new(&exe, ctx.out("custom_stamp.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // ★ The redirect. See the header for everything else that moves with it.
    spec.env
        .push(("APPDATA".to_owned(), home.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- B: Review mode, the Markup tab, the stamp tool --------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, "review")?;
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.markup").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.markup` region after switching to Review. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("markup"))
    {
        return Err(Error::new(
            "the click on the Markup tab produced no tab-selected line, so nothing below would \
             mean anything.",
        ));
    }
    arm_stamp(&session, &driver, ui_rect)?;
    report.note("Markup > Stamp armed the annotation tool");

    // --- C: drag the box, and read what the scan found ---------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let from = frame.to_screen(mapping.doc_to_window(target)?);
    let to = frame.to_screen(mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + BOX_PT,
        y: target.y + BOX_PT,
    })?);
    driver.drag(from, to)?;
    session.settle(24);

    let trace = session.trace()?;
    if trace.events("text-annot-open").next().is_none() {
        return Ok(Some(
            "the drag completed and traced no `text-annot-open` line, so the release did not \
             open the stamp dialog at all. That blocks this check rather than being reported as \
             O172."
                .to_owned(),
        ));
    }
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "a `text-annot-open` line was traced and no `{BODY}` region appeared, so the dialog \
             was created and never drawn."
        )));
    }
    let Some(library) = trace.last(LIBRARY_EVENT) else {
        return Ok(Some(format!(
            "the stamp dialog opened and traced no `{LIBRARY_EVENT}` line, so it never scanned \
             the stamps folder. The scan is guarded on the annotation KIND — see \
             `dialogs::textannot::TextAnnotDialog::open` — and a guard that has stopped matching \
             `TextAnnotKind::Stamp` would look exactly like this."
        )));
    };
    let folder_found = library.get("folder") == Some("true");
    let categories = number(&trace, LIBRARY_EVENT, "categories").unwrap_or(0);
    let stamps = number(&trace, LIBRARY_EVENT, "stamps").unwrap_or(0);
    let unreadable = number(&trace, LIBRARY_EVENT, "unreadable").unwrap_or(0);
    if !folder_found || categories != 1 || stamps != STAMPS {
        return Ok(Some(format!(
            "the scan of the planted stamps folder found folder={folder_found} \
             categories={categories} stamps={stamps} unreadable={unreadable}, and this check \
             plants exactly one collection of {STAMPS} placeable stamps at {}. folder=false \
             means `stamps::folder::user_stamps_dir` did not resolve the redirected %APPDATA%, \
             so check that the child process inherits it. categories=0 with unreadable=1 means \
             the file was opened and `pdfcer_core::stamp_file::read` found no `/Names` -> \
             `/Pages` tree in it, which is a reader change rather than a shell one.",
            home.display()
        )));
    }
    report.note(format!(
        "the dialog scanned the planted folder and offered {stamps} stamps in {categories} \
         category"
    ));

    // --- D: the custom half is on the screen -------------------------------
    if declared(&trace, ui_rect, CUSTOM).is_none() {
        return Ok(Some(format!(
            "{stamps} of the operator's stamps were found and no `{CUSTOM}` region was declared, \
             so the gallery drew none of them. R9 says an unavailable capability renders nothing \
             — but the capability is plainly available here, so this is the feature shipped \
             dead. Regions declared: {}.",
            list(&declared_names(&trace, ui_rect, "text-annot"))
        )));
    }
    if declared(&trace, ui_rect, CUSTOM_0).is_none() {
        return Ok(Some(format!(
            "the `{CUSTOM}` group was declared and `{CUSTOM_0}` was not, so the half exists and \
             its first stamp has no rectangle to press. Regions declared: {}.",
            list(&declared_names(&trace, ui_rect, CUSTOM))
        )));
    }
    report.note("the gallery's custom half is drawn, with a rect per stamp");

    let shot = ctx.out("custom-stamp-gallery.png");
    if crate::capture::window_to_png(&session, &shot).is_ok() {
        report.artifact(shot);
    }

    // --- E: press the first one, and check WHICH one it is -----------------
    if !click_until_traced(&session, &driver, ui_rect, CUSTOM_0, CHOSEN_EVENT)? {
        return Ok(Some(format!(
            "clicking `{CUSTOM_0}` {CLICK_TRIES} times traced no `{CHOSEN_EVENT}` line, so the \
             radio was never heard. Either the rect published for it is not where the control \
             is, or the control is under something — the gallery sits inside a scroll area, and \
             a row scrolled out of view still publishes a rect through `diag::ui_rect`, which is \
             silent about visibility."
        )));
    }
    let trace = session.trace()?;
    let chosen = trace
        .last(CHOSEN_EVENT)
        .ok_or_else(|| Error::new("the chosen line vanished between two reads"))?;
    let name = chosen.get("name").unwrap_or_default().to_owned();
    let source_page = chosen.get("page").unwrap_or_default().to_owned();
    let dynamic = chosen.get("dynamic") == Some("true");
    if name != FIRST_LABEL || source_page != FIRST_SOURCE_PAGE.to_string() {
        return Ok(Some(format!(
            "the FIRST entry in the gallery is `{name}` on source page {source_page}, and the \
             planted collection's first NAME-TREE entry is `{FIRST_LABEL}` on page \
             {FIRST_SOURCE_PAGE}. §7.9.6 sorts a name tree lexicographically, so `#SRIssued` \
             sorts before `SRApproved` while its page is last — a gallery built by enumerating \
             PAGES instead of tree entries produces exactly this, looks entirely plausible, and \
             mislabels every stamp the operator owns. See \
             `fixtures/stamp-collection.PROVENANCE.py` property 1."
        )));
    }
    if !dynamic {
        return Ok(Some(format!(
            "`{FIRST_LABEL}` was selected and reported dynamic=false. The planted collection \
             marks it `#SRIssued`, and the `#` prefix is Acrobat's marker for a stamp whose text \
             it recomputes at placement time. Losing that flag loses the only sentence that \
             tells the operator the date baked into the artwork will never change."
        )));
    }
    report.note(format!(
        "ordinal 0 is `{name}` on source page {source_page}, dynamic — tree order, not page order"
    ));

    // --- F: place it -------------------------------------------------------
    let accept = declared(&trace, ui_rect, ACCEPT).ok_or_else(|| {
        Error::new(format!(
            "no `{ACCEPT}` region on the stamp dialog. It is published through \
             `diag::ui_rect_visible`, which stays silent when the control is off the screen — see \
             `stamp_dialog_reopen`, which is the check for that."
        ))
    })?;
    driver.click_at(frame_of(&session, &trace, ui_rect, ACCEPT)?.declared_center(accept))?;
    session.settle(30);

    let trace = session.trace()?;
    if let Some(refused) = trace.last(REFUSED_EVENT) {
        let refused_raw = &refused.raw;
        return Ok(Some(format!(
            "the collection would not open at placement time: `{refused_raw}`. It \
             opened a moment earlier for the scan, and this check plants it and does not touch \
             it in between, so the path carried on the action is not the path the library \
             offered."
        )));
    }
    if trace.events(REQUESTED_EVENT).next().is_none() {
        return Ok(Some(format!(
            "Add was pressed and traced no `{REQUESTED_EVENT}` line, so `apply` did not fork to \
             the custom route. The arm reads `custom: Some(..)` FIRST and falls through to \
             `textannot::commit` otherwise — a build that reordered those two arms places \
             `Approved` on the sheet instead of the operator's own stamp, silently, and every \
             unit test in front of the fork still passes."
        )));
    }
    let Some(placed) = trace.last(PLACED_EVENT) else {
        return Ok(Some(format!(
            "`{REQUESTED_EVENT}` was traced and `{PLACED_EVENT}` was not, so \
             `EditSession::place_page_artwork` was reached and did not return `Ok`. The funnel's \
             own refusal line carries the engine's error: {}.",
            trace
                .last(&format!("{FUNNEL_EVENT}-refused"))
                .map_or_else(|| "not traced either".to_owned(), |l| l.raw.clone())
        )));
    };
    let placed_raw = &placed.raw;
    let distorted = placed.get("distorted") == Some("true");
    let imported = number(&trace, PLACED_EVENT, "imported").unwrap_or(0);
    if imported == 0 {
        return Ok(Some(format!(
            "the artwork was placed and imported=0 objects. A stamp page carries at least a \
             content stream, and this one carries a font as well, so an empty import means the \
             annotation has an appearance stream referencing nothing — a `/Stamp` that draws a \
             blank rectangle. Line: `{placed_raw}`."
        )));
    }
    if !distorted {
        return Ok(Some(format!(
            "a {BOX_PT:.0} x {BOX_PT:.0} pt square box was dragged for a 200 x 60 pt source \
             page and the engine reported distorted=false. §12.5.5 maps the form `/BBox` onto \
             `/Rect` with INDEPENDENT horizontal and vertical factors, so the two scales cannot \
             be equal here — either the aspect is being silently preserved (which changes where \
             the mark lands, and is a behaviour change nobody asked for) or the flag stopped \
             being computed. Line: `{placed_raw}`."
        )));
    }
    report.note(format!(
        "the artwork arrived: {imported} objects imported, stretched as §12.5.5 requires"
    ));

    // --- G: and the operator is told, OFF the canvas -----------------------
    let said = disclosures_said(&trace);
    if said < OWED_DISCLOSURES {
        return Ok(Some(format!(
            "the placement recorded {said} disclosures and owes {OWED_DISCLOSURES}: the stretch \
             (distorted=true, asserted above) and the dynamic promise (dynamic=true, asserted \
             above). `app::actions::customstamp::disclosures` is the only thing between those \
             two facts and the status bar. Funnel line: `{}`.",
            trace
                .last(FUNNEL_EVENT)
                .map_or_else(|| "not traced".to_owned(), |l| l.raw.clone())
        )));
    }
    if declared(&trace, ui_rect, DISCLOSURE).is_none() {
        return Ok(Some(format!(
            "{said} disclosures were recorded and the status bar declared no `{DISCLOSURE}` \
             region, so the sentences exist and are on no screen. R8b rule 4 puts them \
             off-canvas — nothing is drawn on the mark itself — which makes the status bar the \
             ONLY place they can appear. Status regions declared: {}.",
            list(&declared_names(&trace, ui_rect, "status-group:"))
        )));
    }
    report.note(format!(
        "{said} sentences reached the status bar, and nothing was drawn on the mark"
    ));

    let shot = ctx.out("custom-stamp-placed.png");
    if crate::capture::window_to_png(&session, &shot).is_ok() {
        report.artifact(shot);
    }

    // --- H: the window opens again, on the stamp he just used --------------
    //
    // ★★★ **This is the half no unit test can reach.** `TextAnnotDialog::open`
    // scans the real stamps folder off `%APPDATA%`, so a test inside the crate
    // can hand the window a remembered CUSTOM stamp and watch it fail to
    // resolve, but it cannot plant a library for it to succeed against. This
    // check owns the folder it planted, which makes it the only place the
    // memory's whole round trip -- committed, carried, re-resolved against a
    // fresh scan -- is ever measured. `dialogs::textannot_tests` says so in its
    // own header so a reader does not mistake the gap there for coverage.
    arm_stamp(&session, &driver, ui_rect)?;
    // ⚠ LEFT and down, not right and down. The sweep aims at 2000 pt on a
    // 2383.9 pt wide sheet, so `x + 300 + BOX_PT` is 136 pt off the right edge --
    // a box the engine would still accept and whose artwork would land half off
    // the paper, which is a placement this check would then report as a memory
    // result. The offset only has to MISS the first stamp; which way it goes is
    // free, so it goes the way that stays on the sheet.
    let second = DocPoint {
        page: target.page,
        x: target.x - SECOND_OFFSET_PT,
        y: target.y + SECOND_OFFSET_PT,
    };
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let from = frame.to_screen(mapping.doc_to_window(second)?);
    let to = frame.to_screen(mapping.doc_to_window(DocPoint {
        page: second.page,
        x: second.x + BOX_PT,
        y: second.y + BOX_PT,
    })?);
    driver.drag(from, to)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(gallery) = trace.last(GALLERY_EVENT) else {
        return Ok(Some(format!(
            "the second drag traced no `{GALLERY_EVENT}` line, so either the dialog did not open \
             a second time -- `stamp_dialog_reopen` is the check for that, and a failure there \
             explains a failure here -- or the line is emitted only for a stamp and the kind \
             guard stopped matching. Regions on the dialog: {}.",
            list(&declared_names(&trace, ui_rect, "text-annot"))
        )));
    };
    let gallery_raw = gallery.raw.clone();
    let restored = gallery.get("restored").unwrap_or_default().to_owned();
    let remembered = gallery.get("remembered").unwrap_or_default().to_owned();
    if restored != RESTORED_CUSTOM {
        return Ok(Some(format!(
            "the stamp window opened a second time and reported restored=`{restored}`, not \
             `{RESTORED_CUSTOM}`. `default` means the memory never reached it, so \
             `DialogsState::last_stamp` was not written on accept or not read on open. \
             `standard` means a STANDARD face was remembered where a custom one was placed. \
             `gone` means the memory arrived and did not resolve -- the label or the category it \
             names is not in the folder this check planted, which is what a memory keyed on a \
             page index would do the moment the collection was re-sorted. Line: `{gallery_raw}`."
        )));
    }
    if remembered != REMEMBERED_TOKEN {
        return Ok(Some(format!(
            "the window restored a custom stamp and named it `{remembered}`, not \
             `{REMEMBERED_TOKEN}`. The category comes from the collection's `/Info` `/Title` and \
             the label from its name-tree entry, so a mismatch here says the memory is keyed on \
             something else -- the file stem, or the ordinal. Note the quotes: the category holds \
             a space, and an unquoted value would arrive truncated at it. Line: `{gallery_raw}`."
        )));
    }
    report.note(format!(
        "the second opening came up on {remembered} with no click -- the memory survived a \
         placement and a fresh folder scan"
    ));

    // --- I: and it is that stamp that lands, not merely that label ---------
    //
    // ⚠ The gallery showing the right thing and the commit sending the right
    // thing are two claims. A restored selection the accept path did not read
    // would look exactly like phase H passing, and would place `Approved`.
    let accept = declared(&trace, ui_rect, ACCEPT).ok_or_else(|| {
        Error::new(format!(
            "no `{ACCEPT}` region on the SECOND stamp dialog. `stamp_dialog_reopen` is the check \
             for that defect -- O171 -- and a failure there explains this one."
        ))
    })?;
    driver.click_at(frame_of(&session, &trace, ui_rect, ACCEPT)?.declared_center(accept))?;
    session.settle(30);

    let trace = session.trace()?;
    let requests: Vec<String> = trace
        .events(REQUESTED_EVENT)
        .filter_map(|l| l.get("name").map(str::to_owned))
        .collect();
    if requests.len() < 2 {
        return Ok(Some(format!(
            "Add was pressed on the restored window and the run recorded {} `{REQUESTED_EVENT}` \
             line(s), not 2. The gallery said `{remembered}` was selected, so a commit that did \
             not fork to the custom route means the accept path reads a different field than the \
             one `open` restored.",
            requests.len()
        )));
    }
    if requests.last().map(String::as_str) != Some(FIRST_LABEL) {
        return Ok(Some(format!(
            "the restored window committed `{}`, and the memory named `{FIRST_LABEL}`. The right \
             label was shown and a different stamp was sent -- precisely the failure the memory \
             is stored as a NAME to prevent, since a stored page index names a different entry \
             the moment the collection is re-sorted. Requests this run: {}.",
            requests.last().map_or("nothing", String::as_str),
            list(&requests)
        )));
    }
    report.note(format!(
        "the restored stamp reached the page: {} placements, the last of them `{FIRST_LABEL}`",
        requests.len()
    ));

    let shot = ctx.out("custom-stamp-remembered.png");
    if crate::capture::window_to_png(&session, &shot).is_ok() {
        report.artifact(shot);
    }
    Ok(None)
}
