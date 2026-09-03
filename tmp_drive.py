import io

p = 'tools/ui-verify/src/checks/button_action.rs'
s = io.open(p, encoding='utf-8').read()

# ---- constants -------------------------------------------------------------
s = s.replace(
    '''/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed''',
    '''/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed
/// ★★★ The line `panels::forms::button` writes when it READS an existing
/// button's action — the half that could not ship until `Pass 212.0`.
const READ: &str = "button-action-read"; // ui-text-exempt: a trace event name
/// The Forms panel, opened through the seam.
const FORMS_PANEL: &str = "edit.forms"; // ui-text-exempt: a command id, never displayed''',
    1,
)

# ---- the new final step ----------------------------------------------------
old = """    report.note(format!(
        "★★★ …and the engine wrote it: `{}`",
        applied.raw
    ));
    Ok(None)
}"""
new = """    report.note(format!("★★ the engine wrote it: `{}`", applied.raw));

    // --- E: and the FORMS PANEL can read it back ----------------------------
    //
    // ★★★ **The half that could not ship on the morning of 2026-09-01.**
    //
    // `set_button_action` could write and nothing could read, so a control over
    // an EXISTING button had three possible shapes and all three were bad: show
    // "Nothing" and lie about somebody else's script, invent a one-way "set
    // this button to:" that no form editor has, or make the only way to read an
    // action be to destroy it. The row was declined and the gap was filed.
    //
    // `Pass 212.0` answered it hours later. This step is the proof the answer
    // reached the operator, and its oracle is deliberately the READ rather than
    // the row's pixels: the sentence a reader sees is chosen from four states,
    // and `state=` names which one — a screenshot of "Clear the form" cannot
    // distinguish a correct `Known` from a lucky default.
    driver.press_chord(&[], 0)?; // no-op: keeps the driver alive across the seam below
    let trace = session.trace()?;
    let read = trace
        .events(READ)
        .filter_map(|l| l.get("state").map(str::to_owned))
        .next_back();
    match read.as_deref() {
        Some("known") => {
            report.note("★★★ …and the Forms panel reads it back as a known action");
        }
        Some("none") => {
            return Ok(Some(format!(
                "★★★ THE PANEL READS THE BUTTON AS INERT: `{READ} … state=none`, on a button \\
                 this run has just given a Reset action to and watched the engine accept.\\n\\
                 That is the exact falsehood the reader was requested to prevent — pdfcer \\
                 asserting a fact about the operator's document that it did not check. Look at \\
                 `panels::forms::button::row` and at whether it is asking for the same \\
                 fully-qualified name the author wrote. Trace: {}.",
                session.trace_path().display()
            )));
        }
        Some(other) => {
            return Ok(Some(format!(
                "★★ THE PANEL READS THE BUTTON AS `{other}`: this run authored a `ResetForm`, \\
                 which `Pass 212.0` states round-trips as `Known` — including `Only` vs \\
                 `Except`, which is the thing a reader most easily gets backwards. \\
                 `unmodelled` here means the engine wrote something it cannot decode; \\
                 `foreign` means it decoded something it will not author, on a button pdfcer \\
                 itself just authored. Trace: {}.",
                session.trace_path().display()
            )));
        }
        None => {
            return Err(Error::new(format!(
                "no `{READ}` line, so the Forms panel never drew a push-button row — most \\
                 likely it is not on screen. SKIPPED rather than failed: that is a fact about \\
                 the panel layout this run inherited, not about the reader. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }
    Ok(None)
}"""
assert old in s
s = s.replace(old, new, 1)

# open the Forms panel alongside everything else
s = s.replace(
    'const INVOKE: &str = "mode.edit,edit.form_push_button";',
    'const INVOKE: &str = "mode.edit,edit.forms,edit.form_push_button";',
    1,
)
io.open(p, 'w', encoding='utf-8').write(s)
print('ok')
