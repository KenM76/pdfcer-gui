//! `the_inspector_is_one_master_detail_column` — Objects over Properties, in
//! the room the Tool panel used to take, with rows that fit.
//!
//! # What this is for — `OPERATOR_REQUESTS.md` **O123**, parts 3, 4 and 6
//!
//! > *"Objects and Properties become master–detail in one panel with a
//! > draggable split, ellipsis and tooltip on rows … I'd also like those one to
//! > appear in the space where the tool dock currently shown … Default dock
//! > width 360 px in Edit."*
//!
//! Three claims, and each of them is a layout claim, which on this project has
//! exactly one oracle: **a rendered screenshot.** A unit test can pin the
//! arrangement `app::modes::defaults` intends; nothing but a driven run can say
//! that the dock drew it.
//!
//! # ★★★ The five things it asserts, and why none of them is redundant
//!
//! | # | assertion | the build it fails on |
//! |---|---|---|
//! | 1 | Objects' body and Properties' body are **both** on screen | one behind the other in a tabbed stack, which is what "one panel" would become if somebody merged the two stacks into one |
//! | 2 | Objects sits **above** Properties, in the same x range | a side-by-side split, or the two in different columns |
//! | 2b | the detail pane holds **no document metadata**, *and* the document's own properties are a mounted tab | the arrangement the operator reported on 2026-09-05 — and, through the second half, every build that "fixed" it by drawing nothing |
//! | 3 | a **splitter** publishes between them | a fixed split — the thing part 3 says the pair must not be |
//! | 4 | **no row is elided at the width the dock opens at** on this fixture | the row form that shipped until 2026-09-05, on which every row was elided |
//!
//! ## ★★★ Why 2b is here rather than in a check of its own
//!
//! Because it is a claim about **this pane**, and this is the check that
//! already has the pane's rectangle in hand. The operator's report — *"the
//! document properties are still always visible in the properties tab"* — is
//! precisely a statement that something which is not the detail of the
//! selection was drawn inside the detail half of this master–detail column, so
//! the assertion belongs with the one that establishes the column exists.
//!
//! ⚠ **And it is a PAIR, deliberately.** The obvious assertion — *the metadata
//! region is not inside the Properties body* — passes on a build where the
//! Properties panel draws nothing at all, where the docprops panel failed to
//! mount, and where the dock failed to draw. So it is read together with the
//! presence of `dock.tab.file.document_properties`, which says the metadata
//! went **somewhere** rather than merely leaving. That shape — a negative
//! paired with the positive control that stops it being vacuous — is the one
//! this suite keeps having to relearn.
//!
//! # ⚠ NOT RUN by the session that last edited this file — 2026-09-05
//!
//! ★ Twice over now. It was **passing** before the document-properties move and
//! it has **not been re-run since**, for the same reason as before: the
//! machine's pointer and keyboard belong to another track and a driven run
//! cannot share them. Assertion 2b has therefore never executed once.
//!
//! The four original assertions are unchanged in substance; assertion 4's
//! **failure message** was rewritten because it named two causes that turned
//! out to be the wrong two. This check has **not been re-run against the fixed
//! build**: the machine's pointer and keyboard were held by another track, and
//! a driven run cannot share them. The headless half of the same question is
//! `panels::objects::tests::every_object_row_of_the_a1_sheet_fits_the_measured_pane`,
//! which passes.
//!
//! ## ★★★ What the first run actually found, and what its message got wrong
//!
//! It reported **`8 OF 8 OBJECT ROWS DO NOT FIT`** and offered two
//! explanations: *"either the width regressed to 320 or the rows grew."*
//! Neither was true.
//!
//! - The trace's own `objects-rows` line read `pane=314.0 overflow=473.6` —
//!   the widest row wanted **473.6 pt** where the pane offered **296 pt** of
//!   text room. Re-measured headlessly, the *narrowest* row of this fixture
//!   wanted **306.3 pt**. Every row was over, so no width was going to fix it:
//!   a dock wide enough is about 526 pt, half of an 1,100 pt window.
//! - And the pane was **not the default**. The same trace reads
//!   `mode-changed from=Some("read") to=edit remembered=true` — a **restored
//!   workspace**, which never consults `EDIT_INSPECTOR_WIDTH` at all. So the
//!   message's first candidate ("the width regressed to 320") named a constant
//!   the failing run had not read.
//!
//! ⇒ The fix was in the **row**: `panels::objects` draws a headline (index,
//! kind, the facts that identify the object, one disclosure mark) and hovers
//! the full description. See that module's header §1b.
//!
//! ★ The lesson for this file, and it is the suite's own recurring one: **a
//! failure message that lists candidate causes is a hypothesis, and it goes
//! stale exactly like a comment.** The rewritten message below names the
//! measurement (`overflow=` against `pane=`) instead of guessing, because the
//! trace already carries the number that decides between the candidates.
//!
//! ★ Assertion 4 is the one that needed a channel built for it, and the channel
//! is the application's own `objects-rows` line. That is the app marking its own
//! homework, and it is published anyway because the harness cannot read the text
//! a panel renders — there is no AccessKit reader, no OCR, no text extraction
//! from a screenshot. `SHELL_LAYOUT_PROPOSAL.md` §2.5 states the gap in as many
//! words.
//!
//! ## ★★ So it is made non-circular the way `read_mode_chrome` is: a pixel
//!
//! The **right-hand strip of the Objects pane** is sampled and must be
//! near-uniform — the panel's ground, with no glyph running into it. *"The rect
//! is exact and cheap and would be satisfied by a build that moved the canvas
//! without repainting anything; the pixels cannot be faked by an arithmetic
//! error."* An elision arithmetic that lied in the trace would still leave ink
//! against the pane's right edge, and this is the assertion that sees it.
//!
//! ⚠ Note the polarity, because it is the opposite of every other uniformity
//! assertion in this suite: here `is_uniform` is the **pass**. A strip of pane
//! that is *not* uniform is a row running off the edge.
//!
//! # ★ The fixture is pinned, and it is pinned for a stated reason
//!
//! `fixtures/a1-titleblock.pdf` — an A1 CAD sheet whose text objects carry
//! subset-tagged font names (`AAAAAA+SpaceGrotesk-Bold`). Those are the rows
//! that were being cut mid-character at 320 pt, and they are why
//! `SHELL_LAYOUT_PROPOSAL.md` §2.5 says *"pin a fixture whose text objects carry
//! subset-tagged names"*. On a document of short rows this check could not fail.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment, declared, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode O123 rearranged.
const MODE: &str = "edit";

