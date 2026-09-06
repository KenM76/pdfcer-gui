//! # `refused_character_face` — **a refused character offers the face that can
//! type it**, driven end to end
//!
//! `OPERATOR_REQUESTS.md` **O141**. The operator, 2026-09-05, while trying to
//! fix a typo:
//!
//! > *"if the character isn't available in a pdf are we able to change to a
//! > different font?"*
//!
//! **Yes**, and on the day he asked it every piece already existed — the engine
//! refuses by name and hands back `Refusal::character`, the face chooser has
//! offered the fourteen standard faces since `Pass 162.0`, and `set_font`
//! authors a resource the page does not carry. **Nothing connected the refusal
//! to the chooser**, so the answer to his question lived in the last clause of
//! an error message he never saw. This check drives the connection.
//!
//! ## ★★★ HOW TO FALSIFY IT — read this before trusting a green run
//!
//! [`crate::checks`]' founding rule: *a check that has only ever been seen to
//! pass is indistinguishable from one that cannot fail.* Seven separate
//! mutations must each turn it red, and each names the link it cuts:
//!
//! | mutate | expected verdict |
//! |---|---|
//! | delete the `refusedchar::record(..)` call in `app::status::decline::textedit` | **FAIL** — *"THE REFUSAL LEADS NOWHERE"*: the `⊗` slot still draws and no `properties.refusedchar` region follows the refusal |
//! | make `missing_character` return `None` unconditionally | **FAIL** — same region, plus `said=UnsupportedFont` on the classification line where `FontLacksTheCharacter` is the only reading of a refusal that named a character |
//! | drop the character from `refusedchar::section`'s trace line | **FAIL** — *"THE OFFER DOES NOT NAME THE CHARACTER"*: the block drew and cannot say what it drew about, which is the whole distinction between this and a generic decline |
//! | delete the `face_addable_disclosure` label from `refusedchar::section` | **FAIL** — *"…AND THE DISCLOSURE IS NOT ON SCREEN"*, rule 4's half |
//! | stop `canvas::textedit::plan` writing `LAST_COMMIT`, or drop the `Action::CommitTextEdit` push from `refusedchar::section` | **FAIL** — *"THE OFFER DOES NOT FINISH THE JOB"*: the face swaps and no commit of any kind follows, although the harness pressed nothing |
//! | send the restyle to the current selection instead of `runs: vec![refused.run]` | **FAIL** — *"THE FACE SWAP DID NOT REACH THE RUN THE REFUSAL NAMED"*: the retype comes back on the **same** subset floor |
//! | make `RefusedCharUi::advance` return `true` unconditionally, or freeze `retried` at `false` | **FAIL** — the control point at step 0b, and the state walk at step 7 |
//!
//! ★★ And two mutations that must **NOT** turn it red, because a check that
//! fires on them is asserting a limitation rather than a capability: swapping
//! which of the fourteen is first in `Std14::ALL` (the check clicks the first
//! addable row, whatever it is), and rewording any sentence in
//! `crate::text::panels::face` (the harness reads regions and one trace field,
//! never rendered prose — `check-ui-strings.sh` and the unit tests own the
//! words).
//!
//! ## ★★★ THE DYNAMIC RANGE, and why it is two assertions rather than one
//!
//! The oracle for most of this file is *"a region was published"*, and **a probe
//! whose baseline has no dynamic range cannot produce a verdict.** A build that
//! offered the chooser after every edit — because the block is drawn
//! unconditionally, because the state never retires, because a stale refusal
//! from ten minutes ago is still in the slot — would satisfy a one-sided check
//! *permanently*. This project filed three such false reports in a single day.
//!
//! Two assertions hold the two sides:
//!
//! * **step 0b** — on a freshly opened document that nothing has been done to,
//!   the block must be **absent**. That is what a build drawing it
//!   unconditionally fails.
//! * **step 7** — the block must be seen to **walk its own state machine**:
//!   `offer` → `swapped` → `blocked` on its own trace line, three states each
//!   caused by a different thing happening to the document. A block frozen on
//!   one state cannot produce that sequence.
//!
//! ★ On the branch where the character actually goes in (see below), the
//! stronger classical control applies instead and is asserted there: the offer
//! must be **gone** after a commit that succeeded, and its absence is made
//! non-vacuous by first proving frames were painted after that commit.
//!
//! ## The states it walks, all measured with `pdfcer.exe` first
//!
//! | # | gesture | what must happen |
//! |---|---|---|
//! | 0 | nothing yet | `properties.refusedchar` is **absent** — the control point |
//! | 1 | seed `q`, Ctrl+Enter | the engine refuses; the classification says `FontLacksTheCharacter`; the offer draws and **names `q`** |
//! | 2 | — | the disclosure is on screen and **does not overlap the page** (rule 4) |
//! | 3 | open the chooser, click the first addable row | `format-text` reaches the document |
//! | 4 | **nothing** | the block re-applies the operator's own edit by itself — a commit follows with no further input |
//! | 5 | — | that commit either **lands** (the character goes in, one gesture, and the block retires) or is refused by the engine's base-revision limit, in which case the block must move to `state=blocked` and say so |
//!
//! ## ★★★ WHY STEP 5 HAS TWO ACCEPTED OUTCOMES, AND IT IS NOT A SOFTENED ASSERTION
//!
//! Because the last step is currently blocked by a defect in `pdfcer-core`, and
//! that was **measured rather than inferred**.
//! `pdfcer-gui`'s `canvas::textedit::facewall` holds three experiments:
//!
//! 1. **one** `EditSession`, `format_text` then `edit_text`, located by find
//!    text alone so **no operand this shell computes is in the request** —
//!    refused;
//! 2. the identical pair with a **save and reopen** between them — succeeds,
//!    and the character is in the extracted text;
//! 3. a swap to a face the page **already carries** — succeeds at once.
//!
//! So the trigger is the `/Font` object `format_text` creates: it lives in the
//! session's overlay, and `edit_text` plans with `plan_edit(&self.base, …)`, so
//! the name in the rewritten stream resolves to nothing. Filed as
//! `request_edit_text_resolves_font_names_against_the_base_revision.md`.
//!
//! ⇒ The two outcomes are therefore **the engine fixed and the engine not
//! fixed**, and the check reports which. What it will **not** accept is any
//! other refusal: a second embedded-subset refusal means the swap never reached
//! the run the refusal named, which is the shell defect this file was written to
//! catch, and it is failed by name. Nor will it accept a build that is blocked
//! and does not say so — `state=blocked` is required, because a block still
//! promising the operator that his character is going in, over a document where
//! it did not, is the confident small lie `app::status::decline`'s own header
//! forbids.
//!
//! When the engine ships the fix, `facewall`'s first test goes red and this
//! check should be tightened to REQUIRE the landing branch. Both files say so.
//!
//! ## ★★ Why `q` and not `€`
//!
//! Because it is the operator's own example, and because it is the fact O141
//! says is worth seeing: *"it is narrower than 'no accents and no symbols' …
//! that font will not take a `€`, a `%`, an `@` — **or a plain lowercase `q`**,
//! because no word in those headings has one in it. Nothing about the font is
//! foreign or exotic. Whatever was not printed is simply not in the file."*
//!
//! It is also the safer instrument: the character travels to the application in
//! an environment variable (`PDFCER_DIAG_TYPE`) and comes back through a trace
//! line, and an ASCII character cannot be lost to a codepage on either leg. The
//! non-ASCII half is covered where it belongs — by
//! `text::panels::face`'s `every_sentence_in_the_offer_names_the_character_itself`,
//! which asserts on `€` precisely so a build that formatted it as `\u{20ac}`
//! goes red.
//!
//! ## ★★★ The fixture is PINNED and any `--pdf` is ignored
//!
//! `fixtures/subset-font-floor.pdf`, aimed at **(115.2, 612.0) on page 1**.
//! `fixtures/subset-font-floor.PROVENANCE.md` carries the four measured engine
//! runs and the reason no other fixture in this repository can carry the case:
//! every one of them is either a non-embedded standard-14 face (whose
//! `WinAnsiEncoding` accepts the character and the edit simply works), a fully
//! embedded non-subset face (the `class.embedded && class.subset` floor never
//! fires), or a symbolic built-in-cmap face that refuses **every** edit for an
//! unrelated reason and has no remedy to offer. A check driven against any of
//! those would be measuring something else and would be **unable to fail**.
//!
//! ⚠ **And the operator's own file cannot be used**, which is worth knowing
//! before somebody re-points this check at it. `apartment work - signed.pdf`
//! raises the identical refusal — `this font has no glyph for '€'` on an
//! 8,640-byte `AAAAAA+Arimo-Bold` subset — and its page 2 is written **one show
//! operator per glyph**, so the caret refuses before the font question is ever
//! asked (O140). The engine's command line reaches it because it searches the
//! whole page; a click cannot. So the fixture is the floor and his file is the
//! subject, and any claim made here should be re-measured against his document
//! with `pdfcer.exe` before it is repeated to him.
//!
//! ## What this deliberately does NOT assert
//!
//! **Which** of the fourteen faces was applied. `text-style-applied` carries
//! `change=face` and not the selector, and adding the face name to a diagnostic
//! to satisfy a check would be the harness dictating a trace's contents —
//! `std14_face`'s own ruling, and it binds here for the same reason. The row
//! clicked is the *first* addable one on a fixture whose page fonts are known,
//! which is deterministic enough, and `format-text` landing is the claim that
//! matters because it is the one that says a `/Font` object was written into the
//! operator's document.
//!
//! It also does not assert **what the block says**. The harness cannot read
//! rendered text — there is no accessibility reader and no OCR — so it asserts
//! that the block drew, and that the character it drew about is the one that was
//! refused. The wording is held by unit tests in `crate::text::panels::face` and
//! by `check-ui-strings.sh`.

