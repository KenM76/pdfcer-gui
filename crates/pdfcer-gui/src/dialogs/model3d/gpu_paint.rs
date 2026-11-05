//! The paint callback: create the mesh and its programs on the first paint,
//! upload what the viewer left, and draw the frame with every pass the
//! engine's shaders ask for — the shadow, lamp, extra and see-through maps
//! before the faces, ambient occlusion, the emissive repeat, then the lines
//! and points. A fisheye frame draws the same passes once per lens face and
//! resamples each into the picture (`gpu_lens`), its lines and points then
//! bent through the lens by their own programs.
//!
//! The depth prepass, when the look asks for it, draws the opaque faces'
//! depth first so the faces' shader skips lighting what ends up hidden; the
//! picture is the same on a single-sample target and can differ by part of
//! a sample at silhouettes on a multisampled one (`pdfcer_3d::gpu_prepass`).
//!
//! The seven layered inputs are always handed over in the same order, one
//! placeholder texel standing in for a feature that is off, because the
//! mesh reuses a layered texture by position and its shaders declare every
//! sampler.

use std::sync::{Mutex, PoisonError};
use std::time::Instant;

use eframe::egui_glow;
use native_mesh::{
    Attribute, Fill, Format, Layers, Layout, Lens, LensFace, Mesh, Overlay, Passes, Picture,
    Prepass, Primitive, Repeat, ScreenEffect, Source, Table, Target, Value,
};
use pdfcer_3d::{
    GLSL_ES_300_AO_COMPOSITE_FRAGMENT, GLSL_ES_300_AO_FRAGMENT, GLSL_ES_300_FRAGMENT,
    GLSL_ES_300_LAMP_SHADOW_ALPHA_FRAGMENT, GLSL_ES_300_LAMP_SHADOW_ALPHA_VERTEX,
    GLSL_ES_300_LAMP_SHADOW_FRAGMENT, GLSL_ES_300_LAMP_SHADOW_VERTEX, GLSL_ES_300_LENS_LINE_VERTEX,
    GLSL_ES_300_LENS_POINT_VERTEX, GLSL_ES_300_LENS_RESAMPLE_FRAGMENT, GLSL_ES_300_LINE_FRAGMENT,
    GLSL_ES_300_LINE_VERTEX, GLSL_ES_300_PREPASS_FRAGMENT, GLSL_ES_300_PREPASS_VERTEX,
    GLSL_ES_300_SCREEN_VERTEX, GLSL_ES_300_SHADOW_ALPHA_FRAGMENT, GLSL_ES_300_SHADOW_ALPHA_VERTEX,
    GLSL_ES_300_SHADOW_FRAGMENT, GLSL_ES_300_SHADOW_VERTEX, GLSL_ES_300_VERTEX,
    GLSL_ES_300_WIDE_LINE_VERTEX, LENS_LINE_PIECES, LIGHT_TABLE_WIDTH,
};

use super::gpu::{RETIRED, Shared, lock};
use super::gpu_lens::LensPlan;
use super::gpu_uniforms::{Uniforms, occlusion_uniforms, unit};

/// Vertex attribute widths, matching `pdfcer_3d::VERTEX_FLOATS`: position,
/// normal, uv, flat.
const ATTRIBUTES: [i32; 4] = [3, 3, 2, 1];
const _: () = assert!(3 + 3 + 2 + 1 == pdfcer_3d::VERTEX_FLOATS);
const _: () = assert!(pdfcer_3d::LINE_VERTEX_FLOATS == 18);

/// The fill programs, in the order `prepare` adds them.
const SHADOW: usize = 0;
const LAMP: usize = 1;
const SHADOW_ALPHA: usize = 2;
const LAMP_ALPHA: usize = 3;
const PREPASS: usize = 4;
/// The overlays, in the order `prepare` adds them.
const WIDE_LINES: usize = 0;
const POINTS: usize = 1;
/// The same through a fisheye.
const LENS_LINES: usize = 2;
const LENS_POINTS: usize = 3;

