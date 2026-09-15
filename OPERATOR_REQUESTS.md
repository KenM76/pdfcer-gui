# Operator requests — the standing backlog

Every request Ken makes of pdfcer-gui, with its status and its evidence. It is
read at the start of every session, alongside `PROJECT_PLAN.md`, and it is the
record — not a chat reply, not a session summary, not an agent's memory.

> **Ken:** *"Where do you need to put these requests so they just get
> auto-repeated over and over again so I don't have to keep requesting they be
> done over and over again?"*

**Here.** This file is the answer, and the next section is the contract.

## The contract

1. **Every request you make goes in this file, the moment you make it**, before
   any work starts on it. Not into a chat reply, not into a session summary,
   not into an agent's memory — into this file, which is in git, on disk,
   backed up, and read at the start of every session.
2. **Only you close a row.** I may move a row to *shipped-and-driven*; the row
   does not leave this file until you have used it and said so. A row I believe
   is done but you have not confirmed sits under **Shipped — awaiting your
   verdict**, not deleted.
3. **A row carries evidence, not a claim.** *"Done"* is not a status. The
   status is either a driven check by name, or a note saying exactly what was
   verified and how. If nobody drove it, it says NOT VERIFIED, in those words.
4. **A blocked row names what blocks it and where that is filed.** If it is an
   engine gap, the row names the file in
   `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`. A row that says
   "blocked" with nothing behind it is a row I have not done the work on.
5. **Nothing is silently rescoped.** If I ship half of what you asked for, the
   row stays open and says which half.
6. **A retraction stays beside what it retracts.** When a row turns out to
   have been wrong — the defect was mine, the capability was already there,
   the measurement was of something else — the row is marked RETRACTED and
   stays exactly where it is, with the correction under it. Deleting it
   leaves the next reader free to re-derive the same wrong conclusion from
   the same evidence, and leaves you unable to see that I noticed. O115 is
   the shape to copy.

A request made in conversation lives exactly as long as the conversation:
sessions end, context is compacted, and an ask made in turn three is gone by
turn thirty. A file in the repository has none of those properties, and its
history is in git, so a row cannot quietly disappear.

## Standing expectations — these govern every row below

**A request is scoped to the whole expected behaviour, not to its literal
words.**

> *"I think that is how we are ending up with a gui that when I ask for
> something I get a very narrowly scoped part of what I wanted. Whereas when I
> ask for something, my expectation is usually that everything surrounding that
> request is also done to where it would match the behaviour a user would
> expect. Otherwise I am left typing out every little missing detail."*

The same rule governs what is asked of the engine: *"not adding such things
just because they weren't explicitly asked for i think is how we end up with
partially finished features."*

**A report is a sample, not the scope.**

> *"Please don't just fix the bugs and add the features for the exact tools I am
> outlining. You need to do a proper sweep and diagnosis to ensure all tools and
> features."*

**And believe the report before scoping it.** A report that sounds like a
misunderstanding is still a report, and the ones that sound least precise
have been the most precise. *"I can’t delete an object"* and *"text editing
is weird"* each resolved to one locatable bug. The worst instance: *"text
editing doesn’t work"* was true of most of the text on his own drawings
while every driven check in the harness was green — because the checks drove
fixtures this repository authored, and those fixtures did not have the
property his documents have. A green suite is evidence about the fixtures.
It is not evidence against him.

**The basics get audited as basics.**

> *"We'll have to reconsider how you are going about the canvas later since it
> shouldn't take multiple 3 hour sessions each day to figure out how to get a
> cursor to move and edit text on it, or get shortcuts to work for basic
> functions."*

Every check that asks *"does the thing I built work?"* and none that asks
*"does the thing everyone expects exist?"* is how `Ctrl+P` shipped unbound and
a text caret shipped with no index. The keymap now has a list-shaped gate for
exactly that. **The canvas needs the same treatment and does not have it.**

# OPEN

## O177–O189 — `FEATURE.txt` — thirteen rows from one file, FILED BEFORE ANY WORK

**Source:** `C:\Users\Ken\OneDrive\pdfTests\FEATURE.txt`, written, read the same morning. Every paragraph in it became a row below, in the
order he wrote them, **before any of it was measured** — the rule of this file
is that a request is written down when he says it, not when the work lands.

**Nothing below has been measured yet unless its own row says so.** A row
that says FILED means exactly that: his words are recorded and nothing else.

## O177 — ◑ **BUILT AND DRIVEN** — "the page or pages view should snap back to center of the canvas", and fit-page in two-up should fit BOTH pages

> *"when switching the view from scroll pages to show one page at a time or
> show two pages side by side the page or pages view should snap back to center
> of the canvas. also fit page when in 2 pages side by side views should fit the
> two side by side pages onto the canvas - right now it snaps to fitting one."*

Both are shell work. Neither is filed with the engine.

Driven by `switching_the_page_display_recentres_and_a_facing_fit_fits_the_spread`,
written and run against the broken build first, so the pass is a measurement.

## O178 — ◑ **FILED** — tabs between windows, a tab torn into its own window, and a right-click menu that offers it

> *"it might be a limitation of rust, but with several windows of pdfcer-gui
> open we should be able to drag tabs and items between them as though they were
> tabs within the same window. Also tabs should be able to be dragged into their
> own new window or dragged to be moved onto the tab space. right-click on a tab
> should also have the option to open in its own window."*

## O179 — ◑ **BUILT AND DRIVEN, AND IT WAS A REPORT AGAINST O163** — the search zoom checkbox is unchecked and it still zooms

> *"In the search options when I unselect the zoom option, instead of jumping to
> the page that the text is found on and leaving the page in its current
> position on the canvas it still zooms and repositions the page on the canvas."*

**O163 is the row that built this checkbox** and it is marked *"BUILT, NOT YET DRIVEN"*. He has now driven it and it does not do what it
says. This is not the first time a row marked built-but-not-driven turned out
not to work; the pattern is the row, not the feature.

## O180 — ◑ **BUILT AND DRIVEN** — a trailing space pasted from Excel stops the search finding the text

> *"trailing spaces/tabs/etc stops a search from finding text on the page that
> doesn't have these symbols - this should be an option in the settings to
> include or exclude such items in the search. copy pasting from excel seems to
> give a trailing space that I have to remove to search."*

## O181 — ◑ **FILED** — the Add Text font list does not offer installed fonts, and the Format ribbon's font controls look dead

> *"The dropdown for font selection when adding text doesn't show installed
> fonts as an option. Also it looks like the font ribbon commands under the
> format ribbon menu aren't enabled."*

## O182 — ◑ **FILED, LIKELY AN ENGINE ROW** — white gaps between the image tiles of a colour rendering

> *"OneDrive pdfTests TR-0180__KUBOTA RTV X1130 EMS ROPS FOPS CONCEPT.pdf
> renders with white gaps between the images that make up the colour rendering.
> these gaps don't appear in acrobat."*

## O183 — **FILED** — the dimension row: units, tolerance, live preview, leaders, arrows, and a comment box that should not be there

> *"for a dimension group, I can't set the units to fractions or any of the
> related settings after creation in the properties. This feature also doesn't
> do anything when I override it for a specific dimension. Also the tolerance
> setting is missing for the group, and the setting doesn't work for the
> override. there isn't the same amount of control over how dimension tolerances
> are handled the way there is in Solidworks. There is also not a live preview
> when I make or try to move a dimension, and it is hard to tell when I am
> moving the dimension text itself vs the dimension line. When I click on a
> dimension the comment box pops up, but it isn't needed for these, unless it
> were to contain all of the editing options, and even then it would be nice to
> have a setting to turn it off and just edit from the properties tab. When I am
> creating or selecting an existing radius dimension there is no way to change
> leader location, style of leader (should have the same options as solidworks
> and glyphs to match), or whether it is a radius or diameter, and whether the
> center mark is shown, and how it looks (like solidworks has). Again, it should
> have a live preview, be easy to select the dimension text or the leader to
> relocate. Also we should have control over how arrows are displayed."*

Surveyed in `DESIGNS.md` → *O183 — the nine-part ce-dimension request*, which
takes each clause in turn with the call sites. **Four of the nine are already
shipped, so that much of the report is a discoverability finding rather than a
gap.** Read the survey before scheduling any of it; the table below indexes it
and does not replace it.

| his clause | verdict |
|---|---|
| units cannot be changed after creation | **partly false**, with a different defect underneath |
| the per-dimension override does nothing | **false as a capability claim** — masked by the comment box, clause 7 |
| tolerance missing from the group | **true**, and a written decision rather than an oversight |
| tolerance does not work on the override | **false** — the same mask |
| thinner tolerance control than SolidWorks | a scope question for you, not a defect |
| no live preview, and text versus line is unclear | first half **false**, second half **real** |
| clicking a ce dimension pops the comment box | **the keystone** — it is what hides four of the above |
| radius ce dimensions have no controls | **two shipped, two absent** |
| arrow display is not controllable | **false** — controllable on both tiers |

## O184 — ◑ **BUILT, DRIVEN AND FALSIFIED** — the drag preview outline grew with zoom until it was the width of the canvas

> *"The live preview blue outlines that appear when we drag and object scale
> with zooming in and out of the page instead of being independent of zoom - at
> high zoom levels they end up being the width of the canvas. I think they keep
> the same size as the line widths they are moving and that is ok- but if we set
> the line width view to the one pixel width option the preview lines should
> also be affected by this setting."*

Driven by `preview_width_ignores_zoom`, and **falsified twice against
deliberately re-broken builds, once per ruling** - because a check that has
only ever been green is not evidence.

## O185 — ◑ **BUILT AND DRIVEN — PARTS 1 AND 2; PART 3 HE WITHDREW HIMSELF** — Print must remember, and must have a Cancel that actually reverts

> *"The Print Dialogue window should remember the last settings that were used
> when we press print or close. We should add a cancel button that doesn't save
> the changes we made since opening the dialogue and they revert back to what
> they were when we opened the print dialogue before cancelling. if possible it
> would be great to have another button to save settings with pdf, but I imagine
> that isn't something that is supported by pdf, and if it would make it a
> special setting that only works with our program, then don't implement it."*

**Driven**, two launches against the real binary: change the paper policy, leave
by Cancel, reopen — the window opens on the original; change it again, leave by
*Keep and close*, reopen — the window opens on the change. The check is
`the_print_window_forgets_what_cancel_undid`, and it was falsified three ways -
Cancel wired as Keep, Keep wired as Cancel, and the two swapped — each going red
with the sentence that names which mis-wiring it found.

## O186 — ◑ **BUILT AND DRIVEN — CLAIMS 1 AND 3; CLAIM 2 IS A MEASUREMENT NOBODY HAS MADE, AND IT IS THE SAME SUBJECT AS O174** — the cursor still jumps at deep zoom, and the raster error should be a stop, not an error

> *"At deeper zooms I am still experiencing the cursor jumping, and at some
> point it sometimes repositions to where the object area I was zooming into is
> no longer on screen. Maybe some other variables have to switch to 64 bit at
> night level zoom? I think this sometimes results in similar error to 'This
> page could not be drawn. requested raster size 50411508x32619210 is empty or
> exceeds MAX_PIXMAP_EDGE'. perhaps the zoom is fine, but the cursor has jumped
> somewhere unsuported. If this error is caused by some other limitation that
> will always happen, zoom should stop at the limit and not end up showing an
> error - the canvas will just stop zooming in and can still function. the error
> can still be shown on the bottom bar so the user has some idea as to why
> zooming stopped short of 1 trillion percent."*

## O187 — ◑ **BUILT AND DRIVEN, AND IT WAS A DEFECT AGAINST O151** — the page-preview timeout must be remembered, and 0 must mean never

> *"the draw page previews timeout needs to be remembered, and setting it to 0
> should set it to infinity (never time out)"*

**O151** built *"the drawing page previews checkbox should never automatically
turn off"* and is marked NOT YET DRIVEN. This adds two things it
did not do.

## O188 — ◑ **FILED** — the text in a title block is one lump; he wants the pieces

> *"In text that is grouped together or whatever it is called, such as in my
> title blocks, I would like a way to move the individual text blocks within it
> around, and have the ability to delete them like I can when I add text using
> our Add text tool."*

| his ask | verdict |
|---|---|
| **delete** one piece | **Already shipped.** `delete_text_run` is wired, driven, and measured on his own sheet (18 runs → 17, page objects unchanged). He could do this the whole time. |
| **move** one piece | — **The engine has no verb.** Every other part kind has both halves; text has the delete and not the move. Filed as `G017`. |

Checks: `the_right_click_offers_the_line_you_clicked`.

## O189 — ◑ **FILED** — dragging pages between documents leaves the bookmarks behind

> *"Dragging pages from one open pdf to another doesn't transfer the
> bookmarks."*

## O191 — ◑ **BUILT, FILED LATE THE SAME DAY** — the GitHub landing page should read like something written for a person

> *“please make the git landing page for pdfcer-gui more ordinary human
> readable. Sell people on the features and be concise about each one.”*

**This row is late, and the lateness is the finding.** The work was done
within the hour he asked for it — `README.md` was rewritten from an
engineering status page into a features-first landing page — and then nothing
was written down, because the work getting done is exactly why nothing looked
wrong. A request that is satisfied but unrecorded is indistinguishable, from
the register, from a request that was never made; and the register is what a
cold session reads to know what he has asked for. **Write the row when he
speaks, not when the work lands.**

## O190 — ◑ **MEASURED, NOT SOMETHING YOU ASKED FOR** — a freshly opened document draws one blank frame before the page appears

**Severity: low, and stated as low on purpose.** It is one frame —
roughly 16 ms at 60 Hz — and it happens while a document is opening, when the
window has just been resized and painted anyway. Nobody has reported it,
including you. It is filed because it was *measured*, and an observation that
goes into a chat reply instead of this file is an observation that will be
rediscovered.

## O192 — ◑ **FILED; BUILT AND DRIVEN (, shipped in `v0.5.0-dev.20260913.4`) — NOT CLOSED, that is yours** — the Set Scale dialogue never tells you what the scale is

**Driven:** `set_scale_reads_the_group_it_is_about_to_overwrite`, ten steps
through the real binary. Its assertion is a **count** — exactly one
window constructed across the whole calibration — because *"the window is back
afterwards"* is satisfied by the broken build as well as the fixed one and would
have passed quietly for the defect's entire life.

Cited: `dialogs/scale.rs:183-199`, `dialogs/open.rs:274-282`, `canvas/measure/scale.rs:204-209`, `dialogs/page_size.rs:251-288`, `dialogs/export_dxf.rs:110`, `app/frame.rs:1186-1198`, `scale.rs:220-236`, `panels/dimension_groups/mod.rs:550`.

## O193 — ◑ **FILED; BUILT AND DRIVEN (, shipped in `v0.5.0-dev.20260913.4`) — NOT CLOSED, that is yours** — the Set Scale window has no way to choose which dimension group you are scaling

> *“Also in the set scale window there is no drop-down to select the dimension
> group that I am setting the scale for.”*

**The dialogue targets a group implicitly and never says which one.** With more
than one dimension group in a drawing — which is the normal case on a sheet
carrying details at different scales — there is no way to tell the dialogue
which group the scale is for, and no way to tell from looking at it which one it
picked.

Cited: `panels/dimension_groups/identity.rs:253-254`, `panels/properties/dimension/mod.rs:163-171`, `panels/dimension_groups/identity.rs:172`, `dialogs/scale.rs:506-513`, `canvas/measure/mod.rs:390-392`, `panels/dimension_groups/mod.rs:590-592`, `app/frame.rs:1206`, `scale.rs:124-130`.

## O194 — **FILED** — units everywhere, including kilometres and miles, and the places that still offer only points

**A limitation sentence is a citation with an hours-long shelf life.** This
is the second one in a single morning. Anything written in this file of the form
*"blocked on the engine"* is a dated observation about a repository that answers
in minutes, and must be re-measured before it is quoted, never inherited as a
standing property.

**Driven, not merely tested** (R1). Eleven checks against the release binary,
each on its own copy under its own profile: the page-size round trip through a
saved file, both New-document surfaces, both image-placement surfaces, both
print surfaces, document properties, the load-anomaly panel, the metafile
export, and a drawing dropped onto the thumbnails. **All eleven pass.** Two of
them reported SKIP on the first attempt; both were my fixture choice — a
one-page primary document handed to checks whose whole evidence is a page count
changing — and both pass on a four-page primary. Neither was an application
defect, and the harness said so in its own words rather than going green.

- **Step 4 is half-done and therefore worse than not started**: km, yd and mi
 have `ui_text` entries for their abbreviations; mm, cm, m, in and ft still do
 not. A catalog that is inconsistent is a catalog nobody can audit.
- **Step 5 is not done.** There is a driven check per *surface the conversion
 work touched*, which is not the same thing as a driven check per *surface that
 offers a unit menu*. The second is what the row asks for.
- **The ~30 surfaces with no unit control at all still have none.** Step 2 made
 them agree with each other; it did not give the operator a choice. The
 properties panel still says *"Points, measured to the bottom-left corner"* --
 the exact sentence he was describing when he filed this.
- **The type-size surfaces were deliberately left un-annotated.** The gate never
 flags them, so a marker it does not read is decoration, not enforcement. The
 reasoning is written into the gate's header as a documented hole, next to the
 two others: a bare `/ 72.0` with no `25.4` beside it, and a positional
 `{:.0}` whose argument is a millimetre.

Cited: `text/panels/properties.rs:415-417`, `panels/pages/mod.rs:203`, `panels/docprops/mod.rs:538`, `text/new_document.rs:180-197`, `app/fontband.rs:356`, `panels/properties/text.rs:748`, `panels/properties/tool.rs:212`, `text/forms/mod.rs:583`.

## O195 — ◑ **FILED** — smart select is not available in Review mode

**The driven check to extend, not duplicate:** `tools/ui-verify/src/checks/smart_select.rs`
hard-codes `const MODE: &str = "edit"` at `:100`. Parameterise it.

Cited: `canvas/input.rs:195`, `canvas/clicking.rs:592`, `canvas/textsel/gate.rs:272-273`, `canvas/clicking.rs:444-449`, `gate.rs:338-341`, `app/frame.rs:430`, `canvas/smart.rs:146-149`, `app/conditions/armed.rs:146-147`, `app/modes/capability.rs:20-23`, `capability.rs:116-130`, `clicking.rs:572-584`, `input.rs:196`, `app/conditions/mod.rs:759-761`, `app/dispatch.rs:1397`, `capability.rs:32-45`, `shell/commands/tests.rs:274`, `shell/commands/catalog/view.rs:338`, `shell/manifest/view.rs:271`, `shell/manifest/rail.rs:261`, `app/dispatch/navigate.rs:121`, `rail.rs:431-466`, `MODES_AND_PANELS.md:554-564`, `left_rail.rs:277`.

## O196 — ◑ **FOUND, not yet reported by the operator** — three export windows forget every setting, and nothing in the program remembers them

**The precedent is already built and was built for exactly this.** Operator
request **O166** produced `PrintDialog::open(doc, remembered: &PrintPrefs)` and
the driven check `the_print_window_opens_on_the_settings_you_last_used`
(`tools/ui-verify/src/checks/print_remembered.rs`), whose header reads: *“Before
that day `PrintDialog::open` built every field from a literal, every single
time.”* **That sentence is currently true of three other windows.** Port
`PrintPrefs` to `ExportPrefs` and port the check; this is a known shape, not a
design problem.

| what | where |
|---|---|
| twelve export preferences, read and written | `app/prefs/exporting.rs`, `ExportImagePrefs` / `ExportTextPrefs` / `ExportDxfPrefs` |
| each window seeded from them | `dialogs/export_image.rs`, `dialogs/export_text.rs`, `dialogs/export_dxf.rs`, each `open(doc, remembered)` |
| each window traces the values it **built with** | `export-image-open`, `export-text-open`, `export-dxf-open` |
| the driven check | `tools/ui-verify/src/checks/export_remembered.rs` |

**Driven and falsified** against the release binary over
`fixtures/a1-titleblock.pdf`, `--no-input`: twelve of twelve came back, and a
narrow sabotage of one window's constructor produced `RESULT: FAIL` naming that
window's four rows and nothing else. The check needs no mouse at all — it
opens all three windows in one launch through `PDFCER_DIAG_INVOKE` and reads
three trace lines — so it is the cheapest driven check in the suite and runs
on a machine whose desktop is in use.

Cited: `dialogs/export_image.rs:270-295`, `dialogs/export_text.rs:137-158`, `dialogs/export_dxf.rs:116`.

## O197 — **FILED** — the landing page sells pdfcer short by reading as a CAD tool, when the PDF compatibility is the bigger feature

> *"I think the readme and the git landing page is selling pdfcer-gui short by
> making it sound like it is geared just towards cad drawings. For example we
> spent a huge amount of time making support for CYMK PDFs to the point where,
> and we can't state this, [the industry print-conformance suite]'s test PDFs
> open as well as they do with acrobat reader, and those have nothing to do
> with cad. In fact I would say our pdf compatability is one of our biggest
> features."*

**One name in that quotation is masked, and the masking is the
operator's own instruction being obeyed one step earlier than he meant it.**
He named the licensed print-conformance suite;
`tools/check-suite-name-absent.py` keeps that name out of this public
repository entirely, contents and file names both (operator ruling). Transcribing his sentence verbatim turned that gate red within a
minute of this row being filed — which is the gate working, and it is
recorded here rather than quietly repaired because the next person to write
down something he said out loud will be one keystroke from the same place.
The private map directory's `manifest.json` names the suite in full.

> *"and we can't state this"*

## O198 — ◑ **FILED, AND IT SUBSUMES O181 AND O188** — every piece of text on the SolidWorks drawing must be editable, and the font/bold/italic controls and the properties fields are dead

> *"I really need you to focus on finding ways to make all text editable on the
> sw drawing that is in pdftests folder. Find a way to make it happen. Seems the
> reflow works with each line but still can't edit when the text has been
> reflowed. Also get the font selector and editing tools like old and italic
> working. That entire area is always greyed out in the menu, and the properties
> area is uneditable too. This is true even when I add a new line of text."*

