import io

# ---------------------------------------------------------------- the trait
p = 'crates/pdfcer-gui/src/canvas/target.rs'
s = io.open(p, encoding='utf-8').read()

anchor = "    fn hit_test_rect(&self, page_index: usize, rect: Rect) -> Vec<TargetId>;"
assert anchor in s
add = '''    /// **Is this container worth selecting instead of what is inside it?**
    ///
    /// # ★★★ The question the Smart-Selector forgot to ask
    ///
    /// `canvas::smart::Scope::resolve` maps a leaf to its containing form so
    /// that a first click selects the container and a double-click descends —
    /// `OPERATOR_REQUESTS.md` O70, and the right model for a title block or a
    /// stamp.
    ///
    /// It is the **wrong** model for the commonest form in the world.
    /// Every CAD exporter this project has seen wraps a drawing's whole visible
    /// body in one page-sized form XObject, and a `/BBox` is a clipping extent
    /// (§8.10.1) rather than a claim about ink — so that wrapper contains
    /// everything, wins every click, and "select the container first" becomes
    /// "select the whole drawing, every time".
    ///
    /// ⇒ Which is the operator's **headline complaint**, verbatim, restored by
    /// the feature built to improve selection:
    ///
    /// > *"There are obviously more than one item on the page, but when I click
    /// > on one of the objects all I get is the page selected."*
    ///
    /// ★★ So a container is worth resolving to only when selecting it says
    /// something selecting the leaf does not. A container that holds
    /// **everything on the page** says nothing: it IS the page, under another
    /// name.
    ///
    /// # What it does NOT change
    ///
    /// **Entering** such a form still works, and must. A double-click descends
    /// into it, the Objects panel lists it, and the canvas menu's *"select the
    /// containing form"* reaches it. Reachable on purpose was always the
    /// design; winning by default is what was wrong, both times.
    ///
    /// Defaults to `true` — a provider that cannot measure says yes, which is
    /// the behaviour before this existed.
    fn container_is_worth_selecting(&self, page_index: usize, container: TargetId) -> bool {
        let _ = (page_index, container);
        true
    }

'''
s = s.replace(anchor, add + anchor, 1)
io.open(p, 'w', encoding='utf-8').write(s)

# ---------------------------------------------------------------- the impl
p = 'crates/pdfcer-gui/src/panels/objects/provider/mod.rs'
s = io.open(p, encoding='utf-8').read()
anchor2 = "    fn hit_test_rect(&self, page_index: usize, rect: Rect) -> Vec<TargetId> {"
assert anchor2 in s
impl = '''    /// Measured, not assumed. See the trait for why the question exists.
    ///
    /// # ★★ The measurement, and why it is not "is it page-sized"
    ///
    /// The obvious predicate — compare the form's `/BBox` with the page's media
    /// box — needs the page rect, which this provider does not hold, and it
    /// answers the wrong question anyway. A form can be *smaller* than the page
    /// and still contain every mark on it, which is common: a CAD exporter that
    /// wraps the drawing body but not the margin produces exactly that, and
    /// selecting that wrapper is just as useless.
    ///
    /// So the comparison is against **what is actually on the sheet**: the union
    /// of every page object's bounds. If the container covers essentially all of
    /// it, the container is the page's content and selecting it tells the
    /// operator nothing they did not already know.
    ///
    /// ★ `COVERS_EVERYTHING` is deliberately generous. The failure this guards
    /// against is severe and constant — every click on a CAD drawing — and the
    /// cost of being slightly too generous is that one unusually large title
    /// block stops being offered as a container on the first click, while still
    /// being reachable by double-click, by the Objects panel and by the canvas
    /// menu. That is a mild inconvenience against a headline defect.
    fn container_is_worth_selecting(&self, page_index: usize, container: TargetId) -> bool {
        if page_index != self.page_index {
            return true;
        }
        let Some(bounds) = self.bounds(page_index, container) else {
            return true;
        };
        let mut union: Option<Rect> = None;
        for i in 0..self.objects.objects.len() {
            if let Some(r) = self.bounds(page_index, TargetId::Object(i as u64)) {
                union = Some(match union {
                    Some(u) => u.union(r),
                    None => r,
                });
            }
        }
        let Some(union) = union else { return true };
        // A degenerate union cannot be divided into; say yes rather than
        // producing an infinity and a surprising answer.
        if union.width() <= f32::EPSILON || union.height() <= f32::EPSILON {
            return true;
        }
        let covers = (bounds.width() / union.width()).min(1.0)
            * (bounds.height() / union.height()).min(1.0);
        covers < COVERS_EVERYTHING
    }

'''
s = s.replace(anchor2, impl + anchor2, 1)

const = '''/// **How much of a page's content a container may cover and still be worth
/// selecting**, as a fraction of the union of every page object's bounds.
///
/// ★ 0.9 — a container over nine tenths of everything on the sheet is the
/// sheet. See `CanvasTargetProvider::container_is_worth_selecting` for the
/// defect this number exists to prevent and for why it errs generous.
const COVERS_EVERYTHING: f32 = 0.9;

'''
marker = "pub struct ObjectModelProvider {"
i = s.index(marker)
j = s.rindex('\n', 0, i)
while s[j - 1] == '/' or s[j - 3:j] == '///':
    j = s.rindex('\n', 0, j)
s = s[:j + 1] + const + s[j + 1:]
io.open(p, 'w', encoding='utf-8').write(s)

# ---------------------------------------------------------------- resolve
p = 'crates/pdfcer-gui/src/canvas/smart.rs'
s = io.open(p, encoding='utf-8').read()
old = """        if self
            .entered
            .is_some_and(|f| TargetId::Object(f) == container)
        {
            return target;
        }
        container
    }"""
new = """        if self
            .entered
            .is_some_and(|f| TargetId::Object(f) == container)
        {
            return target;
        }
        // ★★★ **A CONTAINER THAT HOLDS EVERYTHING IS THE PAGE** — 2026-09-01,
        // and this guard is the repair of a defect this module CAUSED.
        //
        // Resolving a leaf to its container is right for a title block and
        // wrong for the commonest form in the world: every CAD exporter wraps
        // the drawing's whole body in one page-sized form, so "select the
        // container first" became "select the whole drawing, every time" — the
        // operator's own headline complaint, restored by the feature built to
        // improve selection:
        //
        //   "There are obviously more than one item on the page, but when I
        //    click on one of the objects all I get is the page selected."
        //
        // ★★ It shipped on 2026-08-31 and was found on 2026-09-01 by a driven
        // check that had SKIPPED — on a stale binary — through both sweeps in
        // between. `a_click_inside_a_form_selects_what_is_drawn_there` is that
        // check, and it is the reason this took a day rather than a week.
        //
        // ★ Entering such a form is untouched. A double-click descends, the
        // Objects panel lists it, the canvas menu reaches it. Reachable on
        // purpose was always the design; winning by DEFAULT is what was wrong,
        // both times.
        if !targets.container_is_worth_selecting(page, container) {
            return target;
        }
        container
    }"""
assert old in s
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8').write(s)
print('ok')
