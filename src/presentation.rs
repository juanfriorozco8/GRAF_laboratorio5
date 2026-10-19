use raylib::prelude::Image;

use crate::{
    HEIGHT, Uniforms, WIDTH,
    framebuffer::Framebuffer,
    primitive::primitive_assembly,
    rasterizer::rasterize,
    shaders::{fragment_shader, vertex_shader},
    triangle::TriangleMode,
    vertex::Vertex,
};

pub const EXPORT_WIDTH: i32 = 2400;
pub const EXPORT_HEIGHT: i32 = 1800;
pub const SAMPLES: i32 = 2;

pub fn sampled_triangles(
    vertices: &[Vertex],
    uniforms: &Uniforms,
    width: i32,
    height: i32,
    samples: i32,
) -> Vec<[Vertex; 3]> {
    assert!(width > 0 && height > 0 && (1..=4).contains(&samples));
    let sample_width = width.checked_mul(samples).expect("Ancho excesivo");
    let sample_height = height.checked_mul(samples).expect("Alto excesivo");
    let scale = (sample_width as f32 / WIDTH as f32).min(sample_height as f32 / HEIGHT as f32);
    let offset_x = (sample_width as f32 - WIDTH as f32 * scale) * 0.5;
    let offset_y = (sample_height as f32 - HEIGHT as f32 * scale) * 0.5;

    // Procesa primero la matriz del curso y luego escala sus coordenadas de
    // pantalla a la cuadricula de muestras. No cambia la forma ni la camara.
    let transformed: Vec<Vertex> = vertices
        .iter()
        .map(|vertex| {
            let mut vertex = vertex_shader(vertex, uniforms);
            vertex.transformed_position.x = vertex.transformed_position.x * scale + offset_x;
            vertex.transformed_position.y = vertex.transformed_position.y * scale + offset_y;
            vertex
        })
        .collect();
    primitive_assembly(&transformed)
}

pub fn render_image(
    vertices: &[Vertex],
    uniforms: &Uniforms,
    mode: TriangleMode,
    width: i32,
    height: i32,
    samples: i32,
) -> Image {
    let triangles = sampled_triangles(vertices, uniforms, width, height, samples);
    let sample_width = width * samples;
    let sample_height = height * samples;
    let scale = (sample_width as f32 / WIDTH as f32).min(sample_height as f32 / HEIGHT as f32);
    let mut framebuffer = Framebuffer::new(sample_width, sample_height);
    let line_width = if mode == TriangleMode::Wireframe {
        scale.round().max(1.0) as i32
    } else {
        1
    };
    let start = -(line_width / 2);
    let end = start + line_width;

    // Rasteriza y procesa una cara a la vez para limitar la memoria temporal
    // a sus fragmentos, conservando el mismo rasterize y los mismos shaders.
    for triangle in &triangles {
        for fragment in rasterize(std::slice::from_ref(triangle), mode) {
            let color = fragment_shader(&fragment);
            let x = fragment.position.x as i32;
            let y = fragment.position.y as i32;
            for dy in start..end {
                for dx in start..end {
                    framebuffer.point(x + dx, y + dy, color);
                }
            }
        }
    }
    if samples == 1 {
        framebuffer.image
    } else {
        framebuffer.resolve(samples)
    }
}

pub fn export_image(vertices: &[Vertex], uniforms: &Uniforms, mode: TriangleMode) -> Image {
    render_image(
        vertices,
        uniforms,
        mode,
        EXPORT_WIDTH,
        EXPORT_HEIGHT,
        SAMPLES,
    )
}
