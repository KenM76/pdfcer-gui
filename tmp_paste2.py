import io

p = 'crates/pdfcer-gui/src/canvas/clipboard.rs'
s = io.open(p, encoding='utf-8').read()

ANCHOR_DOC = '''        /// ★★★ **The point that is placed under the cursor on a paste** —
        /// the clip's centre, in **PDF user space**.
        ///
        /// `OPERATOR_REQUESTS.md` O73: *"when I paste it should paste where
        /// the mouse cursor is sitting."*
        ///
        /// # Why it is captured at COPY time and not derived at paste time
        ///
        /// Because at paste time the source may be gone. The clip outlives the
        /// selection it came from, outlives an undo of the cut that produced
        /// it, and — for a cut — outlives the objects themselves. Deriving the
        /// centre from the document on paste would work in the common case and
        /// fail in exactly the case `Ctrl+X` `Ctrl+V` exists for.
        ///
        /// # Why a CENTRE
        ///
        /// The operator pointing at a spot means *"put it here"*, not *"begin
        /// its bounding box here"*. That is Inkscape's rule and Illustrator's;
        /// top-left is the Word/Explorer convention and belongs to a text
        /// caret rather than to a drawing canvas. Acrobat drops a pasted
        /// comment centred on the click too.
        ///
        /// ★ It is also what preserves relative geometry inside a
        /// multi-object paste **by construction**: one anchor for the whole
        /// clip means one delta, applied to everything, so the arrangement
        /// cannot drift no matter how many items are in it.
        anchor: (f64, f64),'''

# --- Clipped::Markup gains the anchor -------------------------------------
OLD_M = '''    Markup {
        /// The spec, verbatim from `spec_from_dict`.
        spec: Box<MarkupSpec>,
        /// The 0-based page it was copied from.
        page: usize,'''
NEW_M = '''    Markup {
        /// The spec, verbatim from `spec_from_dict`.
        spec: Box<MarkupSpec>,
        /// The 0-based page it was copied from.
        page: usize,
''' + ANCHOR_DOC
assert OLD_M in s
s = s.replace(OLD_M, NEW_M, 1)

# --- Clipped::Content gains the anchor ------------------------------------
OLD_C = '''        count: usize,'''
assert s.count(OLD_C) == 1, s.count(OLD_C)
s = s.replace(OLD_C, OLD_C + '\n' + ANCHOR_DOC, 1)

io.open(p, 'w', encoding='utf-8').write(s)
print('variants patched')
