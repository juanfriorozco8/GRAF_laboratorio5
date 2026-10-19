use crate::{
    fragment::Fragment,
    triangle::{TriangleMode, triangle, triangle_barycentric, triangle_scanline},
    vertex::Vertex,
};

pub fn rasterize(triangles: &[[Vertex; 3]], mode: TriangleMode) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    for [a, b, c] in triangles {
        fragments.extend(match mode {
            TriangleMode::Wireframe => triangle(a, b, c),
            TriangleMode::FilledBarycentric => triangle_barycentric(a, b, c),
            TriangleMode::FilledScanline => triangle_scanline(a, b, c),
        });
    }
    fragments
}
