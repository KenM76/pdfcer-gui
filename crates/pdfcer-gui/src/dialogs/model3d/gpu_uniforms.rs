//! The uniforms the card's programs read, built from the engine's per-frame
//! inputs. Every key a later pass overrides (the emissive repeat) is also
//! set here each frame, so the next frame starts from the frame's own.

use native_mesh::Value;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{
    Bounds, Camera, DrawData, FrameAmbientOcclusion, FrameExtraShadows, FrameLampShadows,
    FrameLights, FrameShadow, LightingScheme, RenderError,
};

/// A program's uniforms.
pub(super) type Uniforms = Vec<(&'static str, Value)>;

/// The main program's samplers and their texture units. Sampler types may
/// not share a unit, so each takes its own even when its feature is off.
pub(super) const SAMPLERS: [(&str, i32); 10] = [
    ("u_texture", 0),
    ("u_shadow_map", 1),
    ("u_lamp_shadow_map", 2),
    ("u_shadow_extra_map", 3),
    ("u_shadow_veil", 4),
    ("u_shadow_transmit", 5),
    ("u_lamp_veil", 6),
    ("u_lamp_transmit", 7),
    ("u_lights", 8),
    ("u_prepass_depth", 9),
];

/// The texture unit of `sampler` in [`SAMPLERS`].
pub(super) fn unit(sampler: &str) -> u32 {
    SAMPLERS
        .iter()
        .find(|(name, _)| *name == sampler)
        .map_or(0, |&(_, u)| u as u32)
}

/// One draw's material finish; `u_finished` false without one.
pub(super) fn finish_uniforms(finish: Option<&pdfcer_3d::DrawFinish>) -> Uniforms {
    let Some(f) = finish else {
        return vec![("u_finished", Value::Int(0))];
    };
    vec![
        ("u_finished", Value::Int(1)),
        ("u_material_ambient", Value::Vec3(f.ambient)),
        ("u_specular", Value::Vec3(f.specular)),
        ("u_emissive", Value::Vec3(f.emissive)),
        ("u_shininess", Value::Float(f.exponent)),
    ]
}

/// The camera's matrices and eye, which the faces, lines and points share.
pub(super) fn camera_uniforms(
    camera: &Camera,
    lights: &FrameLights,
    aspect: f64,
    bounds: &Bounds,
) -> Result<Uniforms, RenderError> {
    Ok(vec![
        ("u_view", Value::Mat4(camera.view_matrix()?)),
        (
            "u_projection",
            Value::Mat4(camera.projection_matrix(aspect, bounds)?),
        ),
        (
            "u_depth_range",
            Value::Vec2(camera.depth_range(aspect, bounds)?),
        ),
        ("u_eye", Value::Vec3(lights.eye)),
        ("u_forward", Value::Vec3(lights.forward)),
        ("u_orthographic", Value::Int(i32::from(lights.orthographic))),
    ])
}

/// The main program's per-frame uniforms past the camera's: the light
/// counts (the lights themselves are the `u_lights` table) and every
/// sampler's unit.
pub(super) fn light_uniforms(lights: &FrameLights) -> Uniforms {
    let mut uniforms: Uniforms = SAMPLERS
        .iter()
        .map(|&(name, u)| (name, Value::Int(u)))
        .collect();
    uniforms.extend([
        ("u_mode", Value::Int(lights.mode.code())),
        ("u_ambient", Value::Vec3(lights.ambient)),
        ("u_count", Value::Int(lights.count as i32)),
        ("u_local_count", Value::Int(lights.local_count as i32)),
    ]);
    uniforms
}

/// The key light's shadow uniforms; the map is bound by the mesh.
pub(super) fn shadow_uniforms(shadow: Option<&FrameShadow>) -> Uniforms {
    let Some(s) = shadow else {
        return vec![("u_shadows", Value::Int(0))];
    };
    vec![
        ("u_shadows", Value::Int(1)),
        ("u_shadow_light", Value::Int(s.light)),
        ("u_shadow_dim", Value::Float(s.dim)),
        ("u_shadow_matrix", Value::Mat4(s.matrix)),
        ("u_shadow_size", Value::Float(s.size as f32)),
        ("u_shadow_to_light", Value::Vec3(s.to_light)),
        ("u_shadow_offset", Value::Float(s.normal_offset)),
        ("u_shadow_bias", Value::Float(s.bias)),
    ]
}

/// The further directional casters' uniforms.
pub(super) fn extra_uniforms(extra: Option<&FrameExtraShadows>) -> Uniforms {
    let Some(e) = extra.filter(|e| e.count > 0) else {
        return vec![("u_shadow_extra", Value::Int(0))];
    };
    vec![
        ("u_shadow_extra", Value::Int(e.count as i32)),
        ("u_shadow_extra_to_light", Value::Vec3s(e.to_light.concat())),
        ("u_shadow_extra_light", Value::Ints(e.lights.to_vec())),
        ("u_shadow_extra_colour", Value::Vec3s(e.colour.concat())),
        ("u_shadow_extra_centre", Value::Vec3(e.centre)),
        ("u_shadow_extra_reach", Value::Float(e.reach)),
    ]
}

/// The lamps' shadow uniforms.
pub(super) fn lamp_uniforms(lamps: Option<&FrameLampShadows>) -> Uniforms {
    let Some(l) = lamps else {
        return vec![("u_lamp_shadows", Value::Int(0))];
    };
    vec![
        ("u_lamp_shadows", Value::Int(1)),
        ("u_lamp_shadow_size", Value::Float(l.size as f32)),
        ("u_lamp_far", Value::Floats(l.far.to_vec())),
    ]
}

/// Whether the see-through layers are read for the directional maps and
/// for the lamps.
pub(super) fn veil_uniforms(directional: bool, lamps: bool) -> Uniforms {
    vec![
        ("u_shadow_veiled", Value::Int(i32::from(directional))),
        ("u_lamp_veiled", Value::Int(i32::from(lamps))),
    ]
}

/// Whether the main pass skips fragments the depth prepass shows hidden,
/// and how far round each pixel it looks: one pixel on a multisampled
/// target, whose edge samples the single-sample prepass cannot see.
pub(super) fn prepass_uniforms(on: bool, samples: i32) -> Uniforms {
    vec![
        ("u_prepass", Value::Int(i32::from(on))),
        ("u_prepass_spread", Value::Int(i32::from(samples > 0))),
    ]
}

/// The emissive repeat's uniforms: the faces unlit and unshadowed, over the
/// frame's own, which reset every key here the next frame.
pub(super) fn repeat_uniforms(
    camera: &Camera,
    options: &ModelRenderOptions,
) -> Result<Uniforms, RenderError> {
    let unlit = FrameLights::new(
        camera,
        &ModelRenderOptions {
            lighting: LightingScheme::None,
            ..*options
        },
    )?;
    Ok(vec![
        ("u_mode", Value::Int(unlit.mode.code())),
        ("u_ambient", Value::Vec3(unlit.ambient)),
        ("u_count", Value::Int(0)),
        ("u_local_count", Value::Int(0)),
        ("u_shadows", Value::Int(0)),
        ("u_lamp_shadows", Value::Int(0)),
        ("u_shadow_extra", Value::Int(0)),
        ("u_finished", Value::Int(0)),
    ])
}

/// The line and point programs' uniforms past the camera's; `u_viewport`
/// is the paint's own.
pub(super) fn line_uniforms(data: &DrawData) -> Uniforms {
    vec![
        ("u_bias", Value::Float(data.line_bias)),
        ("u_crease_cos", Value::Float(data.crease_cos)),
        ("u_line_width", Value::Float(data.line_width)),
        ("u_point_size", Value::Float(data.point_size)),
    ]
}

/// Both ambient occlusion passes' uniforms, for a `size` viewport: the
/// paint's own, which can differ by a pixel from the size the viewer asked.
pub(super) fn occlusion_uniforms(ao: &FrameAmbientOcclusion, size: [i32; 2]) -> Uniforms {
    vec![
        ("u_ao_size", Value::IVec2(size)),
        ("u_ao_orthographic", Value::Int(i32::from(ao.orthographic))),
        ("u_ao_depth_range", Value::Vec2(ao.depth_range)),
        ("u_ao_unproject", Value::Vec2(ao.unproject)),
        ("u_ao_pixels", Value::Float(ao.pixels)),
        ("u_ao_radius", Value::Float(ao.radius)),
    ]
}