/// The command that puts the dock back to this build's default arrangement.
///
/// Fired before anything is measured — see the block in `run` for why. Named as
/// a constant rather than inlined so the assertion that the reset landed and the
/// invocation that performs it cannot drift apart.
const RESET: &str = "view.reset_layout";

/// The environment variable that rings a command chain through the real
/// dispatcher at launch - no pointer, no keystroke.
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";
/// The fixture, pinned. See the module header.
const FIXTURE: &str = "fixtures/a1-titleblock.pdf";
/// The master's body compartment, as the dock reports it.
const OBJECTS_BODY: &str = "dock.body.view.panel_objects";
/// The detail's body compartment.
const PROPERTIES_BODY: &str = "dock.body.file.properties";
/// The region the document's own `/Info` form publishes.
///
/// ★ Named `properties.info` even though it belongs to
/// `crate::panels::docprops` now: the region name was deliberately left alone
/// when the section became a panel, and that module's `REGION` carries the
/// reason. **This check is the one that would notice if it drifted**, so the
/// constant is here and the string is written once.
const DOC_METADATA: &str = "properties.info";
/// The Document properties panel's tab in the dock's strip.
///
/// The positive control for assertion 2b. A mounted tab publishes this whether
/// or not it is the active one, which is what lets *"the metadata moved"* be
/// asserted rather than only *"the metadata is not here"*.
const DOC_PROPERTIES_TAB: &str = "dock.tab.file.document_properties";
/// The splitter between the right side's two stacks.
///
/// ★ Column 0, boundary 0 — the only stack boundary Edit's right side has, and
/// the name is structural rather than generated, which is what lets this check
/// re-read it instead of remembering a coordinate.
const STACK_SPLITTER: &str = "dock.right.0.split.row.0";
/// The line the Objects panel writes about the rows it actually drew.
const ROWS_EVENT: &str = "objects-rows";
/// How wide a strip of the Objects pane's right edge is sampled, in logical
/// points.
///
/// ★ Eight, not one. A single-point column can fall between two glyph stems and
/// report clean on a row that is plainly running off the edge; eight is about
/// one character and cannot.
const EDGE_STRIP_PTS: f32 = 8.0;

/// See the module documentation.
pub struct TheInspectorIsOneMasterDetailColumn;

