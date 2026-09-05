//! # `text::print` — every word the print dialog shows
//!
//! The catalog area for [`crate::dialogs::print`]. One module per surface is
//! the rule this directory's `mod.rs` states; the print dialog is a surface,
//! and it is a large one — three tabs, a preview, a device selector and a
//! commit button whose label is itself a disclosure.
//!
//! ## The copy in here is doing three different jobs
//!
//! Distinguishing them is what keeps the voice consistent, so they are named:
//!
//! 1. **Names.** A radio's label, a heading, a tab. Sentence case, no
//!    trailing period, and they name the *thing*, not the act — "Actual
//!    size", not "Print at actual size".
//! 2. **Disclosures.** Sentences pdfcer owes the operator because pdfcer
//!    inferred something, capped something, or is about to lose something:
//!    [`clip_summary`], [`dpi_capped`], [`raster_note`],
//!    [`commit_with_clipping`]. These are full sentences with punctuation,
//!    they name the number, and they never apologise. `docs/core-api/03`
//!    §6.3 enumerates exactly which values are inferences; every one of them
//!    has a function here.
//! 3. **Refusals.** Why a control is absent or a job cannot go
//!    ([`spooler_unavailable`], [`no_printers`], [`no_pages_selected`],
//!    [`range_unparsable`]). These say what is true and what the operator
//!    can do, and they are deliberately *different sentences* for different
//!    causes — see the next section, which is the single most important
//!    convention in this file.
//!
//! ## ★ Three ways to have no printer, said three ways
//!
//! This mirrors [`crate::text`]'s own three-way open-failure distinction, and
//! for the same reason: an operator must be able to tell from the words alone
//! which of these is true, because the three have completely different
//! remedies.
//!
//! | function | what is actually true | what the operator does |
//! |---|---|---|
//! | [`spooler_unavailable`] | pdfcer could not ask this system about printers **at all** | nothing, in this build |
//! | [`no_printers`] | the spooler answered, and reported none installed | install a printer |
//! | [`device_unavailable`] | this *particular* printer's driver would not describe itself | pick another printer |
//!
//! `pdfcer-print` is explicit that collapsing the first two is a defect:
//! non-Windows `list_printers` returns `Err(Unsupported)` rather than an
//! empty `Vec`, because *"reporting the same value for 'this platform cannot
//! enumerate printers at all' would collapse two different facts into one and
//! send a caller looking for hardware"* (`lib.rs:1859-1866`). The error type
//! carries that distinction across the port in
//! [`crate::dialogs::print::spooler`]; these three sentences are what it is
//! carried *for*.
//!
//! ## Why the commit button's label is a format string
//!
//! [`commit_with_clipping`] exists because the print dialog **is** the
//! confirmation — there is no second gate — so the uncertainty has to be
//! stated *in the disclosure itself* rather than implied by a confirm step
//! existing. That is rule 4 applied to a button. A separate warning label
//! beside the button would be the version an operator can look past.

/// The dialog's window title.
#[must_use]
pub const fn dialog_title() -> &'static str {
    "Print"
}

// ---------------------------------------------------------------------------
// The device, and the three ways there is not one
// ---------------------------------------------------------------------------

/// Label for the printer selector.
#[must_use]
pub const fn printer_label() -> &'static str {
    "Printer"
}

/// Label for the button that opens the **driver's own** properties dialog.
///
/// # Why an ellipsis, and why this exact word
///
/// The ellipsis is the platform convention for "this opens something", and it
/// is doing real work here: the button is beside a combo box, and without it
/// a reader scanning the row has no way to tell that one of the two controls
/// hands them off to another window.
///
/// *Properties* rather than *Setup*, *Preferences* or *Options* because it is
/// the word Windows itself uses on that dialog's own title bar, and because
/// it is what the operator asked for: *"pretty much every program I have ever
/// seen lets you press a properties button beside the selected printer in the
/// drop-down menu to open the printer options."*
#[must_use]
pub const fn properties() -> &'static str {
    "Properties…"
}

/// Hover text for [`properties`].
///
/// # ★ It states the override, and that is the non-obvious half
///
/// The driver's dialog offers orientation and paper alongside media type,
/// quality and finishing. pdfcer **asserts its own** orientation over whatever
/// that dialog set — always, because a `DEVMODE` handed to `CreateDC` carries
/// one and this dialog's radios are what the preview was drawn from. An
/// operator who sets landscape there and gets portrait paper would have no
/// way to find out why, and would reasonably conclude the driver dialog was
/// ignored wholesale. It is not: everything pdfcer does not name survives.
///
/// Paper is the other way round and is stated as such — a sheet chosen in the
/// driver's dialog is adopted by the combo beside it, so the two surfaces
/// cannot end up describing the same job differently.
#[must_use]
pub const fn properties_tooltip() -> &'static str {
    "Opens this printer's own settings — media type, quality, finishing and anything else the driver offers. pdfcer keeps the orientation chosen here in the Print dialog; a paper size chosen there is picked up by the Paper list."
}

/// Disclosure that the driver's settings are being carried with the job.
///
/// Shown only once the operator has been through [`properties`] and accepted
/// it, because before then there is nothing to say: pdfcer sends no `DEVMODE`
/// at all and the device's own defaults apply in full.
///
/// # Why it is worth a line of the operator's attention
///
/// Because the settings it refers to are invisible from this dialog. A print
/// configured for glossy photo paper at best quality looks, from inside
/// pdfcer, exactly like one configured for plain draft. The line is the only
/// evidence in the application that a job is carrying anything beyond what
/// the three tabs show.
#[must_use]
pub const fn properties_held() -> &'static str {
    "This printer's own settings are being sent with the job."
}

/// The driver's properties dialog could not be opened, and why.
///
/// `detail` is `pdfcer-print`'s own error `Display`, passed through for the
/// same reason [`failed`] passes one through.
///
/// **Cancel is not this.** An operator who pressed Cancel gets nothing at
/// all: the engine reports that as `Ok(None)` and it is them declining, not a
/// failure. Showing a message there would be scolding them for using the
/// dialog correctly.
#[must_use]
pub fn properties_failed(detail: &str) -> String {
    format!("This printer's own settings could not be opened. {detail}")
}

