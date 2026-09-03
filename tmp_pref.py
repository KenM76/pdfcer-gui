import io

# ---------------------------------------------------------------- 1. the pref
p = r"crates\pdfcer-gui\src\app\prefs\mod.rs"
s = io.open(p, encoding="utf-8").read()

anchor = "    pub wheel_paging: WheelPaging,"
add = '''    pub wheel_paging: WheelPaging,
    /// **Which chord means which paste, for a form field** —
    /// `OPERATOR_REQUESTS.md` **O58**, operator ruling 2026-08-29.
    ///
    /// See [`PasteChords`]. Read when the shell's keymap is assembled and on
    /// every change to it, never per keystroke: it does not decide what a
    /// command *does*, it decides which key *reaches* it.
    pub paste_chords: PasteChords,'''
assert anchor in s
s = s.replace(anchor, add, 1)

# the enum itself, next to WheelPaging's definition
we = s.index("pub enum WheelPaging")
# back up to the doc comment start of that item
start = s.rindex("/// ", 0, we)
while s.rindex("\n///", 0, start) == start - 1:
    break
enum = '''/// **Which chord pastes a form field as a NEW field, and which as a DUPLICATE.**
///
/// `OPERATOR_REQUESTS.md` **O58**. Ken, 2026-08-29: *"let's make it an option to
/// have it swap to match Acrobat or work the way we have it now."*
///
/// # ★★★ Why this exists at all — the two answers are genuinely different
///
/// Copying a form field has two legitimate meanings: **a new, independent
/// field**, and **another box for the same field** that fills in step with the
/// original. pdfcer offers both on two chords rather than asking in a dialog.
///
/// **Acrobat assigns them the other way round.** Its plain Copy/Paste is the
/// *linked* one — paste a field, leave its name alone, and you get a second
/// widget of the same field — and it has **no paste-as-independent chord at
/// all**; independence comes only from its bulk commands (Place Multiple
/// Fields, Create Multiple Copies), which auto-name and so produce separate
/// fields. Sourced from `Acrobat_Features/forms__field_copy_paste_and_duplication.md`.
///
/// Neither order is obviously right, which is precisely why it is a setting:
///
/// - **[`PdfcerOrder`](Self::PdfcerOrder)** puts the *common intent* on the
///   *common chord*. Copying a title-block field down a column almost always
///   wants independent fields, and Acrobat's own linking default is a
///   documented, unresolved point of user friction — there is a standing
///   Acrobat request for an *"option to unlink form fields when copying"*, and
///   the only workaround Acrobat offers is renaming each duplicate by hand.
/// - **[`AcrobatOrder`](Self::AcrobatOrder)** matches muscle memory. An
///   operator who spends all day in Acrobat and reaches for `Ctrl+V` expecting
///   a linked field gets one, and does not have to hold a second rule in their
///   head for one program.
///
/// # ★★ It swaps the CHORDS, never what a command means
///
/// The two commands keep their meanings and their labels for ever:
/// `edit.paste` is always *"paste as a new field"* and `edit.paste_duplicate`
/// is always *"paste as another box for the same field"*. Only the keys move.
///
/// The alternative — swapping what the *commands* do — was rejected because it
/// makes the labels lie: a ribbon button reading **Paste as duplicate** would
/// paste a new field, and no tooltip can rescue a control whose name is wrong.
/// Moving the binding instead means the ribbon, the context menu, the shortcuts
/// dialog and the keyboard all agree by construction, because every one of them
/// reads the same keymap.
///
/// ⇒ It is applied by [`super::super::shell::manifest::apply_paste_chords`],
/// which rewrites two entries of the shell's keymap. That is the mechanism the
/// framework already has for exactly this: `SHELL_FRAMEWORK.md`'s whole claim is
/// that the keymap is a **manifest**, not code, so an operator preference about
/// keys is a data edit rather than a branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum PasteChords {
    /// `Ctrl+V` pastes a **new** field; `Ctrl+Shift+V` pastes a **duplicate**.
    ///
    /// The default, and the operator's original ruling.
    #[default]
    PdfcerOrder,
    /// `Ctrl+V` pastes a **duplicate**; `Ctrl+Shift+V` pastes a **new** field.
    ///
    /// Acrobat's own assignment.
    AcrobatOrder,
}

impl PasteChords {
    /// Both, in the order the settings pane offers them.
    pub const ALL: &'static [Self] = &[Self::PdfcerOrder, Self::AcrobatOrder];

    /// The chord that reaches `edit.paste` (a NEW field) under this order.
    #[must_use]
    pub const fn new_field_chord(self) -> &'static str {
        match self {
            // ui-text-exempt: keymap chord spellings, matched against the manifest.
            Self::PdfcerOrder => "Ctrl+V",
            Self::AcrobatOrder => "Ctrl+Shift+V",
        }
    }

    /// The chord that reaches `edit.paste_duplicate` under this order.
    #[must_use]
    pub const fn duplicate_chord(self) -> &'static str {
        match self {
            // ui-text-exempt: keymap chord spellings, matched against the manifest.
            Self::PdfcerOrder => "Ctrl+Shift+V",
            Self::AcrobatOrder => "Ctrl+V",
        }
    }
}

'''
s = s[:start] + enum + s[start:]
io.open(p, "w", encoding="utf-8", newline="\n").write(s)
print("prefs ok")