impl Check for TheInspectorIsOneMasterDetailColumn {
    fn name(&self) -> &'static str {
        "the_inspector_is_one_master_detail_column"
    }

    fn defect(&self) -> &'static str {
        "the right dock does not show the object list over that object's properties in one \
         column with a draggable split, or it shows them at a width that cuts the rows, or the \
         detail pane is still carrying the document's own metadata under everything else — all \
         of which are invisible to every unit test, because the arrangement a module intends \
         and the arrangement a dock draws are two different facts"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks ONE thing — the mode segment — \
             and then looks. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    // ★ Pinned, and any `--pdf` is ignored: a document of short rows cannot
    // exhibit the defect, so a sweep's fixture would make this check unable to
    // fail — the vacuity `SHELL_LAYOUT_PROPOSAL.md` §2.5 names.
    let pdf = ctx.source_root.clone().unwrap_or_default().join(FIXTURE);
    let pdf = if pdf.exists() {
        pdf
    } else {
        std::path::PathBuf::from(FIXTURE)
    };
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is not on disk. This check cannot use an arbitrary \
             document: its rows have to be long enough, and carry subset-tagged font names, \
             for the width question to have an answer."
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("master-detail.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env.push((INVOKE_ENV.to_owned(), RESET.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {} on {}", session.pid(), FIXTURE));
    session.settle(40);

    let driver = Driver::new(session.window());

    // ★★★ **RESET THE LAYOUT FIRST, AND ASSERT THE RESET LANDED — 2026-09-05.**
    //
    // The application persists its dock arrangement to `userdata/layout.ron`,
    // and `Session::launch` does not clear it. So without this the check
    // inherits **whatever width a previous run left behind**, and the trace says
    // so plainly: `mode-changed … remembered=true` — a restored workspace, which
    // never reads `EDIT_INSPECTOR_WIDTH` at all.
    //
    // ⇒ This check's own header already spotted the symptom and corrected its
    // *failure message* to warn about `remembered=true`. It did not make the
    // check **hermetic**, so the warning was advice to a human reading a report
    // rather than a property of the run. A measurement of an inherited width is
    // a measurement of an earlier session's furniture.
    //
    // ★★ It is the same defect that made `panels_float_close_and_dock` fail for
    // days — four sections relaunching the binary, each inheriting what the last
    // one saved, and the headline number (`docked=0`) *honest* the whole time.
    // That check's fix is the precedent this follows: reset in the shared path,
    // and **assert the reset landed**, because a reset that silently did nothing
    // is indistinguishable from a clean start and turns every number below into
    // a claim about the wrong document.
    // Assert the reset LANDED before anything downstream is believed —
    // `panel_float::reset_landed`'s precedent, and its exact reason: a section
    // that skipped this and went on to report a width would be reporting the
    // application wrong for the check's own reason.
    {
        let pre = session.trace()?;
        let landed = pre
            .events("layout-reset")
            .last()
            .map(|l| l.raw.clone())
            .filter(|raw| raw.contains("changed="));
        if landed.is_none() {
            return Ok(Some(format!(
                "\u{2605}\u{2605}\u{2605} THE LAYOUT RESET DID NOT LAND. `{RESET}` was fired at launch and \
                 no `layout-reset ... changed=` line came back, so every width below \
                 would be whatever a previous run left in `userdata/layout.ron` rather \
                 than this build's default. A check measuring an inherited arrangement \
                 is measuring an earlier session's furniture - reported as a FAILURE, \
                 not a note, because a silent inheritance is indistinguishable from a \
                 clean start."
            )));
        }
    }

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(30);

    let trace = session.trace()?;

    // --- 1: both bodies are on screen ---------------------------------------
    //
    // ★ Both regions come through `crate::diag::ui_rect_visible`, so their
    // presence is a claim about REACHABILITY and not about layout. A panel
    // behind a sibling tab publishes nothing here at all.
    let Some(objects) = declared(&trace, ui_rect, OBJECTS_BODY) else {
        return Ok(Some(format!(
            "★★★ THE OBJECT LIST IS NOT ON SCREEN. `{OBJECTS_BODY}` was never published in \
             `{MODE}`, so either the Objects panel is not mounted or it is behind another \
             tab — and O123 part 4 puts it at the TOP of the side, in the room the Tool panel \
             used to take. Dock bodies that did publish: {}.",
            list(&driving::declared_names(&trace, ui_rect, "dock.body."))
        )));
    };
    let Some(properties) = declared(&trace, ui_rect, PROPERTIES_BODY) else {
        return Ok(Some(format!(
            "★★★ THE PROPERTIES PANEL IS NOT ON SCREEN ALONGSIDE THE OBJECT LIST. \
             `{OBJECTS_BODY}` drew at {objects:?} and `{PROPERTIES_BODY}` did not. That is the \
             failure master–detail is defined against: the detail must be visible WITH the \
             master, not one tab away from it. Dock bodies that did publish: {}.",
            list(&driving::declared_names(&trace, ui_rect, "dock.body."))
        )));
    };
    report.note(format!("master {objects:?}, detail {properties:?}"));

    // --- 2: the detail is UNDER the master, in the same column ---------------
    if properties.min.y < objects.max.y - 1.0 {
        return Ok(Some(format!(
            "★★ THE TWO ARE NOT STACKED. The object list ends at y={:.1} and the properties \
             begin at y={:.1}, so they are beside each other rather than one over the other. \
             O123 asks for master OVER detail; side by side halves the width that the same \
             request widened to 360 pt.",
            objects.max.y, properties.min.y
        )));
    }
    let dx = (properties.min.x - objects.min.x).abs() + (properties.max.x - objects.max.x).abs();
    if dx > 2.0 {
        return Ok(Some(format!(
            "★★ THE TWO ARE IN DIFFERENT COLUMNS. The object list spans x {:.1}..{:.1} and the \
             properties span x {:.1}..{:.1}. One column is what makes them one panel; two \
             columns is two panels that happen to be adjacent.",
            objects.min.x, objects.max.x, properties.min.x, properties.max.x
        )));
    }

    // --- 2b: ★★★ the detail pane is the detail of the SELECTION, and of
    // nothing else — the operator, 2026-09-05 -------------------------------
    //
    // > *"the document properties are still always visible in the properties
    // > tab. it needs to get out of there and be in its own document properties
    // > tab."*
    //
    // The file's own `/Info` form was drawn at the foot of this pane on every
    // frame, under everything, with no condition of any kind. It is
    // `crate::panels::docprops` now. This asserts that it is no longer INSIDE
    // the detail pane's rectangle.
    //
    // ★★★ **The negative alone is vacuous, and the pairing is the point.**
    // `properties.info` is published through `ui_rect_visible`, so a panel that
    // is mounted behind a sibling tab publishes nothing at all — which means
    // "the region is not inside the detail pane" is satisfied by a build that
    // deleted the metadata form outright, by one where the panel failed to
    // mount, and by one where the whole dock failed to draw. Every one of those
    // is worse than the defect being fixed.
    //
    // ⇒ So the tab strip is read for the panel's own tab. `dock.tab.<id>` is
    // published for a mounted tab whether or not it is the active one — the
    // strip draws every tab, only the BODY is the active one — so this says
    // *the metadata moved*, where the negative alone says only *the metadata is
    // not here*.
    if let Some(metadata) = declared(&trace, ui_rect, DOC_METADATA) {
        let inside = metadata.min.x >= properties.min.x - 1.0
            && metadata.max.x <= properties.max.x + 1.0
            && metadata.min.y >= properties.min.y - 1.0
            && metadata.max.y <= properties.max.y + 1.0;
        if inside {
            return Ok(Some(format!(
                "★★★ THE DOCUMENT'S OWN PROPERTIES ARE STILL IN THE SELECTION INSPECTOR. \
                 `{DOC_METADATA}` drew at {metadata:?}, inside `{PROPERTIES_BODY}` at \
                 {properties:?} — so the file's title, author, subject and keywords are on \
                 screen in the pane whose subject is what the operator picked, which is the \
                 report this assertion exists for. The metadata belongs to \
                 `{DOC_PROPERTIES_TAB}`."
            )));
        }
    }
    let tabs = driving::declared_names(&trace, ui_rect, "dock.tab.");
    if !tabs.iter().any(|name| name == DOC_PROPERTIES_TAB) {
        return Ok(Some(format!(
            "★★★ THE DOCUMENT PROPERTIES PANEL IS NOT MOUNTED. No `{DOC_PROPERTIES_TAB}` tab \
             was published in `{MODE}`, so the metadata form the operator asked to be moved \
             OUT of the inspector has not been moved anywhere he can reach — and the \
             assertion above passes trivially on exactly that build, which is why the two are \
             read together. Dock tabs that did publish: {}.",
            list(&tabs)
        )));
    }
    report.note("the document's own properties are a tab of their own, not a block in the detail");

    // --- 3: the split is DRAGGABLE, which means a splitter was drawn ---------
    let Some(splitter) = declared(&trace, ui_rect, STACK_SPLITTER) else {
        return Ok(Some(format!(
            "★★★ THE SPLIT IS FIXED. `{STACK_SPLITTER}` was never published, so there is no \
             handle between the object list and its properties and the operator cannot give \
             either of them more room. O123 asks for a DRAGGABLE split in as many words. \
             Splitter regions that did publish: {}.",
            list(&driving::declared_names(&trace, ui_rect, "dock.right"))
        )));
    };
    if !splitter.is_substantial() {
        return Ok(Some(format!(
            "★★ THE SPLITTER HAS NO GRAB AREA: {splitter:?}. A handle a pointer cannot land on \
             is a fixed split with a rectangle in the trace."
        )));
    }
    report.note(format!("the split is draggable at {splitter:?}"));

    // --- 4a: no row was elided, per the application's own report -------------
    let Some(rows) = trace.last(ROWS_EVENT) else {
        return Ok(Some(format!(
            "★★ THE OBJECT LIST PUBLISHED NO `{ROWS_EVENT}` LINE, so it drew no rows at all — \
             on a fixture chosen because it has many. Either the panel failed to decompose the \
             page, or the report was removed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let elided: usize = rows
        .get("elided")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| Error::new(format!("`{ROWS_EVENT}` carries no `elided=` count")))?;
    let visible: usize = rows
        .get("visible")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| Error::new(format!("`{ROWS_EVENT}` carries no `visible=` count")))?;
    // ★ `pane=` and `overflow=` go in the NOTE, not only in the failure
    // message, so a PASS records the margin it passed by. A run that passes at
    // 295 pt of a 296 pt room is one edit from failing and looks identical, in
    // a report, to one that passes at half the width.
    let pane = rows.get("pane").unwrap_or("?").to_owned();
    let overflow = rows.get("overflow").unwrap_or("?").to_owned();
    report.note(format!(
        "{visible} rows drawn, {elided} shortened; pane={pane} pt, widest shortened row \
         overflow={overflow} pt"
    ));
    if visible == 0 {
        return Err(Error::new(format!(
            "the object list drew zero rows on {FIXTURE}, so there is nothing to measure. \
             SKIPPED rather than failed: this check's subject is the width, not the \
             decomposition."
        )));
    }
    if elided > 0 {
        return Ok(Some(format!(
            "★★★ {elided} OF {visible} OBJECT ROWS DO NOT FIT. The pane offered {pane} pt and \
             the widest shortened row wanted {overflow} pt. \
             ★ READ THOSE TWO NUMBERS BEFORE FORMING A THEORY: if the overshoot is small the \
             row form grew a clause; if it is half as much again, something put the FULL \
             object description back on the row, which is the state that shipped until \
             2026-09-05 and which no dock width can fix (the widest description on this \
             fixture is 473.6 pt). ⚠ Do NOT reach for `EDIT_INSPECTOR_WIDTH`: check the trace \
             for `remembered=true` first — a restored workspace never reads it. \
             The shortened rows are still readable on hover, so this is a legibility failure \
             and not a lost capability. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 4b: …and the pixels agree, which is what makes 4a worth having ------
    let path = ctx.out("master-detail.png");
    let image = crate::capture::window_to_png(&session, &path)?;
    report.artifact(path.clone());
    let frame = session.frame()?;
    let strip = crate::geom::LRect::new(
        crate::geom::Pt::new(objects.max.x - EDGE_STRIP_PTS, objects.min.y),
        crate::geom::Pt::new(objects.max.x, objects.max.y),
    );
    let uniformity =
        crate::pixels::region_not_uniform(&image, frame.logical_to_capture_pixels(strip));
    // ⚠ Polarity: uniform is the PASS here. See the module header.
    if !uniformity.is_uniform() {
        return Ok(Some(format!(
            "★★★ INK IS RUNNING INTO THE OBJECT PANE'S RIGHT EDGE ({}). The application \
             reported zero shortened rows and the last {EDGE_STRIP_PTS} pt of the pane are not \
             the panel's ground, so something is being drawn past where the elision arithmetic \
             thinks the text ends. That is the case the trace channel alone cannot see, and it \
             is why this assertion exists. The capture is at {}.",
            uniformity.summary(),
            path.display()
        )));
    }

    report
        .note("★★ Objects over Properties, one column, draggable split, and every drawn row fits");
    Ok(None)
}
