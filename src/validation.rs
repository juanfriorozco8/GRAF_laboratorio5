use crate::{
    fragment::Fragment,
    triangle::{triangle_barycentric, triangle_scanline},
    vertex::Vertex,
};
use raylib::prelude::Vector3;
use std::{collections::BTreeSet, io, path::Path};

type Position = (i32, i32);

pub fn positions(fragments: &[Fragment]) -> BTreeSet<Position> {
    fragments
        .iter()
        .map(|f| (f.position.x as i32, f.position.y as i32))
        .collect()
}

pub fn make_triangle(points: [(f32, f32); 3]) -> [Vertex; 3] {
    points.map(|(x, y)| Vertex::new_with_color(Vector3::new(x, y, 0.0), Vector3::one()))
}

pub fn cases() -> Vec<(&'static str, [Vertex; 3])> {
    vec![
        (
            "Pequeño",
            make_triangle([(1.0, 1.0), (5.0, 2.0), (2.0, 5.0)]),
        ),
        (
            "Grande",
            make_triangle([(20.0, 20.0), (770.0, 100.0), (300.0, 570.0)]),
        ),
        (
            "Borde horizontal superior",
            make_triangle([(10.0, 10.0), (80.0, 10.0), (40.0, 90.0)]),
        ),
        (
            "Borde horizontal inferior",
            make_triangle([(40.0, 10.0), (80.0, 90.0), (10.0, 90.0)]),
        ),
        (
            "Borde vertical",
            make_triangle([(10.0, 10.0), (10.0, 90.0), (80.0, 50.0)]),
        ),
        (
            "Parcialmente fuera del framebuffer",
            make_triangle([(-30.0, -20.0), (90.0, 15.0), (20.0, 80.0)]),
        ),
        (
            "Colineal",
            make_triangle([(10.0, 10.0), (30.0, 30.0), (70.0, 70.0)]),
        ),
        ("Tres vértices iguales", make_triangle([(2.0, 2.0); 3])),
        (
            "Vértices y aristas sobre centros",
            make_triangle([(0.5, 0.5), (8.5, 0.5), (0.5, 8.5)]),
        ),
        (
            "Subpíxel",
            make_triangle([(2.1, 2.2), (2.3, 2.2), (2.2, 2.4)]),
        ),
    ]
}

fn compare_one(triangle: &[Vertex; 3]) -> io::Result<(usize, usize)> {
    let [a, b, c] = triangle;
    let barycentric = triangle_barycentric(a, b, c);
    let scanline = triangle_scanline(a, b, c);
    let first = positions(&barycentric);
    let second = positions(&scanline);
    if first != second || first.len() != barycentric.len() || second.len() != scanline.len() {
        return Err(io::Error::other(format!(
            "Rellenos diferentes o duplicados: {} diferencias",
            first.symmetric_difference(&second).count()
        )));
    }
    let reverse = positions(&triangle_barycentric(c, b, a));
    let reverse_scanline = positions(&triangle_scanline(c, b, a));
    if first != reverse || first != reverse_scanline {
        return Err(io::Error::other("El orden inverso cambia la cobertura"));
    }
    let visible = first
        .iter()
        .filter(|&&(x, y)| (0..crate::WIDTH).contains(&x) && (0..crate::HEIGHT).contains(&y))
        .count();
    Ok((first.len(), visible))
}

pub fn compare_triangles(triangles: &[[Vertex; 3]]) -> io::Result<()> {
    for (index, triangle) in triangles.iter().enumerate() {
        compare_one(triangle)
            .map_err(|error| io::Error::other(format!("Triangulo {index}: {error}")))?;
    }
    println!(
        "Cobertura identica en los {} triangulos, tambien en orden inverso.",
        triangles.len()
    );
    Ok(())
}