use crate::checks::driving::{self, INVOKE_EVENT, SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The fixture, relative to the source root. See the module header: it is
/// pinned, and a `--pdf` is reported as ignored rather than honoured.
const FIXTURE: &str = "fixtures/subset-font-floor.pdf";

/// The fixture's one font, as the engine spells it in its own refusal.
///
/// A constant rather than two string literals inside two failure messages,
/// because the name carries the engine's pre-rename resource prefix and each
/// literal would otherwise need its own exemption marker on
/// `tools/gates/check-old-name-absent.sh`. One marked line is better than
/// several, and the two messages cannot now drift on how the font is spelled.
const FIXTURE_FONT: &str = "SUBSET+pdfceSubsetDemo"; // old-name-exempt: a PDF resource / BaseFont name the ENGINE writes into the file, quoted verbatim from its own output. It is data in the file format, not prose about this project, and the engine deliberately stopped the rename at that boundary because changing it would alter the bytes of every document pdfcer has ever produced. Same ruling as O141's.

/// The point on that fixture, in PDF user space, 0-based page.
///
/// The centre of the run `ABC`, whose box `extract-text --pages 1 --json`
/// reports as `[72.0, 588.0, 158.4, 636.0]`.
const AIM: (usize, f64, f64) = (0, 115.2, 612.0);

/// The fixture's page box. Stated rather than read, because the fixture is
/// pinned and a `--page-size` cannot apply to a document the caller did not
/// choose.
const PAGE: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};

/// Reset the dock, take Edit, mount Properties, arm the caret — one per frame.
///
/// ★ `view.reset_layout` first, and it is not decoration: the application
/// persists its dock layout across runs and the harness does not clear it, so a
/// launch inherits whatever the previous launch left — including a previous
/// *driven* one. `typo_refusal`'s own header records the run this project spent
/// reading last run's furniture. Its arrival is asserted below rather than
/// assumed.
///
/// ★★ `file.properties` before `edit.text`, and `mode.edit` before both: the
/// dock follows the ribbon mode on the same frame, so a panel mounted before the
/// mode moved would be mounted into the workspace this check is about to leave.
/// `std14_face` learned that the expensive way.
const INVOKE: &str = "view.reset_layout,mode.edit,file.properties,edit.text";

/// The character seeded into every draft this check opens.
///
/// See the module header for why it is `q` rather than `€`. Seeded rather than
/// typed because `sys::vk` is a deliberately closed list of non-character
/// virtual keys and this machine cannot inject an arbitrary character — and the
/// keystroke is not the subject here, the refusal after the commit is.
///
/// ★★ It replaces the draft's whole text rather than appending to it
/// (`keys::typing` does `draft.text.clear()` before inserting the seed), so the
/// commit is `find="ABC" replace="q"` — which is exactly the command line
/// measured in the fixture's provenance note, on both sides of the face swap.
const SEED: &str = "q";