*"Seems the reflow works with each line but still can't edit when the text has
been reflowed."* The clause that turned out to carry the finding is **"seems"**:
reflow works near the top of a sheet and stops working further down, which is
exactly what a numbering that drifts produces.

**(B) The engine's, and FILED as `request_G015`.** With the right index, page 0
still refuses - with *"text was added to this page this session... save and
reopen before reflowing this page"*, **on the first frame after you open the
file, before you have touched anything.** The guard tests a structural property
(does the page have more than one content stream) and SolidWorks wrote that
sheet with **eight**. Measured across the set: 1 of 36 sheets. Two things are
wrong with it - the sentence blames you for something you did not do, and its
remedy cannot work, because the streams are in the file and survive a save and
reopen. There is an accidental workaround, and it is absurd enough that I will
not offer it to you: making one unrelated text edit on the page collapses the
eight streams into one, after which reflow is allowed. Filed with the
measurement, the guard's source, and a preferred fix that keeps the protection
the guard was written for.

*"That entire area is always greyed out in the menu, and the properties area is
uneditable too. This is true even when I add a new line of text."* Three dead
surfaces, and they were never three faults. **Every one of the five Font
controls, every Properties text field and every restyle verb in this shell is
gated on the same condition: is a TEXT object selected.** A pick that cannot
produce a text selection switches all of them off at once - which is exactly
how a capability that is registered, enabled by its own conditions and green on
every pinned fixture reaches you as *"that entire area is always greyed out"*.

**What was measured.** Nine aims on page 1 of your drawing, one per distinct
font size on the sheet (5, 6, 8, 9, 10, 11.8, 12, 13.2 and 16 pt), each aim two
units inside the run's own box, driven against the real binary.

## O199 — ◑ **FOUND, NEEDS YOUR VERDICT — nothing has been changed** — not something you asked for: the Save button no longer looks like the three controls beside it

You did not ask for this row. It is here because the File tab's Save group now
carries four controls in a row — **Save**, **Save as…**, **Save a copy…** and
**Save compacted…** — and the last three were drawn as a family while the first
was not. Which way that should go is a taste call about the control you press
most often, so it is a question rather than a change.

**The three share one body and differ by one interior mark each.** A square
sheet with a shutter across the top, then a pencil for *Save as* (a new name), a
second body showing behind for *Save a copy* (a second file), and an arrow that
stays inside the body for *Save compacted* (the same file, smaller). That
grammar is right, and none of the three can be read as either of the others at
the 16 px they ship at.

**Save's glyph is a different shape entirely** — a rounded body with a cut
corner and one label bar, no shutter and no interior mark. So the family is
correct and Save is now the odd member of it.

**It was left alone deliberately, and the reason is on the record rather than
left to be rediscovered:** Save is the bare one, the press you make fifty times
a day without reading the label, and at 16 px those interior marks are the only
thing separating four otherwise identical icons. Redraw Save onto the shared
body and there are four near-twins where there are now three and a distinct one.

**Two ways to go, and nothing is blocked either way.** Leave Save as the odd one
and let its outline keep doing the work of telling the most-used control apart;
or redraw it onto the family body and let the interior marks carry the whole
distinction. It is the picture on your Save button, so you should see the change
before it ships rather than after.

## O176 — ◑ **MEASURED, NEEDS YOUR VERDICT** — not something you asked for: on a big sheet at fit zoom, a small form field is all grip and no body

You did not ask for this row. It is here because a full drive of the shipped
build found something you would hit on an ordinary A1 sheet and would have
no way to name, and because the answer is a taste call about your own
drawings rather than an engineering one. **Nothing has been changed. This is
a question.**

Two separate checks, both driving the real program, both stopped for the
same reason.

## O175 — ◑ **BUILT, DRIVEN AND FALSIFIED** — "in our view ribbon area we need an option to show the stuff that is off page or not"

> *"release asap with this one small change before you continue work in the
> rest: in our view ribbon area we need an option to show the stuff that is off
> page or not (and when not showing the stuff that is off page there shouldn't
> be a gap between pages where the stuff is, so it just goes back to looking
> before we added the view things that are off the page feature). by default,
> read doesn't show off page items, review and edit do show off page items.
> these settings can be changed by the user and their preference is remembered
> for each read review edit modes."*

Checks: `the_off_page_toggle_is_per_mode_and_remembered`.

## O174 — ◑ **MEASURED AND FIXED, NOT YET DRIVEN BY A CHECK** — "this pdf `A-591.pdf` causes problems zooming past about 1600% - the view appears to jump to another location and when I pan back to where something is visible it appears to be distorted."

**They are filed as one row because you saw them together, and separated in
the analysis because they fail in different places.** This project has a standing
lesson about exactly this shape — *"and at other junctions too" is the
load-bearing clause* — from the day one reported symptom at one zoom turned out
to be seven distinct causes. Nothing here may be called closed because the first
of the two reproduces and gets fixed.

### Status

## O173 — ◑ **BUILT AND DRIVEN** — "we should have an easy way to make pdfce-gui our default opener for pdfs. Ask once with a don't show me again check box option. Then it should be in the top of our settings as a button to execute the changeover." <!-- old-name-exempt: HIS words, quoted verbatim. He typed `pdfce-gui`; correcting an operator's own sentence inside the row that records it would stop the row being a quotation. -->

* **The driven check is WRITTEN and has NOT BEEN RUN**, because the machine was
 yours all session. It is
 `tools/ui-verify/src/checks/default_app_offer.rs`, registered in the roster,
 and it drives five phases: delete the sandbox's seeded preference, launch with
 nothing open, assert the offer's body and **all three** of its controls are
 declared, capture the window, tick the box and press *Not now*, then **launch
 the same profile a second time and assert it does not ask again**. That last
 phase is the half of *"ask once"* no unit test can reach — the unit tests prove
 the preference is written; only a second launch proves it is read back.
 Until it has actually run, this row stays ◑.
* **It asserts the two buttons by NOT guessing a size.** Both buttons and the
 checkbox publish their rect only when it is inside their own clip rect, so the
 check asking "was it declared?" *is* the check asking "was it on the screen?" —
 which is your O171 sentence, mechanically, in a second window.
* **It will never press the affirmative button.** That button writes ten registry
 values on whatever machine runs the sweep and opens Windows' own settings page
 in front of whoever is at it. The check drives the decline route only.
* **One consequence worth knowing about:** every ui-verify sandbox is a fresh
 profile, so the offer would have opened in front of **every other driven
 check in the sweep** and taken their pointer presses. The sandbox now writes one preference to decline
 it in advance — which is why the driven check's first act is to delete that
 file, and it says so in three places.
* **A fourth region was added while writing the check**, and it is not
 bookkeeping: *Not now* was the one control in the answer row that no harness
 could see. Your sentence was plural — *"those buttons should always be
 available"* — and a check that could see one of the two would have reported the
 row as reachable on a build where half of it had been clipped away.

## O172 — ◑ **BUILT AND DRIVEN** — "We also need to make it easy to add our own custom stamps and use them, preferrably exactly the same way acrobat does."

**That was filed with the engine and the engine answered the same day.**
Placing is built, and it is driven end to end by
`custom_stamp_reaches_the_page`: a planted stamp collection, the real pointer
on the gallery, a drag onto a drawing, and the artwork asserted on the page.
Your sentence is answered in all three of its parts — add, use, and the same
route Acrobat uses.

*"Exactly the same way Acrobat does"* is the useful part of this row, because
it names the whole expected behaviour rather than one verb: in Acrobat a custom
stamp is a **menu entry beside the standard ones**, grouped by the category it
was authored under, chosen the same way, placed the same way, and it remembers
the last one used. That is the target — not "a second, different route for
stamps you made yourself".

**Built to that description, not to the literal ask.** Your stamps appear in
the same gallery as the standard ones, under the category you authored them
into, picked with the same click and placed with the same drag. There is no
separate "custom stamp" route, because a separate route is the failure this
paragraph names.

## O171 — ◑ **BUILT AND DRIVEN** — "After I place the first stamp and go to make a second one the window for the options pops up but is undersized so I can't see the add or cancel button. Those buttons should always be available, and if there isn't size for all the features they get scrolled in their own space."

> *"Those buttons should always be available, and if there isn't size for all
> the features they get scrolled in their own space."*

Checks: `the_second_stamp_dialog_still_has_its_buttons`.

## O170 — ✅ **ANSWERED AND ACTED ON** — "the stamp icon gap - are you using the latest engine? It just did some work on stamps."

**Your words, mid-session.** A question, not a request, and it is
filed anyway because the answer changed the build.

## O169 — ◑ **ALL THREE HALVES BUILT AND DRIVEN; the placing half stopped being blocked and is now driven by `custom_stamp_reaches_the_page`** — "if acrobat has a way of adding custom stamps or text, we need the same feature too with the same import/export to make the stamps as Adobe has and is compatible with adobe's"

**Your words.** Filed here today because it has become GUI work — the engine
half landed as `Pass 288.0` and is already inside the pin we ship from, so the
next move is ours. Filed before any of it was built, per rule 1.

**Placing a custom stamp onto a drawing from inside pdfcer now works.** The
engine shipped the artwork-import verb the same day the request was filed, and
the shell drives it: `tools/ui-verify/src/checks/custom_stamp.rs` —
*`custom_stamp_reaches_the_page`* — plants a stamp collection under a
redirected `%APPDATA%`, picks one of "his" stamps out of the gallery with the
real pointer, drops it on a drawing and asserts the artwork arrives. Nine hops,
none of them assumed.

**Placing a custom stamp onto a drawing from inside pdfcer.** A custom stamp's
artwork *is a page*, and the engine has no verb that draws one page's artwork
onto another page. Filed as
`request_a_custom_stamp_can_be_read_and_authored_but_never_placed_on_a_page.md`.

## O166 — ◑ **BUILT AND DRIVEN** — "the printer dialogue box needs to remember our last settings"

**Your words**, in the same sentence as O167. Filed before any work
started, per rule 1 of this file.

### Status

◑ **Built. Not yet driven, and that is the whole of what is
outstanding.**

## O168 — ◑ **DONE** — "release new version asap"

**Your words**, arriving mid-build.

Shipped as `v0.5.0-dev.20260910.1`, on GitHub and in your OneDrive folder. It
carries the engine bump to **v0.50.0** and the stamp size chooser above
(**O162**, the half of it that could be built).

## O167 — ◑ **BUILT AND DRIVEN** — "we also need the option to auto select paper size based on the page sizes in the pdf"

**Your words.** Same sentence as O166, filed as its own row because
it can ship on its own and because it is a genuinely different mechanism.

### Status

◑ **Built and driven.**

## O165 — ◑ **STANDING ORDER, opened** — "check for the next pdfcer engine release periodically then build and release with the new features when there is a new version"

**This row does not close.** It is a standing instruction, not a task, so it
stays under OPEN permanently and records each time it fired rather than being
ticked off. Given while you were away from the machine, on your phone, with the
PC handed over.

1. **Poll the engine repository** for a new release, on a cadence, without being
 asked again.
2. When one lands: **bump the pin, build, drive, refresh `FEATURES.md`, package
 from a clean tree, publish to both OneDrive and GitHub** — the full release
 ritual, not just the bump.
3. **"With the new features"** is the load-bearing phrase. A pin bump that
 compiles is not the deliverable. Each release note in the engine names
 capabilities, and this project's job is to make them *reachable* — a verb the
 shell never calls is a capability you do not have. So every bump is followed
 by a read of what shipped and a check that a route to it exists here.

## O164 — ◑ **BUILT, HALF DRIVEN** — not something you asked for: pdfcer now tells you when a file contradicted itself and pdfcer had to decide

You did not request this row and it is here anyway, because it changes what the
program says to you and rule 1 of this file is that you should never have to
discover a change by tripping over it.

Some PDFs are self-contradictory — the same entry is written twice with two
different values in one place. pdfcer has always had to pick one to open the
file at all (it keeps the **second**). Until now it picked silently. A file
that says two things and gets read as one thing, with no mention of it, is the
exact shape of the problem you named on the redaction nagging: **the program
knowing something about your document and not saying so.**

Checks: `load_anomalies_are_listed_in_document_properties`, `load_anomalies_reach_the_status_bar`.

**NOT DRIVEN.**

## O163 — ✅ **BUILT, NOT YET DRIVEN** — "add a checkbox option to our search bar called zoom - when I unchecked just jump to the page and highlight the found item as before but don't change the zoom"

Not driven through the window: you were at the machine, and a driven check
takes the desktop. The `ui-verify` check that asserts the zoom readout is
identical either side of a find jump on a mixed-size set is the outstanding
work.

**NOT DRIVEN.**

## O153 — ✅ **BUILT, NOT YET DRIVEN** — a follow-on from O150: two identical lines on one page can now be told apart

Not something you asked for — it came out of measuring O150 — but it is a real
capability you did not have this morning and may as well know about.

If a line of text on a drawing was **drawn in separate pieces** (which is how
your CAD exporter writes almost everything) **and the same words appeared
somewhere else on that sheet**, pdfcer refused to edit it. It told you the
words appeared N times and it could not tell which one you meant.

**NOT VERIFIED.**

## O162 — ◑ **HALF BUILT, NOT YET DRIVEN** — "if I drew the stamp too small for the text to fit, resizing just stretches the entire object as is — I should be able to double-click again and drag and edit just the box size without affecting the text. Box size properties should also be available to edit in the position and size separate from the text size."

What you have now is the engine scaling the stamp's *picture* — letters and
frame together — because that is the only resize it offers for a stamp. What
you are asking for is a **re-bake**: the box changes, the text stays the size
you set, and the words re-flow or re-centre inside the new box — the way a
text box already behaves. That is the engine's stamp builder redrawing at a new
rectangle, which it does not do yet; **re-asked as wanted, with your words**
(`request_resize_annotation_refuses_a_pdfcer_authored_stamp_as_foreign.md`).
When it ships: the Properties panel gets width/height for the BOX (re-bake)
alongside the existing text size, and a second drag mode (double-click, then
drag) resizes the box without scaling the text. Until then: resize scales
the whole stamp; to get a bigger box with the same text size, delete and
place again larger.

**You can set a stamp's text size when you place it, and the box now grows to
hold the words instead of cutting them off.** The stamp window has a **Size**
list beside the stamp faces: *Fit the box I drew* — which is what it has
always done — and then 8, 10, 12, 14, 18, 24, 36, 48 and 72 pt.

⬜ **The other half is still not possible and it is still the engine's, not a
decision here.** Changing the size of a stamp **already on the page** needs a
verb that does not exist: the one that edits a placed annotation's style
carries only its icon and its colour, and the function that could tell pdfcer
what size an existing stamp renders at is not public. So there is no
width/height-for-the-box field in Properties yet, and there is nothing to fill
a text-size field from. Both are filed with the engine rather than guessed at.

☑ **Driven, later the same day.** Released immediately on your word, so it
shipped on unit tests alone; the driven check was built straight afterwards. It
presses the Size chooser in the running program and asserts that **24 pt**
reached the engine — and it was made to fail against two planted defects, one of
them the chooser opening on the engine's 12 pt rather than on your drawn box,
before its green was quoted. Details under **O168**.

## O161 — ◑ **FILED, MEASURED HEADLESSLY, NEEDS YOUR VERDICT** — "I should also be able to unselect things of redaction that i selected for redaction"

**Your words.** Two routes exist and both were measured without a
screen: (1) the Redact panel lists every mark with a **Remove** on its row;
(2) on the canvas a mark **is** a selectable object and **Delete** on it takes
the mark off through the engine's own unmark verb (one undo step, tooltip
*"remove a redaction mark"*). A unit test now holds both halves of route 2.
What headless cannot see is why it did not work for you at the keyboard.
The likely suspect: with the redaction tool still armed, a click on the page
starts a new mark instead of selecting one — switch to the select tool (or
press Escape) first, then click the mark and press Delete. If that is not it,
say what you clicked and in which mode, and it gets driven.

## O160 — **THREE MORE, MID-MORNING, EACH MEASURED ON YOUR OWN FILES AND FIXED** — "I still get this error … found 35 piece(s) of the supposedly-removed text still in it … I STILL can't adjust the size of a stamp on the canvas, or by entering a different size in the properties box … the text comes out vertical, and there is no control to set the angle"

**Your words.** And your follow-up, which was exactly right: *"if
it is rejecting based on a search of finding other items with the same text
elsewhere then that is a bug and not a feature."*

## O159 — **FOUR REPORTS IN ONE SENTENCE, EACH MEASURED ON YOUR OWN FILES** — "there's still no way to edit the size of a placed stamp, and some text in sw41177 still isn't editable, and we're back to the apply redactions box that just tells me we can't do it and I can't edit or delete nodes on the freehand markup draw shape"

**Your words, mid-morning.** You were at the machine, so nothing was
driven; every answer below was measured headlessly against the three SW41177
sheets copied to scratch, with the engine at the revision this build pins.

**First: which build.** `pdfcer-gui1` is last night's (22:06); **`pdfcer-gui2`
is this morning's** (06:21). The Enter-key fix and the sticky fix are in `gui2`
only. Everything below lands in the *next* build after this row.

Filed as `request_resize_annotation_refuses_a_pdfcer_authored_stamp_as_foreign.md`.

## O158 — 🔶 **WIRED, NOT YET DRIVEN** — "the draw a line that follows the pointer tool — I can't edit the nodes that make it"

**Your words.**

The freehand tool draws an **ink stroke**, and until this morning that was the
one markup kind whose points pdfcer could *read* and could not *change*.
Polygon, polyline, line and cloud all let you drag a node; freehand did not.

## O157 — ✅ **FIXED AND DRIVEN** — "the Markup Items don't have a live preview — the bounding box stays the same size when I drag the handles"

> *"NO RESIZE PREVIEW WAS DRAWN … the resize reached the engine and not one
> preview box was published, so the operator held a corner and watched nothing
> happen."*

## O156 — **YOUR ARCHITECTURE QUESTION, ANSWERED HONESTLY** — "I would think some of the function that handles the handle box and resizing would use the same function across almost all objects. Seems you have to program specifically for each one?"

**You are right, and where it is true it is a defect rather than a cost.**

## O155 — ✅ **FIXED — text boxes resize now, and stickies correctly never will** — "when will being able to drag on the canvas resize the Text Box and Stamp"

**A sticky lands where you click.** A reader anchors a sticky's icon at the
**top-left** of its box, not the bottom-left, so the box hangs *down* from the
click. A resize attempted from the Properties panel's width/height fields on a
sticky answers with a sentence — *"drawn at one fixed size … drag the note to
move it"* — rather than a silent nothing.

## O154 — ✅ **FIXED — the engine shipped it fifteen minutes after I asked** — "the Text box Markup tool — pressing enter shows one line with a `?` for each new line"

`request_encode_winansi_turns_a_newline_into_a_question_mark_before_wrap_lines_can_split_on_it.md`,
with the fix we think it wants and the two traps in testing it.

## O152 — ✅ **ANSWERED AND INSTRUMENTED** — "does our project check for the latest version of egui to compile with?"

**Your words:** *"does our project check for the latest version of
egui to compile with?"*

`Cargo.toml` asks for `egui = "0.35"`. In Cargo that means **`>=0.35.0` and
`<0.36.0`** — a wall, not a floor. So `cargo update` resolves the newest 0.35
and reports everything current, **truthfully, about a question nobody meant to
ask.** It cannot ever mention 0.36, and nothing else was looking.

## O151 — ✅ **BUILT, NOT YET DRIVEN** — "the drawing page previews checkbox should never automatically turn off"

> *"also the drawing page previews checkbox should never automatically turn
> off. You can add a box next to the checkbox to enter a timeout value when the
> user unchecks the draw page previews."*

**NOT VERIFIED.**

## O150 — **CAUSE FOUND, after two wrong diagnoses and one number that was a sample of a single cell — each font on each sheet carries only what it already draws, and that is between SIX and 59 of the 95 keys on your keyboard** — "I can't edit some of the text… the BOM only sometimes works"