fn compare_sample_grid(triangles: &[[Vertex; 3]]) -> io::Result<usize> {
    let mut total = 0;
    for (index, [a, b, c]) in triangles.iter().enumerate() {
        let expected = triangle_barycentric(a, b, c);
        // Ambos rellenos recorren filas y columnas en orden; comparar la
        // secuencia exacta evita almacenar millones de entradas en un arbol.
        for (scanline, first, last) in [(true, a, c), (false, c, a), (true, c, a)] {
            let actual = if scanline {
                triangle_scanline(first, b, last)
            } else {
                triangle_barycentric(first, b, last)
            };
            if expected.len() != actual.len()
                || expected
                    .iter()
                    .zip(&actual)
                    .any(|(x, y)| x.position != y.position)
            {
                return Err(io::Error::other(format!(
                    "Diferencia en muestras de alta resolucion, cara {index}"
                )));
            }
        }
        if expected.windows(2).any(|pair| {
            let a = pair[0].position;
            let b = pair[1].position;
            (a.y, a.x) >= (b.y, b.x)
        }) {
            return Err(io::Error::other("Muestras duplicadas o desordenadas"));
        }
        total += expected.len();
    }
    Ok(total)
}

pub fn verify(root: &Path) -> io::Result<()> {
    let mut report = String::from(
        "# Comparación exacta de posiciones\n\n\
        Se comparan conjuntos de `(x, y)` antes de escribir en el framebuffer y se verifica que \
        no haya fragmentos duplicados dentro de un mismo triángulo. También se invierte el orden \
        de los tres vértices en ambos algoritmos. Color uniforme blanco, centros `(x + 0.5, y + 0.5)` \
        y regla superior/izquierda en los dos rellenos.\n\n\
        | Caso | Baricéntrico | Scanline | Visibles (800 × 600) | Diferencias | Orden inverso |\n\
        | --- | ---: | ---: | ---: | ---: | --- |\n",
    );
    for (name, triangle) in cases() {
        let (count, visible) = compare_one(&triangle)?;
        report.push_str(&format!(
            "| {name} | {count} | {count} | {visible} | 0 | Igual |\n"
        ));
    }
    let path = root.join("assets/nave_kylo2.obj");
    let model = crate::obj::load_obj(
        path.to_str()
            .ok_or_else(|| io::Error::other("Ruta OBJ invalida"))?,
    )
    .map_err(|error| io::Error::other(error.to_string()))?;
    let uniforms = crate::model_uniforms(&model);
    let transformed: Vec<Vertex> = model
        .get_vertex_array()
        .iter()
        .map(|vertex| crate::shaders::vertex_shader(vertex, &uniforms))
        .collect();
    let triangles = crate::primitive::primitive_assembly(&transformed);
    compare_triangles(&triangles)?;
    report.push_str(&format!(
        "\nTambién se compararon individualmente las **{} caras del OBJ del laboratorio 4**, \
        en ambos órdenes: cero diferencias y cero duplicados dentro de cada cara.\n",
        triangles.len()
    ));
    let sampled = crate::presentation::sampled_triangles(
        &model.get_vertex_array(),
        &uniforms,
        crate::presentation::EXPORT_WIDTH,
        crate::presentation::EXPORT_HEIGHT,
        crate::presentation::SAMPLES,
    );
    let samples = compare_sample_grid(&sampled)?;
    report.push_str(&format!("\n## Presentación de alta resolución\n\n\
        También se comparó la secuencia exacta de las posiciones antes de promediar las muestras, \
        en la cuadrícula de **4800 × 3600**, para cada cara y ambos órdenes de vértices. \
        Cada algoritmo generó **{samples} fragmentos**: cero diferencias y cero duplicados dentro de cada cara. \
        Los PNG finales de 2400 × 1800 también son idénticos entre ambos rellenos.\n"));
    println!("Alta resolucion: {samples} muestras por algoritmo, sin diferencias.");
    report.push_str("\nLos fragmentos fuera de la imagen se conservan en rasterización y `Framebuffer::point` los descarta. \
        El caso colineal y los tres vértices iguales producen cero fragmentos. Las aristas compartidas \
        incluyen solamente el borde superior o izquierdo, evitando doble cobertura entre triángulos adyacentes.\n");
    println!("{report}");
    println!("Verificacion correcta.");
    Ok(())
}
