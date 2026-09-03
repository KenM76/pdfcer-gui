import io


def patch(path, pairs):
    s = io.open(path, encoding='utf-8').read()
    for old, new in pairs:
        if old not in s:
            raise SystemExit('MISS in %s: %r' % (path, old[:110]))
        s = s.replace(old, new, 1)
    io.open(path, 'w', encoding='utf-8').write(s)
    print('patched', path)


patch('crates/pdfcer-gui/src/dialogs/unsaved.rs', [
    # --- the outcome ------------------------------------------------------
    ('''pub enum Outcome {
    /// Write a copy first; resume only if a file was actually written.
    SaveCopy,''',
     '''pub enum Outcome {
    /// ★★★ **Write the file the operator opened, then resume** —
    /// `OPERATOR_REQUESTS.md` O65.
    ///
    /// Offered only when the document has a file to be written over. Resumes
    /// on a successful write and on nothing else, exactly as [`Self::SaveCopy`]
    /// does and for the identical reason: a save that did not happen must
    /// never be a route to discarding the work it was supposed to preserve.
    SaveInPlace,
    /// Write a copy first; resume only if a file was actually written.
    SaveCopy,'''),
    # --- the dialog carries whether a Save is possible --------------------
    ('''    /// Set by a button, drained by the owner.
    outcome: Option<Outcome>,''',
     '''    /// ★ **Whether this document has a file to save over**, captured at open
    /// time from `app::save::has_a_file`.
    ///
    /// Decides whether the Save button is drawn at all. R9: a never-saved
    /// document renders **nothing** rather than a greyed Save, because "this
    /// has never been written anywhere" is a standing property of the
    /// document, not a temporary condition a hover sentence could resolve —
    /// and the operator already has the control that fixes it, one button to
    /// the right.
    has_file: bool,
    /// Set by a button, drained by the owner.
    outcome: Option<Outcome>,'''),
    ('''    pub fn new(intent: PendingIntent, edits: u64) -> Self {
        Self {
            intent,
            edits,
            outcome: None,
            cancelled: false,
        }
    }''',
     '''    pub fn new(intent: PendingIntent, edits: u64, has_file: bool) -> Self {
        Self {
            intent,
            edits,
            has_file,
            outcome: None,
            cancelled: false,
        }
    }'''),
    # --- the buttons ------------------------------------------------------
    ('''        ui.horizontal(|ui| {
            let save = ui.button(t::save_copy_button());
            crate::diag::ui_rect(REGION_SAVE, save.rect);
            if save.clicked() {
                self.outcome = Some(Outcome::SaveCopy);
            }''',
     '''        ui.horizontal(|ui| {
            // ★★★ **Save, when there is a file to save over** — O65.
            //
            // First, because it is the answer that loses nothing AND changes
            // nothing about where the operator's work lives. The order of this
            // row runs from least to most destructive and this is the new
            // least.
            //
            // Absent, not greyed, on a never-saved document (R9). Its own
            // region is published either way a reader might expect — no: the
            // region is published only when the control is drawn, so a driven
            // check can tell "the build has no Save button" from "the Save
            // button is off screen", which two adjacent recorded findings say
            // is otherwise the same screenshot.
            if self.has_file {
                let save_now = ui.button(t::save_button());
                crate::diag::ui_rect(REGION_SAVE_IN_PLACE, save_now.rect);
                if save_now.clicked() {
                    self.outcome = Some(Outcome::SaveInPlace);
                }
            }
            let save = ui.button(t::save_copy_button());
            crate::diag::ui_rect(REGION_SAVE, save.rect);
            if save.clicked() {
                self.outcome = Some(Outcome::SaveCopy);
            }'''),
])

# --- the region constant ---------------------------------------------------
s = io.open('crates/pdfcer-gui/src/dialogs/unsaved.rs', encoding='utf-8').read()
import re
m = re.search(r'^(const REGION_SAVE: &str = .*)$', s, re.M)
assert m, 'REGION_SAVE not found'
s = s.replace(m.group(1), m.group(1) + '''

/// The Save-over-the-open-file button's region — `OPERATOR_REQUESTS.md` O65.
///
/// Its own name rather than sharing [`REGION_SAVE`], because the two buttons
/// mean different things to the file on disk and a check that could not tell
/// them apart would pass on a build that had silently swapped one for the
/// other.
const REGION_SAVE_IN_PLACE: &str = "dialog.unsaved.save-in-place"; // ui-text-exempt: trace region name, never displayed''', 1)
io.open('crates/pdfcer-gui/src/dialogs/unsaved.rs', 'w', encoding='utf-8').write(s)
print('region added')