/// One line vertex's attributes (`pdfcer_3d::LINE_VERTEX_FLOATS`): position,
/// colour, midpoint, two normals, kind, shaded.
const LINE_VERTEX: [Attribute; 7] = [
    Attribute {
        location: 0,
        size: 3,
        offset: 0,
    },
    Attribute {
        location: 1,
        size: 4,
        offset: 12,
    },
    Attribute {
        location: 2,
        size: 3,
        offset: 28,
    },
    Attribute {
        location: 3,
        size: 3,
        offset: 40,
    },
    Attribute {
        location: 4,
        size: 3,
        offset: 52,
    },
    Attribute {
        location: 5,
        size: 1,
        offset: 64,
    },
    Attribute {
        location: 6,
        size: 1,
        offset: 68,
    },
];
/// A segment as one instance: both ends' attributes the wide-line shader reads.
const SEGMENT: [Attribute; 10] = [
    LINE_VERTEX[0],
    LINE_VERTEX[1],
    LINE_VERTEX[2],
    LINE_VERTEX[3],
    LINE_VERTEX[4],
    LINE_VERTEX[5],
    LINE_VERTEX[6],
    Attribute {
        location: 7,
        size: 3,
        offset: 72,
    },
    Attribute {
        location: 8,
        size: 4,
        offset: 84,
    },
    Attribute {
        location: 9,
        size: 3,
        offset: 112,
    },
];

/// An owned [`Fill`].
struct FillPlan {
    layer: i32,
    program: usize,
    uniforms: Uniforms,
    source: Source,
}

/// An owned [`Layers`].
struct LayerPlan {
    sampler: &'static str,
    array: bool,
    format: Format,
    size: i32,
    layers: i32,
    fills: Vec<FillPlan>,
    redraw: bool,
}

impl LayerPlan {
    /// One cleared texel: the input for a feature that is off.
    fn placeholder(sampler: &'static str, array: bool, format: Format) -> Self {
        Self {
            sampler,
            array,
            format,
            size: 1,
            layers: 1,
            fills: Vec::new(),
            redraw: false,
        }
    }
}

/// The paint callback: create or update the GL objects and draw.
pub(super) fn draw(
    shared: &Mutex<Shared>,
    info: &egui::PaintCallbackInfo,
    painter: &egui_glow::Painter,
) {
    let gl = painter.gl();
    let mut guard = lock(shared);
    let s = &mut *guard;
    let Some(target) = ready(gl, s, info, painter) else {
        return;
    };
    let started = Instant::now();
    let size = [target.viewport[2], target.viewport[3]];
    let plans = layer_plans(s);
    let fills: Vec<Vec<Fill>> = plans.iter().map(|p| borrow_fills(&p.fills)).collect();
    let layers = layers(&plans, &fills);
    let tables = [Table {
        sampler: "u_lights",
        unit: unit("u_lights"),
        width: LIGHT_TABLE_WIDTH as i32,
        data: &s.light_table,
    }];
    let ao_uniforms = s.occlusion.map(|ao| occlusion_uniforms(&ao, size));
    let (lens_ao, lens_uniforms) = lens_owned(s, size);
    let mut line_uniforms = s.line_uniforms.clone();
    line_uniforms.push(("u_viewport", Value::Vec2(size.map(|v| v as f32))));
    let overlays = overlays(s, &line_uniforms);
    let samples = s.mesh.as_ref().map_or(0, Mesh::samples);
    let mut uniforms = s.uniforms.clone();
    uniforms.extend(super::gpu_uniforms::prepass_uniforms(s.prepass, samples));
    let repeat = s.repeat.as_ref().map(|(uniforms, e)| Repeat {
        uniforms,
        blend: [*e; 4],
    });
    let faces = s
        .lens
        .as_ref()
        .map(|l| lens_faces(l, &lens_ao, repeat))
        .unwrap_or_default();
    let passes = Passes {
        tables: &tables,
        layers: &layers,
        screen: ao_uniforms.as_deref().map(screen_effect),
        repeat,
        overlays: &overlays,
        lens: lens_uniforms.as_deref().map(|uniforms| Lens {
            faces: &faces,
            uniforms,
            colour: "u_face_colour",
            depth: "u_face_depth",
        }),
        prepass: (s.prepass && s.lens.is_none()).then(|| Prepass {
            program: PREPASS,
            uniforms: &s.uniforms,
            sampler: "u_prepass_depth",
            unit: unit("u_prepass_depth"),
        }),
    };
    let Some(mesh) = s.mesh.as_mut() else { return };
    let result = mesh.draw(gl, &uniforms, &s.draws, &passes, &target);
    finish(s, result, started);
}