/// **pdfcer could not ask this system about its printers at all.**
///
/// The first of the three no-printer sentences (module docs). It is what this
/// build says when the print spooler itself could not be queried.
///
/// # ★ This sentence used to be a lie, and the lie is instructive
///
/// It read *"This build cannot reach a print device"* for the whole of
/// v0.1.0, which was true when it was written — `pdfcer-print` was not a
/// dependency — and became false the moment the manifest line landed while
/// the adapter's four calls were left refusing. So the dialog told every
/// operator that the program they were running could not print, on a machine
/// with printers, in a build that had the printing crate linked into it.
///
/// The wording is now about the **spooler**, not the build, because that is
/// the only thing this sentence can honestly be about: a query was attempted
/// and it failed. [`crate::dialogs::print::spooler::Unavailable`] carries the
/// engine's own account of *why*, and [`spooler_detail`] is how it is shown —
/// a general sentence an operator can act on, plus the specific one they can
/// quote at whoever administers the machine.
///
/// Deliberately **not** "no printers were found": that would be a claim about
/// the operator's hardware made on evidence pdfcer does not have. It also
/// covers the honest non-Windows case, where `pdfcer-print`'s every entry
/// point returns `Unsupported`.
///
/// Says plainly that the capability is absent rather than showing controls
/// the shell would then ignore — the same choice, for the same reason, as
/// [`crate::text::open_needs_password`].
#[must_use]
pub const fn spooler_unavailable() -> &'static str {
    "pdfcer could not ask this system about its printers, so there is nothing to \
     print to. Nothing has been sent."
}

/// The engine's own account of a spooler failure, shown beneath
/// [`spooler_unavailable`].
///
/// # Why two sentences rather than one
///
/// They answer different people. [`spooler_unavailable`] tells the operator
/// what happened and that nothing was printed; this tells them — or whoever
/// they forward it to — *which* thing failed, in `pdfcer-print`'s own words,
/// which already carry the remedy ("the Print Spooler service may be
/// stopped", "run `pdfcer list-printers` to see the names this machine
/// knows").
///
/// Merging them would mean either dropping the specific half or building one
/// sentence by concatenation, and a concatenated sentence is the one that
/// reads badly in exactly the cases nobody tested. This is the same split
/// [`failed`] already uses for a spool that was attempted and refused.
#[must_use]
pub fn spooler_detail(detail: &str) -> String {
    format!("The print system reported: {detail}")
}

/// **The spooler answered, and this system has no printers installed.**
///
/// The second sentence. A true statement about the machine, which is why it
/// may not be used for the case above.
#[must_use]
pub const fn no_printers() -> &'static str {
    "This system reports no printers. Install one, then reopen this dialog."
}

/// **This particular printer's driver would not describe itself.**
///
/// The third. Everything the preview draws — the sheet, the printable
/// rectangle, the unprintable margins — comes from the device's own reported
/// geometry, so without it there is no honest picture to draw. Saying so is
/// better than drawing a plausible sheet: a guessed rectangle is exactly the
/// "confidently wrong" preview the whole feature exists to prevent.
#[must_use]
pub const fn device_unavailable() -> &'static str {
    "This printer's driver did not report its paper size, so pdfcer cannot show \
     what the sheet will look like. Choose another printer."
}

/// The dialog was asked to draw with no document open.
///
/// Reachable only if a document is closed while the dialog is up. The dialog
/// closes itself in that case; this is the sentence for the spool path, which
/// must refuse rather than assume.
#[must_use]
pub const fn no_document() -> &'static str {
    "No document is open, so there is nothing to print."
}

// ---------------------------------------------------------------------------
// The tab strip
// ---------------------------------------------------------------------------

/// Tab 1's label.
#[must_use]
pub const fn tab_pages_layout() -> &'static str {
    "Pages & Layout"
}

/// Tab 1's hover text — the question the tab answers.
#[must_use]
pub const fn tab_pages_layout_tooltip() -> &'static str {
    "Which pages print, and how each one lands on the sheet."
}

/// Tab 2's label.
#[must_use]
pub const fn tab_copies_finishing() -> &'static str {
    "Copies & Finishing"
}

/// Tab 2's hover text.
#[must_use]
pub const fn tab_copies_finishing_tooltip() -> &'static str {
    "How many sheets come out, in what order, and on how many sides."
}

/// Tab 3's label.
#[must_use]
pub const fn tab_comments_resolution() -> &'static str {
    "Comments & Resolution"
}

/// Tab 3's hover text.
#[must_use]
pub const fn tab_comments_resolution_tooltip() -> &'static str {
    "What is painted onto each page, and how finely."
}

// ---------------------------------------------------------------------------
// Tab 1 — Pages & Layout
// ---------------------------------------------------------------------------

/// Heading over the page-range radios.
#[must_use]
pub const fn pages_heading() -> &'static str {
    "Pages"
}

/// "All N pages" — the count is in the label so the operator can see what
/// "all" costs before choosing it.
#[must_use]
pub fn range_all(pages: usize) -> String {
    if pages == 1 {
        "All 1 page".to_owned()
    } else {
        format!("All {pages} pages")
    }
}

/// The page currently on the canvas.
#[must_use]
pub const fn range_current() -> &'static str {
    "Current page"
}

/// The typed-range radio.
#[must_use]
pub const fn range_custom() -> &'static str {
    "Pages"
}

/// Hover text for the range box.
///
/// **States the syntax by example**, because the syntax is shared verbatim
/// with `pdfcer` — see [`crate::dialogs::print::tabs::parse_page_range`]
/// for why there is exactly one parser — and an operator who learns it here
/// can use it there.
#[must_use]
pub const fn range_hint() -> &'static str {
    "Page numbers or ranges, for example 3 or 1-4 or 5,1-2. \
     Numbers are the ones printed on the page, starting at 1."
}

/// The typed range names no page in this document.
///
/// A refusal, not a correction. The parser yields *nothing* rather than a
/// guess for malformed input, precisely so this sentence can be shown and the
/// commit button can go absent — instead of printing a range nobody asked
/// for.
#[must_use]
pub const fn range_unparsable() -> &'static str {
    "That range does not name any page in this document."
}

