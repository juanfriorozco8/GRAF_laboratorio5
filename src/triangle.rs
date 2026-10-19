use crate::{fragment::Fragment, line::line, vertex::Vertex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
    FilledScanline,
}

impl TriangleMode {
    pub const ALL: [Self; 3] = [
        Self::Wireframe,
        Self::FilledBarycentric,
        Self::FilledScanline,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Wireframe => "wireframe",
            Self::FilledBarycentric => "baricentrico",
            Self::FilledScanline => "scanline",
        }
    }
}

pub fn triangle(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let mut fragments = line(a, b);
    fragments.extend(line(b, c));
    fragments.extend(line(c, a));
    fragments
}

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

fn edge(a: Point, b: Point, p: Point) -> f64 {
    (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
}

// Con area positiva e Y hacia abajo, incluye los bordes superior e izquierdo.
fn is_top_left(a: Point, b: Point) -> bool {
    b.y < a.y || (b.y == a.y && b.x > a.x)
}

struct Coverage {
    points: [Point; 3],
    area: f64,
    inclusive: [bool; 3],
}

impl Coverage {
    fn new(a: &Vertex, b: &Vertex, c: &Vertex) -> Option<Self> {
        let mut points = [a, b, c].map(|vertex| Point {
            x: f64::from(vertex.transformed_position.x),
            y: f64::from(vertex.transformed_position.y),
        });
        if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return None;
        }
        let area = edge(points[0], points[1], points[2]);
        if area == 0.0 {
            return None;
        }
        if area < 0.0 {
            points.swap(1, 2);
        }
        let [a, b, c] = points;
        Some(Self {
            points,
            area: area.abs(),
            inclusive: [is_top_left(b, c), is_top_left(c, a), is_top_left(a, b)],
        })
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        let p = Point {
            x: f64::from(x) + 0.5,
            y: f64::from(y) + 0.5,
        };
        let [a, b, c] = self.points;
        let weights = [edge(b, c, p), edge(c, a, p), edge(a, b, p)];
        weights
            .into_iter()
            .zip(self.inclusive)
            .all(|(weight, inclusive)| {
                let coordinate = weight / self.area;
                coordinate > 0.0 || (coordinate == 0.0 && inclusive)
            })
    }

    fn rows(&self) -> std::ops::Range<i32> {
        let min = self
            .points
            .iter()
            .map(|p| p.y)
            .fold(f64::INFINITY, f64::min);
        let max = self
            .points
            .iter()
            .map(|p| p.y)
            .fold(f64::NEG_INFINITY, f64::max);
        (min - 0.5).ceil() as i32..(max - 0.5).ceil() as i32
    }
}

pub fn triangle_barycentric(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let Some(coverage) = Coverage::new(a, b, c) else {
        return Vec::new();
    };
    let min_x = coverage
        .points
        .iter()
        .map(|p| p.x)
        .fold(f64::INFINITY, f64::min)
        .floor() as i32;
    let max_x = coverage
        .points
        .iter()
        .map(|p| p.x)
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil() as i32;
    let mut fragments = Vec::new();
    for y in coverage.rows() {
        for x in min_x..max_x {
            if coverage.contains(x, y) {
                fragments.push(Fragment::new(x as f32, y as f32, a.color, 0.0));
            }
        }
    }
    fragments
}

pub fn triangle_scanline(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let Some(coverage) = Coverage::new(a, b, c) else {
        return Vec::new();
    };
    let points = coverage.points;
    let mut fragments = Vec::new();
    for y in coverage.rows() {
        let center_y = f64::from(y) + 0.5;
        let mut intersections = [0.0_f64; 3];
        let mut count = 0;
        for i in 0..3 {
            let mut start = points[i];
            let mut end = points[(i + 1) % 3];
            if start.y > end.y {
                std::mem::swap(&mut start, &mut end);
            }
            // [y_min, y_max): las horizontales no cruzan; un vertice cuenta una vez.
            if center_y >= start.y && center_y < end.y {
                let t = (center_y - start.y) / (end.y - start.y);
                intersections[count] = start.x + t * (end.x - start.x);
                count += 1;
            }
        }
        if count != 2 {
            continue;
        }
        let left = intersections[0].min(intersections[1]);
        let right = intersections[0].max(intersections[1]);
        // Corrige solo los extremos con la regla compartida, para evitar
        // discrepancias numericas. El interior no reevalua baricentricas.
        let mut start_x = ((left - 0.5).floor() as i32).saturating_sub(1);
        let mut end_x = ((right - 0.5).ceil() as i32).saturating_add(1);
        while start_x < end_x && !coverage.contains(start_x, y) {
            start_x += 1;
        }
        while start_x < end_x && !coverage.contains(end_x - 1, y) {
            end_x -= 1;
        }
        for x in start_x..end_x {
            fragments.push(Fragment::new(x as f32, y as f32, a.color, 0.0));
        }
    }
    fragments
}