/// `layout-reset scope=… changed=…`. ★ `changed=false` is a good answer: it
/// means the layout was already default. What matters is that the reset ran.
const RESET_EVENT: &str = "layout-reset";
/// `text-edit-caret kind=… page=… run=… len=…` — a click opened a draft.
const CARET_EVENT: &str = "text-edit-caret";
/// `text-edit-declined reason=…` — a click did not.
const DECLINED_EVENT: &str = "text-edit-declined";
/// `edit-text-refused page=… n=… detail=…` — the funnel's error arm.
const REFUSED_EVENT: &str = "edit-text-refused";
/// `edit-text page=… n=… epoch=… disclosures=…` — the funnel's SUCCESS arm, and
/// the negative control's oracle.
///
/// ★ Deliberately not `text-edit-*`: `vector_edit`'s label is the bare verb name
/// and a module's own summary line takes a suffix, which is what
/// `tools/gates/check-trace-names.py` exists to keep true. Matching is on the
/// exact first token, so the two never collide.
const APPLIED_EVENT: &str = "edit-text";
/// `edit-text-classified page=… run=… kind=… one_operator=… character=…
/// said=…` — the shell's own classification of the engine's refusal.
const CLASSIFIED_EVENT: &str = "edit-text-classified";
/// `refused-char page=… run=… character=… font=… faces=… state=…` — the offer
/// block's own line, published on every frame it draws.
const OFFER_EVENT: &str = "refused-char";
/// `text-style-applied page=… change=… applied=… of=…`.
const STYLE_EVENT: &str = "text-style-applied";
/// The label `vector_edit` writes when the restyle reached the engine.
const FORMAT_APPLIED: &str = "format-text";
/// The offer block itself, published only on the frames it draws.
const OFFER_REGION: &str = "properties.refusedchar";
/// Its face chooser — the control that opens the list.
const OFFER_FACE_REGION: &str = "properties.refusedchar.face";
/// ★★★ Rule 4's off-canvas report, drawn above the chooser.
const OFFER_DISCLOSURE_REGION: &str = "properties.refusedchar.disclosure";
/// The heading over the rows pdfcer would ADD, inside the popup.
const POPUP_ADDABLE_REGION: &str = "properties.refusedchar.face.addable";
/// The same disclosure again, inside the popup.
const POPUP_DISCLOSURE_REGION: &str = "properties.refusedchar.face.disclosure";
/// The first row offering a face the document does not contain.
const POPUP_NEW_REGION: &str = "properties.refusedchar.face.new";
/// The page on the canvas, for the *off-canvas* half of rule 4 and for saying
/// whether a sheet was on screen at all.
const PAGE_REGION: &str = "page";
/// The Properties panel's dock tab, so the body can be brought to the front.
///
/// ★ A docked pane that is not in front publishes **nothing**, which is
/// indistinguishable from a panel with nothing to say. This project filed one
/// such report; `dock.tab.<id>` is published for exactly this.
const PROPERTIES_PANEL: &str = "file.properties";
/// How many notches to spend scrolling the panel for the chooser.
const SCROLL_ATTEMPTS: usize = 6;

/// See the module documentation.
pub struct ARefusedCharacterOffersAFaceThatCanTypeIt;

impl Check for ARefusedCharacterOffersAFaceThatCanTypeIt {
    fn name(&self) -> &'static str {
        "a_refused_character_offers_a_face_that_can_type_it"
    }

    fn defect(&self) -> &'static str {
        "an edit is refused because the run's font has no code for the character just typed, and \
         the operator is told only that the edit was refused — while the engine named the \
         character, the shell already has a chooser that offers faces which carry it, and \
         set_font already writes one; so the answer to \"can we change to a different font?\" is \
         yes and is unreachable from the moment the question arises"
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

/// Do two rectangles share any area?
///
/// [`LRect`] carries `contains_rect` and not this, deliberately — *"can the
/// operator click this?"* is a containment question. The question here is the
/// opposite one and it is rule 4's: **is any part of this sentence drawn over
/// the page?** Overlap by a pixel would be enough to make the answer *yes*, so
/// the weaker predicate is the right one.
fn overlaps(a: LRect, b: LRect) -> bool {
    a.min.x < b.max.x && b.min.x < a.max.x && a.min.y < b.max.y && b.min.y < a.max.y
}