/// Label in front of the odd/even radios.
#[must_use]
pub const fn subset_label() -> &'static str {
    "Subset"
}

/// No odd/even filtering.
#[must_use]
pub const fn subset_all() -> &'static str {
    "Every page"
}

/// Odd document pages only.
#[must_use]
pub const fn subset_odd() -> &'static str {
    "Odd only"
}

/// Even document pages only.
#[must_use]
pub const fn subset_even() -> &'static str {
    "Even only"
}

/// Hover text over the subset row.
///
/// **Says which numbering is meant**, because the answer is not obvious and
/// getting it wrong prints the wrong half of the document. `pdfcer-print`
/// (`lib.rs:1217-1224`): *"an operator printing '2-9, odd' means document
/// pages 3, 5, 7, 9 — the numbers printed on the paper."*
#[must_use]
pub const fn subset_tooltip() -> &'static str {
    "Odd and even mean the page numbers printed on the paper, not positions \
     within the range above. The subset narrows the range; the two combine."
}

/// Heading over the sizing radios.
#[must_use]
pub const fn sizing_heading() -> &'static str {
    "Sizing"
}

/// Scale up or down to fill the printable area.
#[must_use]
pub const fn scale_fit() -> &'static str {
    "Fit to the printable area"
}

/// One PDF point to one point of paper.
#[must_use]
pub const fn scale_actual() -> &'static str {
    "Actual size"
}

/// Reduce an oversized page; never enlarge a small one.
#[must_use]
pub const fn scale_shrink() -> &'static str {
    "Shrink oversized pages only"
}

/// An explicit percentage.
#[must_use]
pub const fn scale_custom() -> &'static str {
    "Custom scale"
}

/// Hover text over the sizing group.
///
/// **Names the difference between Fit and Shrink**, which is the one thing
/// about this group an operator can get wrong without noticing. `pdfcer-print`
/// keeps them as separate modes because collapsing them *"silently blows a
/// business card up to A4"* (`lib.rs:490-494`), and a UI that does not say so
/// re-creates the confusion the engine avoided.
#[must_use]
pub const fn sizing_tooltip() -> &'static str {
    "Fit scales in both directions, so a small page is enlarged to fill the \
     sheet. Shrink oversized pages only ever reduces."
}

/// Suffix on the custom-scale spinner.
#[must_use]
pub const fn percent_suffix() -> &'static str {
    " %"
}

/// Heading over the orientation radios.
#[must_use]
pub const fn orientation_heading() -> &'static str {
    "Orientation"
}

/// Decide per page from its own shape.
#[must_use]
pub const fn orientation_auto() -> &'static str {
    "Auto, from each page's shape"
}

/// Force portrait.
#[must_use]
pub const fn orientation_portrait() -> &'static str {
    "Portrait"
}

/// Force landscape.
#[must_use]
pub const fn orientation_landscape() -> &'static str {
    "Landscape"
}

// ---------------------------------------------------------------------------
// Tab 2 — Copies & Finishing
// ---------------------------------------------------------------------------

/// Label in front of the copy-count spinner.
#[must_use]
pub const fn copies_label() -> &'static str {
    "Copies"
}

/// The collation checkbox, phrased as the *un*-collated option.
///
/// Phrased this way round because collated is the default and the checkbox
/// therefore describes the change, not the state. "Collate" as a checked-by-
/// default box reads as a feature being switched off, which is the more
/// confusing of the two framings.
#[must_use]
pub const fn uncollated() -> &'static str {
    "Group each page's copies together, rather than repeating the whole set"
}

/// Print the sequence back to front.
#[must_use]
pub const fn reverse() -> &'static str {
    "Print back to front"
}

/// Hover text for reverse — names the reason it exists.
#[must_use]
pub const fn reverse_tooltip() -> &'static str {
    "For a printer that stacks face-up, so the finished pile is in order."
}

/// Heading over the duplex radios.
///
/// The whole group is **absent** on a device whose driver does not report
/// duplex support, rather than greyed — see the tab body for why, and
/// `docs/core-api/03` §6.3 item 4 for the engine's side of it. There is
/// deliberately no "your printer cannot do this" sentence here: no setting in
/// this dialog would ever make it possible, so there is nothing to explain
/// and nothing to hope for.
#[must_use]
pub const fn duplex_heading() -> &'static str {
    "Two-sided"
}

/// One side only.
#[must_use]
pub const fn duplex_off() -> &'static str {
    "One-sided"
}

/// Two-sided, flipped on the long edge — the usual book binding.
#[must_use]
pub const fn duplex_long() -> &'static str {
    "Two-sided, long-edge binding"
}

/// Two-sided, flipped on the short edge — notepad binding.
#[must_use]
pub const fn duplex_short() -> &'static str {
    "Two-sided, short-edge binding"
}

/// Label for the tray-by-sheet-size checkbox.
///
/// # ★ This control was deleted on 2026-08-17 and is back on 2026-08-18
///
/// It was removed because it did nothing: `DeviceSettings::pick_tray_by_page_size`
/// was a field `pdfcer-print` declared and read nowhere, so the job spooled,
/// the paper came out of the default tray, and nothing reported that the
/// request had been dropped — indistinguishable, from the operator's side,
/// from a driver that had declined it.
///
/// The engine now honours it (`DMBIN_FORMSOURCE`, asserted only when this box
/// is ticked). The control is therefore backed, and the reason it was removed
/// no longer holds.
///
/// **What is worth carrying forward is the removal, not the restoration.**
/// Deleting a control that succeeds while doing nothing is the correct move
/// and it is a harder call than deleting one that visibly fails, because
/// there is no symptom to point at.
#[must_use]
pub const fn tray_by_size() -> &'static str {
    "Let the printer choose the tray from each page's size"
}

/// Hover text for [`tray_by_size`].
#[must_use]
pub const fn tray_tooltip() -> &'static str {
    "Useful when a document mixes sheet sizes and the printer has a tray loaded for each. Off, every sheet is fed from the printer's usual tray."
}