/// Retire old meshes and prepare `s`; the target to draw into, or `None`
/// when the frame has failed or the last picture was presented again.
fn ready(
    gl: &egui_glow::glow::Context,
    s: &mut Shared,
    info: &egui::PaintCallbackInfo,
    painter: &egui_glow::Painter,
) -> Option<Target> {
    for mesh in RETIRED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .drain(..)
    {
        mesh.destroy(gl);
    }
    if s.failed.is_some() {
        return None;
    }
    if let Err(error) = prepare(gl, s) {
        s.failed = Some(error);
        return None;
    }
    let target = target(s, info, painter);
    let again = s.drawn
        && s.mesh
            .as_ref()
            .is_some_and(|m| m.present_again(gl, &target));
    (!again).then_some(target)
}

/// Where the picture is drawn: the callback's viewport and clip, in pixels.
fn target(s: &Shared, info: &egui::PaintCallbackInfo, painter: &egui_glow::Painter) -> Target {
    let vp = info.viewport_in_pixels();
    let clip = info.clip_rect_in_pixels();
    Target {
        viewport: [vp.left_px, vp.from_bottom_px, vp.width_px, vp.height_px],
        clip: [
            clip.left_px,
            clip.from_bottom_px,
            clip.width_px,
            clip.height_px,
        ],
        destination: painter.intermediate_fbo(),
        background: s.background,
        read_back: s.trace.is_some(),
    }
}

/// The layered textures of `plans`, each with its borrowed `fills`.
fn layers<'a>(plans: &'a [LayerPlan], fills: &'a [Vec<Fill<'a>>]) -> Vec<Layers<'a>> {
    plans
        .iter()
        .zip(fills)
        .map(|(p, f)| Layers {
            sampler: p.sampler,
            unit: unit(p.sampler),
            array: p.array,
            format: p.format,
            size: p.size,
            layers: p.layers,
            fills: f,
            redraw: p.redraw,
        })
        .collect()
}

/// Record the outcome of a draw, tracing a new picture.
fn finish(s: &mut Shared, result: Result<Option<Vec<u8>>, String>, started: Instant) {
    match result {
        Ok(rgba) => {
            s.drawn = true;
            s.directional_redraw = false;
            s.lamp_redraw = false;
            let ms = started.elapsed().as_secs_f64() * 1000.0;
            if let (Some(rgba), Some(head)) = (rgba, s.trace.take()) {
                let fields = super::picture::pixel_fields(&rgba, to_bytes(s.background));
                let prepass = u8::from(s.prepass);
                crate::diag::trace(|| {
                    format!("{head} gpu=1 gpu-prepass={prepass} ms={ms:.1}{fields}")
                }); // ui-text-exempt: diagnostic trace, never displayed
            }
        }
        Err(error) => s.failed = Some(error),
    }
}