> *"In `SW41177.pdf` I can't edit some of the text, for example `#2 USE SPACERS
> 8 9 10 11 IF REQUIRED.` The BOM I can, but it only sometimes works — in fact I
> thought it had also stopped working, but after trying multiple times in
> multiple places it started working."*

Filed as
`request_a_spanning_find_cannot_be_anchored_at_a_pinned_operator_so_a_repeated_bom_cell_is_uneditable.md`.
It is a small ask — start the existing cross-piece search at the piece you
clicked instead of at the top of the page — and it makes the refusal
unreachable from a click.

## O149 — ✅ **FIXED — "the redaction feature regressed back to just giving me the 'don't apply yet' button"**

> *"also the redaction feature regressed back to just giving me the 'don't apply
> yet' button."*

## O148 — ✅ **DONE — "we should have export/import for that"** — the import half, three days late and driven

> *"also the engine can export PDFs as text. we should have export/import for
> that."*

Export shipped that day. **Import could not**, and the reason is worth one
sentence: pdfcer had no way to *make* a page — only to copy one from somewhere
else. That was filed at the engine, answered, and wired today.

## O145 — ✅ **FIXED — "the object gets larger with each enactment of the tool"** — the engine shipped it the same afternoon

> *"fixed the rotate bug in the review objects where the object gets larger with
> each enactment of the tool."*

## O146 — ✅ **DONE — "the angle should be editable from the properties"**

> *"also the angle should be editable from the properties."*

## O147 — ✅ **DONE — "the box outlined when an object is selected should be in the same angled orientation as the object"**

> *"and the box outlined when an object is selected should be in the same angled
> orientation as the object."*

## O144 — ◑ **BUILT, AND NOT ONE PART OF IT HAS BEEN DRIVEN** — "getting full editing working for the Markup tools", and Adobe's own colours

> *"getting full editing working for the Markup tools. Also make sure you've
> used the same default colours and style look for these things as Adobe."*

**A text box's words go stale if you edit them.** Changing the note text on a
text box changes what the program records and not what is painted on the page.
The program now tells you so, twice, at the moment you do it — rather than
letting you find out later. Filed with the engine as the fix.

So: **built, awaiting your verdict, and awaiting a driven run before anybody
should call it more than that.** Driving it is the first job of the next
session. Everything above is in the release on OneDrive in **`pdfcer-gui2`**;
**`pdfcer-gui1` from 08:31 is your fallback** if any of it misbehaves.

**NOT DRIVEN.**

## O141 — ✅ **COMPLETE — "IF THE CHARACTER ISN'T AVAILABLE IN A PDF ARE WE ABLE TO CHANGE TO A DIFFERENT FONT?" — yes, and the letter now goes in on the same press**

**The last step of this works.** Choose the face and the letter
goes in, in the same press, with no save and no reopen. Driven end to end; the
sentence about saving and reopening is deleted, because it now describes nothing.

## O141 — ◑ **BUILT AND DRIVEN, AWAITING YOUR VERDICT — "IF THE CHARACTER ISN'T AVAILABLE IN A PDF ARE WE ABLE TO CHANGE TO A DIFFERENT FONT?" — YES, and now pdfcer offers it at the moment you hit the wall**

> *"if the character isn't available in a pdf are we able to change to a
> different font?"*
>
> —, asked while you were trying to fix the `clien` typo

> *"this font has no glyph for '€', and pdfcer cannot add one to a font that is
> already embedded. Keep this edit to characters the font already uses, or
> choose a font that covers it."*

> *"This text is now set in Helvetica, and the 'q' still would not go in: pdfcer
> cannot type into a font it has just added to a file until that file has been
> saved and opened again. Save this document, open it, and type the 'q' once
> more — it will go in then. This limit is pdfcer's own and is on the list to
> fix."*

So the trigger is the face pdfcer had to *add*, and nothing else. Filed at
the engine as
`request_edit_text_resolves_font_names_against_the_base_revision.md`, with the
reproduction. **The three measurements are kept as tests**
(`canvas::textedit::facewall`), so the day the engine fixes it the first one goes
red and pdfcer finds out from a test run rather than from somebody re-reading a
paragraph — which is how this project has been wrong about the engine four times
in a fortnight.

- `request_classify_font_reads_fontdescriptor_from_the_type0_parent_where_it_never_is.md`
 — pdfcer tells you an embedded font is **not** embedded on most real
 documents, so the sentence about whose letterforms your client will see is the
 wrong way round. Your apartment file shows all three of its own reports
 disagreeing about one font.
- `request_hit_test_has_no_distance_bound_so_a_click_on_blank_paper_finds_text.md`
 — unrelated to fonts; see the note on O142 below.

Cited: `edit.rs:31362`, `text_edit/addtext.rs:96`.

Checks: `a_refused_character_offers_a_face_that_can_type_it`.

## O142 — ✅ **FIXED, AND IT HAD BEEN FIXED FOR TWO DAYS WITHOUT ANYONE NOTICING** — a click on empty paper starts new text

> *"How do I make new text when I click on the canvas and expect to edit there?
> Same problem as the previous."*

> *"the point named an existing run, which is a fact about this fixture rather
> than about the feature"*

**Not something you reported.** It was found while looking for
blank paper on your file, recorded in this project's own feature register, and
**re-measured from scratch today before being filed**, because two requests this
week were sent to the engine on diagnoses that did not survive re-measurement.
It is on your list because the request it breaks is yours.

> *"How do I make new text when I click on the canvas and expect to edit there?
> Same problem as the previous."*

**Filed at the engine** as
`request_hit_test_has_no_distance_bound_so_a_click_on_blank_paper_finds_text.md`,
with the measurement and a copy-pasteable reproduction. Deliberately **not**
worked around here: from outside, a cursor that lands 3 points past the end of a
line (correct, and you use it) and one that lands 215 points away are the same
answer, so any distance limit invented here would be a second opinion about the
engine's own geometry and would drift from it.

## O140 — ✅ **YOUR TYPO IS FIXED** — the correction goes in now, on your own file, in one gesture

> *"on page 2 there is a spelling mistake — clien instead of client. if I try to
> edit the edit is not accepted. **the lines I added below `price)` are
> editable, but everything else that existed when I got the pdf is not.**"*
>
> —, on `apartment work - signed.pdf`

> *"on page 2 there is a spelling mistake — clien instead of client. if I try to
> edit the edit is not accepted. **the lines I added below `price)` are
> editable, but everything else that existed when I got the pdf is not.**"*
>
> —, on `C:\Users\Ken\OneDrive\pdfTests\apartment work.pdf`

> *"pdfcer cannot change these words. The program that made this file wrote the
> line one letter at a time, and pdfcer rewrites a whole piece of text at once —
> so there is no piece here that holds the word you are correcting. Text you
> added with pdfcer is written a line at a time, which is why those lines do
> edit. Your document is unchanged; this limit is on the list to fix."*

The morning's diagnosis — filed at the engine as a feature request — was that
your document's fonts are `Identity-H` and cannot be written into. **That is
wrong, and it is retracted.** On your own page 2, with the engine's own command
line:

Both are filed. The second is the small one and is the route to your fix.

`tools/ui-verify`'s **`a_refused_typo_fix_says_why_it_was_refused`**, driven on
your own file at your own typo, **PASS**. It clicks the word, seeds the `t`,
commits, and asserts the `⊗` slot draws afterwards — and then, in the same
process, commits an edit that **succeeds** and asserts the slot stays silent,
because a check that only ever watches something appear cannot tell a working
program from one that shows that sentence always.

## O136 — ◑ **BUILT, AWAITING YOUR VERDICT** — the document's own properties have their own tab now, and the Properties tab is only ever about what you picked

> *"the document properties are still always visible in the properties tab. it
> needs to get out of there and be in its own document properties tab."*

* **Unit tests:** the arrangement of all three modes is restated by hand in two
 tests that transcribe the panel lists literally, so this change could not slip
 through as an incidental; the two panels cannot share one command; the new
 panel is reachable from the ribbon and registered.
* ⬜ **NOT DRIVEN.** No window was launched. The two driven checks that cover
 this surface — `properties_metadata_round_trips` and
 `the_inspector_is_one_master_detail_column` — were **updated and not run**,
 and each says so in its own header. Another track had the machine's pointer.
 Both were passing before this change.

## O135 — ◑ **BUILT, AWAITING YOUR VERDICT** — read mode was a room with no visible door

> *"I didn't see a way to get back out of read mode. if there is a shortcut for
> this it should have a note what the key combo is in the top bar that holds the
> window controls."*

**You were right, and the reason is worse than a missing label.** Read mode
(`Ctrl+H`) hides the ribbon and the panels — and the only control that turns it
off, View ▸ Window ▸ Read mode, **is on the ribbon**. So the moment it is on,
the control that undoes it is hidden by the thing it toggles. The only route
left was a chord that nothing on screen named.

**Status:** BUILT. **Verified:** unit tests only — the title in all
four forms including with no document open, the bar's line, R128 (it cannot
change the bar's height), and the identity between the advertised chord and the
manifest's binding. A driven check,
`read_mode_says_how_to_get_back_out`, is **written and NOT RUN**; it needs no
pointer and no keystroke, so it can be run at any time. Its own header says so
in its first section.

## O134 — ✅ **FIXED AND IN YOUR BUILD — deleting pages from a drawing set works** (guard kept)

> ### ENGINE FIXED IT — `Pass 251.1`
>
> **Not yet in our lock**, and deliberately so: a driven sweep owns the pointer
> and bumping the engine would swap the binary under a run in progress. It goes
> in with `RESUME.md`'s queued bump.
>
> **The cause, in their words, and it is exactly the shape the three-level
> fixture was built to expose:** `delete_pages_with`'s splice loop rewrites a
> node's `/Count` only when its **own** `/Kids` changed —
> `if kept.len() == kids.len() { continue; }`. On a nested tree the ancestors
> above the immediate parent keep all their kids (their child node survives,
> just lighter), so every one of them was skipped. The `leaves_under` /
> `lost_under` tallies **were already computed for them**; they simply were not
> consumed.
>
> `page-copy --cut` is fixed by the same change, *"exactly as your
> byte-identical-output measurement implied."*
>
> **What happens to our guard when the bump lands:** it should stop firing,
> and it was built to notice — its own test distinguishes *engine fixed* from
> *guard unwired* by re-reading the written file. **The guard stays**, because
> it is a proof at the boundary rather than a workaround for one defect, and the
> same class can return through any writer. What must change is the sentence: it
> currently says *"this is a fault in pdfcer"*, which becomes false for a file
> pdfcer wrote with the fixed engine.
>
> Also scoped by the engine off the back of today: **`Pass 254.0` — the
> line-weights-off display mode (O137), marked "Next up, operator-requested"**,
> and `Pass 255.0` — markup-shape vertex read/edit, backlogged.

> *"I tested deleting pages from a pdf. when I open the document in Acrobat
> there are blank pages at the end of the document equalling the number of
> pages I deleted."*

**Status:** the guard is **shipped and unit-tested (23 assertions, falsified
three ways)**. The engine fix is filed as
`request_delete_pages_leaves_ancestor_count_stale_on_a_nested_page_tree.md`
with the full measurement and our fixture offered to them.

⬜ **NOT DRIVEN.** The driven check `a_save_that_would_produce_blank_pages_is_refused`
is written and registered and **has not been run** — another track owned the
screen. In those words rather than implied, per rule 3 of this file.

## O133 — ◑ **FIXED, AWAITING YOUR VERDICT** — a comment you drew could be turned but not moved or resized, because its own note window was sitting on top of it

Both were filed as defects and neither is one. They are recorded
here because a wrong bug report costs as much as a bug.

Checks: `dragging_a_markup_moves_it`.

## O132 — ◑ **ALL THREE PARTS ARE NOW BUILT —, AWAITING YOUR VERDICT** — you cannot edit or delete the nodes of a shape you have drawn

> *"I also can't edit or delete nodes of a markup shape once it is drawn."*

One sentence, and when it was measured it turned out to be **three different
things**, wearing one complaint. **This paragraph read *“Two of them are fixed
today. The third cannot be fixed here at all, and it is filed rather than
fudged”*, and it stopped being true within six hours.** All three
are fixed, and the fourth thing at the bottom of this row — the right-click menu
— was built. **Nothing here is closed; it is done and waiting on
your verdict.**

**Verified:** 14 unit tests, six of them new and every one **falsified** — the
guard was removed, the test was watched go red, and the code was put back.
Three of them run against the real engine on a real document rather than a
stand-in.
⬜ **NOT DRIVEN.** `a_corner_can_be_added_and_taken_away` is written,
registered and has never seen a running window; the machine's pointer belonged
to another job all day. Its own header says so.

Filed as `request_a_markup_shapes_vertices_cannot_be_read_or_edited.md`.

## O131 — ◑ **FIXED, AWAITING YOUR VERDICT** — you could copy a comment in Review and had nowhere to put it, and the Objects rows were all cut short

**Verified:** ten unit tests, including one that drives the real dispatcher.
⬜ **NOT driven.** The machine was in use by another job.
`a_paste_review_may_not_do_says_so` is written and registered and has never
been run.

**Verified:** four unit tests, two of which measure the real fixture with the
real font. ⬜ **NOT driven** — same reason.
`the_inspector_is_one_master_detail_column` should now pass and has not been
re-run.

## O130 — ◑ **BUILT, AWAITING YOUR VERDICT** — you can read a sticky note by clicking it, in Read mode, the way Acrobat does

> *"check how the review functions work. unless something has changed I could
> add a yellow sticky note but even in read mode I don't think I could figure
> out how to read it. the review features should look and act the same as they
> do in Acrobat Reader. check that these are fully editable while you are at
> it."*

**You were right, and it was worse than you thought.** There was no way to read
a note's words *anywhere on the page*, in any mode. The only place a comment's
text appeared at all was the Comments panel — which lives on the **Markup**
tab, and Read mode is shown File and View only. So in Read mode there was **no
route to a comment at all.** A reading mode that cannot read the comments is a
PDF reader with its posture exactly backwards.

Three things Acrobat has are missing because **the engine cannot do them yet**,
not because we ran out of time. Each is filed and the engine session answers
within the hour:

| | |
|---|---|
| **Reply to a comment** | pdfcer can *read* a reply thread and has no way to *write* one. Filed |
| **Accepted / Rejected / Completed** | Acrobat's review status. The engine has no notion of it at all — not read, not written. Filed |
| **Change a sticky's icon or colour after placing it** | write-once today: to change either you delete and place another. Filed |

## O129 — ◑ **BUILT, AWAITING YOUR VERDICT** — pdfcer can now tell you WHO signed a document, using the trust list your Acrobat has already downloaded

⬜ **NOT VERIFIED, in those words.** The headless tests pass and every one was
falsified. The driven check
(`signature_trust_is_reported_as_its_own_fact`) is written, registered and
**was not run** — you may have been at your keyboard. And **nothing in this
repository has ever seen a signature come back `trusted`**: that needs a real
certificate from a real authority, which this project cannot commit and cannot
keep current. **The one thing most worth doing next is on your machine**:
turn the setting on, press *Show what is in it*, and tell me the numbers it
reports.

## O128 — ✅ **BOTH HALVES SHIPPED AND BOTH HALVES DRIVEN —** — export and import as text

**Ken, verbatim:**

> *"also the engine can export PDFs as text. we should have export/import for
> that."*

> ### The import half — the engine has no route, and this is filed
>
> **`pdfcer-core` cannot turn a text file back into PDF page content**, in any of
> the three senses *"import text"* could mean: there is no document builder, no
> page-level text replace, and `add_ocr_layer` takes positioned words from the
> recogniser rather than a file. The nearest verb, `add_text`, places one run on
> **one** page and *emits* overflow past the sheet rather than paginating — so a
> two-page text file would produce one page with the second page painted off the
> edge, invisible and present. That is a data-loss trap wearing the shape of a
> feature, so it was not built.
>
> **Nothing was drawn for it.** No greyed control, no control that declines when
> pressed, no tooltip implying a round trip.
>
> Filed at
> `request_there_is_no_route_from_a_text_file_back_into_a_pdf.md`, asking for
> either a paginating text placer or a page-level text replace.

Checks: `export_text_writes_the_documents_words`.

**NOT VERIFIED.**

## O127 — ◑ **PARTLY DONE** — added text duplicates on move, Enter cannot make a new line, and Reflow does nothing

**Ken, verbatim, from a real editing session:**

> *"there's a bug I've come across where if you add text once it works, but if
> you add text a second or third time it will make duplicates of you try to move
> the instances after and make a duplicate for every new text box that you added
> regardless of which one you move, with the exception that if you make a text
> box, switch tools and make another one, then the first one doesn't start
> making duplicates. also can the enter key create new lines when we are editing
> or creating text? I also haven't seen the reflow option actually work with
> anything when I press it."*

*"if you make a text box, switch tools and make another one, then the first one
doesn't start making duplicates."*

| # | cause | where | status |
|---|---|---|---|
| 1 | duplicates on move | **`pdfcer-core`**, `edit.rs` `text_edit_command`'s `if first_edit {` gate | **filed** — reproduced by a test here; the fix is not ours to make |
| 2 | Enter cannot make a new line | this shell, `canvas::textedit::keys` | **fixed** |
| 3 | Reflow does nothing | this shell — it was answering, in the wrong slot | **fixed**, with one gate deliberately kept |

Reproduced by `crates/pdfcer-gui/tests/added_text_duplicates_on_a_later_edit.rs`
— three tests, written to pass on the broken engine and go **red** on the fixed
one, in `engine_overlay_skew.rs`'s shape. Filed as
`request_added_content_is_duplicated_by_the_next_content_edit.md`.

**A second, worse defect found while diagnosing it:** `reflow_block` guards
only on `contents[0]`, which `add_text` does not touch — so a reflow after an
add **silently deletes the added text**. Filed in the same request. See 3.

**The `edit_epoch != 0` gate is KEPT**, and that is the one thing here a
reader will want to argue with. It looks over-broad next to the engine's own
condition, and it is — but the engine's condition does not cover `add_text`, and
a reflow permitted after one **silently deletes the added text** (see 1). Lifting
it needs the engine change that is now filed. Until then reflow still refuses
after any edit, and now says so where he will see it.

## O126 — ◑ **OPEN** — float / close / dock on every panel, search on Layers, selection highlights its layer (✅ **done**), and the export list is short

**Ken, verbatim, after approving the A7 mockup:**

> *"I checked the mockup and it is perfect. And you understand that there are
> options to float, close, and dock those panels, and that there is a search to
> implement on the layers and selecting an object highlights that layer? No
> shortcuts or lazy half-implementation. Be sure to implement everything to the
> fullest of what would be expected by a user. I think we might also have a few
> more export options available that what is shown."*

**The relation was requested rather than invented**, filed as
`request_which_layer_is_this_object_on.md`, and `pdfcer-core`'s `Pass 250.0`
answered it the same session. `oc: Option<ObjId>` now sits on `PathObject`
(`vector/decompose.rs:386`), `TextObject` (`:484`) and `ImageObject` (`:780`),
read through `VectorObject::oc()` (`:1064`) and `FormLeaf::oc()` (`:1300`).
Engine v0.38.0.

Building the page-object route beside the annotation route found **two engine
divergences**, both filed on the `ENGINE_BACKLOG.md` row rather than absorbed:

**Not verified:** the driven check
(`ui-verify selecting_an_object_names_its_layer`) is written, registered and
**has not been run** — the machine was possibly in use. The unit tests, including
the two that would go red if the answer stopped following the selection, are
green.

> *"add a select-all glyph. I didn't refuse that."*

**The driven sweep filed this as A5:** *"a floated panel opens an
empty window, and Dock all recovers nothing"* — `panels_float_close_and_dock`
reporting no viewport-tagged `ui-rect` and `panels-dock-all docked=0`. The
same check had also been failing with `moved=false` for days.

⬜ **NOT DRIVEN.** The repaired check has **not been run** — the pointer
belonged to another track for the whole session. The row stays ◑ until it is.

## O125 — ✅ **FIXED** — redaction refuses to do anything, and it should not have to save a new file every time

**Ken, verbatim:**

> *"I really hate how when I search for text to redact, or select a text object
> on the screen to redact, pick the text to redact, then click apply redaction it
> refuses to redact anything because it always finds text that wasn't redacted,
> and it always finds all of the text is found that I selected. I really really
> would like if it still redacted the things it could redact, warn that there
> were things it couldn't redact (because every time I have tried the tool there
> always is and it always counts everything I selected as unredactable) but still
> make the changes it could. What is the purpose of a redaction tool that refuses
> every time to do any work? Also why does it have to save to a new file right
> away? Why can't it just wait on saving until I choose to save over the existing
> file or save as a new file?"*

> *"if the engine can't hold it we need to leave it up to the user to decide to
> overwrite the original or save to a new file, and just add another option to
> save a copy before overwriting the orginal. If someone is saving their changes
> while redacting they aren't going to keep having to save a new file every
> time."*

### Part 1 is a bug, not a policy, and "always" is the tell

### Part 2 — the forced save-as, and why his objection defeats its argument

**Part 2 was answered by the engine, in full, within hours of being asked.**
The request `request_apply_redactions_into_the_session.md` went out at midday
saying the deferral could not be built —
`redact::apply_redactions` took a `&Document` and returned `Vec<u8>`, and
`EditSession` had no way to take the result back. `Pass 250.1`
shipped `EditSession::apply_redactions` the same afternoon. So the fallback
ruling in this row is **not** what was built: the deferral itself was.

## O124 — ✅ **FIXED** — the canvas fades content at the edges of the view; it should render true

> *"the canvas does a fading around the edges on stuff shown at the edges of the
> view. I don't want this. it should render true."*

## O123 — ◐ **PARTLY DONE** — A7: one dock, master–detail, a one-line tool status, and selection controls in the left rail

**Ken, verbatim:**

> *"Also I want A7 from the plan implemented. I never understood why there is a
> tool dock when everything can be in object and properties. I'd also like those
> one to appear in the space where the tool dock currently shown. to recap 'The
> Tool panel becomes a one-line tool status (name, one sentence, "Put this tool
> down"); its buttons duplicate the ribbon and go. Objects and Properties become
> master–detail in one panel with a draggable split, ellipsis and tooltip on
> rows. Layers, Signatures and Fonts join Pages and Bookmarks as tabs in one
> dock instead of a second dock with a fixed split. Default dock width 360 px in
> Edit, remembered per mode.' What I'd also added in the bar at the left side
> that we are adding: the navigate selectors and some other related selection
> controls (lasso tool when we implement one, etc) and these will fold up into a
> drop down arrow if space becomes scarce."*

### His version is the one that ships: one tabbed dock, not a second dock with a fixed split

Checks: `the_armed_tools_settings_are_in_properties`, `the_first_frame_names_the_armed_tool`, `the_inspector_is_one_master_detail_column`.

## O122 — ◑ **OPEN** — an "Open in Acrobat" button beside Read / Review / Edit

**Ken, verbatim:**

> *"also beside our read-review-edit buttons at the top there should be an open
> in acrobat button which will open the active pdf in acrobat reader or pro
> depending on what is installed - we'll have to add a feature to automatically
> locate and open the installed acrobat on the system, and have a setting where
> people can change it. When clicked it will check if the file has been changed
> (forms filled out for example, etc) and ask to save changes first, but if it
> hasn't changed it will note the file will be closed when opened in acrobat
> with and ok button to continue - there will be a cancel button as well."*

### Status

**Built. NOT VERIFIED by a driven run** — see "The driven check that
was not run" below, and the reason, which is that you were at your keyboard.

`tools/ui-verify/` is another track's tree this session and I stayed out of it,
so the check below is specified rather than added. **I did not run it, and I did
not launch the GUI or Acrobat at any point** — you were at your keyboard, and a
driven run takes your screen.

## O121 — ◑ **OPEN** — the test harness took your screen twice after you said you were back, and the fix is a lock rather than an apology

**Blocked until** the track currently rewriting `tools/ui-verify/` lands, so the
two do not collide. It is perhaps forty lines.

## O120 — ◑ **OPEN, AND IT SHOULD HAVE BEEN OPEN SINCE** — export to PNG / JPEG / SVG, and copy-paste into Word and Inkscape

**Ken, verbatim:**

> *"can you add the ability to export page(es) to png, jpg, svg. note that there
> had better be full support (including transparency where supported!). Also I'd
> like to be able to copy and paste anything to other software - like copy and
> paste vector graphics into word or inkscape for example if possible."*

The note landed in `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`
marked *"informational, no reply needed; consume when convenient"*, and **that
is exactly the shape of thing this project has already been burned by**: no
gate reads the request channel, no test fails, and "when convenient" never
arrives on its own. It was found only because a session read the
channel looking for something else.

### Status

**Still owed: the driven check.** See item 3 below. Nothing in this pass was
opened in a running binary either; the desktop was owned by a concurrent track
and `ui-verify` was deliberately not run.

1. ✔ **Clipboard placement.** `crates/native-clipboard` — eleven Win32 symbols
 across `user32`, `kernel32` and `gdi32`, and it does not name a PDF concept
 anywhere, which is what would let `egui-shell` take it unchanged.
 `crate::clipboard::place` is the shell half: it produces the payload, asks
 the degradation predicate, and maps `ClipFormat` → `Slot` in one function.
2. ✔ **A selection, as opposed to a page** — and it **fell out cleanly**, which
 is why it was taken rather than deferred again. Both ends already existed:
 `EditSession::copy_objects` → `ObjectClip::to_pdf` gives a standalone
 one-page PDF whose `/MediaBox` is the *selection's* bounds (so what lands in
 Word is the line-work at its own size, not floating in a page-sized empty
 rectangle), and `svg::export_svg` / `emf::export_emf` take a plain
 `&Document` — the `_view` forms are the session variants and a freshly parsed
 clip has no session. Four lines of plumbing between two things already there.
 It is **page content only**: an annotation-only selection names no content
 objects, so such a copy takes the whole page rather than refusing, which is
 the honest answer because a markup's vector form is on the page.
 The file-export window still does not offer a selection, and that is
 unchanged and deliberate: *"export this selection to a file"* has a question
 inside it that a clipboard copy does not — what are the file's bounds? — and
 the clipboard answers it by construction.
3. **The driven check.** The window has never been opened in a running
 binary — not for the first three formats and not for EMF. R1's bar is *an
 operator can reach it in a real build*, and by that bar this is not
 finished. Stated here rather than implied by a green test count, and stated
 again because neither the second nor the **third** pass cleared it: the
 desktop was owned by a concurrent track on both days and `ui-verify` was
 deliberately not run. The driven check for the copy-out is written and
 unrun — `Copy as vector` on a page, then a paste into a real Word document
 through combridge, asserting an inline shape with `svgBlip` in the OOXML,
 which is exactly the measurement the engine made when it fixed the order.

## O119 — ✅ **BUILT** — the engine can now put a password on a document and set what it allows. You said yes.

When you asked for the encryption and signature tab (O108), the audit came back
with an answer that changed the ask: the engine could **read** protected files
and could not **make** one. So the tab was scoped to tell you things — what a
document is encrypted with, which password opened it, what it says it allows —
and the missing half was filed as a request.

**The window has never been opened in a running binary.** The headless suite is
green — the model and the dialog have 22 tests between them — and the
`ui-verify` check that drives the real thing is **written and was not run**: the
desktop was owned by a concurrent track that day. R1's bar is *an operator can
reach it in a real build*, and by that bar this is not signed off. Stated here
rather than implied by a green test count.

## O118 — ✅ **SHIPPED** — "in those featurerequests did you see the new glyphs and layout?"

A question about pictures and layout is answered by rendering the mockups
headlessly and looking at them, never by reading their JSON or HTML as source
text. A glyph is adopted only when a command or role in this build would use it
today.

### Awaiting your decision — the Save family reads as three floppies and one odd one out

The four Save commands read, left to right, as a **rounded square with a bar**
and then three **floppy disks** with different interiors. `save-as`,
`save-copy` and `save-compact` are drawn as a family — one shared body, one
interior difference each — and Save's own glyph is not that body. The family
grammar is right; **Save is the odd member of it.**

**Not fixed, deliberately.** That glyph is your own art on the most-used
control in the application, and redrawing it to match three new siblings is a
change you should see before it ships rather than after.

## O117 — ◑ **OPEN** — one driven check is FLAKY, which is a defect in the instrument

A flaky check is the same defect as the bad `--doc-point` that produced six
false reports (O115), arriving by a different route: **it
manufactures confident wrong bug reports at random.** This one already did — it
is one of the six I filed and retracted, and I retracted it for the wrong
reason. I attributed it to the page index. The page index was wrong *and* this
check is flaky, and the second fact was hidden behind the first.

Checks: `scrolling_far_keeps_the_canvas_its_pointer_input`.

## O116 — ◑ **OPEN** — an edit the engine refuses is SILENT: you type, you commit, nothing happens, nothing says why

The categories have to come from somewhere. `EditError`'s variants are
`pdfcer-core`'s, and wording one sentence per variant here would be a second
catalog that drifts from theirs. **ASKED** —
`request_can_edit_errors_expose_a_coarse_kind_a_front_end_may_switch_on.md`
asks for a coarse, STABLE discriminant ("unsupported font", "structure frozen",
"not found", "other") a front end may switch on without re-deriving their
diagnosis. It says plainly why the two shortcuts are refused: matching on their
variants is a second copy of their taxonomy that drifts and then tells the
operator the WRONG reason — worse than the silence we have now — and parsing
the `Display` string is greping prose that is theirs to reword.

Checks: `text_edit_on_a_real_drawing`.

## O115 — **RETRACTED** — the "three defects the sweep found" were **my own bad argument**

A harness that cannot start tells you so. **A harness given a bad coordinate
does not fail — it lies fluently.** Each report named a real function, a real
trace event and a real line number, and read exactly like a regression. I filed
four of them as defects and wrote a paragraph claiming one of them disproved an
earlier pasteboard theory.

**Fixed** in `coords.rs`: `from_trace` now compares the caller's page against
the page the **application says it is showing**, on the same trace line as the
rect. Two independent quantities, so the comparison is real. The exact
invocation that fooled me now SKIPs with a message naming both pages and stating
that PAGE is 0-based; the valid invocation still passes.

## O114 — ◑ **INTAKE** — an outside GUI review, twenty findings, one of them a crash

**Ken:** *"new feature request in d:/dev/featurerequests/pdfcer-gui.
I'm not at the PC. it is yours to use."*

**Read at** `D:\Dev\FeatureRequests\pdfcer-gui\` — `REVIEW.md`, `HANDOFF.md`,
`screenshots/INDEX.md`, `mockups/`, `logs/`.

### ✅ A1 — the crash — FIXED THE SAME SESSION

> `pdfcer ▸ Keyboard shortcuts` panicked at `dialogs/host.rs:943`:
> **"viewport callback ran twice"**, on a fresh launch, taking the open
> documents with it. Unsaved markup was lost in the reviewer's session.

### ◑ A2–A20 — every finding read against the source

Five more were fixed beyond the crash (A3's real defect, A4, A5, A9's R9 hole,
and the harness gap); thirteen were confirmed and queued; seven were already
decided against. Every queued defect is now fixed in source — A11
`dialogs/about.rs:42-44`, A12b `canvas/forms.rs:999`, A12c `canvas/forms.rs:488`,
A15f `dialogs/ocr.rs:600`, A16a `dialogs/formfield.rs:104`, A16c
`dialogs/host.rs:465`, Part C `tools/ui-verify/src/checks/theme_page.rs`.

**Five items are amendments to `RIBBON_IA.md`, which is settled and yours to
rule on — none has been actioned.** The sharpest is **A10**: the review wants
the four page-display buttons labelled, `RIBBON_SCALING.md` made them icon-only
on a measurement, and `RIBBON_IA.md:143-147` already argues the reviewer's
side. That one wants a decision from you.

**The mockup's ribbon does not load.** It puts Fonts and Comments on two tabs
each, and `no_command_appears_twice_on_the_tabs` refuses it. Most of the mock is
the shipped manifest re-drawn; the genuinely new parts are a left icon rail
(gated behind the R128 fit-zoom cache), a bundled typeface, a palette, and an
11 pt type floor.

### Eight of the reviewer's statements are factually wrong

Recorded because an incorrect finding, uncorrected, becomes a fact.

- **"Escape closes no dialog."** It closes every one — read from the **child**
  window (`dialogs/host.rs:901`). Escape was sent to the main window.
- **"Dialogs stack."** 18 of 20 kinds are single-instance-guarded; 2 replace in
  place. Four windows open at once were four *different* dialogs.
- **"Set scale was cancelled by drawing a rectangle."** No code path does that.
  There *is* a real stranding on the Escape-during-calibrate route, written
  down at `canvas/placing.rs:39-43`.
- **"The suite drives many dialogs but not Keyboard shortcuts."** It drives it,
  in three checks. The truth was worse than the report: it drove it and passed
  anyway.
- **"Properties gets a quarter of the height", "315 px column", "Objects rows
  truncate without ellipsis"** — a third, 320, and no truncation exists.
- **"First launch is 1116 × 839"** — the declared constant is 1100 × 800; the
  difference is DPI inflation.
- **"The title shows the minute although the decision says the day"** —
  superseded by O101 on your instruction.
- **A16b, "Print's preview column takes half the dialog and should be
  resizable"** — it is 42.5 %, and it is a draggable splitter with a floor and
  a double-click reset. That the reviewer did not find it is itself a
  discoverability finding.

Checks: `dialogs_open_in_their_own_window`.

## O113 — ✅ **DONE** — the clipping hatch should cover only what actually falls outside the printable area, and the button's count follows it

**Ken:** *"also can you make it so the red pattern you put over the
page if it is going to print beyond the printable borders is only over the areas
that extend beyond the printable page? Our drawing get drawn 1:1 and the area
that isn't printed is just empty border."*

## O112 — ✅ **DONE** — the print preview should be resizable, and poppable into its own window

**Ken:** *"also the preview should be adjustable size, and even
better if it has the option to pop out into its own resizeable window - closing
the window pops it back into place on the print window."*

**No window was rendered.** You were at the machine, so `ui-verify` was not run
and no screenshot exists of either the popped window or the collapsed column.
The arithmetic is unit-tested and falsified; **that it looks right is
unverified**, and for a layout change a rendered frame is the only real oracle.

`ui-verify`'s `the_print_preview_pops_into_its_own_window` is written and
registered and **has never been run**. It drives the click, asserts the column's
region is declared before it and **retired** after it (`ui-rect-gone`), asserts
the popped window's body drew, asserts `popped=true preview_w=0.0` and
`options_w == content_w`, then presses Escape and asserts the preview comes
home. Its header says what a first run is most likely to teach it.

### Was blocked behind O111 on purpose

## O111 — ✅ **FIXED** — the print dialog: two permanent scrollbars, it will not close, and Print looks broken

**Ken:** *"I thought the print dialogue box had been fixed by
replacing with a more standard window one but with all our current acrobat style
controls and commands untouched. Instead I have two scroll bars in the pop up
window that won't go away no matter how, and it doesn't close after I hit the
print button that is so far off in the corner it is touching the edge the
window, and it looks greyed out as though it it doesn't do anything even when I
hit print - but it is working, so after many clicks I checked the printer and of
course there was a dozen jobs there because the button just looks greyed out and
broken."*

| # | fix |
|---|---|
| 1 | every width and height in the body is now derived from the space **outside** the scroll area and from constants — never measured inside it — and `auto_shrink` is `[true, true]`. Driven by `print_dialog_body_does_not_deadlock_its_scrollbars`, which reads egui's own `content_size` and `inner_rect` out of a running frame |
| 2 | a **successful** print records its receipt on the application's disclosure row and closes the window. A **failed** one does not close, because the driver's words and the settings that produced them are what the operator needs next |
| 3 | `Host::BODY_MARGIN_PTS` — 12 pt, applied once in the host, so **all fourteen dialogs** gain it rather than each remembering |
| 4 | `Theme::accent_pair` — one named accessor for *"paint this as the emphasised action"*, replacing the translucent `selection.bg_fill` in every dialog's affirmative button |

- **Driven and falsified.** `print_dialog_body_does_not_deadlock_its_scrollbars`
 fails when the original width defect is planted back in (content 784 pt in a
 776 pt viewport) and passes when it is removed.
- **Photographed at five window sizes**, before and after. The before-captures
 show both bars at 1000 × 760 and 1300 × 900 and the clipped-with-no-bar case
 at 700 × 520; the after-captures show no bar where nothing needs scrolling and
 a bar where something does.
- **Print is now in `dialogs_open_in_their_own_window`'s list**, which it had
 never been in — see below.
- Three unit tests on the close-and-report decision, extracted into
 `commit_notes` so that proving the window closes does not require putting a
 job on your printer.

## O110 — ✅ **RELEASED** — release and publish the package on GitHub

**Ken:** *"please release and publish the package on github."*

**The rename had blinded the falsification harness, silently.**
`ui-verify`'s profile for the OLD GUI — the build the checks must be seen to
FAIL against — had all four of its external names swept to the new spelling.
Three of the four fail *quietly*: an env var the old binary does not read leaves
its diagnostics off, and a trace prefix it never prints parses to an empty
trace. The suite would have said *"the old build does not exhibit the defect"*.
Repaired, and held by two falsified tests rather than a comment.

## O109 — ✅ **RELEASED** — release the latest source and exe to GitHub

**Ken:** *"And while you are doing that release the latest source and
exe to github!"*

## O108 — ◑ **AUDITED, and the answer changes the ask** — one ribbon tab for every encryption and signature feature

**Ken:** *"can we get all of the encryption and signature features
that have been implemented in the engine under one new tab in the ribbon?"*

1. **A password prompt**, so an encrypted document opens. Nothing else on this
 list matters if the file cannot be read.
2. **A Security tab** collecting: the encryption state (scheme, key length,
 whether you authenticated as user or owner), the **permission bits** in
 plain words, the signature census, and the byte-range coverage — the four
 questions an operator actually asks of a protected file.
3. **A request to the engine** for the authoring half: encrypt with a password,
 set permissions, remove encryption, sign. It has none of it, and this shell
 must not invent it. ✅ **Filed**, as two files because they are two
 topics: `request_a_document_cannot_be_encrypted_or_have_its_permissions_set.md`
 and `request_a_document_cannot_be_signed.md`.

## O105 — ✅ **DRIVEN** — the radius/diameter tool fits the wrong thing

**Ken:** *"can you check our radius/diameter dimensioning tool?
selecting a point sometimes makes a big circle, and selecting more points around
a hole doesn't always get it to narrow down to the size of the hole."*

*"selecting more points around a hole doesn't always narrow it down"* has two
causes, both from the same design.

**Driven by `three_clicks_round_a_hole_measure_the_hole`**, on a fixture built
to carry your geometry — `fixtures/hole-in-a-big-object.pdf`, **one** path
object holding a 30 pt circle *and* forty unrelated segments across the page.
The check pins that fixture and ignores `--pdf`, because on a document whose
circles are their own objects the defect **cannot occur** and the broken build
would pass.

## O106 — ◑ **BUILT, driven only in unit tests** — a click with nothing under it should still be a point

**Ken:** *"also we should be able to click and it selects a position
on a page if there is no point to select under the cursor (so we can measure off
bitmaps too)."*

## O107 — ✅ **DRIVEN** — see what is in the pick set, and take things out of it

**Ken:** *"also we should be able to unselect points/clicked
locations, and it should have a box in the side panel showing what is part of
our selection and we should be able to delete included points/locations from
there - clicking on a point or location listed should allow us to remove it."*

Checks: `three_clicks_round_a_hole_measure_the_hole`.

## O104 — ⬜ **INVESTIGATED** — a selection cannot be narrowed once it is made

**Ken:** *"also I can't unselect things once I have selected them
for redaction."*

## O103 — ✅ **FIXED BY THE ENGINE, our half rebuilt the same day** — redaction refuses any region that touches an image

**Ken:** *"every time I've tried the redact feature it tells me it
can't because there is objects that weren't redacted."*

`request_redaction_refuses_any_region_that_touches_an_image.md` — (1) gate on
the pixels rather than the bounding boxes; (2) **remove an image that is wholly
covered**, which needs no pixel surgery and is the common "redact this logo"
case; (3) **refuse per region, not per document**, so twelve good marks apply
and the one that cannot is disclosed as a residual like every other carrier.

**Driven, and falsified BOTH WAYS** —
`marking_over_an_image_says_so_before_apply`. It asserts the warning appears on
a document that is nothing but a raster image **and** that a CAD sheet with no
image is marked in silence. A check with only the first half passes just as
happily on a build that warns about every mark on every document, which is worse
than no warning: a caveat attached to everything is one you learn to scroll past,
and the day it matters you scroll past it too. Forcing `images = 0` reddens the
first half; forcing `images = 1` reddens the second.

## O102 — ✅ **DRIVEN** — closing asks about unsaved work, document by document

**Ken:**

> *"also when I close the program it should prompt to save changes if there are
> any, and it should do what other programs do - switch focus to the document
> that is being prompted for, and cycle through each unsaved document while it
> prompts, but also have a save all button that saves all changed documents."*

**NOT DRIVEN.**

## O101 — ✅ **DRIVEN** — the build time in the top bar

**Ken:** *"also in the next release add the local compilation time
to the top bar at the end of the date you added."*

**NOT DRIVEN.**

## O100 — ✅ **DRIVEN; the preset half is ANSWERED and CONSUMED** — new colour-rendering options

**Ken:**

> *"the engine I think has a couple of new options for colour rendering that we
> might need to surface and set for our standards presets."*

`RenderPreset` covers `page_blend_space_source` and does **not** cover the new
axis. That may well be correct — the preset module's own reasoning notes that
*"a third of the grid is axes a standard does not reach"* — so it is filed as a
**question**, not a demand: *does a PDF/X or PDF/A level constrain the spot
colorant device model, and if so should the preset pin it?*

Filed as `done_2026-09-02-spot-device-model-REQUEST.md`.

## O99 — ✅ **DRIVEN** — the tab-order list drags, with the caret

**Ken:**

> *"the tab order list is supposed to be able to be reordered by dragging and
> dropping rows around like we can with pages in the page preview, and have clear
> markers of where the field is going to move to."*

### ⬜ Filed, and NOTHING was built

`request_a_pages_tab_order_cannot_be_changed_at_all.md`, asking for
`reorder_annotations(page, &[usize])` with `reorder_pages`' contract — plus the
one question that cannot be answered from outside: **should it also write
`/Tabs /A`**, so the order you arranged is the order the file *states* rather than
an order some readers happen to follow?

Checks: `tab_order_drag_moves_a_field_and_shows_where`.

## O98 — ✅ **DRIVEN** — the panel points, the canvas lights up

**Ken:**

> *"when we have the fill form panel visible and I click on fields in it instead
> it should highlight the field on the canvas that is being filled."*

> *"the old shell's Forms panel did draw on the canvas: hovering a row
> highlighted the field's rectangle on the page … It was answering a real
> question — 'which of these is the one I am about to type into?' — and the
> answer is welcome under rule 4's fourth clause, which permits 'a snap
> indicator, a hover highlight, a rubber-band, a selection handle — these are
> the cursor'. It is still not carried, and the reason has **changed**: the
> mechanism now exists … so what is missing is only the panel→canvas channel."*

Checks: `clicking_a_form_row_lights_the_field_on_the_page`.

**NOT DRIVEN.**

## O97 — ✅ **DRIVEN** — the display buttons are on two rows

**Ken:**

> *"our display buttons should be on two rows to save space."*

**NOT DRIVEN.**

## O96 — ✅ **DRIVEN** — the fillable fields are shaded

**Ken:**

> *"in our display section we should have an option to shade the form fields like
> acrobat does."*

Checks: `fillable_fields_are_shaded_on_the_page`.

## O95 — ✅ **DRIVEN** — Save As, and then keep editing the NEW file

**Ken:**

> *"we need a Save As option so that we are then making edits in the save as file
> instead of the original just like other programs have it."*

**NOT DRIVEN.**

## O94 — ✅ **BUILT AND DRIVEN** *(back-filled — see above)* — OCRed text can be copied

**Ken:** *"also I can't seem to copy and paste text we have OCRed"*

Checks: `text_on_a_scan_can_still_be_swept_over_the_image`.

## O93 — ✅ **BUILT AND DRIVEN** *(back-filled)* — OCR says what it is doing, and Stop and Cancel differ

**Ken:**

> *"can you make it so the recognizing ocr gives feedback on what it is doing
> when it is running (pages done, words/characters detected, etc) so that the
> user can see that it is doing something and hasn't frozen on large documents?
> Maybe a cancel and stop button too. The cancel throws away what was done, and
> the stop finished the page it is on and keeps the work it has done."*

**Three driven checks, in `tools/ui-verify/src/checks/ocr_progress.rs`**, each
run twice — once on a committed eight-page synthetic fixture and once on his
scan.

Cited: `widgets/spinner.rs:40`.

Checks: `cancelling_ocr_throws_away_what_it_had_done`, `ocr_recognises_a_page_and_the_document_keeps_it`, `ocr_says_how_far_it_has_got_while_it_runs`, `stopping_ocr_keeps_the_pages_it_had_already_done`.

## O92 — ✅ **SHIPPED AND DRIVEN** — reaching an object dropped off the side of the page

**Ken:** *"we should be able to select things offside of the page,
especially since I sometimes drop objects there, and when I do I can't get them
back."*

**Driven**, on a purpose-built fixture, and **falsified** — with `mode_for`
stubbed to return `Enclosed` for both directions it fails with *"THE BAND
REACHED INTO THE MARGIN AND FOUND NOTHING"*; restored, it passes.

## O91 — ✅ **SHIPPED AND DRIVEN** — a clickable table of contents works

**Ken.** The second half of the same message as O90.

> *"also I don't think I have an example in that folder, but I think the what's
> new pdf on the desktop might have a table of contents that you can click on
> and be sent to the appropriate section. I could be wrong though as I am not at
> the PC to try. anyway, it didn't do that in ours, but I didn't confirm in
> Adobe either."*

**You are right that it does not, and the shell-side cause is total: there is no
link-following code path at all.** Not a broken one — none. Clicking a `/Link`
does nothing, and nothing is drawn to suggest it would.

Filed as
`request_a_links_destination_cannot_be_read_so_a_table_of_contents_is_dead.md`.

Checks: `a_link_goes_to_the_page_it_names`, `a_link_it_cannot_follow_says_so_instead_of_jumping`.

## O90 — ✅ **FIXED** — a bookmark lands on the detail it names

**Ken:**

> *"in Acrobat clicking on the nested bookmarks in the drawing package takes you
> to a zoomed in area of the page for the drawing bookmark that was clicked on.
> when we click on ours it just jumps us to the correct page, but doesn't send
> us to the spot on the page the bookmark actually points to."*

Checks: `a_bookmark_lands_on_the_detail_it_names`.

## O89 — ✅ **"I don't see where I am able to edit the color of text, vectors, etc."** — CLOSED: the route exists now, and so does the whole-selection recolour

**Ken.** Two different answers in one sentence.

**Filed:** `request_a_paths_colour_cannot_be_changed_at_all.md`, asking for
the verb **and its reader** — a swatch that cannot show the object's current
colour is one that silently discards it on first touch — plus a named refusal
for spot inks, because writing `DeviceRGB` over a named separation would look
right on screen and destroy the plate on your drawings.

**Status:** ✅ **CLOSED.** Vectors ship, one shape or a whole box-full;
text colour is on the shape you click. The route question was decided rather
than asked, per your standing instruction: *"Always add new features. never ask.
just do."* All three
candidates are now in the build; two of them turned out to be there already, and
the third had shipped **wrong** for the one control you actually went looking
for.

⬜ **NOT VERIFIED, named rather than implied.** The driven check
`clicking_text_offers_its_colour` is written and registered and **has not been
run** — the session that wrote it was forbidden to take the desktop. Its module
header carries the falsification table. What *is* verified: sixteen unit tests
over the two colour readings, two of them falsified by planting the flattening
defect and watching the named test go red (`left: Some(Agreed([255, 0, 0]))`
where `Some(Mixed)` was required, and `left: Agreed([0, 0, 0])` over a selection
containing a spot ink).

## O88 — ✅ **SHIPPED AND DRIVEN** — a right-to-left box takes what it touches

**Ken, on `TR-0461-1500-copy.pdf`:**

> *"I can't box select the tables in the left or right top corners using the
> mouse — it only picks up the lines of each table, so I can't drag the entire
> thing and move it somewhere else, or cut/copy and paste it elsewhere."*

Checks: `a_marquee_over_a_table_takes_its_text_as_well_as_its_lines`.

## O87 — ✅ **NOT A DEFECT — an old build.** Paste lands at the cursor

> **Ken, an hour later:** *"Just realized windows wasn't opening the
> latest version. paste puts things where the cursor is."* And: *"it wasn't you.
> It was me. I had linked the default pdf opener to a different location. I
> thought I had relinked it to the new one but it didn't take."*
>
> ### The part that IS ours, and it is now fixed
>
> **Nothing on screen could have told either of us which build was running.**
> That is the whole cost of this row, and it does not depend on how the wrong
> build got launched — an operator describes a defect that was fixed, and an
> engineer investigates a version nobody is running.
>
> The build stamp existed the entire time. `build.rs` sets it and the About
> window shows it — two clicks behind a menu nobody opens while they are
> working.
>
> **The build date is now in the window title.** Taskbar, Alt-Tab, a
> screenshot, the accessibility window list: all read it, and none of them can
> see a menu. The day rather than the minute, because the question a title has
> to answer at a glance is *"is this today's?"* — the exact time and the commit
> stay in About for anyone comparing two builds precisely.
>
> The diagnostic added while chasing this stays, and it earns its place: the
> paste's fallback now says WHICH half was missing (`offset-no-cursor`,
> `offset-no-anchor`, `offset-neither`) rather than only that it fell back. Two
> causes, opposite investigations, and the trace used to be silent about which.

## O87 (original) — Paste lands near the copy, not at the cursor

**Ken:**

> *"copy and paste still doesn't paste where the cursor is, it just pastes near
> the copied object."*

**Status:** ⬜ **OPEN — instrumented, not fixed.** The next step needs one run
with the diagnostic on, doing the paste **the way you do it** — the route
matters, and a ribbon press and `Ctrl+V` differ precisely in where the pointer
is at the moment the command fires.

## O86 — ✅ **FIXED** — filled fields size themselves to the box

> **Your text now fits the field it is in.** On your Weber form:
>
> | | box | was | now |
> |---|---|---|---|
> | a description row | 27.8 pt | 12 pt | **22.4 pt** |
> | a header field | 13.1 pt | 12 pt | **9.6 pt** |
>
> The second row is the half that was about to bite you: 12 pt in a 13 pt box
> overflows, and every header field on that form is that size.
>
> **And pdfcer now tells you which way it decided**, in the words it uses:
> *"fitted to the field's HEIGHT; make the box taller to change it"*, or
> *"shrunk to fit the field's WIDTH"*, or — the honest third case — *"held at
> pdfcer's legibility floor; the box is too small for this text, which will
> overflow"*.

> *""FullName" is too small for this text. pdfcer held the size at 4.0 pt so
> it stays readable, which means the text will overflow the box — make the field
> taller, or shorten what is in it."*

## O86 (original) — Filled fields come out the wrong SIZE

**Ken:**

> *"when I fill out the form fields on `TRP5188 - Weber Supply.pdf` the font
> doesn't match what is set for the fields. Adobe uses the same font and size as
> in the first filled out row below the headers. 'TC-10 Wheel Chocks' is in the
> font that should be showing for the other fields."*

**Filed:** `request_auto_sized_field_text_is_a_flat_12pt_and_acrobat_fits_the_box.md`,
with both measurements and the derivation.

**Status:** ✅ **FIXED** in `pdfcer-core` `Pass 215.0`.

## O85 — ✅ **NOT A DEFECT — an old build.** Ctrl+S closing the program

**Closed by Ken:** *"the save bug was due to running an old version
and is no longer present."*

Worth keeping rather than deleting, and for the same reason O87 was: **this is
the second report closed by "you were running an old build"**, and two of a kind
is a pattern rather than a coincidence. The published slots alternate, so the
folder he opens is not always the newest — and nothing in the program tells him
which he is running without opening About.

## O85 — ⬜ **"I pressed Ctrl+S to save and it closed"** — NOT REPRODUCED YET

**Ken:** *"can you try doing an edit and save? I did this and
pressed ctrl+s to save and it closed."*

Step 4 is `O65` — *"it closes the document after saving"* — which was fixed and marked **NOT DRIVEN**. It is driven now, for the first time,
and it holds.

**Status:** ⬜ **OPEN, and blocked on one answer from you.** Everything that can
be checked without knowing which edit it was has been checked.

Checks: `ctrl_s_after_an_edit_saves_and_the_program_is_still_running`.

## Q3 — ⬜ TWO THINGS THE ENGINE SHIPPED TODAY THAT ONLY YOU CAN SCOPE

**Not a request of yours — a question to you**, filed here rather than asked in
conversation because that is what this file is for. `pdfcer` released **0.18.0** and two of its headline capabilities have **no GUI surface and no
plan for one**, because whether they should is a product decision rather than an
engineering one.

> **The question:** what would *"edit the internals"* mean as a button?
>
> This is genuinely powerful and genuinely sharp. The signature-preservation
> property is the interesting half. But a menu item called *Edit internals* that
> opens 40 MB of PDF syntax in Notepad is not a feature, it is a trap — and I do
> not know what the safe shape is without knowing what you would use it for.
>
> **If you have a case in mind, tell me the case** and I will design to it. If
> not, it stays on the command line, where the people who want it can find it.

Filed to `D:/dev/rag/egui/`, because the general form is worth keeping: when
two call sites share a predicate, extracting the predicate is half the job —
they must also agree on **when to ask it**.

The program is more correct and the suite is less independent. The check now
looks before it toggles. Also filed to the RAG.

⬜ **One flaky check, named rather than fixed.**
`the_format_tab_offers_font_controls_for_swept_text` skips in the suite and
passes alone — a text-sweep check whose swept-character count varies tenfold is
measuring the harness's drag timing as much as the feature.

## O80 — ✅ **SHIPPED AND DRIVEN** — a page-display choice reaches the next document

`a_page_display_choice_survives_a_close_and_reaches_a_new_document` opens
`four-pages.pdf`, presses **Facing** on the ribbon, closes the window
**immediately with Alt+F4**, and then opens `paragraph.pdf` — **a document the
program has never seen** — in a second process. It opens facing, from the
standing preference.

```
page-display mode=facing source=preference ribbon-mode=read
```

## O80 — ✅ BUILT, not yet driven

> **Two causes, and the second was a function written for an exit path that did
> not exist.**
>
> ✅ **A standing page-display preference.** It was already remembered *per
> document*, written the moment you press the control — but there was no answer
> for a document the program had never seen, so a choice made on one drawing
> meant nothing on the next. Three tiers now: this document's own record, your
> standing preference, then the mode's rule. Pressing a page-display button
> records it as the standing preference, which is your sentence exactly.
>
> ✅ **And the program had no exit hook at all.**
> `LayoutStore::flush` says what it is for in as many words — *"for an exit
> path, which must not lose the last change to a debounce that had not yet
> expired"* — and had **no production caller**. So the layout is written 750 ms
> after it changes and a change made in the last three quarters of a second
> before you closed the window was silently thrown away.
>
> That reaches you through page display: the active ribbon **mode** rides in the
> layout file, and the mode picks the default for a document with no remembered
> entry. Switch to Edit, close quickly, reopen in Read, get continuous.
>
> **Wheel paging was already persisted correctly** — status-bar toggle and
> Settings both write. One caveat found while checking: the toggle is hidden
> under a continuous display, so in Read mode the control you pressed last time
> is not on screen, which reads identically to "it forgot". The page-display fix
> above removes that.
>
> **Verified:** the prefs round trip (which caught that the writer must not
> invent a key for an unstated preference) and a precedence table. **NOT
> driven.**

> *"Also it should remember my page display preferences from my last closing of
> the program. Example if I press show one page at a time and enable flip
> pages."*

**Status:** not investigated.

## O79 — ◑ HALF BUILT, and the other half is a question for you

> ✅ **"The pages I picked in the thumbnails" now exists**, as a fourth scope in
> the Recognise-text window, drawn only when something is picked and labelled
> with the count. The rail's selection is already the operand for delete,
> extract, rotate and the page clipboard; OCR was the one page-scoped verb that
> ignored it.
>
> ⬜ **The half I cannot close from here: All pages and a typed range ALREADY
> EXIST**, and have, in the build you are running. All pages is
> even the default. So *"still only has a button to recognize this page"* is a
> report about the ROUTE, not about the feature — something is wrong with how
> that group presents and I cannot tell what from here.
>
> The candidates are ones this project has been caught by twice: a control
> published to the harness but scrolled out of its pane, or a window whose first
> group is above the fold. There is no check asserting the scope group is
> **visible** rather than merely declared, which is its own defect.
>
###, his answer: *"there is only the option to do the page, no
### radio buttons or anything else."* — and I still cannot reproduce it

> *"Also OCR still only has a button to recognize this page, I should have
> options to do the whole document, or the pages I have selected in the
> thumbnails."*

**Status:** not started.

## O78 — ✅ BUILT, not yet driven

> *"when I change the size of the canvas window, whatever area was centered in
> the current canvas should stay centered, and unless I have manually changed
> the zoom after clicking one of the preset options, the pdf should maintain
> whichever option was selected - Fit Width, Fit Height, or Fit Page. Also when
> starting the view should be centered on the canvas when a pdf is first
> opened."*

**Status:** not investigated.

## O64 — ✅ FIXED BY THE ENGINE, same day, and our tests inverted

> **You can move a picture the moment you place it.** No save, no reopen.
>
> **And the half you did not report was the dangerous one.** Chasing your
> sentence *"I assume this probably affects more than just images"* turned up a
> second symptom nobody had seen: after deleting a page, an edit made on what
> you see as page 1 was being committed to **a different sheet**, silently,
> with no refusal and no message. The engine team reproduced that before fixing
> it. It is gone too.
>
> **What it was:** every content-editing verb in the engine read the document
> *as it was on disk*, while everything that adds content wrote into the
> session. So the picture you had just placed did not exist as far as the verb
> that would move it was concerned — which is exactly why saving and reopening
> made it work.
>
> **How it went:** filed at 12:48 with a reproduction attached, answered at
> 14:25 the same day (`pdfcer-core` Pass 186.0). The three tests that proved the
> defect are now three tests that guard against it coming back, and two of them
> assert an outcome rather than an `Ok` — a verb that transformed nothing and
> reported success would be the same complaint wearing a different face.
>
> **Verified:** `cargo test --workspace`, on every commit.

> *"When I add a new image to a pdf I can't edit it unless I save the document
> first, at which point it closes the document after saving. When I open it I
> can then edit the image. I assume this probably affects more than just
> images."*

**Status:** not investigated.

Filed as `request_edit_verbs_read_the_base_not_the_overlay.md`.

## O65 — ✅ BUILT, not yet driven

> **Save never closed the document. What closed it was the step you took next,
> and the chain was driven rather than reasoned about.**
>
> A successful save recorded `saved_epoch` and **nothing in production read
> that number**. Every surface asking "does this have unsaved edits?" asked a
> different question a save cannot answer — so the tab kept its dot, the next
> Close raised the unsaved-edits prompt, and that prompt's only save button was
> "Save a copy…", a picker, which on success proceeds with the close. Press
> save, get asked for a filename, watch the document close.
>
> Now: one predicate, `save::has_unsaved_edits`, read by the prompt, the tab
> strip and the close arm. And the prompt has a **real Save** when there is a
> file to write over — absent, not greyed, otherwise.
>
> A green test was holding the old answer in place. It refused any label
> reading as a Save over the open file, *"which this build cannot do"* — true
> when written, false since Save landed, never revisited. It
> would have failed this fix.
>
> **Verified:** a truth-table unit test walking all five states. **NOT driven.**

> *"… I can't edit it unless I save the document first, at which point it closes
> the document after saving."*

**Status:** not investigated.

## O66 — ✅ BUILT AND DRIVEN — and driving it found a mirrored placement

> *"Also anything we are inserting like this should have an option in its
> dialogue box to place it with the mouse instead of by positional
> co-ordinates."*

**Status:** built, gates green, driven.

Checks: `the_insert_window_steps_aside_so_you_can_point`.

## O67 — ✅ BUILT AND DRIVEN

> **Drop a drawing onto the thumbnails and its pages go in where you pointed.**
> The caret shows the gap while the file is still in the air, exactly as it
> does when you drag a page from another open document, and dropping several
> files at once stacks them in the order you dragged them.
>
> **Measured:** a 4-page file dropped on the left half of the second thumbnail
> of a 36-page drawing → 40 pages, inserted before page 2.
>
> **The position does not exist in the toolkit and had to be asked of
> Windows.** `winit` receives the drop point from the operating system and
> throws it away — twice, once for the hover and once for the drop — and no
> mouse-move messages arrive during a drag either, so the toolkit's idea of
> where the pointer is was stale from before the drag started. Without that
> one syscall, *"drop it on the thumbnails"* and *"drop it anywhere"* are the
> same event and this could not have been built at all.
>
> **What a drop that is not a drawing still does:** exactly what it did
> before. An image goes to the placement window, an unreadable file opens so
> the parser can say what is wrong with it, and a drop anywhere else opens in
> a tab. Every refusal is a fall-through rather than a message, so the failure
> mode is *"it opened instead of importing"* — visible and undoable — and
> never a file that vanished.
>
> **Not driven, and said plainly:** the caret drawn while a file HOVERS. A
> harness cannot originate a drag from Explorer — that is a protocol between
> two processes — so the check holds a simulated drop back, parks the real
> cursor on a real thumbnail, and lets the application read the real position.
> The drop path is therefore driven end to end; the hover feedback that
> precedes it is not.
>
> **Verified:** driven — `a_drawing_dropped_on_the_thumbnails_becomes_pages`,
> and falsified: a build that ignores the pointer reports `gap=36` and the
> check names it as the position-blind case.

> *"I should be able to drag and drop documents into the thumbnails section of
> another pdf to import the pages."*

**Status:** built, gates green, driven.

## O68 — ✅ BUILT, not yet driven — and it found two more

> *"Also the Merge files and Split files buttons don't do anything."*

**Status:** not investigated.

## O69 — ✅ BUILT, all four, not yet driven

> **The two remaining halves landed the same day.**
>
> ✅ **No box over the nodes.** The outline was stroked in an unconditional loop
> with no rung test anywhere on the path, so at the Part and Node rungs it was
> drawn on top of the very anchors you were trying to see. And the move ghost
> stroked a *translated copy* of it — for node drags too — so dragging a point
> gave you the box AND its ghost, which is O63's *"it just had a perimeter box
> around it"* surviving inside the gesture O63 was about.
>
> ✅ **The nodes are visible and hittable.** The unselected anchor was a 6 px
> HOLLOW square, 1 px of accent over black CAD linework. It is filled now, at
> 7 px, and the catch radius went to 8 — which is what a Bézier control point
> already got, so an anchor stopped being harder to hit than the handle hanging
> off it. Object picking is untouched.
>
> **And on a dense path there were NO dots at all.** The 400-anchor cap
> counted every anchor in the path, so a contour over 400 points drew nothing
> and published no regions — you armed Points, clicked, watched the box change,
> and the program went quiet. It now counts what is **on screen**, so zooming in
> makes the dots appear, which is already the gesture you perform to work on a
> point and which made no difference at all before. And it now says so when the
> cap fires, which on that route it never did.

> **Two of your four complaints are fixed, and one of them was worse than you
> said.**
>
> ✅ **`Edit ▸ Edit Objects` is deleted.** You said we should not need it. It
> was worse than redundant: it was an alias for the **arrow tool**, so pressing
> it after arming Points put you back on the black arrow — the control you had
> been told to press in order to edit a drawing was the one that ENDED node
> editing. Its tooltip promised *"drag an anchor to move that node"*, which is
> the Points tool described exactly, by a button that armed a different one.
>
> ✅ **The Points tool is withheld outside Edit** instead of declining in
> silence. It has always needed Edit mode and its arm said nothing on screen —
> drawn, enabled and inert in two of three modes. The `A` chord still reaches
> it and now says *"Switch to Edit to work on points."*
>
> The route is now one control: **View ▸ Navigate ▸ Points**, in the tool
> palette order every program in this class uses.
>
> ⬜ **Still open: the bounding box drawn around an object whose nodes are
> showing**, and the nodes being hard to see and hit. Both are understood — the
> outline is drawn unconditionally at every rung, and the grip radius is 6 px
> against Inkscape's 8 — and neither is done.

> *"I'm still not entirely clear how to reliably get to a point where I can edit
> nodes. It seems like I click on Edit-> Edit Objects, but also have to click on
> View-> and the node selector under Navigate, then double click several times,
> but then the nodes are hard to see and click on. If we are at a point where we
> are showing the nodes in an editable state there shouldn't be a bounding box
> around the objects. We shouldn't even need an Edit Objects button. In edit
> mode I should be able to click Navigate -> select points, then single click an
> object to see its points and single click the points to edit."*

**Status:** not investigated.

## O70 — ✅ COMPLETE, BUILT AND DRIVEN → 09-01

> **Clicking a wrapped drawing now selects the drawing.** On a CAD sheet whose
> content was placed as one piece — a title block, a stamped detail, a symbol —
> a click used to select one line inside it and there was no way to select the
> piece at all except a Format-tab command you had to know existed. Now a click
> selects the piece and a **double-click goes inside**, which is the Inkscape
> convention you named.
>
> **The switch is in View ▸ Navigate**, beside the four pointer tools, on by
> default, and it remembers itself across restarts. Escape steps back out —
> once to drop the selection, again to leave — because losing the container you
> are working inside to a stray deselect would be worse than the extra press.
>
> **Measured, driving the real binary:** click → the container; double-click →
> `smart-enter` and the object inside it; Escape, Escape → out. The first
> Escape must NOT leave, and the check asserts that too.
>
> **A fixture had to be built for it, and the reason is worth reading.**
> Neither of your drawings could test this at the zoom they open at.
> `SW41177.pdf` contains no wrapped content at all. The benchmark site plan
> does — one container over 10,256 pieces — but pdfcer's click tolerance is six
> screen pixels, which on a sheet opened to fit is about fifteen points of
> paper, and at that radius the big page-level objects win everywhere. So the
> feature is reachable by you, zoomed in, and unreachable by a harness aiming
> at a fitted page. That is a fact about the document, not the feature, and the
> answer was a small purpose-made file.
>
> ### ✅ …and what you go inside can now be MOVED and DELETED —
>
> Drag a line inside a title block and it moves. Press Delete and it goes. Both
> reach the engine as one undoable command, driven and asserted.
>
> For the day between the two halves this shell could *reach* something
> inside a wrapped drawing and not move it, which is worse than not reaching
> it: the selection outline is a promise the gesture then breaks.
>
> ### ✅ …and the chain now runs all the way down —
>
> Double-click again inside a container and you reach the piece's parts, with
> its points drawn, and dragging one commits. The ladder goes exactly as deep
> inside a wrapped drawing as outside one, which is what *"until a double click
> reaches the bottom"* asked for.
>
> Four things had to become true in the same commit, and any one alone would
> have made it worse: the hit test had to ask about the piece rather than a
> page position, the descent guard had to go, the points had to be drawn, and
> the drag had to reach a verb. Descending without the other three would have
> made the selection box **vanish** and offer nothing in its place.
>
> And driving it caught the fifth thing. Everything above was right and the
> drag still refused — one line was still asking *"what kind of parts does this
> page object have?"* about something that is not a page object, so the answer
> was "none" and the refusal arrived after the operator had already entered the
> rung and seen the points.
>
> ### ✅ …and the drag looks the same inside as outside —
>
> Dragging something inside a container shows **the shape moving**, not a box
> round it, and the curve handles are drawn and draggable there too. Driven:
> the preview builds its geometry while the drag is in flight.
>
> Both were one line each once the piece could be asked what it is made of,
> which is the point of having built that first.
>
> ### ✅ …and double-clicking a text box types in it —
>
> The last rung, and the one where *deeper* means something different: below a
> piece of text is **the words**, not a smaller shape. Double-click it and the
> caret lands where you clicked, with the text tool armed — which is what
> Inkscape and Illustrator both do, and what stops you being left with a caret
> blinking while the arrow is still the tool.
>
> **This row is now complete.** Every part of what you asked for in it is built
> and driven: the checkbox, the container click, the descent, the editing
> inside, the preview, the handles, and this.
>
> **Verified:** driven —
> `a_click_selects_the_whole_drawing_and_a_double_click_goes_inside`.

<details><summary>The row as filed</summary>

> *"we should have a checkbox in navigate for a Smart-Selector option, in Edit
> Mode this makes it so if I click on an object and it is enabled to be selected
> in the Smart Selector it puts it in a bounding box with handles to move, resize
> and rotate, if a click selects an object that is made of multiple objects
> (group, form, etc) a double click should bring me further down the chain, until
> a double click reaches the bottom and lets me edit the nodes. If I recall this
> is similar to how Inscape does things and we should follow that convention.
> Selecting a text box or similar item does the same thing, but double-clicking
> inside the bounding box should edit the text."*

**Status:** not investigated.

## O71 — ✅ BUILT AND DRIVEN

<details><summary>The row as filed</summary>

> *"In read mode the regular pointer should also allow us to select images so we
> can copy and paste them as well as text outside of the pdfcergui."*

**Status:** not investigated.

Checks: `read_mode_copies_a_picture_other_programs_can_paste`.

## O72 — ✅ BUILT, not yet driven

> *"Click and hold shouldn't select an object - it should allow me to draw a box
> around objects to select."*

**Status:** not investigated.

## O73 — ✅ BUILT, not yet driven

> *"When I cut or copy and object, when I paste it should paste where the mouse
> cursor is sitting."*

**Status:** not investigated.

## O74 — ✅ BUILT, not yet driven

> *"When I make edits or even just fill out a form I notice all of the page
> previews get re-rendered instead of just the one that is being changed, and it
> seems to really slow down clicking a checkbox in a form. The last thing that
> should matter is updating the preview, and it should just update the pages that
> were actually altered."*

**Status:** not investigated.

## O75 — ✅ BUILT, not yet driven

> *"When I am working in the right side panel objects are getting selected
> through the side panel when I am trying to edit fields in the Properties
> section. Also the Properties section is always showing the This document
> properties instead of just the properties of the objects I am editing."*

**Status:** not investigated.

## O76 — ✅ FIXED — the engine answered and both routes are wired

> **The cause is neither of the two this row guessed.** The outline does not
> thicken because pdfcer wrote a bigger border width. It thickens because
> **nothing was rewritten at all**: the engine redraws a field's appearance for
> Text and Choice fields only, and a check box is a button — so the appearance
> pdfcer itself drew, at the ORIGINAL size with a hard-coded 1 pt stroke, is kept
> and the PDF placement matrix stretches it into the new box. Drag a 12 pt check
> box to 40 pt and its 1 pt border draws at about 3.3 pt.
>
> That is exactly the case the ANNOTATION resize **refuses by name** — *"a
> foreign appearance cannot be rebuilt without replacing somebody else's artwork
> with pdfcer's rendering of it"* — and the widget path takes it silently, on
> artwork pdfcer drew and could therefore rebuild exactly.
>
> ✅ **Shipped: pdfcer stops lying about it.** The sentence was chosen on
> "was it resized" and said *"its contents were redrawn to fit"* — a claim the
> very outcome it was reading denied on the next field. There is now a third
> case saying the contents are stretched, plus the trace this verb never had,
> which its two siblings have always emitted.
>
> **The fix is the engine's** and is filed:
> `request_resizing_a_check_box_stretches_its_appearance.md`. It asks for
> two things — that a button's appearance be redrawn after a `/Rect` change, and
> that `WidgetEdit` carry the same three scale answers `ResizeOptions` already
> takes, so your Tool-row switches reach a form field at all.
>
> ⬜ **Corner radii: nothing to scale, and that is a finding rather than a
> refusal.** A form field's border style carries no radius, and pdfcer's own
> check-box artwork draws square corners. What you are seeing thicken is the
> square artwork, which the engine row fixes. A genuine "scale rounded corners"
> toggle needs a rounded-rectangle primitive to exist first, and that is a
> feature to scope with you rather than improvise under a bug report.

> *"Form shape outlines of checkboxes and such scale when I drag them larger.
> There is supposed to be an option on the menu to choose the behaviour of
> resizing items - when scaling objects scale stroke width by the same proportion
> and when scaling rectangles scale the radii of the rounded corners"*

**Status:** not investigated.

Checks: `a_resized_check_box_is_redrawn_not_stretched`.

## O77 — ⬜ The standing instruction: sweep everything, do not fix only what was named

> *"Please don't just fix the bugs and add the features for the exact tools I am
> outlining. You need to do a proper sweep and diagnosis to ensure all tools and
> features."*

**Status:** in progress.

## O63 — DONE SO FAR (read this before the analysis below)

1. **Everything that is not a canvas gesture.** *"Live preview for everything we
 do"* also covers a colour change, a Bold press, a delete, a redaction mark.
 None of those has a sprite to slide and none is covered yet — see the
 analysis below on why a *rendered* preview is a second away.
2. **The engine's 430 ms commit, on the UI thread.** Filed; unanswered at the
 time of writing. Until it moves off the UI thread the window still stops
 answering the pointer for half a second per edit on a dense drawing, and no
 preview fixes that.
3. **Saying the page is catching up.** The third piece holds the picture; it
 does not yet say why. One sentence on the status line, owed.

Checks: `dragging_a_node_bends_the_line`, `turning_a_field_right_turns_it_right`.

## O63 — ◑ The program keeps up with your hand on the CANVAS — the rest is open

**Ken:** *"we need to make it so we have a live preview as we drag
and move and resize and rotate, etc around the canvas. The live preview should
remain while the update to the pdf structure runs in the background. This should
just cache each one as the user does their edits so to them everything looks
WYSIWYG and the delay in updating the actual isn't noticable. If the user gets
too far ahead, then it will pause and update."*

**Ken, clarifying:** *"to clarify live preview request is for
everything we do."*

**Ken:** *"yeah do both. but to be clear at least last time I checked
if I moved the end of a line, it didn't show me the shape change of the line, it
just had a perimeter box around it. this goes for anything I change right now.
there isn't a real preview like there is in inkscape."*

1. **Opening is free, reading is free.** 3.6 ms to load 5.6 MB; `view()` is
 unmeasurable. The cost is **the content stream**, not the file and not the
 object graph.
2. **`decompose_page` (501 ms) and `move_objects` (434 ms) are within 15 % of
 each other**, which reads as *the verb's cost is essentially one
 decomposition*. If so, every content edit on this page carries the same
 ~450 ms floor — moving one line costs what moving ten thousand costs.
3. **The shell then pays for a second one.** `app::cache::page_objects` is
 keyed on `(page, edit_epoch)` and the commit bumps the epoch, so the
 decomposition is discarded at the moment the verb returns and rebuilt on the
 next frame. **~500 ms of duplicated work per edit, pure loss** — the same
 stream parsed twice because the two parsers cannot see each other across the
 crate boundary. Filed:
 `request_one_edit_costs_two_decompositions_of_the_same_page.md`.

1. **Which half is slow.** See above. Everything below is conditional on (a).
2. **What the preview IS** for a non-geometric edit. There is no sprite for
 "this text is now bold"; the only general preview of an edit is *the page
 rendered with the edit applied*, which is the expensive thing being deferred.
3. **What happens when the engine REFUSES** an edit the preview already showed.
 This has no honest answer yet and it decides the design. Note it is
 **entirely a problem of (a)** — under (b) the engine has already accepted
 before anything is drawn.
4. **The queue depth**, and what *"pause"* looks like. A frozen pointer is worse
 than a slow one.
5. **Undo grouping.** One entry per gesture, not per frame — and `EditSession`
 has no grouping verb, which is already filed.

Where the preview lives, so the next reader does not have to find it:

- `canvas/painting.rs:497` — the shape itself following the pointer, drawn
  above the bounding ghost and below the snap marker, with the rule-4 argument
  for why a cursor may be drawn and applied content may not.
- `app/state/heldpreview.rs:112` and `:176` — the hold that outlives the
  gesture. `shape_preview` is `None` at release; the raster still shows the
  object where it started for a second or two, so dropping the preview there
  makes the object snap back and jump forward.
- `canvas/handledrag.rs:216` — why a node drag previews the pointer position
  rather than a ghost of the curve: the Bézier belongs to the engine.
- `canvas/rotating.rs:340` — a gesture publishing its in-flight outcome, which
  is the shape every canvas gesture uses.
- `canvas/moving/mod.rs:9` — one gesture is one command, which is why the
  preview cannot be emitted per frame as a series of small commits.
- `canvas/present.rs:672` and `canvas/backdrop.rs:88` — the previous texture
  held while the new raster is computed, and the backdrop under it.
- `app/actions/funnel.rs:60` — the commit funnel the release goes through.

## O62b — ✅ Bold stopped using real bold fonts, and nothing failed

**Found, not reported.** It arrived inside the same `cargo update` that built
the O62 release, and it is the reason that release was rebuilt.

Press **Bold** on a page that carries a real bold face — a title block set in
Calibri with Calibri-Bold sitting right there in the page's own font list — and
pdfcer **thickened the Calibri instead of using the Calibri-Bold**. Artificially.
Into the saved file. Every other viewer would show the fake.

## O62 — ◐ Turn a form field's box · Say something other than the measurement — BUILT, NOT DRIVEN

**Ken:** *"finish those 2 then release."*

**Status:** ◐ **BUILT AND RELEASED, NOT DRIVEN.** Two checks written, blocked on
the machine, first job next session.

Checks: `a_form_field_can_be_copied_and_pasted_both_ways`, `turning_a_field_right_turns_it_right`.

## O61 — ✅ pdfcer tells you when a document phones home · ✅ AND buttons can now be given actions

> ### CLOSED — you can make a button do something, seven ways
>
> **Draw a button and pdfcer asks what pressing it should do.** Seven answers:
>
> | | reaches |
> |---|---|
> | Nothing | — |
> | Clear the form | nothing outside the document |
> | Go to a page | nothing outside the document |
> | Move through the pages (next / previous / first / last) | nothing outside the document |
> | Show or hide fields | nothing outside the document |
> | Open a web address | writes an address; pdfcer never opens it |
> | Send the form's data | writes an address and a declaration; pdfcer sends nothing and has no way to |
>
> Every one of the seven says which of those it is, in a sentence under the
> chooser — including the five that reach nothing, so the two that do are not
> the only ones carrying a line. A disclosure that appears only on the risky
> choice is one people learn to skip.
>
> **The submit says four things nobody can guess**, and Acrobat says none of
> them: hidden fields are sent, fields whose characters are masked are sent as
> plain text, a field that names a file on your computer sends that file's
> contents, and the message carries this document's own location on disk.
>
> **No web address is blocked.** An unencrypted one is *said* to be
> unencrypted and you decide. Blocking it would be pdfcer inventing a rule the
> standard does not state.
>
> **And nothing is marked on the page.** A button that submits looks exactly
> like one that does nothing, because that is how the saved file will look.
> What it does is said off the canvas, never on it.
>
> ### The part worth your attention, because it is a process failure rather than a feature
>
> **The engine shipped this and this shell did not notice for two
> days.** The reply even said *"please check your own copy — your surface is now
> saying something untrue."* It was read, filed and answered, and the Button
> tool stayed greyed anyway, because nothing here failed when the capability
> landed.
>
> That is fixed rather than apologised for: a new build gate now fails whenever
> pdfcer gains something this shell neither uses nor has written a sentence
> about. On its first run it found five more.

**Ken:** *"I think pdfcer added support for several button features
and protections for outgoing submits. implement everything available."*

The filed request stands and is unanswered. Nothing to implement
here yet — and I would rather tell you that than build something that looks like
it works.

**Status:** ✅ **the phone-home disclosure is SHIPPED AND DRIVEN.** ⬜ **button
actions remain an engine policy decision, filed, unanswered.**

## O60 — ✅ Redact by selecting on the canvas · ✅ AND push buttons that actually do something

> ### THE SECOND HALF CLOSED
>
> *"do push buttons work for some features and can we now add them?"* — **yes to
> both, and the answer to the first half was always yes.** Buttons in somebody
> else's form have always kept working when pdfcer saves the file; it recognises
> every action type it meets and preserves all of them. Only pdfcer's **own**
> buttons were inert, and they are not any more.
>
> The full account is on **O61**, which is the row that asked for it.

**Ken:** *"the redaction tool — am I able to select objects on the
canvas and redact them that way yet? I only tried it when it only worked with
the search box and it didn't work for some things. it just told me it couldn't.
also do push buttons work for some features and can we now add them?"*

**You were right that it couldn't, and right about why.** There were exactly two
routes: the search box, which reaches *text pdfcer can read as text*, and *mark
whole page*, which reaches everything. On a CAD drawing almost everything worth
redacting is in the gap between them — a title-block value drawn as **vector
strokes**, a scanned stamp, a logo, a signature image, a run in a font whose
encoding cannot be mapped. There is nothing you could have typed that would have
found any of them. *"It couldn't"* was the program being honest about a route,
not a bug in it.

**Driven and falsified:** `a_selected_object_can_be_marked_for_redaction`. It
asserts the mark count went up **and** that nothing was applied — a build that
quietly applied on marking would look completely correct and be the worst defect
this feature could have.

**Filed:** `request_a_push_button_that_does_nothing_is_the_only_kind_we_can_make.md`

**Status:** ✅ **redaction-by-selection SHIPPED AND DRIVEN.** ⬜ **push-button
actions are an engine policy decision, filed, awaiting their ruling.**

## O59 — ✅ Cut, copy and paste for **everything**: the engine shipped it, the shell has not consumed it

**Ken, to the engine session:** *"can you make sure we have cut,
copy, and paste available for everything and if not implement?"* → *"yes do all
without stopping."* Then to this session: *"latest release of core engine
ready."*

**Driven and falsified:** `cutting_a_redaction_mark_is_refused_before_anything_is_removed`,
against the engine's own three-mark fixture. With the gate stubbed out it fails
on its first assertion.

**Driven:** `pages_can_be_copied_and_pasted` — copy, paste, and the document
goes from **4 pages to 5**. The oracle is the page count, not the trace lines:
every intent line would be present and correct on a paste that inserted nothing.

In the **Bookmarks panel**, beside Rename and Remove, because that is where
every other bookmark verb already is: a bookmark is edited where it is seen.
Copy and Cut act on the selected one **and everything filed under it**; Paste
puts them under the selection, or at the top level when nothing is selected —
the same rule Add already uses, so there is one place to learn it.

**Driven and falsified:** `a_bookmark_subtree_can_be_copied_and_pasted` — the
outline goes from **5 bookmarks to 6**, three runs in a row. With the paste
stubbed out it fails on the applied-line assertion.

**Status:** ✅ **ALL THREE DONE — awaiting your verdict.** Cut refuses what it
cannot carry, pages copy and paste, bookmarks copy and paste.
The engine half is pinned and verified — run `bash tools/gates/run-all.sh` and
`cargo test --workspace` for the current figures.

## O58 — ✅ Copy and paste a form field: `Ctrl+V` pastes a NEW field, `Ctrl+Shift+V` pastes a DUPLICATE

**Ken:** *"wire the request. ctrl v for paste as new. ctrl shift v
for paste as duplicate."*

Engine request:
`D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\request_form_fields_cannot_be_pasted_and_half_of_it_already_works.md`
— asks for one shape, `copy_field` / `paste_field` carrying a serialisable
`FieldClip` with a `NewField` / `AdditionalWidget` policy. Serialisable because
he copies **between drawings**, and the engine confirmed this morning that
`ObjectClip::to_bytes` does not carry annotations.

**Neither chord is blocked on the reply.** Both ship off what exists; until the
verb lands, `Ctrl+V` discloses what did not come along — off-canvas, in the
status line, never marked on the page (Rule 4).

1. **`Ctrl+V` did nothing.** `egui-winit` raises `Event::Paste` only when the OS
 clipboard holds non-empty text and swallows the keystroke otherwise. The
 existing content-copy path writes a marker for exactly this reason; the new
 field-copy path did not, because that workaround lives at each copy site.
 *A documented platform trap does not protect a code path written after it* —
 filed to `D:/dev/rag/egui/`.
2. **`Ctrl+Shift+V` would have pasted a NEW field.** egui's own
 `is_paste_command` does **not** exclude Shift, so `Ctrl+Shift+V` becomes the
 same `Event::Paste` as `Ctrl+V` with the raw key swallowed and no modifier on
 the event. The plausible assumption — that it arrives as an ordinary key and
 the generic keymap handles it — is wrong, and shipping on it would have made
 the chord silently do the other thing.

*"shouldn't we make it so we are able and then do follow Acrobat's
conventions?"*

**Yes, and it took no work.** The engine had already shipped it. The request
this shell filed at 11:56 was answered at **13:03** — `Pass 167.0`,
`pdfcer_core::formclip`, `copy_field` / `paste_field` — and the reply was sitting
unread in the channel while the workaround was being published.

*"let's make it an option to have it swap to match Acrobat or work the way we
have it now."*

**Driven, both ways, and falsified.** `a_form_field_can_be_copied_and_pasted_both_ways`
drives the default order; `the_acrobat_paste_order_swaps_which_chord_does_which`
drives the other in a second process and asserts the **mirror** — under the
Acrobat order `Ctrl+V` must add a box *without* a name and `Ctrl+Shift+V` must
add one. Measured: it does.

**Status:** ✅ **SHIPPED, LOSSLESS, OPTIONAL AND DRIVEN — awaiting your
verdict.** Nothing is open on this row and no question is outstanding.

Cited: `edit.rs:9364`, `edit.rs:13523`, `canvas/selection/annot.rs:189`.

## O57 — ✅ The grips swallow small objects — BOTH halves closed

**Not his report — found by a driven check**, and filed here
because the remaining half is a design decision that is his to make.

> ***"Always add new features. never ask. just do."***

⬜ **NOT DRIVEN.** The window was not launched. `ui-verify --check
mouse_work_survives_every_render_tier` is the check that found this and it has
not been re-run against the fix.

> *"you should also try zooming in on the atoms of the banana pdf file and see
> what happens when you try to draw a box around a molecule and move it, or
> select the ion and move it, or edit the nodes at that scale. you should check
> all the mouse actions and capabilities out at each scale where our scaling
> algorithm changes."*

`ui-verify --check mouse_work_survives_every_render_tier` drives
`banana.pdf`, steers a closed loop onto the pair of cells at PDF (540, 560) —
**0.85 pt across, drawn life size** — and at every rung selects them, presses
the exact centre of the published `canvas.selection-outline`, and drags 40 pt.
Measured, with the grip box the application itself publishes.

**Status:** ⬜ **OPEN — half shipped, half is a question.** The shipped half is
in `canvas::handles::MIN_BODY_STRIP_PX` with its measurement. **Driven** by `mouse_work_survives_every_render_tier`, which fails on today's
build with the table above; it was first found by `widget_move`, whose own press
had to be moved to a quarter of the box width to get past the grips at all —
which is itself the report.

## O56 — "Confirm that you have built every editable surface into the GUI that has been implemented in pdfcer"

> *"confirm that you have built every editable surface into the GUI that has
> been implemented in pdfcer. continue and loop until the handoff items and these
> other things are done."*

**Status:** **PARTIALLY SHIPPED, and the row stays open until every gap in
`EDITABLE_SURFACES.md` is either wired or carries a dated reason.** Shipped: the note editor, author-time opacity, bookmark rename and delete,
both settings fixes, the instrument and the register.

**NOT DRIVEN.** `a_note_can_be_written_onto_a_shape_that_exists` and
`a_bookmark_can_be_renamed_and_removed` are written and have never run — the
machine was his. Every driven claim in this row is owed a sweep.

## O55 — A fit should CENTRE, and a canvas resize should keep the fit until the operator leaves it

> *"I want the buttons that zoom to page, width, height etc when pressed to
> center the page, or width, or height on the canvas window. if the canvas
> window is resized the pdf should resize to match unless the person has
> changed the zoom or panned around."*

**Status:** **DONE, driven and falsified.**
`a_fit_command_puts_the_page_on_screen` gained a resize phase;
`a_pan_leaves_the_fit` is new. With the pan fix removed the latter reports
`margins l=8.0 r=8.0 t=108.4 b=108.3` — dead centre — against
`l=-39.0 r=-105.0 t=120.4 b=-27.3` for the correct build.

## O54 — The highlight tool should follow text the way Acrobat's does, and paragraph reflow should be offered

> *"Also the highlight tool - it's great that we can just drag a box to highlight
> an area, but we should be able to drag it along to just highlight text too like
> it works in adobe. Also I think the paragraph reflow was implemented ages ago
> in the pdfcer core, so we should have that option too."*

**Status:** **(b) DONE**, not driven — he was at the machine. The
`ui-verify` check and its fixture are written and unrun:
`reflowing_a_paragraph_rewraps_it` against `fixtures/paragraph.pdf`, whose six
short ragged lines pack to five, so *"it ran and changed nothing"* fails rather
than passing.

**Status:** **BOTH HALVES DONE.** Neither is driven.

## O53 — "always always always": the canvas is the primary surface, and a checkbox proved it is not yet

> *"I'm noticing that when I make a checkbox, I can't select it on the canvas to
> move or resize. Note that if the engine is capable, I should be able to select
> the object and do all of the ordinary editing one would expect a GUI editor to
> be able to do. It is great that there is a properties box that allows this, but
> **always always always** I need objects on the canvas to be clickable and
> editable as one would expect given our research of other programs."*

**When a driven check needs a step the operator would never know to take, that
step is a bug report.** It was recorded as scenery instead. The lesson is filed.

**Status:** **ACCEPTED.** The ruling is recorded as a standing rule
in agent memory as well as here, because it governs every future feature rather
than this one.

Checks: `dragging_a_form_field_moves_it`.

## O52 — Colour default becomes *Match other PDF viewers*, and the old formula goes entirely

> *"under the colour setting we are going to change our default to Match other
> PDF viewers. you can also remove the The old pdfcer formula from that section,
> even the code for it."*

**He has now looked at it again and changed his mind.** That is not a defect
report and must not be filed as one. What it means practically is that a doc
comment saying *"by operator ruling"* becomes a lie the moment the default
flips, and **the ruling it cites is the one being reversed** — so the change
carries a documentation obligation the code change alone does not discharge.

| piece | whose | state |
|---|---|---|
| the `#[default]` on `CmykIntent` | **`pdfcer-core`** | filed with the engine |
| deleting the `Naive` variant and its colour maths | **`pdfcer-core`** | filed with the engine |
| the third radio button and its copy | this shell | **done** |
| the divergence note | this shell | **done** — deleted, not reworded |
| what a fresh install gets today | this shell | **done** — seeded, see below |

`D:\Dev\pdfcer\` is read-only to this project until fold-in, so the first two
are a hand-off rather than a change I make. The request is
`request_cmyk_default_flips_and_the_naive_formula_goes.md`.

**Status:** **BOTH HALVES DONE.** The engine landed
`Pass 153.0` the same afternoon — `Calibrated` is its default and `Naive` is
deleted — so the shell-side seed is **gone**, exactly as its own tripwire
instructed. It fired on the first build after the `cargo update`, which is the
whole reason it was written as a `debug_assert_ne!` rather than a comment.

## O51 — Inkscape-style scale toggles: line weight follows a resize, if you say so

> *"if that was the resize question about scaling line weight, etc with resize
> it got the answer wrong. default should be what it said, but there should be
> an option that they do scale with resize. Inkscape has options for this and I
> want the same."*

*"Blocked on `resize_annotation`, which the engine is building to this shape."*
It had **shipped** — `Pass 151.0`, with `ResizeOptions` carrying all three
fields — and this shell was already calling it. The row was a record of a
conversation rather than of the code, which is this project's fourth-commonest
defect and the reason the standing rule is to verify an absence claim against
source before writing it down.

**Status:** **DONE.** Not driven — he is at the machine. The
check is written and unrun: `the_line_weight_switch_reaches_the_resize`, which
reads `stroke=` off the applied line because a screenshot cannot tell a border
that thickened because `/BS /W` changed from one that thickened because the
placement matrix scaled it.

## O50 — A permanent font-folder setting, with a checkbox for the OS's own fonts

**Status:** **SHIPPED AND DRIVEN.** The checkbox is in
Settings ▸ Fonts, off by default, and lists the two folders it resolves to
underneath it — including the per-user one at
`…\AppData\Local\Microsoft\Windows\Fonts`, which is where a plain
double-click installs a font on this machine.

Checks: `settings_headings_legible`.

## O49 — Zoom loses its place past about 300,000%, and I am telling you rather than quietly widening the test

**Status:** **FIXED AND DRIVEN.** Both checks are GREEN, and
the drift now holds at **33 % of tolerance all the way to 105 billion percent**
— it used to breach at 292,415 %.

## O48 — Removing embedded fonts does not make the file smaller, and I think that is the wrong answer

**Status:** **SHIPPED AND DRIVEN.** File ▸ Save ▸ *Save a
compacted copy…*, always to a new file, with the three losses stated before the
picker opens.

## O47 — ✅ **ANSWERED PROPERLY** — should pdfcer embed its OWN fonts?

### ✅ Status: it is **option 3** — a checkbox, off by default, disclosed

> *"OFF by default, and not for a technical reason: the bundled faces are
> BSD-3-Clause … and embedding one puts it inside a document you then
> distribute — which carries that licence's attribution condition with it.
> **That is your decision to make, so pdfcer does not make it for you.**"*
> — `D:\Dev\pdfcer\crates\pdfcer-cli\src\main.rs:1598`

**No window was rendered.** You were at the machine, so `ui-verify` was not run.
`embedding_works_with_no_font_folder_at_all` was rewritten to drive **both**
positions — the offer made and declined as the window opens, then the box
ticked and the commit reaching the engine with `substituted=true` — and **has
not been run.** A check asserting only that the box is off would pass on a build
that ignored the box entirely, which is why both are driven; that both fire is
still unproven.

## O46 — Editing should work like every other graphics program. It does not.

> *"The interface for this has gotten so wonky. I can't figure out how to click
> on objects to edit them. For example on the [conformance suite's composite
> page] There are obviously more
> than one item on the page, but when I click on one of the objects all I get is
> the page selected. When I double click on an object it doesn't select — it
> still only has the whole page selected. Also when I have an object selected
> like text the Tool tab doesn't switch to giving me the editable stuff for that
> object. And if I add an image I Expect to click on it to resize but dragging
> doesn't resize. Editing should work like 99% of the graphics programs out
> there. It should be intuitive. I have no idea why I can't just click an
> object, see its properties and edit them in the tool tab, move them, adjust
> them, etc, just like any other software. Somehow we have the most convoluted
> system, or it just isn't working for most of the things I expect it to.*
>
> *Also how do I OCR more than one page? Why does the tool stop at one? Why do I
> have to save a copy instead of just go back into my pdf and save over it or
> save from there? Where is the option to select more than one page? How did we
> end up with the most useless and un-userfriendly of options for the OCR? What
> program in the world works this way out of all the OCR programs out there?"*

> *"BEFORE you make any changes, you are going to go and research how GUIs for
> graphics editing programs handle editing and navigating and then you are going
> to write a manual on how things should work in pdfcergui, and in parallel while
> you do that you are going to write up the manual for how to currently use
> pdfcergui, then you are going to compare the two documents and see the
> difference and determine just what you need to change."*

| # | Complaint | Status |
|---|---|---|
| 1 | Clicking an object selects the whole page instead | ✅ **CONFIRMED BY YOU** — *"I checked and clicking works."* Driven by the harness the same day, then used by you on your own file |
| 2 | Double-clicking an object still selects the whole page | **DONE.** A click lands on the real object, and a double-click descends into a form-interior object's subpaths and nodes as well — `canvas/selection/mod.rs` carries the note *“A LEAF DESCENDS TOO”*. The engine has six form-scoped edit verbs and the shell calls all six |
| 3 | Selecting text does not change the Tool tab to that object's editable properties | **done** — the Properties panel reads the canvas selection, and the status bar names what is selected |
| 4 | An inserted image cannot be resized by dragging | **done** — a press on an unselected object selects it and the same drag moves it; placement now arrives selected, so its grips are already up |
| 5 | OCR does one page only — no page range, no multi-page selection | **done** — All pages / this page / a typed range, All being the default |
| 6 | OCR forces Save-a-copy instead of saving into the open document | **done** — recognition is an ordinary edit; `Ctrl+S` saves it, `Ctrl+Z` takes it out |
| 7 | The whole editing model is unintuitive next to other graphics software | in progress — the three research documents are the deliverable, and 2 is the remainder of the click work |

The audit's verdict was that the engine did not enter form XObjects, so a
page-sized form was a page-sized hit target that won every click at every point.
That was filed as an engine request and **`pdfcer-core` answered it
the next day** — Passes 136.0, 136.1 and 136.2:

| | |
|---|---|
| **You cannot edit an object inside a form** | not a shell decision: `pdfcer-core` writes a paint-order edit to the *page's* content stream, and a form-interior object lives in the form's. `FormLeaf::is_editable()` is `false` for every one of them today. Select the form and move that, or wait for the engine |
| **Double-click will not descend into one** | the Part and Node rungs exist to act on geometry, and there is no geometry to act on here. It stops at the whole object rather than descending into something you then cannot change |
| **The measure tools cannot pick a line inside a form** | the engine's line-pick does not see the leaf list. Filed. On the benchmark CAD sheet that is 10,256 lines the tool cannot see, and it was equally true before today — it was just hidden behind the selection defect |
| **`pdfcer object-list --hit` still answers with the form** | the CLI has not consumed the deep hit test, and its help says it is authoritative for the GUI's behaviour, which is now false. Filed |

**DRIVEN, AND THEN CONFIRMED BY THE OPERATOR.**

*"All I get is the page selected"* was a precise and accurate report: he was
selecting a page-sized object. It just was not the one I guessed.

Cited: `edit.rs:14728`, `app/actions/vector.rs:773`, `vector/decompose.rs:1373`, `canvas/clicking.rs:805`, `canvas/smart.rs:146`, `vector/linepick.rs:475`, `canvas/measure/mod.rs:833`, `pdfcer-cli/src/main.rs:9277`, `canvas/target.rs:571`.

Checks: `a_click_inside_a_form_selects_what_is_drawn_there`.

**NOT YET DRIVEN.**

## O45 — Selecting a standard leaves Save greyed out

**Fixed the same day. Unit-tested and falsified; NOT yet
driven** — see the status at the end, which says so in those words.

> *"When I go to settings and select some of the standards the save button is
> greyed out and I can't save the change."*

### Status

## O44 — Four things the first COMPLETE driven run found. Two were real, two were the test.

**Found:**, by the first `ui-verify` run in this project's history in
which every declared check actually launched. **All four resolved the same day**
— and the honest tally is that **two were defects in the program and two were
defects in the tests**, which is worth stating plainly rather than counting four
fixes.

Every claim below was driven, and every driven claim was falsified first.

Filed as *"typing a width and pressing Apply does nothing"* — and the real
defect is that **Apply could not be seen**.

## O43 — Vertical text should behave like vertical text

**Shipped the same day. Driven check written and NOT YET
RUN** — see the status below, which says so in those words.

> *"I have text placed vertically on the bottom left corner of the SW41177.pdf.
> In Adobe when I hover over it the I cursor re-orients itself to match the text
> orientation, and when I select the text it shades each letter as part of the
> same block. when I copy and paste into notepad, I get the text on one line as
> expected. I need pdfcergui to have the same behaviour. as it is now the I
> cursor doesn't reorient and it pastes each letter onto its own line."*
>
> *"The last page has the vertical text."*

The engine request is filed —
`request_extraction_drops_the_writing_direction.md` — asking for
direction-aware segmentation and for the direction to be published. When that
lands, the shell-side recovery becomes a fallback and then deletes.

### Status, stated honestly

| | |
|---|---|
| your own file | **verified.** The stamp on page 36 of `SW41177.pdf` comes back as one line: `W:\Engineering\Products\SAM\SW41177 Toyota Pick up ROPS\SW41177-WELDED FOPS.SLDDRW`. Run it yourself: `cargo test -p pdfcer-gui --lib the_operators_own_vertical_stamp -- --ignored --nocapture` |
| unit tests | 22 new, against real extractions of a real fixture at 0°, 90°, 180°, 270° and 30°. Falsified in both directions before being quoted |
| **the driven check** | **WRITTEN, NOT RUN.** `ui-verify rotated_text_selects_and_copies_as_one_line` drives the release binary, sweeps the string, asserts `chars=6 quads=1`, asserts the cursor traced `deg=90`, and reads the OS clipboard from outside the process. It needs the pointer and the foreground, and you were using the PC. **Say when and it runs.** |
| the 30° case | the band it *marks* is a true parallelogram; the wash it *paints* is that band's bounding box, so it over-covers at the corners. Named rather than discovered. Quadrant rotations — every one a CAD exporter emits — are exact |

## O42 — Let me set the colour-blending buffer size myself

**Measured and filed with the engine the same day; needs
one change there before the setting can exist.**

> *"can the size of the buffer be increased? Allow the user to set the size up
> to the maximum possible?"*

**Shipped.** Settings ▸ Colour ▸ **"Colours changing when you
zoom"**. Type `default`, or a size — `512mib`, `1.5gb`, or a plain number of
bytes. It is uncapped, with no guard and no preflight, exactly the treatment you
chose for the zoom limit; the window states the cost and does not prevent the
choice.

**Driven:** `ui-verify blend_space`, and the funnel that carries the number to
the renderer has its own test — falsified by breaking the one line that carries
it, because a settings field that saves a number and changes nothing would be
worse than not offering it.

## O41 — Colours change with the zoom level

**Cause found and disclosed the same day; the real fix is
filed with the engine.**

> *"seems I get different results depending on Zoom level. The [shading] boxes
> for example on zoom out the colors between our rendering and the references
> don't match, but they do when I am zoomed in. up to 474% they are mismatched,
> but at 579% they match. There's little problems like this in the rendering in
> others too, so probably all of them are related to one bug hopefully."*

**Driven, on the file that shows it.** At 801 % zoom — well past the 534 % where
this same page used to lose its ink — the trace now reads
`cmyk_buffer=true refused=0`. Before the change it read `refused=1` and the
status bar apologised. `ui-verify blend_space` asserts it, and its assertion was
falsified by disabling the mechanism and watching it go red.

**Verified:** driven — `ui-verify blend_space` zooms past the crossing and
checks the line appears, and that it is absent below it.

## O40 — Only one standard was selectable in Settings

**Shipped.**

> *"in the settings for the standards compatibility I can only select
> (ISO15930-1, -4). I want to be able to select all of them and especially
> PDF/X-4 (ISO 15930-7)."*

**You were describing it exactly.** The control worked out which preset was
selected by comparing your settings against each one — and all eight of the
PDF/X and PDF/A presets set *identical* rendering answers. So whichever you
clicked, it matched PDF/X-1a first and the dot jumped back there. All nine are
selectable now.

## O39 — All the form buttons working, and clicking a field shows its properties

**Shipped.**

> *"can you get all the form buttons on the ribbon working next along with
> adding all the form feature buttons. when I click one I should be able to
> click on the canvas to place the position or drag a box for size then a pop up
> lets me set the details for the feature."*
>
> *"remember last settings and leave push buttons on the ribbon but greyed out
> for now. also don't forget that when I click on an existing form field on the
> page it's properties should come up in our side pane for editing it's
> properties."*

**You can change a placed field's properties now.** Click a form field on the
page and the Properties pane offers **Required**, **Read only**, a **tooltip**,
and — for a text field — **multiple lines**, **hide as typed**, **equal cells**
and a **maximum length**. A drop-down gets its own two. Each is one press and
one Ctrl+Z.

**Verified:** driven. `ui-verify form_field` launches the real program, arms the
tool, clicks the page, watches the field get created, then clicks an existing
field and checks the Properties pane actually drew.

## O38 — A rendering preset for PDF/X-4 (ISO 15930-7) conformance, and a standards selector

> *"I'd like a preset setting for rendering things to what the [print
> conformance suite] page needs to render correctly. We can't call it [that],
> but since it is for conformance to PDF/X-4 (ISO 15930-7)... I noticed touching
> some of our presets caused some test to show up as failed... maybe we should
> have a dropdown to select view options between the different standards."*

**Your black-generation question turned out to be the wrong question, in a
useful way.** I filed it as *contentious* — your ruling versus a
conformance render. The engine's answer: **no setting of it is conformant**,
because every PDF/X level guarantees a measured definition of ink and this
control picks among fixed built-in tables. So the two were never in tension.
It is one control standing in for something pdfcer cannot do yet, and the preset
says so on screen rather than leaving a colour conversion that silently did not
happen.

## M1 — The PC starts pdfcer unreliably. The laptop does not. It is the PC.

**SETTLED by your laptop test, and the conclusion is the useful
part: pdfcer is exonerated.** The same portable build, the same files, works
normally on the laptop and fails roughly one launch in three on the PC. That is
a machine difference, not a program defect, and no more of my time goes on it.

**What this costs, and it is worth knowing rather than rediscovering:** the
automated test suite launches a fresh copy of pdfcer for every check, so on the
PC about a third of them cannot start. Those show up as skips that look like
failures. Any future session driving the suite **on this PC** should expect that
and not go hunting.

## E3 — OCR put every word in the wrong place on rotated pages

**Not asked — found by the engine and fixed.**.

Scanned pages are usually rotated by the *scanner driver* writing a rotation
flag rather than by turning the pixels. pdfcer honoured that flag when drawing
the page and **not** when placing the recognised words — so on any quarter-turn
page, every word ended up on the wrong axis at the wrong scale.

## E2 — "Redact every match" could report success and leave the text in the file

**Not asked — found by the engine and fixed.**.

The sibling of E1 below, and the dangerous one. Some PDFs store text with no
record of which letters it is — it renders and prints perfectly, and nothing can
search it. Ask pdfcer to redact every occurrence of a name in such a file and it
would mark nothing, report success, and leave the name in the document. Then you
send it.

## E1 — Find said "no matches" over text it could never have searched

**Not asked — found by the engine and acted on.**.

**The defect you would never have reported as a bug**, because it does not look
like one. A search can return "No matches" for a word that is plainly on the
page. Two situations produce that identical answer: the word really is not
there, or **the document stores its text in a way that records no letters** —
so nothing could ever have matched. The text renders perfectly. It prints. It
simply cannot be searched, and Find used to answer that with a confident "No
matches".

## O37 — All the font tools Word has

**SHIPPED AND DRIVEN** — awaiting your verdict.
`RIBBON_SCALING.md` §6c.

> *"We should also have all the font tools available that Word does."*

- ✅ **Step 2 done** — the engine hand-off is filed as
 `request_restyle_an_existing_text_run.md`, deliberately as ONE request. It
 asks the two questions only the engine can answer: whether a restyle is
 representable for an arbitrary run at all (swapping Helvetica for
 Helvetica-Bold changes every advance width, so the run reflows or overruns),
 and whether the honest scope is narrower — restyling only text pdfcer itself
 authored, where the metrics are already known. If it is narrower, we would
 rather disclose a narrow capability than ship a wide-looking one.
- **Step 3** — the IA amendment. pdfcer's text lives under Edit ▸ Content and the
 contextual **Format** tab; there is no Home tab, and the Format tab is the
 natural home for anything acting on a selection.

Checks: `restyling_selected_text_reaches_the_document`.

## O36 — Sections re-wrap onto more rows, and the scroll arrow is authorised

> *"put it in the plan to update so that tools within sections will re-wrap
> onto more rows when I resize, and do the scroll like Word. BTW the Font
> section in Word will wrap tools onto 3 lines when the window is narrowed
> enough, and other tools wrap in a similar way too."*

## O35 — Image quality worse than Acrobat on normal pages

**Shipped.**

> *"there was also an update to an image quality setting to discard smaller
> details than the screen sees a while ago that I think has been enabled by
> default because image quality is a little worse on normal pages than it was
> whereas before it was on par with acrobat reader — this setting should be an
> option in our settings and disabled by default."*

**You named the mechanism exactly.** Images drawn smaller than their own pixel
grid were point-sampled: one texel per output pixel, the rest discarded. That
is every scan and every CAD raster at anything under 1:1, and the engine's own
note on it says *"aliasing, shimmer, dropped hairlines"*.

**Engine hand-off filed** — `pdfcer-core` grades its own default "a guess" and
names the exact evidence that would flip it: a viewer-behaviour comparison.
You just supplied one, against Acrobat, on your own drawings.

## O34 — The print dialog grows for ever after printing

**Shipped.**

> *"the print dialogue has a bug that when I press print, instead of closing
> after printing it just keeps expanding its size in little steps to infinity."*

**Verified:** four unit tests including one that reproduces the overflow in a
real laid-out frame and fails on the old ordering. **NOT driven** — reproducing
it end-to-end would mean sending a job to your printer.

## O33 — Does the ribbon get the scroll arrow, and do groups re-wrap?

**Partly shipped**.
**One decision open — yours.**

Two questions, and they have different answers.

## O32 — The commands whose tab was decided by mode exposure, not by subject

> *"the current commands for each are fine as is for now. there were just some
> commands you made a decision to put in a different tab than where they would
> normally go because exposure was tab based and not command based."*

**Status:** **FOUND AND LISTED. The operator's decision, per command — nothing
moved.** The mechanism that forced them is gone; whether each *should* move is
a `RIBBON_IA.md` question and the IA is his.

> *"a command refused in a mode where the operator plainly needs it is evidence
> that the command's tab is wrong, not that the mode gate needs an exception."*
> — `RIBBON_IA.md` §5.7

> §5.1: *"**Moved off this tab:** `Copy this page's text` and `Copy the whole
> document's text` go to **Edit ▸ Clipboard**"*
> §5.4: `| | Copy page text · Copy document text | **G** *(from File)* |`
> §5.4: `| **Forms** | Fill form | **G** |`
> §7: `| Edit ▸ Forms ▸ Fill Form | Edit ▸ Forms |`

## O31 — Improve the ribbon: learn from Word

> *"can you improve the ribbon bar? if you can learn how word handles when to
> have text labels, organization on two rows for some commands, and how it
> handles narrowing the window. for one thing it puts an arrow at the end to
> press to move over if there isn't room for all commands. also we should have
> flexibility to show or hide and commands and shift the space used depending
> on what exists. this would allow greater flexibility of where to place
> commands for read, review and edit modes, as what remains shown can be mixed
> on tabs. if you can, drive word as it is installed on this machine."*

**Status:** **RESEARCHED AND STAGED. S1 and S2 done; S3 and S4
designed and not built.** The whole of it is `RIBBON_SCALING.md`.

## O28 — A fit control must place the view, not only set the scale

> *"If I press the Fit width or fit page button the view should center to the
> width as well or center the page."*

**Status:** **FIXED**, and driven —
`a_fit_command_puts_the_page_on_screen` pans thirty notches into the pasteboard
before pressing each button, asserts it got there, and then measures the page's
drawn rect against the canvas's. Measured after Fit page: page
`296,272.. 764,633` in a canvas of `288,143.. 772,762` — margins of 8 and 8
horizontally, 128.5 and 128.6 vertically. **Falsified**: with the placement
disabled the same run reports *"part of the page is outside the canvas; the
vertical margins are 261.5 and −4.4, so the page is not centred"*.

## O29 — Fit height, because Acrobat has it

**Status:** **FIXED**, and driven in the same check. Measured after
Fit height on the 1584 × 1224 sheet: page `288,151.. 1068,754` in a canvas of
`288,143.. 772,762` — the full height on screen, and the width overflowing by
296 points, which is the mode doing exactly what it is for.

## O30 — In single-page view, choose what the wheel does

> *"when in single page view there should be an option on screen near the
> button to scroll or flip through pages, or the current way it is now when the
> scroll wheel is used."*

**Status:** **FIXED**, and driven —
`the_wheel_turns_pages_when_the_operator_asks_it_to` makes five separate
claims, in order: the default is **silent** (a build that flipped
unconditionally could not pass), the toggle is on screen beside the page
buttons, the **very next** notch turns a page, rolling back returns to the page
before, and under a continuous display the control is **not drawn at all**.

## O26 — Zoom out throws the page off screen into a corner

> *"the zoom in function works flawlessly now. The panning works. Zoom out has
> a small bug where it sometimes seems to reposition the page so that it is off
> screen in the far bottom left corner. This happened when I zoomed back from
> around 2 million% but seems to happen at other junctions too."*

**Status:** **SEVEN CAUSES FOUND AND FIXED**, in two clusters:
O26a-d below, which relocate the page at ordinary zooms and were never about
zooming out in particular, and O26e-g, the missing hand-over out of the `f64`
position tier. Every one of them moves the page by a whole page or more.
Driven, with pixels for the first. A residual is filed separately as O27.

## O26e / O26f / O26g — the hand-over back out of the `f64` tier

**Status:** **FIXED**, and driven. The operator's *"from around
2 million %"* is the same number as O24f's, and it is not a number he picked
either time: `SUB_PIXEL_CONTENT_EXTENT / page_height` is where the position
hands over between the `f32` scroll offset and the `f64` `DeepAnchor`.

## O27 — The `f32` scroll tier jitters above about 100,000 %

**Found:**, while driving O26. **Not reported by the operator.**

With all four O26a-d causes and all three O26e-g pieces fixed, an anchored zoom
notch still moves the view by **10–35 screen pixels** on the `scroll` tier
above roughly 130,000 %. On the `deep` tier the same measurement is **±0.05
px** across four readings — exact.

## O25 — Panning far, or zooming out, leaves the new area blank

**Status:** **FIXED.** Driven, and the check fails on a build with
the defect restored.

Checks: `panning_at_deep_zoom_stays_where_it_was_put`, `panning_past_the_overscan_renders_the_new_area`, `the_page_still_renders_at_every_decade_of_zoom`.

## O24i / O24j — Screenshots at maximum zoom, and the two defects they found

**Status:** **CONFIRMED, and it was not confirming before he asked.**

Checks: `the_page_still_renders_at_every_decade_of_zoom`, `zooming_does_not_throw_away_where_the_operator_panned`.

## O24h — "Can you test up to maximum zoom please?"

**Status:** **DONE. Nothing had to be called good enough.**

`ui-verify` on `banana.pdf`: **36 verified, 0 failed, 36 skipped** — the skips
are checks needing a `--doc-point` or a fixture this sheet cannot provide.

Checks: `measure_hover_shows_what_it_will_take`.

## O24e / O24f / O24g — Zoom throws the view away, twice, and `−` undoes a hundredfold

> *"there is a little bug where if I am zoomed out to about page size, pan the
> cells to the center of the screen, then start to zoom, the page snaps back to
> near the center position. … I do lose the view at 2000000% magnification.
> Also clicking the negative button to zoom back snaps me back to 800% when I
> am over 800%."*

**Status:** **ALL THREE FIXED.** Driven, and the driven check fails
on a build with the defects present.

`ui-verify --check zooming_does_not_throw_away_where_the_operator_panned`.
Pans off-centre, then Ctrl+wheels **one notch at a time** with the pointer on
the viewport centre, following the page point under that centre.

## O24c / O24d — The page lurches backwards mid-pan, and bounces when zoomed

> *"As I drag using the middle mouse button the pan will follow and work, but
> if I pan a little too far it jumps back in the opposite direction I was
> moving the mouse towards … It isn't exactly in the same place as it started.
> When I zoom in the image does seem to disappear from the screen sometimes …
> if I pan the other direction and cross the same area where I experienced the
> jump the pan location jumps back to being correct."*

> *"Up to 800% things work perfect. Over that … it seems to refresh the image
> zoom, then reposition to the cursor location, which causes the image to
> bounce around a bit before settling under the cursor."*

**Status:** **FIXED.** Unit-tested. Driven confirmation still
owed — the operator was at the machine and `ui-verify` cannot take the
foreground from him.

`ui-verify` now refuses to report PASS on a run in which no reading described
a region raster (`REGION_TIER_REQUIRED`). A check that cannot fail is not
evidence, and this one was being quoted as evidence.

`canvas-pos` gained `paint=`, `region=` and `ext=`. `ui-verify` recomputes
`region_on_screen` from those **independently** and compares — so a future
change back to the wanted region is caught rather than merely absent from the
tests. `RenderKey::region`'s round-trip is pinned bit-exact.

## O24b — "Can the huge intermediate be fixed? Is that why panning jumped back?"

**Status:** **ANSWERED AND MEASURED.**

`ui-verify --check panning_at_deep_zoom_stays_where_it_was_put`, on
`banana.pdf`, rolling the wheel three notches and reading the position again
ninety frames later.

## O24 — A setting for the maximum zoom

**Status:** **RECORDED. NOT STARTED.** The engine claim is
**verified** — see below — and it changes what this row is. The setting is the
small half.

> *"when you complete the step 2 zoom release to git and put on OneDrive."*

> *"can you build the first step and build the second one and put it as an
> option to use instead for higher zoom capability? that way I can test out
> both in case there are performance issues introduced at lower zoom. I don't
> want to lose our capability to pan around a page and still see high detail
> as we pan. I don't want the affect that other readers have where you always
> have to wait for detail to render after panning to a new area."*

> *"how do we get to the insanely high limit? … I've seen readers hit over
> 4000%, and none are limited to a mere 1000%. You should be able to have a
> new algorithm take over for bigger zooms?"*

Checks: `multi_node_move_moves_every_picked_anchor`.

## O23 — Free navigation: any part of the page to anywhere on screen, and objects off the page still reachable

**Status:** **BOTH HALVES BUILT AND DRIVEN — A, B.**

A is a scroll-extent change. B is about what the canvas draws and hit-tests at
all. They are filed together because he asked for them together and because A is
a precondition for B — there is no point being able to select something you
cannot scroll to — but they will not be one change.

Cited: `mapping.rs:189`, `canvas/mod.rs:662`, `canvas/mod.rs:688`, `canvas/interact.rs:372-375`, `interact.rs:352`, `interact.rs:1310`.

Checks: `a_band_dragged_into_the_margin_reaches_an_object_off_the_page`, `a_band_that_starts_in_the_margin_reaches_an_object_off_the_page`, `a_pan_keeps_the_fit_and_the_resize_keeps_the_position`, `an_object_off_the_page_is_actually_drawn`, `an_object_off_the_page_survives_being_zoomed_in_on`, `resize_scales_a_shape`, `rotate_handle_turns_a_selection`, `scrolling_far_keeps_the_canvas_its_pointer_input`.

## O22 — An object near the top of the view cannot be rotated: its handle is off-canvas

**Status:** **CONFIRMED BY DRIVING, WITH NUMBERS. NOT FIXED.** The fix is a
convention question and is not being improvised.

> *"Above the top edge, centred, by the stem's length. **The one grip whose
> centre is OUTSIDE the box**, which is what the offset is for."*

Cited: `canvas/handles.rs:335`, `canvas/handles.rs:100`, `handles.rs:271`, `handles.rs:260-267`, `handles.rs:269`.

Checks: `resize_scales_a_shape`, `rotate_handle_turns_a_selection`.

## O21 — Move, resize and rotate ANY object; click nodes, select several, move them — all with live preview

**Status:** **ENGINE CONFIRMED against `D:\Dev\pdfcer` source.**
You were right, with two boundaries worth knowing. Asking for it to be
confirmed rather than assumed was the correct instinct and it paid: the
confirmation also caught **a claim in this very file that was false**, and I
had re-published it an hour earlier — see `O20`.

> *"A handle is currently offered for an object that can never be transformed,
> and the operator finds out by dragging it. That is a real gap."*

> *"I've never seen a program that doesn't live preview any change, and yet here
> I am having to ask for all the minute details as if you'd never been trained
> on it."*

Cited: `crates/pdfcer-core/src/edit.rs:7512`, `vector/edit.rs:996`, `edit.rs:8486`, `edit.rs:8542`, `canvas/moving.rs:560`, `app/actions/vector.rs:469`, `canvas/resizing.rs:172`.

## O20 — Dragging and rotating TEXT on the canvas

**Status:** **RECORDED. NOT STARTED.** Two separate things behind one
sentence, and they are in very different states, which is why they are written
out rather than merged.

Cited: `overlay.rs:220`.

**NOT YET DRIVEN.**

## O19 — In single-page mode, an option to turn the page when you scroll past its end

**Status:** **RECORDED, NOT STARTED.**

## O18 — Ctrl+C on selected TEXT puts "1 object copied from pdfcer" on the clipboard

**Status:** **CONFIRMED BY THE OPERATOR** — *"copy paste now
works!"* Fixed in all three places.

> *"A canvas draft claims these chords too … Ctrl+C mid-word must not copy the
> page's text selection: the operator is composing, and the selection they made
> before the caret landed is not what those two keys mean any more."*

Checks: `ctrl_c_copies_text_to_the_os_clipboard`.

## O17 — Selection is governed by a FILTER on the status bar, not by two menus at the top

**Status:** **PARTS A AND C BUILT. THE POPUP SHIPPED BROKEN THE
FIRST TIME AND IS FIXED.** Parts B and D not started.

**Open question, and it is a convention question rather than a preference
one:** whether entering edit-on-an-object in Edit mode is the single click that
selected it, or a second click / double-click. He named the alternative himself
— *"or double clicking if that is the more common convention"* — which is the
right instinct and the right person to be asked. **The class answer is
double-click**: PowerPoint, Illustrator, Figma, Visio and Acrobat all use
single-click-selects, double-click-enters. Single-click-enters exists mainly in
programs with no selection concept at all. Proposed, for his ruling: **click
selects, double-click enters the object's editor**, with Enter as the keyboard
equivalent on a selection.

Checks: `select_filter_changes_what_a_click_hits`.

## O15 — Text editing should be MULTI-LINE

**Ask:** *"I should be able to make it multi line."*
**Status:** **SHIPPED AND DRIVEN.** Awaiting your verdict.

## O16 — Reassemble lines into paragraphs, and move between blocks with the arrow keys

**Ask:** *"there was an acrobat feature in the original pdfcer-gui
that attempted to reassemble individual lines into paragraphs and the cursor
would move to the next block of text using the navigation keys."*
**Status:** **SHIPPED AND DRIVEN.** Awaiting your verdict.

## O14 — The conventions sweep: fourteen gaps, found by asking

**Ask:** by you, as *"how can you learn from these other programs so that you
can build the missing parts more effectively?"*
**Status:** the mechanism is built. These are what it found on its first run.

 **Driven:** eight of the thirteen, one launch each, no mouse touched.
 Every one opened at its declared size and none needed to grow.

 **NOT VERIFIED**, named rather than implied: three of the five reachable
 only by a gesture — Insert pages, Set scale and the unsaved-changes
 question. The note editor is now driven, and see the row below for what
 that found.

Checks: `measure_perimeter_traces_and_closes`.

## O8 — **A Save button.** Not Save As. Save.

**Ask:** *"can I please have a save button like every other
program in existence has? We're on week two of this and just have a save as
button."*
**Status:** **SHIPPED and DRIVEN.** Awaiting your verdict.

Checks: `save_writes_over_the_file_you_opened`.

## O9 — A **length** tool: the perimeter tool that never closes

**Ask:** *"add a length tool that works like the perimeter tool
without needing to close the profile."*
**Status:** **SHIPPED and DRIVEN.** Awaiting your verdict.

## O10 — Neither measuring tool previews while you trace

**Ask:** *"both these tools need a preview just like the measure
tool has."*
**Status:** **FIXED**, awaiting your verdict.

## O11 — Move, resize and rotate a placed image on the canvas

**Ask:** *"there was no way to reposition, resize, or rotate it
on the screen. Can I please please please have that too?"*
**Status:** **MOVE AND RESIZE SHIPPED AND DRIVEN. ROTATE IS NOT — see
below.** Awaiting your verdict.

**Driven, on your own drawing:**

**The rotate grip exists.** `canvas/rotating.rs` holds it: `Grip::Rotate`
(`canvas/handles.rs:175`), its hit test (`handles.rs:412`, ahead of the eight
resize grips), its painter (`overlay.rs:222` via `draw_grips`), a rotate ghost
(`overlay.rs:612`), Shift-snapping to 15°, and the commit through
`transform_objects` (`canvas/rotating.rs:273`).

Checks: `geometry_fields_resize_a_shape`, `resize_scales_a_shape`, `shift_constrains_a_resize`.

## O12 — Move text after placing it

**Ask:** *"can I please please please have the capability to move
the text after?"*
**Status:** **SHIPPED.** Select the text and drag it. Same verb as
O11, exactly as asked for — a placed image and a placed text run are the same
shape in a content stream, so they got one verb rather than two.

**NOT YET DRIVEN** on a text object specifically — the driven checks aim at a
shape, because that is what the fixture's `--doc-point` names. The verb is the
same one three passing checks exercise.

## O13 — Insert image does not appear until you save and reopen

**Ask:** *"I tried a new document and inserted an image. Nothing
appeared on screen or in the tree, but after saving and reopening the image was
there."*
**Status:** **FIXED and DRIVEN.** Awaiting your verdict.

**O4 is still open** — that one is the engine corrupting `/Contents` when it is
already an indirect array, which is what your CAD sheets use, and it produces a
file pdfcer cannot reopen at all. Filed and unchanged.

## O1 — Editing text on the canvas, and editing text in a text box

**Ask:** *"Still no editing text on top of the canvas. Or editing text on a
text box."*

**Status:** **PARKED at your instruction** — not closed, not solved, parked:
*"put the text editing aside again."* The findings stand and the engine request
stays open. Do not pick it back up without him.

Root cause under investigation. It will be either an engine gap (a
request, filed) or a wrong call on my side (a fix). Either way this row records
the answer.

**Driven:** `text_edit_on_a_real_drawing` now asserts the commit named a form,
and the old "this is an absent capability" skip has been **inverted** — if a
build ever refuses with the old reason again, that check fails loudly.
**NOT YET RUN.**

**(b) — not yet driven.** Nothing claimed.

## O2 — Cut / copy / paste of PAGE CONTENT (`Ctrl+X` / `Ctrl+C` / `Ctrl+V`)

Restated: *"can you get
cut copy and paste working for objects I select on the canvas?"*
**Scope set by you:** *"oh I might want all cases so we shouldn't be
restrictive in our ask."*
**Status:** **SHIPPED AND DRIVEN.** Awaiting your verdict.

- **Across two pdfcer windows.** Within one window it is lossless. Between two
 processes it needs the clip registered under a private Windows clipboard
 format, which is a call this shell does not make yet.
- **Copying to another program** — Illustrator, SolidWorks — needs the selection
 rendered as a standalone one-page PDF, which the engine has filed separately
 and deliberately did *not* fold into the same bytes: a one-page PDF cannot
 carry which byte range was which object, so re-deriving it on the way back in
 would make a pdfcer→pdfcer paste worse than a pdfcer→Illustrator one.
- **Dimensions and form fields** are annotations rather than page content, so
 these verbs cannot reach them at all. Filed.

I was going to ask for `duplicate_objects` alone, on the argument that Ctrl+V in
one document decomposes into *duplicate + offset* and `move_objects` already
exists. That is true and it would have covered same-document duplication only —
not pasting into the other tab, not the system clipboard, not dimensions or form
fields. You stopped that, and the filed request is the whole capability.

## O3 — Perimeter measuring tool

**Ask:** click around to make a shape, sum the segment lengths
into one dimension; right-click to add segments; drag the endpoints to adjust
the shape; all the scaling options of the other dimension tools.
**Status:** OPEN — **and no longer blocked. O3 is the single deliverable for
the next build**, on your instruction: *"In the next version I just need the
perimeter measuring tool to work with the group scale stuff the same as the
other dimensioning tools have."* Everything left is shell work. Filed; the engine
shipped the whole thing the same day: a
`Perimeter` kind carrying its vertices and an open/closed flag, verbs to move,
insert and remove a vertex, and a preflight so a right-click menu can be greyed
correctly rather than by guessing.

Checks: `measure_perimeter_traces_and_closes`.

## O4 — Insert image does nothing

**Ask:**, restated — *"No it always hasn't worked."*
**Status:** **BOTH CAUSES FIXED.** Awaiting your verdict.

**The engine's.** `add_image` corrupted the page's `/Contents` whenever it was
an indirect reference to an array — which is what every CAD-exported sheet uses.
The verb returned success, the status bar reported the resolution, the picture
was not on the page, and **the saved file could not be reopened by pdfcer at
all**. Filed with an eight-line repro; fixed in `Pass 111.0`. Files already
damaged by an older build now open, render, and say so through a counted
disclosure rather than being silently patched.

## O7 — Selecting text inside a draft

Recorded because it is the obvious next thing
after the caret and I would rather name the gap than leave it implied.

Shift+arrow, Ctrl+A, and dragging across a draft to select part of it, so that
typing replaces the selection. Not started.

## Q1 — ANSWERED: the rest of the line MOVES ALONG

> *"It should move along."*

## O5 — Horizontal / vertical dimension constraint, from a drop-down

**Status:** OPEN, not started. `LinearPick::constraint` exists in the shell and
is never written outside tests — there is no control for it. No engine work
needed.

## O6 — The scale ratio field follows the dimension you set

**Ask:** *"when I set the dimension the editable ratio one shown
should change to match it."*
**Status:** OPEN, not started.

# SHIPPED — awaiting your verdict

Rows here are built, gated and driven. **They stay here until you have used
them and said so**, then they move to CLOSED with the date you confirmed.

## S1 — Move a placed dimension, with a live preview

**Shipped.**
Press inside a selected ce dimension and drag: the dimension line follows,
previewed from the same function a committed dimension is drawn from, and the
release commits `place_dimension`. The measured points never move, so the
printed number cannot change. Linear dimensions only — angular refuses at the
press rather than starting a drag that could not finish.
**Verified:** unit tests only (4). **NOT yet driven through the harness.**

## S2 — The measure sidebar no longer hides half its controls

**Shipped.**
The group list was a four-column grid 209 pt wider than its own column, clipped
with no scrollbar in that axis. Now a block per group, and the panel measures
its own overflow so it cannot come back quietly.
**Verified:** `no_row_in_this_panel_outruns_a_narrow_dock`, which failed at
209 pt before the change. **NOT yet looked at on screen.**

## S3 — `Ctrl+P` opens Print

**Shipped.**
It had never been bound. Print was on the ribbon, in the QAT and in a menu, so
every surface that lists commands showed it and only the keyboard did not.
`the_keymap_offers_the_chords_a_document_application_must` now asserts the whole
list of universal chords rather than this one line.
**Verified:** unit test. **NOT yet driven.**

## S4 — Imperial sheet sizes

**Shipped.**

## S5 — Multiple documents, page drag between them, tab reorder

**Shipped.**
**Verified:** driven — `document_tabs`, `page_drag_between_documents`,
`tab_reorder`.

# CLOSED

## O137 — ✅ **DONE — "the button never worked but I do want that display option!"** — and it now tells you when it has nothing to do

**Ken:**

> *"awhile ago you told me you removed the button to show all lines without
> their thickness — thin lines or something like cad has. The button never
> worked but I do want that display option!"*

## O138 — ✅ **THE BAND WAS DRIVEN, AND THE THREE-ROW CHANGE HAD REACHED NOTHING ON SCREEN**

**Not a new request — the measured close of the ribbon half of his report,** *"it looks like the edits to the ribbon got halfway
done"*, and of the sixteen item-level differences `RIBBON_IA.md` records.
Full ledger: `RIBBON_IA.md` and its two latest amendments.

The release binary, launched off screen at 1400 × 900 with
`PDFCER_DIAG_VIEWPORT` and read back through its own `ribbon.item.*` trace:
**six groups on the File tab, every one of them one row, two collapsed into
captioned buttons, and 126 pt of band unspent.** The row budget had gone from
two rows to three the day before; nothing consumed it, because
`plan::wrap_group` only wraps a group past 440 pt on one row and **no group
is that wide.** The amendment that raised the row count argued its table from
the mock's rectangles and this shell's unit tests, and said in its own closing
block that the product's rectangles had never been captured.

## O139 — ✅ **"YOU DRAW A CLOUD AND THE COMMENT LIST DOES NOT SHOW IT" WAS THE TEST, NOT THE PROGRAM — and the two witnesses were one witness copied**

> *"THE COMMENTS PANEL DOES NOT SEE THE ANNOTATION THAT WAS JUST AUTHORED: it
> listed 12 before the drag and 12 after it. The engine traced `add-markup`, so
> the annotation IS on the session."*

Checks: `a_comment_can_be_read_on_the_page_in_read_mode`, `a_note_can_be_written_onto_a_shape_that_exists`, `copying_a_sticky_note_carries_the_whole_comment`, `save_copy_round_trip`, `undo_redo_round_trip`.

## O143 — ✅ **DONE — "once you are done that please make a new release and also on GitHub"**

**The newest engine breaks our build**, and finding that out *before*
packaging rather than after is the rule working. `RESUME.md` names it as the
next action: `invocation_set` changed shape, one call site — and behind that
sits `Pass 257.0`, the engine's answer to the O141 measurement filed today.

You said you saw *"a lot of residual background tasks"*. Measured rather than
assumed: no `pdfcer-gui`, `ui-verify`, `cargo`, `rustc` or stray shell
processes were running on the machine. The entries are completed tasks that
stay listed. One genuinely stuck watchdog was killed earlier in the session.