/// ★ Disclosure for a driver that did not advertise tray-by-size.
///
/// # Why the control is still offered, which inverts this project's usual rule
///
/// R83 says never offer an affordance the hardware cannot honour — which is
/// why there is no duplex control on a simplex device. It does **not** apply
/// here, and `pdfcer-print` declined this project's proposal to gate the
/// control the same way, with a measurement:
///
/// > *"`DC_BINS` on Microsoft Print to PDF returns nothing at all, while that
/// > same device's `dmDefaultSource` is already `DMBIN_FORMSOURCE` — it picks
/// > by form by default. A bool would have collapsed 'the driver said
/// > nothing' into 'no', and told the operator a device cannot do the thing
/// > it was already doing."*
///
/// So a query that answered *"I do not know"* is not a query that answered
/// *"no"*, and hiding the control on that basis would remove a working
/// capability on the commonest Windows printer there is. It stays, with this
/// line under it, and the request is still sent.
#[must_use]
pub const fn tray_not_advertised() -> &'static str {
    "This printer does not advertise tray selection by sheet size. The request is still sent; the driver may ignore it."
}

// ---------------------------------------------------------------------------
// Paper — the sheet, and the fact that asking for one is only asking
// ---------------------------------------------------------------------------

/// Heading over the paper selector.
#[must_use]
pub const fn paper_heading() -> &'static str {
    "Paper"
}

/// The paper entry meaning "say nothing; use whatever this printer is set to".
///
/// # Why it is not called "Default"
///
/// Because *default* invites the reading "the default for this document" or
/// "pdfcer's default", and it is neither: it is the sheet named in this
/// printer's own Windows settings, which the operator may have changed
/// yesterday for a different job in a different program. Naming the source
/// rather than the status is what makes the entry checkable.
///
/// It is also genuinely different from picking the same size explicitly. This
/// entry sends **no paper request at all**, so a driver that would have
/// ignored one is not being asked to; the explicit entries are requests, with
/// everything [`paper_is_a_request`] says about them.
#[must_use]
pub const fn paper_device_default() -> &'static str {
    "From the printer's own settings"
}

/// One entry in the paper list: the driver's name for it, and its size.
///
/// # Why the size is repeated when the name usually contains it
///
/// Because *usually* is not *always*, and the exceptions are the ones that
/// matter. `"A4"` and `"Letter"` are self-describing; `"Roll Paper 24in"`,
/// `"User Defined"`, `"Custom"`, `"Photo Paper (Borderless)"` and
/// `"Oversize"` are not, and a plotter operator choosing between three roll
/// entries has nothing else to go on. Repeating it costs a familiar entry
/// nothing and rescues the unfamiliar ones.
///
/// Millimetres, not points, and for the same reason [`sheet_from_driver`]
/// gives: the operator is matching this against a ream label or a roll box.
#[must_use]
pub fn paper_form(name: &str, size_pt: (f64, f64)) -> String {
    let mm = |pt: f64| (pt * 25.4 / 72.0).round() as i64;
    format!("{name} — {} × {} mm", mm(size_pt.0), mm(size_pt.1))
}

/// Shown in place of the paper list when the driver enumerated none.
///
/// # Not an error, and not the same as an empty list being a bug
///
/// A driver is entitled to answer nothing. `DC_PAPERS` is a query, not an
/// obligation, and a device with one fixed sheet has a defensible reason to
/// list none. The honest response is to say the list is missing and carry on
/// printing on whatever the printer is set to — which is exactly what this
/// build did for its whole life before the list existed.
///
/// Absent rather than an empty greyed combo: R9. A combo with nothing in it
/// is a control that cannot ever act.
#[must_use]
pub const fn paper_not_listed() -> &'static str {
    "This printer did not list any paper sizes. The job will use whatever the printer's own settings name."
}

/// ★ **The sheet a chosen paper actually means: a request, not a setting.**
///
/// # Why this sentence exists, in the engine's own measurement
///
/// `pdfcer-print` reported, while building the paper path: **two drivers were
/// found silently ignoring a paper request.** The `DEVMODE` goes out with
/// `DM_PAPERSIZE` asserted, the driver does as it pleases, and Win32 offers
/// no acknowledgement to read. There is nothing pdfcer can check and nothing
/// it can retry.
///
/// So this is an inference pdfcer cannot verify and the operator cannot see
/// until the paper is already out of the machine — rule 4's *fuzzy, never
/// sneaky*, and the half of that rule people forget: **an inference the
/// operator cannot see still owes a report.**
///
/// # Why the disclosure is here and not on the preview
///
/// Rule 4 again, the clause that is most often got backwards. The preview
/// draws the requested sheet exactly as it draws any other — no dashed
/// outline, no amber tint, no "provisional" styling. Marking it would be a
/// second rendering path for the same picture, and the operator's own
/// objection to the old shell was that *"the nagging and red flagging made
/// for a lot of extra bugs in the visibility when editing."*
///
/// # Why it names a first-sheet check
///
/// Because that is the only verification available to anybody. Telling an
/// operator that something might silently fail, without telling them how they
/// would know, is a sentence that raises anxiety and resolves nothing.
#[must_use]
pub fn paper_is_a_request(sheet: Option<(f64, f64)>) -> String {
    let Some((w_pt, h_pt)) = sheet else {
        return "pdfcer asks the printer for this sheet. A driver may ignore the request without reporting it, so check the first sheet that comes out.".to_owned();
    };
    let mm = |pt: f64| (pt * 25.4 / 72.0).round() as i64;
    format!(
        "Planned for {} × {} mm. pdfcer asks the printer for this sheet; a driver may ignore the request without reporting it, so check the first sheet that comes out.",
        mm(w_pt),
        mm(h_pt),
    )
}

