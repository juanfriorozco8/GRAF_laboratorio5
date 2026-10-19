use crate::{rasterizer::rasterize, triangle::TriangleMode, vertex::Vertex};
use std::{hint::black_box, io, time::Instant};

const ITERATIONS: usize = 100;
const REPETITIONS: usize = 7;

pub fn run(
    triangles: &[[Vertex; 3]],
    selected: Option<TriangleMode>,
    scene: &str,
) -> io::Result<()> {
    if cfg!(debug_assertions) {
        return Err(io::Error::other(
            "El benchmark requiere modo optimizado: cargo run --release -- --benchmark-triangles",
        ));
    }
    let modes: Vec<TriangleMode> = TriangleMode::ALL
        .into_iter()
        .filter(|&mode| selected.is_none_or(|selected| selected == mode))
        .collect();
    let counts: Vec<usize> = modes
        .iter()
        .map(|&mode| black_box(rasterize(black_box(triangles), mode)).len())
        .collect();
    let mut samples = vec![Vec::with_capacity(REPETITIONS); modes.len()];
    println!(
        "Escena: {scene} | {} triangulos | {ITERATIONS} ejecuciones x {REPETITIONS} series",
        triangles.len()
    );
    for repetition in 0..REPETITIONS {
        // Rota el orden entre series para no favorecer siempre al mismo modo.
        for offset in 0..modes.len() {
            let index = (repetition + offset) % modes.len();
            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let fragments = rasterize(black_box(triangles), modes[index]);
                black_box(fragments);
            }
            samples[index].push(start.elapsed().as_secs_f64() * 1000.0);
        }
    }
    for (index, &mode) in modes.iter().enumerate() {
        let mut sorted = samples[index].clone();
        sorted.sort_by(f64::total_cmp);
        let median = sorted[REPETITIONS / 2];
        println!("{}: series de 100 (ms) = {:?}", mode.name(), samples[index]);
        println!(
            "{}: {} fragmentos | mediana de 100 = {:.3} ms | {:.5} ms/ejecucion",
            mode.name(),
            counts[index],
            median,
            median / ITERATIONS as f64
        );
    }
    Ok(())
}
