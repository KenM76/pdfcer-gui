//! Overlays: programs with their own vertices and attribute layout, drawn
//! over the finished faces into the same multisampled buffer.

use glow::HasContext as _;

use crate::program::Program;
use crate::{Layout, Overlay, Primitive};

/// An overlay's program, vertex array and buffer.
pub(crate) struct OverlayProgram {
    program: Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
}

impl OverlayProgram {
    /// Link `vertex` and `fragment` and lay out `layout` over a new buffer.
    pub(crate) fn new(
        gl: &glow::Context,
        vertex: &str,
        fragment: &str,
        layout: &Layout<'_>,
    ) -> Result<Self, String> {
        let program = Program::new(gl, vertex, fragment)?;
        // SAFETY: objects created here are kept in the result or deleted.
        unsafe {
            let (vao, vbo) = match (gl.create_vertex_array(), gl.create_buffer()) {
                (Ok(a), Ok(b)) => (a, b),
                (a, b) => {
                    if let Ok(a) = a {
                        gl.delete_vertex_array(a);
                    }
                    if let Ok(b) = b {
                        gl.delete_buffer(b);
                    }
                    program.delete(gl);
                    return Err("overlay buffers could not be created".to_owned());
                }
            };
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            for a in layout.attributes {
                gl.enable_vertex_attrib_array(a.location);
                gl.vertex_attrib_pointer_f32(
                    a.location,
                    a.size,
                    glow::FLOAT,
                    false,
                    layout.stride,
                    a.offset,
                );
                gl.vertex_attrib_divisor(a.location, layout.divisor);
            }
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            Ok(Self { program, vao, vbo })
        }
    }

    /// Replace the vertices.
    pub(crate) fn set_vertices(&self, gl: &glow::Context, vertices: &[f32]) {
        let bytes: Vec<u8> = vertices.iter().flat_map(|f| f.to_ne_bytes()).collect();
        // SAFETY: the buffer is this overlay's own; the slice outlives the call.
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::STATIC_DRAW);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }

    /// Draw `overlay` into the bound framebuffer, whose state the caller set.
    pub(crate) fn draw(&mut self, gl: &glow::Context, overlay: &Overlay<'_>) {
        self.program.use_with(gl, overlay.uniforms);
        // SAFETY: the vertex array is this overlay's own; GL raises an error
        // rather than read past the buffer.
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            match overlay.primitive {
                Primitive::InstancedStrip {
                    vertices,
                    instances,
                } if instances > 0 => {
                    gl.draw_arrays_instanced(glow::TRIANGLE_STRIP, 0, vertices, instances);
                }
                Primitive::Points { count } if count > 0 => {
                    gl.draw_arrays(glow::POINTS, 0, count);
                }
                _ => {}
            }
        }
    }

    /// Delete every object.
    pub(crate) fn delete(self, gl: &glow::Context) {
        // SAFETY: the objects are this value's own.
        unsafe {
            gl.delete_buffer(self.vbo);
            gl.delete_vertex_array(self.vao);
        }
        self.program.delete(gl);
    }
}

/// Set the overlays' state and draw them into the bound framebuffer. Turns
/// on shader-sized points where the context is desktop GL, which needs it.
pub(crate) fn draw_all(
    gl: &glow::Context,
    programs: &mut [OverlayProgram],
    overlays: &[Overlay<'_>],
) {
    if overlays.is_empty() {
        return;
    }
    let desktop = !gl.version().is_embedded;
    // SAFETY: state changes on the caller's current context.
    unsafe {
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LEQUAL);
        gl.depth_mask(true);
        gl.enable(glow::BLEND);
        gl.blend_equation(glow::FUNC_ADD);
        gl.blend_func_separate(
            glow::SRC_ALPHA,
            glow::ONE_MINUS_SRC_ALPHA,
            glow::ONE,
            glow::ONE_MINUS_SRC_ALPHA,
        );
        if desktop {
            gl.enable(glow::PROGRAM_POINT_SIZE);
        }
    }
    for o in overlays {
        if let Some(p) = programs.get_mut(o.overlay) {
            p.draw(gl, o);
        }
    }
    // SAFETY: state changes on the caller's current context.
    unsafe {
        if desktop {
            gl.disable(glow::PROGRAM_POINT_SIZE);
        }
        gl.disable(glow::BLEND);
        gl.bind_vertex_array(None);
    }
}

/// The widest point the context draws, in pixels.
pub(crate) fn widest_point(gl: &glow::Context) -> f32 {
    let mut range = [1.0_f32; 2];
    // SAFETY: a query into a two-float slice.
    unsafe { gl.get_parameter_f32_slice(glow::ALIASED_POINT_SIZE_RANGE, &mut range) };
    let [_, widest] = range;
    widest
}