/// The sheet the job was laid out on, when the operator asked for no
/// particular one.
///
/// # ★ This sentence used to end "pdfcer cannot change it", and that is no
/// # longer true
///
/// It read, for as long as there was no paper control:
///
/// > *"Paper: 595 × 842 pt (210 × 297 mm), from this printer's own settings
/// > in Windows. pdfcer cannot change it — set it in the printer's preferences
/// > and reopen this dialog."*
///
/// Which was correct, disclosed the right thing, and named the only remedy
/// there was. It became false on 2026-08-18, when `pdfcer-print` shipped
/// `PaperSelection` and this dialog grew a list — and a sentence like that
/// does not announce its own expiry. It was found because the work that
/// falsified it was the work that changed this file; had the paper control
/// been added anywhere else, the dialog would have offered a paper list with
/// a line under it saying paper could not be chosen.
///
/// **The general lesson, which this project has now paid for three times:**
/// a disclosure that names a limitation is a claim with a shelf life, and it
/// expires silently. `check-string-gaps.sh` can find a malformed literal; no
/// gate can find a true sentence that stopped being true.
///
/// # Why it names millimetres as well as points
///
/// Points are the document's unit and the one the rest of this dialog speaks,
/// so they come first. Millimetres are the unit the operator's paper is sold
/// in and the one they will compare against — *"210 × 297"* is recognisable
/// as A4 in a way that *"595 × 842 pt"* is not, and recognising it is the
/// whole purpose of the line.
///
/// # The `None` case is not an error
///
/// It is simply "no plan yet" — no printer chosen, or the device would not
/// describe itself. The device-unavailable sentence covers the second and the
/// selector covers the first, so this line steps back rather than adding a
/// third refusal to a column that already has one.
#[must_use]
pub fn sheet_from_driver(sheet: Option<(f64, f64)>) -> String {
    let Some((w_pt, h_pt)) = sheet else {
        return "Paper comes from this printer's own settings in Windows.".to_owned();
    };
    // 1 pt = 1/72 inch, 1 inch = 25.4 mm. Rounded to whole millimetres: the
    // operator is matching this against a ream label, and a tenth of a
    // millimetre of driver rounding is noise that makes a familiar size look
    // unfamiliar.
    let mm = |pt: f64| (pt * 25.4 / 72.0).round() as i64;
    format!(
        "Planned for {} × {} pt ({} × {} mm), from this printer's own settings in Windows.",
        w_pt.round() as i64,
        h_pt.round() as i64,
        mm(w_pt),
        mm(h_pt),
    )
}

// ---------------------------------------------------------------------------
// Tab 3 — Comments & Resolution
// ---------------------------------------------------------------------------

/// Heading over the annotation-scope radios.
#[must_use]
pub const fn comments_heading() -> &'static str {
    "Comments and forms"
}

/// Page content, links and form-field widgets — no review markup.
///
/// The **default for printing**, which differs from the renderer's own
/// `DocumentAndMarkups` default. Deliberate on both sides: the canvas should
/// show markup, and a print should not carry review comments unless asked.
#[must_use]
pub const fn scope_document() -> &'static str {
    "Document"
}

/// Everything above, plus review markup.
#[must_use]
pub const fn scope_markups() -> &'static str {
    "Document and markups"
}

/// Everything above, restricted to stamps.
#[must_use]
pub const fn scope_stamps() -> &'static str {
    "Document and stamps"
}

/// Form-field widgets only, over blank page content.
#[must_use]
pub const fn scope_fields_only() -> &'static str {
    "Form fields only"
}

/// Heading over the resolution disclosure.
#[must_use]
pub const fn resolution_heading() -> &'static str {
    "Resolution"
}

/// The standing note that pdfcer prints rasters, not vectors.
///
/// **Always true, so a caption rather than a warning.** A banner that fires
/// on every job trains an operator to stop reading banners — which is how the
/// *conditional* disclosure beneath it ([`dpi_capped`]) would come to be
/// ignored too.
#[must_use]
pub const fn raster_note() -> &'static str {
    "pdfcer renders each page to an image at the resolution below and sends \
     that image. Text and lines are not sent as vectors."
}

/// pdfcer chose a resolution the operator did not.
///
/// The conditional half of the resolution disclosure, and it exists because
/// `JobResolution::capped` is pdfcer's own memory judgement rather than
/// anything the device or the document asked for (`docs/core-api/03` §6.3
/// item 3). It names all three numbers — what will be used, what the device
/// could do, and what lifting the cap would cost — because an operator
/// deciding whether to raise it needs the cost, not just the fact.
#[must_use]
pub fn dpi_capped(dpi: u32, device_dpi: u32, uncapped_page_mb: u64) -> String {
    format!(
        "Printing at {dpi} DPI. This printer can do {device_dpi} DPI, but one page \
         at that resolution costs pdfcer about {uncapped_page_mb} MB of memory, so \
         pdfcer capped it. Raise the cap if you need the detail."
    )
}

/// Suffix on the DPI spinner.
#[must_use]
pub const fn dpi_suffix() -> &'static str {
    " DPI"
}

// ---------------------------------------------------------------------------
// The preview
// ---------------------------------------------------------------------------

/// "Sheet i of n" — which sheet of the **job** is showing.
///
/// Says *sheet*, not *page*, and the distinction is load-bearing: the stepper
/// walks the job's own sequence, which may be a custom range, odd/even
/// filtered, reversed, or repeated for copies. Calling position 3 "page 3"
/// would name a document page the job might not even contain.
#[must_use]
pub fn preview_position(index: usize, total: usize) -> String {
    format!("Sheet {index} of {total}")
}

/// Step to the previous sheet of the job.
#[must_use]
pub const fn preview_previous() -> &'static str {
    "Previous"
}

/// Step to the next sheet of the job.
#[must_use]
pub const fn preview_next() -> &'static str {
    "Next"
}

/// Put the preview back to fit, centred.
#[must_use]
pub const fn preview_zoom_fit() -> &'static str {
    "Fit"
}

/// Hover text for Fit.
#[must_use]
pub const fn preview_zoom_fit_tooltip() -> &'static str {
    "Show the whole sheet, centred."
}

/// Zoom the preview out one step.
#[must_use]
pub const fn preview_zoom_out() -> &'static str {
    "Zoom out"
}

/// Zoom the preview in one step.
#[must_use]
pub const fn preview_zoom_in() -> &'static str {
    "Zoom in"
}

/// Draw one PDF point as one screen point.
#[must_use]
pub const fn preview_zoom_actual() -> &'static str {
    "100%"
}

/// Hover text for the actual-size button.
#[must_use]
pub const fn preview_zoom_actual_tooltip() -> &'static str {
    "Draw the sheet at its true size on this screen."
}