/// Was `region` published on any frame after trace line `after`?
fn published_after(trace: &crate::trace::Trace, region: &str, after: usize) -> bool {
    trace
        .lines
        .iter()
        .any(|l| l.event == "ui-rect" && l.lineno > after && l.get("name") == Some(region))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks page text, commits an edit, opens \
             a combo box and clicks a row in it, and none of the four can be simulated from a \
             trace. Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are and there is no offer to observe.",
            ctx.profile.name
        ))
    })?;

    // ★ The fixture is PINNED. See the module header: on a document whose font
    // is not an embedded subset the refusal under test cannot occur, so a
    // sweep's fixture would make this check unable to fail.
    let pdf = ctx.source_root.clone().unwrap_or_default().join(FIXTURE);
    let pdf = if pdf.exists() {
        pdf
    } else {
        std::path::PathBuf::from(FIXTURE)
    };
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is not on disk. It is a byte-for-byte copy of \
             `D:\\Dev\\pdfcer\\fixtures\\synthetic\\text\\subset-simple-embedded.pdf`, which the \
             engine generates with `tools/gen-subset-font-fixtures.py`; \
             `fixtures/subset-font-floor.PROVENANCE.md` says so and says why no other document \
             in this directory can carry the case."
        )));
    }
    if ctx.pdf.is_some() {
        report.note(format!(
            "--pdf was supplied and is IGNORED: this check pins {FIXTURE}, whose one font is an \
             embedded SUBSET carrying only the three letters its page prints — the only shape \
             that reaches the R-INV-1 floor this feature answers"
        ));
    }
    if ctx.target.is_some() {
        report.note(
            "--doc-point was supplied and is IGNORED: the aim is the centre of the fixture's \
             own `ABC` run, at (115.2, 612.0) on page 1",
        );
    }
    let target = DocPoint::new(AIM.0, AIM.1, AIM.2);

    let mut spec = LaunchSpec::new(&exe, ctx.out("refused-character-face.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_TYPE".to_owned(), SEED.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} on {} with PDFCER_DIAG_INVOKE={INVOKE} and \
         PDFCER_DIAG_TYPE={SEED}",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    session.settle(50);
    // Maximised so the right-hand dock has room for a panel body. A panel
    // squeezed to nothing publishes regions nobody can press, and this check
    // reads three of them.
    session.maximize();
    session.settle(24);
    let driver = Driver::new(session.window());

    // --- 0a: the dock reset LANDED -----------------------------------------
    let trace = session.trace()?;
    let Some(reset) = trace.events(RESET_EVENT).last() else {
        return Err(Error::new(format!(
            "`view.reset_layout` was requested through `PDFCER_DIAG_INVOKE` and no \
             `{RESET_EVENT}` line followed, so the dock holds whatever the previous launch \
             persisted. Refusing to continue rather than reading last run's furniture.\n\
             ★ `{INVOKE_EVENT}` is the SHELL's line for a ribbon click and is deliberately not \
             what is asserted: `PDFCER_DIAG_INVOKE` dispatches straight into `dispatch_command` \
             and never touches the ribbon. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the dock layout was reset: `{}`", reset.raw));

    if declared(&trace, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nowhere to put a caret. Regions beginning `page`: {}.",
            list(&declared_names(&trace, ui_rect, "page"))
        )));
    }
    driving::raise_dock_tab(&session, &driver, ui_rect, PROPERTIES_PANEL)?;
    session.settle(14);

    // --- 0b: ★★★ THE CONTROL POINT -----------------------------------------
    //
    // Nothing has been refused yet, so the offer must not be on screen. Without
    // this every region read below could be one declared from the first frame,
    // and the check would be green on a block no refusal ever raised — which is
    // also, exactly, the failure mode the negative control at the end guards
    // from the other side.
    let trace = session.trace()?;
    if let Some(early) = declared(&trace, ui_rect, OFFER_REGION) {
        return Ok(Some(format!(
            "★★★ THE OFFER IS ON SCREEN BEFORE ANYTHING WAS REFUSED: `{OFFER_REGION}` is \
             declared at {early:?} on a freshly opened document where no edit has been \
             attempted.\n\
             `panels::properties::refusedchar::section` returns `false` and draws nothing —\
             heading included — until a refusal has been recorded and adopted, on \
             `properties::disclose`'s rule that a heading present on every frame trains an \
             operator to stop reading the region under it. A block drawn unconditionally would \
             make every assertion below vacuous AND would be a permanent font-chooser under a \
             document nobody has edited. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the control point holds: nothing refused, no offer on screen");

    // --- 1: the refusal ----------------------------------------------------
    let at = aim(ctx, &session, PAGE, target)?;
    driver.click_at(at)?;
    session.settle(26);

    let trace = session.trace()?;
    let Some(caret) = trace.events(CARET_EVENT).last().map(|l| l.raw.clone()) else {
        let declined = trace
            .events(DECLINED_EVENT)
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Err(Error::new(format!(
            "the click at ({:.1}, {:.1}) placed no caret{}. This fixture's only run is `ABC` at \
             [72.0, 588.0, 158.4, 636.0] and the aim is its centre, so a miss here is the harness \
             or the hit test rather than this check's subject. SKIPPED. Trace: {}.",
            target.x,
            target.y,
            declined.map_or_else(String::new, |r| format!(" — `{DECLINED_EVENT} reason={r}`")),
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the click placed a caret: `{caret}`"));

    driver.press_chord(&[vk::CONTROL], vk::ENTER)?;
    session.settle(34);

    let trace = session.trace()?;
    let Some(refused) = trace.events(REFUSED_EVENT).last() else {
        return Err(Error::new(format!(
            "the commit of `{SEED}` was NOT refused — no `{REFUSED_EVENT}` line. On this fixture \
             the engine's own command line refuses it by name (`R-INV-1 (embedded-subset floor): \
             character U+0071 'q' … which font '{FIXTURE_FONT}' (an embedded SUBSET) \
             does not already carry on this page`), so either the fixture has been regenerated \
             without its subset tag or its `/FontFile2`, or the seed did not reach the draft — \
             look for `text-edit-seeded len=1`. SKIPPED rather than failed: this says the \
             instrument is not set up, not that the build is wrong. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let refusal_at = refused.lineno;
    let refusal_raw = refused.raw.clone();
    report.note(format!("★★ the engine refused the commit: `{refusal_raw}`"));

    // ★★ The classification, which is a SEPARATE line and PRECEDES the refusal:
    // it is written from inside `vector_edit`'s closure through
    // `Result::inspect_err`, and the funnel's error arm runs after the closure
    // returns. So it is looked for by event and must not be anchored
    // `> refusal_at`, which would assert the opposite of the true ordering.
    let classified = trace.events(CLASSIFIED_EVENT).last();
    let said = classified.and_then(|l| l.get("said")).map(str::to_owned);
    let character = classified
        .and_then(|l| l.get("character"))
        .map(str::to_owned);
    let (Some(said), Some(character)) = (said, character) else {
        return Ok(Some(format!(
            "THE REFUSAL WAS NOT CLASSIFIED: no `{CLASSIFIED_EVENT} … character=… said=…` line \
             after `{refusal_raw}`.\n\
             `EditError::Refused` carries `Refusal::character`, and \
             `app::status::decline::textedit::missing_character` reads it — one datum the coarse \
             `RefusalKind` structurally cannot hold. Without it every font refusal collapses back \
             into one sentence, and the operator with a two-click fix is told pdfcer cannot edit \
             their text. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★★ and it was classified: character={character}, sentence={said}"
    ));

    // ★★★ The mapping a wrong build gets backwards. `EditError::Refused(_)` maps
    // to `RefusalKind::UnsupportedFont` WHOLESALE, so the repertoire refusal
    // (a face swap fixes it) and the unreadable-encoding refusal (nothing does)
    // arrive as one category. `Refusal::character` is `Some` for exactly the
    // first, which is why the datum is read rather than a trigger id.
    if said != "FontLacksTheCharacter" {
        return Ok(Some(format!(
            "★★★ THE REFUSAL NAMED A CHARACTER AND WAS WORDED AS THOUGH IT HAD NOT: \
             `{CLASSIFIED_EVENT}` reports `character={character}` and `said={said}`, where \
             `FontLacksTheCharacter` is the only reading those two facts support.\n\
             `EditRefusal::UnsupportedFont` says *pdfcer cannot write new letters into this \
             text*, which is true of a font whose code-to-glyph relation is unrecoverable \
             (R-INV-2/3/4, where `character` is `None`) and false here: this font is perfectly \
             readable and simply does not carry one letter. Telling the operator the first when \
             the second is true withholds a fix that is two clicks away — a confident wrong \
             reason, which `RefusalKind`'s own header calls strictly worse than the silence it \
             replaced. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 2: ★★★ THE OFFER — it drew, and it names the character -------------
    let trace = session.trace()?;
    let Some(offer) = trace
        .events(OFFER_EVENT)
        .filter(|l| l.lineno > refusal_at)
        .last()
    else {
        let shot = ctx.out("refused-character-face.no-offer.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★★★ THE REFUSAL LEADS NOWHERE. The engine refused by name, the shell classified it \
             as `{said}` and wrote the status bar's sentence — and no `{OFFER_EVENT}` line \
             followed, so `panels::properties::refusedchar` never drew.\n\
             **That is the whole of O141.** The engine refuses by name; the refusal carries the \
             character; the face chooser has offered the fourteen standard faces since \
             `Pass 162.0`; `set_font` authors a resource the page does not carry — measured on \
             the operator's own file, where the `€` went in. Every piece exists and nothing \
             joins them, so the answer to *\"are we able to change to a different font?\"* lives \
             in the last clause of an error message.\n\
             Three candidates. (1) **`refusedchar::record` is not called** — it is one `if let` \
             in `app::status::decline::textedit::record_edit_text_refusal`, beside the sentence \
             that DID reach the bar. (2) **The Properties panel is not on screen**: a docked pane \
             that is not in front publishes nothing, and this check raises `dock.tab.\
             {PROPERTIES_PANEL}` before it starts. (3) **The block was adopted and immediately \
             retired** — `RefusedCharUi::advance` clears on any epoch change it did not cause, \
             and a build that mis-read the epoch would flash the offer for one frame. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let offer_at = offer.lineno;
    let offer_raw = offer.raw.clone();
    let named = offer.get("character").map(str::to_owned);
    if named.as_deref() != Some(character.as_str()) {
        return Ok(Some(format!(
            "★★★ THE OFFER DOES NOT NAME THE CHARACTER THAT WAS REFUSED. The block drew — \
             `{offer_raw}` — and reports `character={named:?}` where the refusal reported \
             `character={character}`.\n\
             Naming it is the one thing this surface exists to do that the status bar cannot: \
             `Declined::line` returns `&'static str`, so the `⊗` slot cannot interpolate a \
             runtime character and `disclosure_line` truncates it to 45 % of the bar besides. A \
             block that draws a heading and a font list without saying which keystroke the \
             document refused has moved the same generic decline to a wider surface — and on a \
             pasted line the operator may never have seen themselves type the character. Trace: \
             {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the offer drew and NAMED the character: `{offer_raw}`"
    ));

    if !published_after(&trace, OFFER_REGION, refusal_at) {
        return Ok(Some(format!(
            "the offer traced `{offer_raw}` and declared no `{OFFER_REGION}` region after the \
             refusal, so the block computed itself and never reached a rectangle. Regions \
             beginning `properties.`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "properties.")),
            session.trace_path().display()
        )));
    }

    // --- 3: ★★★ RULE 4 — the disclosure, ON SCREEN and OFF THE CANVAS -------
    //
    // Swapping the face is the operator's own instruction, so the changed
    // letterforms are not pdfcer marking its own uncertainty and must NOT be
    // badged, tinted or outlined on the page. But a standard-14 face is a NAME,
    // not a font program — *"no font program is embedded and no bytes of glyph
    // outline were added"* — so the client's reader supplies the letterforms.
    // That is the part the operator cannot see on his own screen, and it is
    // exactly the case rule 4's surviving half exists for: it owes an
    // **off-canvas** sentence. Both halves are asserted, and the geometric one
    // is the half no unit test in the workspace can observe.
    let trace = session.trace()?;
    let Some(disclosure) = declared(&trace, ui_rect, OFFER_DISCLOSURE_REGION) else {
        let shot = ctx.out("refused-character-face.no-disclosure.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★★★ THE OFFER IS ON SCREEN AND THE DISCLOSURE IS NOT: `{OFFER_REGION}` is declared \
             and `{OFFER_DISCLOSURE_REGION}` is not.\n\
             pdfcer authors these faces *\"with widths, embedding nothing\"*, so text set in one \
             is drawn with the READER'S OWN COPY — correct on this machine, possibly different \
             on the client's. Rule 4 forbids marking the canvas for it and REQUIRES an \
             off-canvas report, and a sentence that is catalogued, unit-tested and never painted \
             has discharged nothing.\n\
             Two candidates, and the screenshot beside this report separates them. (1) **It is \
             drawn and clipped away** — `ui_rect_visible` publishes nothing for a rect outside \
             the clip, and this panel is one `ScrollArea`. (2) **It is not drawn** — it is one \
             `ui.label` in `refusedchar::section`, immediately above the chooser. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if let Some(page_rect) = declared(&trace, ui_rect, PAGE_REGION)
        && overlaps(disclosure, page_rect)
    {
        let shot = ctx.out("refused-character-face.disclosure-on-canvas.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★★★ THE DISCLOSURE IS DRAWN OVER THE PAGE. `{OFFER_DISCLOSURE_REGION}` is at \
             {disclosure:?} and `{PAGE_REGION}` is at {page_rect:?}, and the two overlap.\n\
             Rule 4 has two halves and this build has swapped them: the report is REQUIRED and it \
             is required to be **off** the canvas — *\"no badge, tint, red flag, dashed outline \
             or 'provisional' layer drawn into the page view\"*, because a second rendering path \
             for the same content is two paths that drift, and because the editing canvas must \
             look exactly like the saved file. A sentence floating over the sheet is that \
             prohibition broken by the surface written to honour it. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(
        "★★★ rule 4 is discharged: the disclosure is on screen, in the panel, and shares no \
         area with the page — nothing marks the canvas",
    );

    // --- 4: take the offer --------------------------------------------------
    let Some(combo) = driving::scroll_to(
        &session,
        &driver,
        ui_rect,
        OFFER_REGION,
        OFFER_FACE_REGION,
        SCROLL_ATTEMPTS,
        report,
    )?
    else {
        return Ok(Some(format!(
            "★★ THE OFFER NAMES THE CHARACTER AND OFFERS NO WAY OUT: `{OFFER_REGION}` drew and \
             `{OFFER_FACE_REGION}` was never declared after {SCROLL_ATTEMPTS} scroll notches.\n\
             A better-worded refusal with no control beside it is O141 answered with a diagnosis \
             instead of a route — which is the state the row was filed about: *\"that last clause \
             is the answer to your question, and it is buried in an error message.\"* Regions \
             beginning `{OFFER_REGION}`: {:?}. Trace: {}.",
            driving::live_names(&session.trace()?, ui_rect, OFFER_REGION),
            session.trace_path().display()
        )));
    };
    // ★ The popup's rows must be absent before the combo is clicked, or every
    // region below could be one that was on screen with the popup shut —
    // `std14_face`'s control point, and the same defect wearing a green tick.
    if declared(&session.trace()?, ui_rect, POPUP_NEW_REGION).is_some() {
        return Ok(Some(format!(
            "`{POPUP_NEW_REGION}` is declared before the chooser was clicked, so the popup's \
             rows are on screen with the popup shut. That is a layer or a visibility defect \
             rather than a chooser one, and it would make the assertions below vacuous. Trace: \
             {}.",
            session.trace_path().display()
        )));
    }
    driver.click_at(session.frame()?.declared_center(combo))?;
    session.settle(26);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, POPUP_ADDABLE_REGION).is_none() {
        let shot = ctx.out("refused-character-face.no-addable.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        driver.press(vk::ESCAPE)?;
        return Ok(Some(format!(
            "★★ THE OFFER'S FONT LIST HOLDS NOTHING THE DOCUMENT DOES NOT ALREADY CONTAIN: the \
             chooser opened and declared no `{POPUP_ADDABLE_REGION}` region.\n\
             This fixture's one font is `{FIXTURE_FONT}`, which is none of the fourteen, \
             so `face::choices` should answer fourteen addable rows — and the page's own faces \
             cannot be the answer here by construction, because the block only exists because \
             they refused the character. An offer built from the page's own fonts is the offer \
             that cannot work. Regions beginning `{OFFER_FACE_REGION}`: {:?}. Trace: {}.",
            driving::live_names(&trace, ui_rect, OFFER_FACE_REGION),
            session.trace_path().display()
        )));
    }
    // ★ The disclosure a second time, inside the popup. Its absence here is a
    // different defect from its absence above: this one is `face::popup_body`'s
    // copy, shared verbatim with the *This text* section and the ribbon's Font
    // group, so losing it loses it in three surfaces at once.
    if declared(&trace, ui_rect, POPUP_DISCLOSURE_REGION).is_none() {
        driver.press(vk::ESCAPE)?;
        return Ok(Some(format!(
            "the chooser opened inside the offer and declared `{POPUP_ADDABLE_REGION}` without \
             `{POPUP_DISCLOSURE_REGION}`. `face::popup_body` draws them one after the other, so \
             a heading without its disclosure is a two-line edit that dropped one — and it is \
             dropped from all three surfaces that share that body. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let Some(row) = declared(&trace, ui_rect, POPUP_NEW_REGION) else {
        driver.press(vk::ESCAPE)?;
        return Ok(Some(format!(
            "the chooser drew the `{POPUP_ADDABLE_REGION}` heading and no `{POPUP_NEW_REGION}` \
             row under it, so the group is a caption over an empty band. Trace: {}.",
            session.trace_path().display()
        )));
    };
    driver.click_at(session.frame()?.declared_center(row))?;
    // ★ Waited for by its OUTCOME rather than by a frame count. A restyle costs
    // one provenance extraction per run — 392 ms on the operator's benchmark
    // sheet — and a fixed settle that was generous on this 1.7 KB fixture would
    // be a flake on a loaded machine, reported as *"the font list does nothing"*.
    // `std14_face::wait_for_verdict`'s shape, bounded the same way.
    let waited = wait_for(&session, &[STYLE_EVENT, "text-style-declined"], offer_at)?;
    report.note(format!(
        "the restyle answered after {waited} ms of wall clock"
    ));

    let trace = session.trace()?;
    let Some(styled) = trace
        .events(STYLE_EVENT)
        .filter(|l| l.lineno > offer_at)
        .last()
    else {
        return Ok(Some(format!(
            "★★ A FACE WAS CHOSEN FROM THE OFFER AND NOTHING HAPPENED: no `{STYLE_EVENT}` line \
             after the block drew.\n\
             `face::popup_body` returns the chosen selector and `refusedchar::section` turns it \
             into one `Action::TextStyle` OUTSIDE the popup closure, because nothing mutates from \
             a widget. A return value dropped between those two is exactly this symptom, and an \
             operator would report it as 'the font list does nothing'.\n\
             ★ Note the operand: this block sends `runs: vec![<the run the REFUSAL named>]`, not \
             the current selection — the caret is already gone by the time the block is on \
             screen, because `Ctrl+Enter` calls `commit_into` and then `abandon` whether or not \
             the engine accepted. A build that used the selection here would find none and act \
             on nothing. Trace: {}.",
            session.trace_path().display()
        )));
    };
    // ★ Copied out immediately, on `typo_refusal`'s pattern: the trace this
    // line borrows is about to be shadowed by a fresh capture, and a scalar
    // carried forward is easier to read than a borrow whose lifetime the reader
    // has to reason about.
    let styled_at = styled.lineno;
    let styled_raw = styled.raw.clone();
    if trace.last(FORMAT_APPLIED).is_none() {
        return Ok(Some(format!(
            "the offer computed `{styled_raw}` and no `{FORMAT_APPLIED}` line followed, so the \
             action was raised and its apply arm never ran. Nothing reached the document, which \
             from the operator's chair is indistinguishable from the list doing nothing. Trace: \
             {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the offer was TAKEN and it reached the document: `{styled_raw}` — `format_text` \
         wrote the `/Font` resource itself, in the same undo command"
    ));

    // --- 5: THE OFFER FINISHES THE JOB, WITH NO SECOND GESTURE ------------
    //
    // Nothing is clicked and nothing is typed from here on, and that is the
    // assertion. Until 2026-09-05 this step clicked the text again and pressed
    // `Ctrl+Enter` a second time, because that is what the operator had to do:
    // `Ctrl+Enter` calls `commit_into` and then `abandon` whether or not the
    // engine accepted, so the words he wrote were thrown away by the refusal
    // and he had to produce them again from memory.
    //
    // They travel with the refusal now (`canvas::textedit::Committing`), so the
    // block re-raises `Action::CommitTextEdit` itself on the frame the face
    // lands. A commit line after this point therefore has exactly one possible
    // author, because the harness has stopped touching the machine.
    let waited = wait_for(&session, &[APPLIED_EVENT, REFUSED_EVENT], styled_at)?;
    session.settle(12);
    let trace = session.trace()?;
    let landed = trace
        .events(APPLIED_EVENT)
        .filter(|l| l.lineno > styled_at)
        .last()
        .map(|l| (l.lineno, l.raw.clone()));
    let refused_again = trace
        .events(REFUSED_EVENT)
        .filter(|l| l.lineno > styled_at)
        .last()
        .map(|l| l.raw.clone());
    if landed.is_none() && refused_again.is_none() {
        let shot = ctx.out("refused-character-face.no-retype.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "\u{2605}\u{2605}\u{2605} THE OFFER DOES NOT FINISH THE JOB. The face was swapped \
             {waited} ms ago and **no commit of any kind followed** \u{2014} neither \
             `{APPLIED_EVENT}` nor `{REFUSED_EVENT}` \u{2014} although the harness pressed \
             nothing after the face was chosen.\n\
             Taking the offer must re-apply the edit the operator already typed. Three things \
             can break that and each leaves this exact silence: the refusal was recorded \
             without `canvas::textedit::last_commit`, so `RefusedCharacter::typed` is `None` \
             and there is nothing to retype; `plan` stopped writing that slot, so the words \
             were never kept; or `refusedchar::section` reached the `swapped` arm and pushed \
             no `Action::CommitTextEdit`. From the operator's chair all three are the font \
             list changing his letters and abandoning his edit. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "\u{2605}\u{2605}\u{2605} the offer RE-APPLIED the operator's own edit with no second \
         gesture \u{2014} a commit followed the face swap after {waited} ms and the harness \
         pressed nothing"
    ));

    // --- 6: which of the two outcomes, and both are accounted for ----------
    //
    // THIS IS WHERE THE ROUTE CURRENTLY STOPS, AND THE REASON IS THE ENGINE'S
    // \u{2014} measured, not inferred.
    //
    // `pdfcer-gui`'s `canvas::textedit::facewall` is one `EditSession` calling
    // `format_text` then `edit_text`, located by find text alone so that no
    // operand this shell computes is in the request. It is refused. The same
    // pair with a save and a reopen between them succeeds; a swap to a face the
    // page ALREADY carries succeeds at once. So the trigger is the `/Font`
    // object `format_text` creates, which lives in the session's overlay while
    // `edit_text` resolves font names against `self.base`
    // (`pdfcer-core/src/edit.rs`, `plan_edit(&self.base, ...)`). Filed as
    // `request_edit_text_resolves_font_names_against_the_base_revision.md`.
    //
    // So this step asserts the shell's half and NAMES the engine's, and it is
    // written so that the day the engine ships the fix this check goes green by
    // the better route and says so in its own notes. It cannot pass by
    // accepting any refusal: a second subset-floor refusal would mean the swap
    // never reached the run, which is the shell defect this file was written to
    // catch, and that is failed by name below.
    if let Some((landed_at, landed_raw)) = landed {
        report.note(format!(
            "\u{2605}\u{2605}\u{2605} THE CHARACTER WENT IN: `{landed_raw}` \u{2014} from a \
             refusal that named it, through a face the document did not contain, to an edit \
             that reached the page, in ONE gesture.\n\
             \u{26a0} If this is the branch you are reading, the engine has shipped the fix \
             for `request_edit_text_resolves_font_names_against_the_base_revision.md`. \
             `pdfcer-gui`'s `canvas::textedit::facewall::\
             the_engine_cannot_type_into_a_face_it_just_swapped_in` will be RED, and this \
             check should be tightened to REQUIRE this branch rather than accept it."
        ));
        if let Some(why) = offer_must_retire(ctx, &session, &trace, landed_at, &landed_raw) {
            let shot = ctx.out("refused-character-face.offer-survived.png");
            if crate::capture::window_to_png(&session, &shot).is_ok() {
                report.artifact(shot);
            }
            return Ok(Some(why));
        }
        report.note(
            "\u{2605}\u{2605}\u{2605} and the offer RETIRED for it: frames were painted after \
             the successful commit and none of them declared the block. The instrument speaks \
             for the character the font could not type and is silent for the one it could",
        );
        return Ok(None);
    }

    let refused_raw = refused_again.unwrap_or_default();
    if refused_raw.contains("R-INV-1") || refused_raw.contains("SUBSET") {
        return Ok(Some(format!(
            "\u{2605}\u{2605}\u{2605} THE FACE SWAP DID NOT REACH THE RUN THE REFUSAL NAMED. \
             The retype was refused by the **same embedded-subset floor** as the first \
             commit: `{refused_raw}`.\n\
             `format-text` reported that it landed, so the restyle went somewhere \u{2014} just \
             not to the run under the caret. The operand is what to read: \
             `refusedchar::section` sends `runs: vec![<the run the REFUSAL named>]`, not the \
             current selection, because the caret is gone by the time the block is on screen. \
             A build that used the selection here would restyle nothing, or something else. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    // The engine's own limit. The shell's obligation is to SAY so, off the
    // canvas, in the operator's terms \u{2014} and the block's third state is that
    // sentence. A block that stayed on `swapped` would be promising the
    // character is going in while it is not.
    let blocked = trace
        .events(OFFER_EVENT)
        .filter(|l| l.lineno > styled_at)
        .filter_map(|l| l.get("state").map(str::to_owned))
        .last();
    if blocked.as_deref() != Some("blocked") {
        return Ok(Some(format!(
            "\u{2605}\u{2605} THE RETYPE WAS REFUSED AND THE BLOCK DOES NOT SAY SO. The engine \
             answered `{refused_raw}`, and the last `{OFFER_EVENT}` state after the swap is \
             {blocked:?} rather than `blocked`.\n\
             `RefusedCharUi::retried` is what tells the two apart, and it does it by \
             arithmetic rather than by reading the engine's prose: the swap moves \
             `doc.edit_epoch` once, and had the retype landed it would move again and the \
             block would have retired. A block still drawing `swapped` is one promising the \
             operator that his character is going in, over a document where it did not \u{2014} \
             which is the confident small lie `app::status::decline`'s own header forbids. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "\u{2605}\u{2605} the retype was refused by the ENGINE \u{2014} `{refused_raw}` \u{2014} \
         and the block moved to `state=blocked`, which is the sentence naming the measured \
         cause and the remedy (save, reopen, type it once more). This is \
         `request_edit_text_resolves_font_names_against_the_base_revision.md`, filed \
         2026-09-05 with a copy-pasteable reproduction; `pdfcer-gui`'s \
         `canvas::textedit::facewall` asserts all three of its measurements, so the day it \
         ships this check takes the branch above instead"
    ));

    // --- 7: THE DYNAMIC RANGE, in the branch that keeps the block alive ----
    //
    // The oracle in steps 1-4 is *"a region was published"*, and a probe whose
    // baseline has no dynamic range cannot produce a verdict. Step 0b holds one
    // side of it \u{2014} the block is absent on a document nothing has been done
    // to, so a build drawing it unconditionally is already red. This holds the
    // other: the block has moved through THREE distinct states on its own trace
    // line, `offer` -> `swapped` -> `blocked`, each caused by a different thing
    // happening to the document. A block that never advanced could not produce
    // that sequence, and a stale-refusal block frozen on one state could not
    // either.
    let walked = states_walked(&trace);
    if walked != ["offer", "swapped", "blocked"] {
        return Ok(Some(format!(
            "\u{2605}\u{2605}\u{2605} THE BLOCK DID NOT WALK ITS OWN STATE MACHINE, so the \
             assertions above are not a verdict. `{OFFER_EVENT}` reported the state sequence \
             {walked:?}, and the three states this route passes through are `offer` (a \
             character was refused), `swapped` (the face landed and the operator's edit was \
             re-raised) and `blocked` (the retype came back refused).\n\
             A build that drew the block unconditionally, or never retired a stale refusal, \
             would satisfy every region assertion above for ever while producing one state \
             and never leaving it. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "\u{2605}\u{2605}\u{2605} and the instrument has dynamic range: the block walked \
         {walked:?} across the run, three states each caused by a different thing happening \
         to the document \u{2014} it is not a region drawn unconditionally"
    ));

    Ok(None)
}

/// The distinct `refused-char` states the run passed through, in order and with
/// runs of the same value collapsed.
///
/// Collapsed because the line is published on **every frame the block draws**,
/// so the raw sequence is hundreds of repetitions of three values and says
/// nothing a reader can use. What is being asserted is the block's path through
/// its own state machine, and a path is a sequence of transitions.
fn states_walked(trace: &crate::trace::Trace) -> Vec<String> {
    let mut walked: Vec<String> = Vec::new();
    for state in trace.events(OFFER_EVENT).filter_map(|l| l.get("state")) {
        if walked.last().map(String::as_str) != Some(state) {
            walked.push(state.to_owned());
        }
    }
    walked
}

/// **The negative control for the branch where the character goes in**: an edit
/// that SUCCEEDED must take the offer off the screen.
///
/// Split out because it is reached from one arm of [`drive`] rather than from
/// its end, and because what it does is one idea: *the block speaks for the
/// character the font could not type and is silent for the one it could*.
///
/// The absence is made non-vacuous first. A build that had simply stopped
/// repainting would publish no regions at all and would pass an
/// absence-of-region test for free, so the count of frames painted after the
/// commit is asserted before the absence is.
fn offer_must_retire(
    ctx: &CheckContext,
    session: &Session,
    trace: &crate::trace::Trace,
    landed_at: usize,
    landed_raw: &str,
) -> Option<String> {
    let _ = ctx;
    let frames_after = trace
        .lines
        .iter()
        .filter(|l| l.event == "ui-rect" && l.lineno > landed_at)
        .count();
    if frames_after == 0 {
        return Some(format!(
            "no `ui-rect` line at all after the successful commit, so the application drew no \
             frame this check can read and the absence asserted below would be vacuous \u{2014} \
             it would hold on a build that had simply stopped painting. Trace: {}.",
            session.trace_path().display()
        ));
    }
    if !published_after(trace, OFFER_REGION, landed_at) {
        return None;
    }
    Some(format!(
        "\u{2605}\u{2605}\u{2605} THE OFFER IS STILL ON SCREEN AFTER AN EDIT THAT SUCCEEDED, so \
         the positive half of this check is NOT a verdict. `{OFFER_REGION}` was published on a \
         frame after `{landed_raw}`, across {frames_after} painted frames.\n\
         `RefusedCharUi::advance` is not retiring: the state machine ends the report on any \
         edit-epoch change it did not itself cause, and the one it does cause \u{2014} the face \
         swap \u{2014} is consumed, so the retype after it must clear the block. A permanent \
         \"type it again\" under a document where the character has already gone in is the \
         same defect class as a decline that never retires, and `app::status::decline`'s \
         retirement rule exists to forbid exactly it. Trace: {}.",
        session.trace_path().display()
    ))
}

/// **Wait until one of `events` appears past trace line `after`**, and answer
/// how long that took in milliseconds.
///
/// ★ Bounded, and the ceiling is generous rather than tight: a wait that gives
/// up early reports a working feature as inert, which is the most expensive kind
/// of wrong this harness can be. On timeout it returns rather than erroring —
/// the caller's own assertion is what says which event was missing and what that
/// means, and it says it far better than a generic timeout could.
///
/// `after` rather than a whole-capture `last(..)` for this crate's standing
/// reason: a whole-capture search is a fossil finder, and every event this check
/// waits for has a predecessor it must be later than.
fn wait_for(session: &Session, events: &[&str], after: usize) -> Result<u128> {
    const CEILING_MS: u128 = 20_000;
    let started = std::time::Instant::now();
    loop {
        session.settle(4);
        let trace = session.trace()?;
        if trace
            .lines
            .iter()
            .any(|l| l.lineno > after && events.contains(&l.event.as_str()))
        {
            return Ok(started.elapsed().as_millis());
        }
        if started.elapsed().as_millis() > CEILING_MS {
            return Ok(started.elapsed().as_millis());
        }
    }
}
