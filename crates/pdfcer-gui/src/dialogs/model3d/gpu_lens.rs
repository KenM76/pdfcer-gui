//! A fisheye frame on the card, the engine's recipe (`pdfcer_3d::FrameLens`):
//! each lens face is drawn as a perspective frame from its own camera, lit
//! and shadowed from the fisheye camera so camera-fixed lights and the
//! caster do not turn with the face, with ambient occlusion of its own; the
//! faces are resampled into the picture, then the lines and points are
//! bent through the lens straight into it. `gpu_paint` hands this to the
//! mesh as `native_mesh::Lens`.

use native_mesh::Value;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{Bounds, Camera, FrameAmbientOcclusion, FrameLens, FrameLights, RenderError};

use super::gpu_uniforms::{self as u, Uniforms};

/// One face of a lens frame, for the paint.
#[derive(Clone)]
pub(super) struct FacePlan {
    /// The face's size, pixels.
    pub(super) size: [i32; 2],
    /// The face camera's view, projection and depth range, the fisheye's
    /// eye and forward, and the prepass off: over the frame's uniforms.
    pub(super) uniforms: Uniforms,
    /// The face's ambient occlusion, when drawn.
    pub(super) occlusion: Option<FrameAmbientOcclusion>,
    /// The resample pass's uniforms for this face.
    pub(super) resample: Uniforms,
}

/// A lens frame, for the paint.
#[derive(Clone)]
pub(super) struct LensPlan {
    pub(super) faces: Vec<FacePlan>,
    /// The resample pass's uniforms every face shares but `u_viewport`.
    pub(super) uniforms: Uniforms,
}

/// The lens frame of fisheye `camera` drawn `size`, lit by `lights` (built
/// from `camera`), over `background`.
pub(super) fn plan(
    camera: &Camera,
    lights: &FrameLights,
    options: &ModelRenderOptions,
    bounds: &Bounds,
    (size, background): ([u32; 2], [f32; 4]),
) -> Result<(LensPlan, FrameLens), RenderError> {
    let lens = FrameLens::new(camera, size[0], size[1])?;
    let faces = lens
        .faces
        .iter()
        .map(|f| face(f, lights, options, bounds))
        .collect::<Result<Vec<_>, _>>()?;
    let uniforms = vec![
        (
            "u_depth_range",
            Value::Vec2(camera.depth_range(1.0, bounds)?),
        ),
        ("u_lens_mapping", Value::Int(lens.mapping)),
        ("u_lens_focal", Value::Float(lens.focal)),
        ("u_background", Value::Vec4(background)),
    ];
    Ok((LensPlan { faces, uniforms }, lens))
}

fn face(
    f: &pdfcer_3d::LensFace,
    lights: &FrameLights,
    options: &ModelRenderOptions,
    bounds: &Bounds,
) -> Result<FacePlan, RenderError> {
    let size = [f.width, f.height];
    let aspect = f64::from(f.width) / f64::from(f.height.max(1));
    let mut uniforms = u::camera_uniforms(&f.camera, lights, aspect, bounds)?;
    uniforms.extend(u::prepass_uniforms(false, 0));
    let [right, up, forward] = f.axes;
    let resample = vec![
        ("u_face", Value::Int(f.face as i32)),
        ("u_face_right", Value::Vec3(right)),
        ("u_face_up", Value::Vec3(up)),
        ("u_face_forward", Value::Vec3(forward)),
        ("u_face_scale", Value::Vec2(f.scale)),
        ("u_face_size", Value::Vec2(size.map(|v| v as f32))),
        (
            "u_face_depth_range",
            Value::Vec2(f.camera.depth_range(aspect, bounds)?),
        ),
    ];
    Ok(FacePlan {
        size: size.map(|v| v as i32),
        uniforms,
        occlusion: FrameAmbientOcclusion::new(&f.camera, options, bounds, size)?,
        resample,
    })
}

/// The lens line and point programs' uniforms but `u_viewport`: the
/// fisheye's eye and basis, the lens, its depth range, and `lines`, the
/// draws' own.
pub(super) fn line_uniforms(
    camera: &Camera,
    lens: &FrameLens,
    bounds: &Bounds,
    lines: &Uniforms,
) -> Result<Uniforms, RenderError> {
    let [right, up, forward] = lens.basis;
    let mut out = vec![
        ("u_eye", Value::Vec3(lens.eye)),
        ("u_right", Value::Vec3(right)),
        ("u_up", Value::Vec3(up)),
        ("u_forward", Value::Vec3(forward)),
        ("u_lens_mapping", Value::Int(lens.mapping)),
        ("u_lens_focal", Value::Float(lens.focal)),
        ("u_lens_near", Value::Float(lens.near)),
        (
            "u_depth_range",
            Value::Vec2(camera.depth_range(1.0, bounds)?),
        ),
        ("u_orthographic", Value::Int(0)),
    ];
    out.extend(lines.iter().cloned());
    Ok(out)
}