/// The magnification readout.
///
/// **A percentage of ACTUAL size, never of the fit.** A number expressed
/// against the fit would change whenever the window was dragged, without the
/// operator touching a zoom control — so it would report the window, not the
/// sheet, and would be useless for the one question the preview exists to
/// answer ("will this fine print clear the margin?").
#[must_use]
pub fn preview_zoom_percent(percent: u32) -> String {
    format!("{percent}% of actual size")
}

/// The gesture hint under the preview.
#[must_use]
pub const fn preview_pan_hint() -> &'static str {
    "Drag to pan, Ctrl+wheel to zoom"
}

/// Move the preview into a window of its own — operator request O112.
///
/// ★ **"Pop out"** is the phrase the product class has settled on — a browser's
/// picture-in-picture, an editor's detached panel, a chat client's detached
/// call window all use it or a near synonym, and the operator used it himself:
/// *"the option to pop out into its own resizeable window"*. Using his word
/// rather than a tidier one ("Detach", "Open in new window") costs nothing and
/// means the control is named the thing he went looking for.
#[must_use]
pub const fn preview_pop_out() -> &'static str {
    "Pop out"
}

/// Hover text for the pop-out button.
///
/// ★★ It states the way BACK, in the same breath as the way out. A control that
/// moves a surface somewhere else owes the operator the return trip before they
/// take it — otherwise the first thing they do after popping it out is hunt the
/// print dialog for a button to put it back, and there is not one, because
/// closing the window is the gesture.
#[must_use]
pub const fn preview_pop_out_tooltip() -> &'static str {
    "Show the preview in its own window, which you can resize and move to another screen. \
     Closing that window puts the preview back here."
}

/// The pop-out window's title bar.
///
/// ★ Deliberately **not** the document's name. The title bar's job is to make
/// this window findable in the taskbar beside the print dialog it came from,
/// and *"Print preview"* is what a person scanning a task list is looking for.
/// A file name there would sit beside the main window's file name and the two
/// would be told apart only by whatever the shell appended.
#[must_use]
pub const fn preview_window_title() -> &'static str {
    "Print preview"
}

/// The job selects no pages, so there is nothing to preview.
#[must_use]
pub const fn no_pages_selected() -> &'static str {
    "This job selects no pages, so there is nothing to preview and nothing to print."
}

/// **Content will be lost off the edge of the printable area.**
///
/// Shown for the whole job, always, not only for the sheet on screen — a
/// multi-page job's clip is frequently on a sheet the operator is not looking
/// at, and a count that only appeared when you happened to step onto the
/// offending sheet would be a disclosure you could miss by not scrolling.
///
/// This is the GUI half of the divergence `pdfcer-print` was built for:
/// *"Acrobat's documented behaviour here is to clip SILENTLY … pdfcer reports
/// it instead"* (`lib.rs:522-528`). That divergence is worth nothing if the
/// shell reduces it to a number an operator can look past, which is why the
/// same fact also reaches [`commit_with_clipping`].
#[must_use]
pub fn clip_summary(clipped: usize, total: usize) -> String {
    if clipped == 1 {
        format!("1 of these {total} sheets will lose content outside the printable area.")
    } else {
        format!("{clipped} of these {total} sheets will lose content outside the printable area.")
    }
}

/// **A CEILING on how many sheets will lose content**, for the state where
/// some have been examined and some have not — operator request O113,
/// 2026-09-04.
///
/// # Why this sentence exists rather than a reworded [`clip_summary`]
///
/// `clip_summary` states a number that was *counted*: every sheet it names has
/// a page box exceeding the printable rectangle, or (once every clipped sheet
/// has been previewed) has been measured to carry ink out in the band. This
/// one states a number that was **bounded**. With some sheets examined and
/// some not, the count is `known_inked + unexamined`, and the true figure can
/// be anywhere from `known_inked` up to that — see
/// `dialogs::print::verdicts`' header for the inequality.
///
/// ★★★ **The hedge is a correction, not a weakening.** Saying "will" of a
/// number nobody measured would be the invented claim; the two words that
/// change — *"Up to"* and *"may"* — are the difference between reporting a
/// measurement and reporting a bound, and they appear exactly when the number
/// stops being a measurement. Nothing softens `clip_summary` itself: where
/// nothing has been subtracted, that sentence is still what is shown, in the
/// words it has always used.
///
/// # Why it does not name which sheets
///
/// Because the answer would be a list that grows as the operator steps through
/// the preview, on a surface they are not looking at. The preview's own
/// caption names the sheet on screen; this line is the job.
#[must_use]
pub fn clip_summary_at_most(clipped: usize, total: usize) -> String {
    if clipped == 1 {
        format!("Up to 1 of these {total} sheets may lose content outside the printable area.")
    } else {
        format!(
            "Up to {clipped} of these {total} sheets may lose content outside the printable area."
        )
    }
}

/// **The sheet on screen overhangs the printable area, and the overhang is
/// empty paper** — operator request O113, 2026-09-03.
///
/// # ★★★ The sentence that stops a warning and a picture contradicting
///
/// > *"can you make it so the red pattern you put over the page if it is going
/// > to print beyond the printable borders is only over the areas that extend
/// > beyond the printable page? Our drawing get drawn 1:1 and the area that
/// > isn't printed is just empty border."*
///
/// [`clip_summary`] above counts sheets whose **page box** exceeds the
/// printable rectangle. That count is a plan-time geometric fact and it is
/// still exactly true. Since O113 the hatch beside it is not geometric: it
/// samples the raster and covers only what actually carries ink. So on the
/// operator's own 1:1 drawings the two disagree — a sentence saying content
/// will be lost, over a picture that visibly loses none — and an operator
/// resolving that disagreement resolves it by trusting neither half.
///
/// This is the resolution. It does **not** contradict the count and does not
/// soften it; it adds the one thing the count could not know, which is what is
/// actually printed on the part that will be cropped.
///
/// # Why it names the SHEET and says "on screen"
///
/// Because that is the only sheet whose raster exists. The preview renders the
/// page it is showing and no other, so this is a statement about one sheet, and
/// wording it as though it covered the job would be a claim nothing checked.
/// The job-wide count above stays the statement about the job.
///
/// # Why "nothing is printed there" rather than "nothing will be lost"
///
/// The stronger phrasing would be a promise about the outcome, and the ink test
/// has a threshold in it (`dialogs::print::ink::INK_MAX_LEVEL`) — a mark
/// lighter than about 4% grey is treated as paper. Saying what was *observed*
/// on the sheet is a claim this code can support; saying what *will* happen at
/// the printer is one it cannot.
#[must_use]
pub const fn overhang_is_blank() -> &'static str {
    "This sheet hangs over the printable area, but nothing is printed there — the overhang is \
     blank."
}

