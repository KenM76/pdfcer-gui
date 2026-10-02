# Word-like text editing: gap matrix and build plan

Goal (Ken): *text editing just works in all cases as if this were a Word document.*
This builds on the text-edit diagnosis, which covers G074 (finds that cross text objects),
G075 (the subset sieve), G076 (no paragraph verb) and the preview's wrong-size
fallback. Those four are not repeated here, except as dependencies.

**Evidence base.** I drove the current release build, copied to scratch, off-screen
(`PDFCER_DIAG_VIEWPORT=-4200,-4200,1400,900`, scripted pointer). The fixtures are in
`wl\`:

- `word.pdf`: Word COM export. TrueType WinAnsi fonts plus one Type0.
- `chrome.pdf`: Chrome headless print. All fonts are Type0 Identity-H with ToUnicode.
- Copied corpus fixtures: `synthetic-image-only`, `three-text-fields`, `encrypted-aes-128`,
  `rotated-text` and `unreadable-type3`.

Rows tagged **(driven)** were observed on the binary. Rows tagged **(code)** were read
from source by me or a sub-audit.

Verdicts: **works**, **partial**, **missing**, **refuses**.
Fix classes: **GUI** (this repo only) or **ENGINE** (needs a request; the number is given).

## 1. The matrix

### A. Typing and the draft

| # | Expectation | Verdict | Root cause | Fix |
|---|---|---|---|---|
| A1 | Any character I type appears | refuses | Capital Q, R and W were refused in a Word ArialMT run, giving `text-edit-key-refused` (driven). The subset font lacks the glyph, and nothing falls back. | ENGINE G075(b) + G078; GUI interim, step 3 |
| A2 | Paste puts in what I pasted | partial | The `Event::Paste` arm bypasses the sieve, so a refused character only fails at commit (code). | GUI |
| A3 | Pasting several lines gives several lines | missing | Newlines are stripped, and a test pins "one\ntwo" → "onetwo" (code). | GUI interim (newline → space); then G076 |
| A4 | Enter starts a new line or paragraph | refuses | Enter in existing text is declined with `run-cannot-hold-a-newline` (driven). | ENGINE G076 |
| A5 | Ctrl+Z undoes my last keystrokes while typing | missing | Ctrl+Z is dead inside a draft (driven). | GUI |
| A6 | Escape cancels what I typed | by decision | Escape commits the draft: O223 decided it, so Ctrl+Z after Escape is the way back. | none |
| A7 | Ctrl+S while typing saves including my typing | missing | Ctrl+S is dead while composing (code). | GUI |
| A8 | Backspace at a line start joins it to the line above | refuses | Declined with `canvas-delete-declined reason=composing-text` (driven). | ENGINE G076 |
| A9 | Delete at a line end joins the next line | refuses | Same as A8 (driven). | ENGINE G076 |
| A10 | Ctrl+Backspace / Ctrl+Delete delete a word | missing | No binding (code). | GUI |
| A11 | Tab inserts a tab | missing | Tab does nothing in a draft (code). | GUI (insert spaces to the next stop); ENGINE G079 for real tabs |
| A12 | IME input (CJK, emoji picker) works | missing | No `Event::Ime` handling (code). Dead keys probably work. | GUI |
| A13 | The live preview looks like the final text | partial | Chrome PDFs always preview in the UI font, `shaped=0` (driven). The `shape()` fallback triggers on a glyph-count mismatch or `outlines.skipped`; which one is unknown. The Down-arrow line preview is also unshaped (driven). | GUI investigation |
| A14 | Typed text keeps the run's kerning and justification | partial | A justified Word line committed as Pin and was not re-justified (driven). Kerning is lost in the edited span (code). | ENGINE G079 |
| A15 | Long text wraps instead of running off the page | missing | Overflow grows and overlaps. Acrobat and PDF-XChange do the same, but Word users do not expect it. | ENGINE G076/G079 |

### B. Caret and selection

| # | Expectation | Verdict | Root cause | Fix |
|---|---|---|---|---|
| B1 | Double-click selects a word | missing | On unopened text it did not select; typing afterwards grew the length 88 → 89 (driven). | GUI |
| B2 | Triple-click selects a line or paragraph | missing | Not implemented (code, driven). | GUI (line); paragraph after G076 |
| B3 | Dragging across lines selects text | refuses | With the text tool, a drag creates a new text box (driven). | GUI |
| B4 | Shift+Up/Down extends the selection | refuses | It commits and drops the selection (code). | GUI |
| B5 | Up/Down moves the caret between lines | works | The caret moves; its preview is unshaped (A13) (driven). | - |
| B6 | Ctrl+A selects all text in the box or paragraph | missing | No visible effect (driven). | GUI |
| B7 | The caret lands where I click on rotated text | partial | The first click ignores the engine's `hit_test`, and the fallback `caret_box` is axis-aligned (code). | GUI |
| B8 | Copy and paste keep formatting | partial | The clipboard carries plain text only, in both directions (code). | GUI (HTML/RTF out first); paste-with-format after G079 |

### C. Formatting

| # | Expectation | Verdict | Root cause | Fix |
|---|---|---|---|---|
| C1 | Ctrl+B, Ctrl+I and Ctrl+U on a selection | missing | Dead while composing (driven, code). | GUI (the engine's `format_text` takes a substring `find`) |
| C2 | Restyling a selection restyles only that selection | partial | The GUI applies the style to the whole run (code). The engine supports a substring. | GUI |
| C3 | Font, size and colour | works | Run granularity only, so this inherits C2. | GUI via C2 |
| C4 | Underline is real text formatting | partial | It exists only as an annotation (code). | GUI (drawn rule in the content stream via the engine's line verbs); ask if absent |
| C5 | Alignment buttons (left, centre, right, justify) | missing | The engine detects alignment, but no GUI control exists (code). Placement decided by Ken: in both places, see step 7. | GUI; re-justify needs G079 |
| C6 | Superscript and subscript | missing | The engine has `set_rise`/`set_script`, but no command reaches them, and reflow flattens them (code). | GUI; reflow keep needs G079 |
| C7 | Bullets, numbering and hanging indent | missing | `BlockKind` has only `Paragraph` (code). | ENGINE G079 |

### D. Structure

| # | Expectation | Verdict | Root cause | Fix |
|---|---|---|---|---|
| D1 | Edit a centred line | refuses | `NotFound`, `SplitAcrossPieces` (driven). | ENGINE G074 |
| D2 | Edit a table cell | refuses | `NotFound` (driven). The row is one line in the model (code). | ENGINE G074 + G080 |
| D3 | Edit a bullet item | refuses | `NotFound` (driven). | ENGINE G074, then G079 |
| D4 | Rewrap or edit a paragraph from Chrome or Edge | refuses | `reflow_apply` refuses composite fonts (`refuse_if_composite`). Every Chrome face is Type0 (measured). | ENGINE G079 |
| D5 | Rewrap a paragraph with a bold word | refuses | One font per block (code). | ENGINE G079 |
| D6 | Delete a paragraph | missing | No command for it (code). | GUI (loop `delete_text_run` into one undo) |
| D7 | Two-column text stays in its column | partial | Row-major line forming (code). This is unconfirmed for gutters; G080 asks the engine to confirm it. | ENGINE G080 |
| D8 | Add a new text box | works | Only the 14 standard fonts, and no alignment (code). | GUI (embed-a-system-font picker exists elsewhere; reuse) |

### E. Document cases

| # | Expectation | Verdict | Root cause | Fix |
|---|---|---|---|---|
| E1 | Clicking text on a scan explains why it can't be edited and offers OCR | refuses | A click silently becomes Add text, logged as `text-edit-became-add reason=no-run-under-the-click` (driven). The OCR message names a nonexistent "Tools > OCR"; the real place is File › Recognise (code). | GUI |
| E2 | Clicking a form field edits the field value | refuses | The click becomes Add text (driven). | GUI (route to the field editor) |
| E3 | Clicking a text box comment (FreeText) edits it | refuses | The click becomes Add text (code). | GUI (route to the annotation editor) |
| E4 | Edit a password-protected PDF I can open | refuses | `edit_text` refuses any `/Encrypt` (code). The message names a nonexistent "Protect > Remove security"; the real path is File › Security › Encrypt… → "Remove the protection entirely". | GUI message now; ENGINE G077 |
| E5 | Type 3 font text | partial | Editing works partly, and the message is misleading (code). | GUI message; ENGINE G081 cause |
| E6 | Font without ToUnicode | partial | The message is good (code). | - |
| E7 | CJK text | partial | Horizontal works partly. Vertical writing is not guarded and is probably laid out wrong (code). | ENGINE G081 (refuse by name) |
| E8 | Right-to-left text (Arabic, Hebrew) | missing | No bidi handling (code). | ENGINE, later; not filed |
| E9 | Find and replace | partial | Find only. The engine replaces one hit per call (code). | GUI, about 3–4 days |
| E10 | Spell check | missing | Not implemented. | GUI, about 3–5 days |
| E11 | A refusal tells me why and what to do | partial | `Unsupported(String)` falls through to a generic "That change was refused…". Offers are written as prose, not buttons. Sentences of 40–60 words are truncated in the status line (code). | GUI now; ENGINE G081 for causes |

## 2. Build order (GUI)

The order follows what users hit most in Word, LibreOffice and Chrome PDFs. Typing in
body text comes first, then clicks landing on the wrong thing, then formatting, then
structure. Effort is in engineer-days, including the driven check.

| Step | Work | Rows | Effort | Waits on |
|---|---|---|---|---|
| 1 | **Draft basics:** draft-level undo stack (Ctrl+Z/Ctrl+Y); Ctrl+S commits then saves; paste runs through the sieve; newline → space on paste, with a status note; Ctrl+Backspace/Delete; Tab → spaces to the next 0.5 in stop; Ctrl+A selects the run. Escape keeps committing (O223 supersedes A6) | A2 A3 A5 A7 A10 A11 B6 | 2 | Done; driven by `the_draft_keys_do_what_a_word_processor_does` |
| 2 | **Clicks reach the right thing:** text tool on an image-only page → status "This page is a picture of text — Recognise text" with a button, and no Add text; on a form widget → field value editor; on FreeText → its editor; correct both wrong menu paths; refusals become a short sentence plus a button | E1 E2 E3 E4(msg) E5(msg) E11 | 2–3 | Done; driven by `a_text_tool_click_reaches_what_is_under_it`. The encrypted-file button is unit-tested, not driven: its password prompt is reachable only through the OS input driver |
| 3 | **Never refuse a keystroke (interim):** when the sieve rejects a character, commit that substring re-faced via `FormatRequest::new(page, find).embedded_font(plan)` with the nearest installed face. Disclose it off-canvas. Undo is one entry. **Built first:** every refused key is named in a notice under the editor box (`canvas::textedit::refused`), with one click to the nearest face that has them all through the Properties font-change path, and the held keys typed back in once it lands. The notice is a pre-commit affordance about keys not in the document, on its own layer, so it does not mark applied content (R8b). **Still to build:** the automatic substring re-face | A1 | 2–3 | Replaced by G075(b)/G078 when they land |
| 4 | **Chrome preview:** instrument `shape()` to log which branch (count mismatch or `outlines.skipped`) fires, then fix; also the Down-arrow line preview | A13 | 1–2 | - |
| 5 | **plan.rs narrowing:** send the engine only the operators an edit touches; a cross-object edit is refused as a split | - | 1 | Done; G074 is in the pin, so a one-font line crosses objects and only a font seam narrows |
| 6 | **Selection gestures:** double-click word, triple-click line, text-tool drag selects when it starts on text (box only on blank), Shift+Up/Down extends, rotated first click via `hit_test` | B1 B2 B3 B4 B7 | 2 | - |
| 7 | **Formatting from the selection:** Ctrl+B/I/U and the Properties fields apply to the selected substring (`format_text` with `find`); superscript/subscript commands; Delete paragraph; HTML clipboard out; alignment buttons (left, centre, right, justify) **in both places**: on the ribbon beside font and size, and in the text Properties panel, both dispatching the same command. O273: inside a draft Ctrl+B/I/U act on the selected characters or on the typing style at the caret, outside one on the selected runs; bold and italic take a real face from `nearface` before a synthetic one, and disclose the synthetic; underline and strikethrough are path segments tied to the run (a G request if the engine cannot keep them tied); ribbon toggles beside Bold and Italic show pressed when the selection carries the style | C1 C2 C3 C6 D6 B8 C5 | 3 | - |
| 8 | **Find and replace:** a dialog, Replace / Replace all as one undo, with skips reported | E9 | 3–4 | Benefits from G074 (more hits succeed) |
| 9 | **IME:** `Event::Ime` preedit/commit into the draft | A12 | 1–2 | - |
| 9b | **O274, a Keyboard Shortcuts page in Settings:** every command with its chord, grouped by tab, plus a "While editing text" group; rebind by pressing, clear to disable, reset to default; a clash is shown before saving; writes the operator layer `userdata/shell.ron` through the existing merge | - | 2 | - |
| 10 | **Paragraph draft:** Enter, Backspace-join, wrap on overflow, multi-line paste, triple-click paragraph, alignment buttons with re-justify | A4 A8 A9 A15 A3 C5 C7 D3 D4 D5 | 4–6 | G074, G076, G079 |
| 11 | **Table cells:** cell draft, Up/Down between cells | D2 D7 | 2 | G080 (+ G074) |
| 12 | **Encrypted editing:** edit when `/P` permits and save incrementally; otherwise one sentence plus an Unlock button | E4 | 1 | G077 |
| 13 | **Refusal precheck:** call `edit_capability` at caret placement so a refusal never follows typing; per-cause sentences | E11 E7 | 1 | G081 (answered FIXED and in the pin) |
| 14 | **Spell check** | E10 | 3–5 | - |
| 15 | **RTL and vertical writing** | E7 E8 | unknown | ENGINE, not filed |

GUI-only steps 1–9 come to about **17–24 days** and need no engine change. Steps 10–13
are blocked on requests.

## 2a. Operator decisions

- **Alignment buttons go in both places** (Ken: *"alignment buttons should go in both places."*): on the ribbon beside font and size, and in the text Properties panel. Both dispatch the same command, so they cannot disagree. Recorded as O271 in `OPERATOR_REQUESTS.md`; built in step 7.

## 3. Engine requests and what waits on them

| Request | Topic | Unblocks |
|---|---|---|
| G074 (existing) | Finds that cross text objects | D1 D2 D3; improves step 8 |
| G075 (existing) | Subset sieve | A1 for Q, R, W-class glyphs present in the full font |
| G076 (existing) | Paragraph verb | Step 10 |
| **G077** (new) | Edit encrypted documents within `/P`; named permission refusal | Step 12 |
| **G078** (new) | Fallback face for characters the run's font cannot take | Replaces step 3's interim |
| **G079** (new) | Re-wrap keeps per-word state and kerning; composite fonts; lists; re-justify | Step 10; C5, C6 keep through reflow |
| **G080** (new) | Table cells (and columns) as blocks | Step 11 |
| **G081** (new) | Typed refusal causes, `NotFound` reasons, vertical refused, `edit_capability` | Step 13; E7 safety |

## 4. Verification drive per step

Every drive uses a scratch copy of the release exe (never the published one), with
`PDFCER_DIAG=1`, `PDFCER_DIAG_VIEWPORT=-4200,-4200,1400,900` and the scripted pointer
file. There is no OS input. The pass condition is the trace and a `shot` crop; a test
passing does not count. Fixture positions are in PDF points (`wl\pos.py`).

1. **`word.pdf`**:
   - Click para 1, line 2 (y 653.3) and type `abc`, then Ctrl+Z ×3. Pass: the draft length returns to its original and nothing commits.
   - Type `x`, then Escape. Pass: `edit-text` commits (O223).
   - Type `x`, then Ctrl+S. Pass: `edit-text` precedes `save`.
   - Paste `one\ntwo`. Pass: the result reads `one two`, with a status note.
   - Paste `Q`. Pass: a sieve verdict before any commit.
2. Click with the text tool:
   - On `synthetic-image-only`. Pass: no `text-edit-became-add`, and the status names File › Recognise.
   - On `three-text-fields` at (680, 248). Pass: the field editor opens.
   - On a FreeText fixture. Pass: the annotation editor opens.
   - On `encrypted-aes-128`. Pass: the message names the real menu path, and its button opens the dialog.
3. **`word.pdf`** body: type `QRW`. Pass: three characters are committed. Text extraction shows `QRW`. The trace shows a re-face disclosure off-canvas, and one undo removes all three.
4. **`chrome.pdf`**: type into body text. Pass: the trace shows `shaped=1`, and the draft crop matches the committed crop. Press Down, then type. Pass: `shaped=1`.
5. The REPORT.md drive for the narrowing.
6. **`word.pdf`**:
   - Double-click a word, then type `z`. Pass: the word is replaced (the length shrinks).
   - Triple-click. Pass: the selection spans the line.
   - A text-tool drag from line 2 to line 3. Pass: no new text box, and a selection exists.
   - Shift+Down. Pass: the selection grows and nothing commits.
   - On `rotated-text`: click a known glyph. Pass: the caret index matches `hit_test`.
7. **`word.pdf`**:
   - Select one word and press Ctrl+B. Pass: only that word's resource changes (extraction plus a resource dump).
   - Superscript a character. Pass: `Ts` is set on that span.
   - Delete paragraph. Pass: the block is gone, and one undo restores it.
8. Replace all `Widget` → `Gadget` on `word.pdf`. Pass: the count reported equals the hits found, minus the reported skips, and one undo restores them all.
9. Feed a scripted IME preedit/commit pair. Pass: the committed CJK string is present in the draft. The verb exists in the pointer script or is added.
10. **`word.pdf`** para 1:
    - Press Enter mid-line. Pass: two paragraphs result.
    - Type past the right margin. Pass: the text wraps and nothing passes the column edge (crop).
    - On `chrome.pdf`, repeat both. Pass: identical behaviour.
11. **`word.pdf`** table: type into `Widgets` at (90, 451.1). Pass: the glyphs stay left of the column rule, and Down lands in the cell below.
12. **`encrypted-aes-128`**: edit a run. Pass: the document stays encrypted after save and reopens with the same password.
13. A Type 3 run on `unreadable-type3`: placing the caret. Pass: the refusal arrives before any keystroke, named by cause.
14. Misspell a word. Pass: a suggestion appears in the panel, with no canvas marking beyond the cursor-class squiggle the IA permits.

## 5. Housekeeping found during the audit

- G074, G077 and G081 have verdict rows in `ENGINE_BACKLOG.md`; G075, G076, G078–G080 and G082 are listed there as filed requests.
- Wrong menu paths in two refusal messages (E1, E4) are one-line string fixes; they are included in step 2.