/// Ambient occlusion's two passes with `uniforms`.
fn screen_effect<'a>(uniforms: &'a [(&'static str, Value)]) -> ScreenEffect<'a> {
    ScreenEffect {
        uniforms,
        depth: "u_depth",
        colour: "u_colour",
        first_output: "u_ao",
    }
}

/// A fisheye frame's per-face occlusion uniforms, and its resample
/// uniforms with the paint's `size` as `u_viewport`; empty and `None` for a
/// planar frame.
fn lens_owned(s: &Shared, size: [i32; 2]) -> (Vec<Option<Uniforms>>, Option<Uniforms>) {
    let Some(lens) = &s.lens else {
        return (Vec::new(), None);
    };
    let ao = lens
        .faces
        .iter()
        .map(|f| f.occlusion.map(|ao| occlusion_uniforms(&ao, f.size)))
        .collect();
    let mut uniforms = lens.uniforms.clone();
    uniforms.push(("u_viewport", Value::Vec2(size.map(|v| v as f32))));
    (ao, Some(uniforms))
}

/// The mesh's faces of `plan`, `ao` each face's occlusion uniforms, each
/// repeating the emissive pass `repeat`.
fn lens_faces<'a>(
    plan: &'a LensPlan,
    ao: &'a [Option<Uniforms>],
    repeat: Option<Repeat<'a>>,
) -> Vec<LensFace<'a>> {
    plan.faces
        .iter()
        .zip(ao)
        .map(|(f, ao)| LensFace {
            size: f.size,
            uniforms: &f.uniforms,
            screen: ao.as_deref().map(screen_effect),
            repeat,
            resample: &f.resample,
        })
        .collect()
}

