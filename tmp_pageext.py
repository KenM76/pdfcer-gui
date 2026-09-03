import io

p = 'crates/pdfcer-gui/src/panels/objects/provider/mod.rs'
s = io.open(p, encoding='utf-8').read()

# ---- field -----------------------------------------------------------------
old = """    page_index: usize,
    objects: PageObjects,
    to_canvas: Transform,
    to_pdf: Option<Transform>,"""
new = """    page_index: usize,
    objects: PageObjects,
    to_canvas: Transform,
    to_pdf: Option<Transform>,
    /// **The page's own extent in canvas units**, or `None` when this provider
    /// was built from parts and nobody supplied one.
    ///
    /// ★★★ Held for exactly one question:
    /// [`crate::canvas::target::CanvasTargetProvider::container_is_worth_selecting`],
    /// which needs to know whether a form covers the whole sheet. It is
    /// `page_device_geometry(page, 1.0)`'s first two returns, which were
    /// discarded here until 2026-09-01 — the transform was wanted and the size
    /// was not.
    ///
    /// ★ `None` makes that predicate answer `true`, which is the behaviour
    /// before it existed. A provider that cannot measure must not guess.
    page_extent: Option<egui::Vec2>,"""
assert old in s
s = s.replace(old, new, 1)

# ---- build -----------------------------------------------------------------
old = """        let (_, _, to_canvas) = page_device_geometry(page, 1.0);
        Ok(Self {
            page_index,
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
        })"""
new = """        let (w, h, to_canvas) = page_device_geometry(page, 1.0);
        Ok(Self {
            page_index,
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
            // ★ At scale 1.0 these ARE the page's canvas-space extent, which is
            // the space `bounds` answers in. Taking them here rather than
            // re-deriving from the crop box keeps one source for the geometry.
            page_extent: Some(egui::vec2(w as f32, h as f32)),
        })"""
assert old in s
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8').write(s)
print('build patched')