// ---------------------------------------------------------------------------
// The footer — the one irreversible control in the application
// ---------------------------------------------------------------------------

/// Leave without printing.
///
/// Says **Close**, not Cancel: nothing has started, so there is nothing to
/// cancel, and a Cancel button next to a Print button invites the reading
/// that a job is in flight and this stops it.
#[must_use]
pub const fn close() -> &'static str {
    "Close"
}

/// Send the job. The plain label, when nothing will be clipped.
#[must_use]
pub const fn commit() -> &'static str {
    "Print"
}

/// Send the job, **with the clip count in the button's own label**.
///
/// The dialog is the confirmation and there is no second gate, so the
/// uncertainty is stated in the disclosure rather than implied by a confirm
/// step existing. Putting it *in the label* rather than beside the button is
/// the difference between a warning the operator has to have read and one
/// they can have looked past — it is on the control their hand is already on.
#[must_use]
pub fn commit_with_clipping(clipped: usize) -> String {
    if clipped == 1 {
        "Print — 1 sheet will be clipped".to_owned()
    } else {
        format!("Print — {clipped} sheets will be clipped")
    }
}

/// Send the job, **naming a count that has actually been measured** —
/// operator request O113, 2026-09-04.
///
/// # Why a second sentence rather than a reworded [`commit_with_clipping`]
///
/// The two say different things and both are needed.
///
/// `commit_with_clipping` says *"N sheets will be **clipped**"* — a geometric
/// fact about page boxes and the printable rectangle, taken at planning time
/// with no raster in hand. It is exactly true and it is what the button says
/// when nothing better is known.
///
/// This one says *"N sheets will **lose content**"*, and it may only be shown
/// when every clipped sheet in the job has been rendered by the preview and
/// its overhang tested for ink. `N` is then the number that really will lose
/// something, which is smaller than the geometric count whenever the operator
/// prints a 1:1 CAD sheet whose border is empty paper — *"the area that isn't
/// printed is just empty border."*
///
/// ★★ **Reusing the old sentence for the corrected number would have been the
/// defect.** With two of five clipped sheets known blank, *"Print — 3 sheets
/// will be clipped"* is plainly false: five are clipped. The count changed
/// what it counts, so the sentence has to say what it now counts. That is a
/// correction, and it is the opposite of softening a true statement to match a
/// better one.
#[must_use]
pub fn commit_losing_content(losing: usize) -> String {
    if losing == 1 {
        "Print — 1 sheet will lose content".to_owned()
    } else {
        format!("Print — {losing} sheets will lose content")
    }
}

/// Send the job, **naming a ceiling** — operator request O113, 2026-09-04.
///
/// Shown when some clipped sheets have been examined and found blank and
/// others have not been looked at. The number is `known_inked + unexamined`:
/// the most sheets that could possibly lose something, with every sheet nobody
/// has looked at still counted, because a claim about an unexamined sheet
/// would be invented.
///
/// # ★ The two words carrying the whole difference
///
/// *"up to"* and *"may"*. They are here because the number is a bound rather
/// than a count, and they are **absent** from [`commit_losing_content`] and
/// from [`commit_with_clipping`] because those two report numbers that were
/// measured — one by the ink test, one by the geometry. A hedge that appeared
/// on all three would say nothing at all; appearing on exactly the one bounded
/// number is what makes it informative.
///
/// # The singular has no "up to", and that is not an inconsistency
///
/// *"Up to 1 sheet"* reads as a quantity discount. *"1 sheet **may** lose
/// content"* carries the same uncertainty in the word that is doing the work,
/// which is `may` in both forms.
#[must_use]
pub fn commit_may_lose_content(at_most: usize) -> String {
    if at_most == 1 {
        "Print — 1 sheet may lose content".to_owned()
    } else {
        format!("Print — up to {at_most} sheets may lose content")
    }
}

/// Confirmation that the job went out.
#[must_use]
pub fn sent(pages: usize) -> String {
    if pages == 1 {
        "Sent 1 page to the printer.".to_owned()
    } else {
        format!("Sent {pages} pages to the printer.")
    }
}

/// The job did not go out, and why.
///
/// `detail` is `pdfcer-print`'s own error `Display`, passed through rather than
/// rewritten — for the same reason [`crate::text::canvas_render_failed`] does
/// it: those errors are structured, specific diagnostics, and replacing one
/// with "an error occurred" throws away the only part of the sentence that
/// helps.
///
/// **Says nothing came out.** A failed spool can leave an operator wondering
/// whether half a job reached the tray, and the first line of the answer
/// belongs in the message.
#[must_use]
pub fn failed(detail: &str) -> String {
    format!("Nothing was sent to the printer. {detail}")
}

/// ★ **The driver would not report its settings, so the job carried only what
/// pdfcer sets itself.**
///
/// Shown beside [`sent`] after a job whose `SettingsSource` came back
/// `Synthesised`, and after no other.
///
/// # Why this is a disclosure and not an error
///
/// The job printed. Paper came out. Nothing failed, and if the operator was
/// changing nothing but orientation they may not be able to tell the
/// difference.
///
/// What was lost is everything the driver holds that pdfcer does not model:
/// media type, print quality, colour handling, output bin, stapling, and the
/// whole vendor-private half of a `DEVMODE` — which on the printers measured
/// while this was built was between 920 and 7,972 bytes, up to 97 % of the
/// structure. A synthesised `DEVMODE` has no private tail to carry any of it.
///
/// So the failure mode is a print that is *subtly* wrong — plain where glossy
/// was configured, draft where best was — and it looks like a printer problem
/// rather than a pdfcer one. That is precisely the class of thing rule 4
/// exists for: pdfcer chose something the operator did not ask for, and it
/// says so.
///
/// # Why it names the remedy
///
/// Because there is one, and it is one button away: opening the driver's own
/// properties dialog produces a real `DEVMODE`, which the next job carries.
#[must_use]
pub const fn settings_synthesised() -> &'static str {
    "This printer would not report its current settings, so the job was sent with only the settings shown here — media type, quality and finishing fell back to the driver's own. Open Properties… before printing again to send them."
}