/// The lines as wide instanced quads, then the points; through a fisheye,
/// each segment cut into `LENS_LINE_PIECES` bent pieces.
fn overlays<'a>(s: &Shared, uniforms: &'a [(&'static str, Value)]) -> Vec<Overlay<'a>> {
    let (lines, points, pieces) = match s.lens {
        Some(_) => (LENS_LINES, LENS_POINTS, LENS_LINE_PIECES as i32),
        None => (WIDE_LINES, POINTS, 1),
    };
    let mut out = Vec::new();
    if s.segments > 0 {
        out.push(Overlay {
            overlay: lines,
            uniforms,
            primitive: Primitive::InstancedStrip {
                vertices: 4,
                instances: s.segments * pieces,
            },
        });
    }
    if s.points > 0 {
        out.push(Overlay {
            overlay: points,
            uniforms,
            primitive: Primitive::Points { count: s.points },
        });
    }
    out
}

fn borrow_fills(plans: &[FillPlan]) -> Vec<Fill<'_>> {
    plans
        .iter()
        .map(|f| Fill {
            layer: f.layer,
            program: f.program,
            uniforms: &f.uniforms,
            source: f.source,
        })
        .collect()
}

/// The seven layered inputs, in their fixed order.
fn layer_plans(s: &Shared) -> [LayerPlan; 7] {
    let [key, key_veil, key_transmit, extra] = directional_plans(s);
    let [lamp, lamp_veil, lamp_transmit] = lamp_plans(s);
    [
        key,
        lamp,
        extra,
        key_veil,
        key_transmit,
        lamp_veil,
        lamp_transmit,
    ]
}

/// A depth pass over the opaque draws with `program`, then the translucent
/// ones with `alpha` keeping the solid range, into `layer`.
fn depth_fills(s: &Shared, layer: i32, programs: [usize; 2], view: &Uniforms) -> Vec<FillPlan> {
    let mut out = vec![FillPlan {
        layer,
        program: programs[0],
        uniforms: view.clone(),
        source: Source::Opaque,
    }];
    if let Some(keep) = s.keep {
        out.push(alpha_fill(layer, programs[1], view, keep));
    }
    out
}

/// A translucent pass with `program` keeping alphas in `keep`.
fn alpha_fill(layer: i32, program: usize, view: &Uniforms, keep: [f32; 2]) -> FillPlan {
    let mut uniforms = view.clone();
    uniforms.extend([("u_keep", Value::Vec2(keep)), ("u_texture", Value::Int(0))]);
    FillPlan {
        layer,
        program,
        uniforms,
        source: Source::Translucent,
    }
}

/// The key map, its veil and transmit arrays (layer 0 the key light's,
/// `1 + e` extra `e`'s), and the extra maps.
fn directional_plans(s: &Shared) -> [LayerPlan; 4] {
    let mut key = LayerPlan::placeholder("u_shadow_map", false, Format::Depth);
    let mut veil = LayerPlan::placeholder("u_shadow_veil", true, Format::Depth);
    let mut transmit = LayerPlan::placeholder("u_shadow_transmit", true, Format::Coverage);
    let mut extra = LayerPlan::placeholder("u_shadow_extra_map", true, Format::Depth);
    let Some(shadow) = s.shadow else {
        return [key, veil, transmit, extra];
    };
    let programs = [SHADOW, SHADOW_ALPHA];
    let view = |m: [f32; 16]| vec![("u_shadow_matrix", Value::Mat4(m))];
    let extras = s.extra.filter(|e| e.count > 0);
    let matrices: Vec<[f32; 16]> = std::iter::once(shadow.matrix)
        .chain(
            extras
                .iter()
                .flat_map(|e| (0..e.count).filter_map(|i| e.matrix(i))),
        )
        .collect();
    key.size = shadow.size as i32;
    key.redraw = s.directional_redraw;
    key.fills = depth_fills(s, 0, programs, &view(shadow.matrix));
    if let Some(e) = extras {
        extra.size = e.size as i32;
        extra.layers = e.layers() as i32;
        extra.redraw = s.directional_redraw;
        extra.fills = (1..matrices.len())
            .flat_map(|l| depth_fills(s, l as i32 - 1, programs, &view(matrices[l])))
            .collect();
    }
    if let Some(range) = s.veil {
        for plan in [&mut veil, &mut transmit] {
            plan.size = shadow.size as i32;
            plan.layers = matrices.len() as i32;
            plan.redraw = s.directional_redraw;
            plan.fills = matrices
                .iter()
                .zip(0..)
                .map(|(&m, l)| alpha_fill(l, SHADOW_ALPHA, &view(m), range))
                .collect();
        }
    }
    [key, veil, transmit, extra]
}

/// The lamp cube faces, layer `6 i + f`, and their veil and transmit arrays.
fn lamp_plans(s: &Shared) -> [LayerPlan; 3] {
    let mut map = LayerPlan::placeholder("u_lamp_shadow_map", true, Format::Depth);
    let mut veil = LayerPlan::placeholder("u_lamp_veil", true, Format::Depth);
    let mut transmit = LayerPlan::placeholder("u_lamp_transmit", true, Format::Coverage);
    let Some(lamps) = s.lamps else {
        return [map, veil, transmit];
    };
    let faces: Vec<(i32, Uniforms)> = (0..lamps.count)
        .flat_map(|i| (0..6).map(move |f| (i, f)))
        .filter_map(|(i, f)| {
            let matrix = lamps.face_matrix(i, f)?;
            let far = lamps.far.get(i).copied().unwrap_or(0.0);
            let uniforms = vec![
                ("u_lamp_matrix", Value::Mat4(matrix)),
                ("u_lamp_far", Value::Float(far)),
            ];
            Some(((6 * i + f) as i32, uniforms))
        })
        .collect();
    let shape = |plan: &mut LayerPlan| {
        plan.size = lamps.size as i32;
        plan.layers = (lamps.layers() as i32).max(1);
        plan.redraw = s.lamp_redraw;
    };
    shape(&mut map);
    map.fills = faces
        .iter()
        .flat_map(|(l, u)| depth_fills(s, *l, [LAMP, LAMP_ALPHA], u))
        .collect();
    if let Some(range) = s.veil {
        for plan in [&mut veil, &mut transmit] {
            shape(plan);
            plan.fills = faces
                .iter()
                .map(|(l, u)| alpha_fill(*l, LAMP_ALPHA, u, range))
                .collect();
        }
    }
    [map, veil, transmit]
}

/// Create the mesh and its passes' programs on the first paint, and upload
/// what the viewer left for this one.
fn prepare(gl: &egui_glow::glow::Context, s: &mut Shared) -> Result<(), String> {
    if s.mesh.is_none() {
        let mut mesh = Mesh::new(gl, GLSL_ES_300_VERTEX, GLSL_ES_300_FRAGMENT, &ATTRIBUTES)?;
        if let Err(error) = link(gl, &mut mesh) {
            mesh.destroy(gl);
            return Err(error);
        }
        s.widest_point = mesh.widest_point();
        s.mesh = Some(mesh);
    }
    let Some(mesh) = s.mesh.as_mut() else {
        return Ok(());
    };
    if let Some([lines, points]) = s.overlay_upload.take() {
        for (overlay, vertices) in [
            (WIDE_LINES, &lines),
            (POINTS, &points),
            (LENS_LINES, &lines),
            (LENS_POINTS, &points),
        ] {
            mesh.set_overlay_vertices(gl, overlay, vertices);
        }
    }
    let Some((vertices, pictures)) = s.upload.take() else {
        return Ok(());
    };
    mesh.set_vertices(gl, &vertices);
    let Some(pictures) = pictures else {
        return Ok(());
    };
    let borrowed: Vec<Picture> = pictures
        .iter()
        .map(|p| Picture {
            width: p.width,
            height: p.height,
            rgba: &p.rgba,
            wrap: p.wrap,
        })
        .collect();
    mesh.set_pictures(gl, &borrowed)
}

/// Link every program besides the main one, in the indices' order.
fn link(gl: &egui_glow::glow::Context, mesh: &mut Mesh) -> Result<(), String> {
    let fills = [
        (
            GLSL_ES_300_SHADOW_VERTEX,
            GLSL_ES_300_SHADOW_FRAGMENT,
            SHADOW,
        ),
        (
            GLSL_ES_300_LAMP_SHADOW_VERTEX,
            GLSL_ES_300_LAMP_SHADOW_FRAGMENT,
            LAMP,
        ),
        (
            GLSL_ES_300_SHADOW_ALPHA_VERTEX,
            GLSL_ES_300_SHADOW_ALPHA_FRAGMENT,
            SHADOW_ALPHA,
        ),
        (
            GLSL_ES_300_LAMP_SHADOW_ALPHA_VERTEX,
            GLSL_ES_300_LAMP_SHADOW_ALPHA_FRAGMENT,
            LAMP_ALPHA,
        ),
        (
            GLSL_ES_300_PREPASS_VERTEX,
            GLSL_ES_300_PREPASS_FRAGMENT,
            PREPASS,
        ),
    ];
    for (vertex, fragment, index) in fills {
        let added = mesh.add_program(gl, vertex, fragment)?;
        if added != index {
            return Err(format!("fill program {added} linked as {index}")); // ui-text-exempt: diagnostic trace, never displayed
        }
    }
    let segment = Layout {
        stride: 144,
        divisor: 1,
        attributes: &SEGMENT,
    };
    let point = Layout {
        stride: 72,
        divisor: 0,
        attributes: &LINE_VERTEX,
    };
    let pieces = Layout {
        divisor: LENS_LINE_PIECES,
        ..segment
    };
    let lines = [
        (GLSL_ES_300_WIDE_LINE_VERTEX, segment, WIDE_LINES),
        (GLSL_ES_300_LINE_VERTEX, point, POINTS),
        (GLSL_ES_300_LENS_LINE_VERTEX, pieces, LENS_LINES),
        (GLSL_ES_300_LENS_POINT_VERTEX, point, LENS_POINTS),
    ];
    for (vertex, layout, index) in lines {
        let added = mesh.add_overlay(gl, vertex, GLSL_ES_300_LINE_FRAGMENT, &layout)?;
        if added != index {
            return Err(format!("overlay {added} linked as {index}")); // ui-text-exempt: diagnostic trace, never displayed
        }
    }
    mesh.set_screen_programs(
        gl,
        GLSL_ES_300_SCREEN_VERTEX,
        [GLSL_ES_300_AO_FRAGMENT, GLSL_ES_300_AO_COMPOSITE_FRAGMENT],
    )?;
    mesh.set_lens_program(
        gl,
        GLSL_ES_300_SCREEN_VERTEX,
        GLSL_ES_300_LENS_RESAMPLE_FRAGMENT,
    )
}

/// `colour` 0–1 as the bytes a read-back holds.
fn to_bytes(colour: [f32; 4]) -> [u8; 4] {
    colour.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
}
