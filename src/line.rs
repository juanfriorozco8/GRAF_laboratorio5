use crate::{fragment::Fragment, vertex::Vertex};

// Bresenham del laboratorio 4: genera fragmentos antes de escribir pixeles.
pub fn line(start: &Vertex, end: &Vertex) -> Vec<Fragment> {
    let a = start.transformed_position;
    let b = end.transformed_position;
    let mut x = a.x.round() as i32;
    let mut y = a.y.round() as i32;
    let x2 = b.x.round() as i32;
    let y2 = b.y.round() as i32;
    let dx = (x2 - x).abs();
    let dy = -(y2 - y).abs();
    let sx = if x < x2 { 1 } else { -1 };
    let sy = if y < y2 { 1 } else { -1 };
    let mut error = dx + dy;
    let mut fragments = Vec::new();
    loop {
        fragments.push(Fragment::new(x as f32, y as f32, start.color, 0.0));
        if x == x2 && y == y2 {
            break;
        }
        let doubled_error = 2 * error;
        if doubled_error >= dy {
            error += dy;
            x += sx;
        }
        if doubled_error <= dx {
            error += dx;
            y += sy;
        }
    }
    fragments
}