/// ★★★ Why the Custom percentage field is greyed —
/// `OPERATOR_REQUESTS.md` O77's sweep.
///
/// `dialogs::print::tabs` has always argued that greying is the correct side
/// of R9 here *because* the field is only **temporarily** unavailable: one
/// click on the radio beside it makes it live. R9's other half is that the
/// argument has to reach the operator, and it never did — the control was
/// greyed with no hover explanation of any kind, so the reasoning existed only
/// in a source comment.
///
/// ★ It names the remedy and where the remedy is. *"Choose Custom"* alone
/// would be true and would still leave him looking for what to choose it on;
/// the radio is immediately to the left and saying so costs three words.
#[must_use]
pub fn scale_custom_disabled() -> &'static str {
    "Choose Custom on the radio beside this field to set your own percentage."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ The three no-printer sentences must be genuinely different.
    ///
    /// Not a tautology test — the same argument as
    /// `crate::text::tests::the_three_open_failures_read_differently`. The
    /// value of the distinction is that an operator can tell from the words
    /// alone which of "this build cannot print", "you have no printers" and
    /// "this printer would not answer" is true, because the three have
    /// different remedies. Three functions producing near-identical prose
    /// would satisfy the type system and defeat the design.
    #[test]
    fn the_three_no_printer_sentences_read_differently() {
        let a = spooler_unavailable();
        let b = no_printers();
        let c = device_unavailable();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    /// The commit label carries the count, and the count is visible in it.
    ///
    /// This is the whole disclosure mechanism: if the number ever stopped
    /// appearing in the string, the button would silently become an ordinary
    /// Print button on a job that loses content.
    #[test]
    fn the_commit_label_states_the_clip_count() {
        assert!(commit_with_clipping(7).contains('7'));
        assert!(commit_with_clipping(1).contains('1'));
        // And it must not read like the plain label, or the disclosure is
        // invisible at a glance.
        assert_ne!(commit_with_clipping(1), commit());
    }

    /// Singular and plural are both grammatical.
    ///
    /// Cheap to get wrong ("1 sheets will be clipped"), and prose that reads
    /// as machine output is prose an operator trusts less — which matters
    /// most on exactly the sentences that are trying to warn them.
    #[test]
    fn the_counted_sentences_are_grammatical_at_one() {
        assert!(commit_with_clipping(1).contains("1 sheet will"));
        assert!(clip_summary(1, 4).contains("1 of these 4 sheets"));
        assert!(commit_losing_content(1).contains("1 sheet will"));
        assert!(commit_may_lose_content(1).contains("1 sheet may"));
        assert!(clip_summary_at_most(1, 4).contains("1 of these 4 sheets"));
        assert!(sent(1).contains("1 page to"));
        assert!(range_all(1).contains("1 page"));
        assert!(!range_all(1).contains("1 pages"));
    }

    /// ★★★ **The three commit labels are three different claims**, and an
    /// operator must be able to tell which one they are being shown from the
    /// words alone — operator request O113, 2026-09-04.
    ///
    /// | label | what it claims | when |
    /// |---|---|---|
    /// | [`commit_with_clipping`] | N page boxes exceed the printable area | nothing examined |
    /// | [`commit_losing_content`] | N sheets really do lose ink | every clipped sheet examined |
    /// | [`commit_may_lose_content`] | **at most** N sheets lose ink | some examined, some not |
    ///
    /// The hedge is the load-bearing distinction: it must be present on the
    /// bounded claim and absent from the two measured ones. A wording change
    /// that put "may" on all three, or took it off the ceiling, would collapse
    /// three states into one sentence and hide exactly the difference the
    /// count was made better to expose.
    #[test]
    fn the_three_commit_labels_are_distinguishable_claims() {
        let geometric = commit_with_clipping(3);
        let measured = commit_losing_content(3);
        let bounded = commit_may_lose_content(3);
        assert_ne!(geometric, measured);
        assert_ne!(measured, bounded);
        assert_ne!(geometric, bounded);

        assert!(
            bounded.contains("may") && bounded.contains("up to"),
            "the ceiling must hedge, or a number nobody measured reads as one that was: {bounded}"
        );
        for measured_claim in [&geometric, &measured] {
            assert!(
                !measured_claim.contains("may"),
                "a measured count must NOT hedge — softening a true statement to match a \
                 better one is how the next defect gets built: {measured_claim}"
            );
        }
        // And all three still carry the number, which is the whole
        // disclosure mechanism.
        for label in [&geometric, &measured, &bounded] {
            assert!(label.contains('3'), "the count vanished from {label}");
        }
    }

    /// The bounded job-wide sentence hedges where its measured twin does not.
    ///
    /// [`clip_summary`] serves both the geometric and the measured state — in
    /// the first it is the unchanged shipped wording, in the second it is
    /// verified — so the only sentence that may hedge is the ceiling's.
    #[test]
    fn only_the_bounded_summary_hedges() {
        assert!(!clip_summary(2, 5).contains("may"));
        assert!(clip_summary_at_most(2, 5).contains("may"));
        assert!(clip_summary_at_most(2, 5).contains("Up to 2"));
    }

    /// The capped-resolution disclosure names all three numbers.
    ///
    /// An operator deciding whether to raise the cap needs the cost of doing
    /// so, not merely the fact that a cap exists. Dropping any one of the
    /// three turns a decision aid back into a notification.
    #[test]
    fn the_dpi_disclosure_names_what_it_costs() {
        let message = dpi_capped(300, 1200, 139);
        for number in ["300", "1200", "139"] {
            assert!(message.contains(number), "missing {number} in {message}");
        }
    }
}
