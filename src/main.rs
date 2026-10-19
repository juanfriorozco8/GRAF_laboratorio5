mod benchmark;
mod fragment;
mod framebuffer;
mod line;
mod obj;
mod presentation;
mod primitive;
mod rasterizer;
mod shaders;
mod triangle;
mod ui;
mod validation;
mod vertex;

use framebuffer::save_image;
use obj::{Obj, load_obj};
use primitive::primitive_assembly;
use raylib::prelude::*;
use shaders::vertex_shader;
use std::{env, error::Error, io, path::Path};
use triangle::TriangleMode;
use vertex::Vertex;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

pub struct Uniforms {
    pub model_matrix: Matrix,
}

fn create_model_matrix(translation: Vector3, scale: Vector3, rotation: Vector3) -> Matrix {
    let rotate = |v: Vector3| {
        let (sy, cy) = rotation.y.sin_cos();
        let (sx, cx) = rotation.x.sin_cos();
        let (sz, cz) = rotation.z.sin_cos();
        let x = cy * v.x + sy * v.z;
        let z = -sy * v.x + cy * v.z;
        let y = cx * v.y - sx * z;
        let z = sx * v.y + cx * z;
        Vector3::new(cz * x - sz * y, sz * x + cz * y, z)
    };
    let x = rotate(Vector3::new(1.0, 0.0, 0.0));
    let y = rotate(Vector3::new(0.0, 1.0, 0.0));
    let z = rotate(Vector3::new(0.0, 0.0, 1.0));
    Matrix {
        m0: scale.x * x.x,
        m4: scale.x * y.x,
        m8: scale.x * z.x,
        m12: translation.x,
        m1: scale.y * x.y,
        m5: scale.y * y.y,
        m9: scale.y * z.y,
        m13: translation.y,
        m2: scale.z * x.z,
        m6: scale.z * y.z,
        m10: scale.z * z.z,
        m14: translation.z,
        m3: 0.0,
        m7: 0.0,
        m11: 0.0,
        m15: 1.0,
    }
}

fn model_uniforms(model: &Obj) -> Uniforms {
    // Conserva los 32 grados de giro y 22 de inclinacion del laboratorio 4.
    let rotation = Vector3::new(22.0_f32.to_radians(), -32.0_f32.to_radians(), 0.0);
    let orientation = Uniforms {
        model_matrix: create_model_matrix(Vector3::zero(), Vector3::one(), rotation),
    };
    let positions: Vec<Vector3> = model
        .vertices
        .iter()
        .map(|&position| {
            vertex_shader(
                &Vertex::new_with_color(position, Vector3::one()),
                &orientation,
            )
            .transformed_position
        })
        .collect();
    let min_x = positions.iter().map(|v| v.x).fold(f32::INFINITY, f32::min);
    let max_x = positions
        .iter()
        .map(|v| v.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = positions.iter().map(|v| v.y).fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|v| v.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let margin = 45.0;
    let scale = ((WIDTH as f32 - 2.0 * margin) / (max_x - min_x).max(0.001))
        .min((HEIGHT as f32 - 2.0 * margin) / (max_y - min_y).max(0.001));
    let translation = Vector3::new(
        WIDTH as f32 / 2.0 - (min_x + max_x) * 0.5 * scale,
        HEIGHT as f32 / 2.0 + (min_y + max_y) * 0.5 * scale,
        0.0,
    );
    Uniforms {
        model_matrix: create_model_matrix(
            translation,
            Vector3::new(scale, -scale, scale),
            rotation,
        ),
    }
}

struct Options {
    benchmark: bool,
    verify: bool,
    export: bool,
    triangle: bool,
    mode: Option<TriangleMode>,
}

fn parse_options() -> Result<Option<Options>, io::Error> {
    let mut options = Options {
        benchmark: false,
        verify: false,
        export: false,
        triangle: false,
        mode: None,
    };
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--benchmark-triangles" => options.benchmark = true,
            "--verify-triangles" => options.verify = true,
            "--export" => options.export = true,
            "--triangle" => options.triangle = true,
            "--mode" => {
                options.mode = Some(match arguments.next().as_deref() {
                    Some("wireframe") => TriangleMode::Wireframe,
                    Some("barycentric") => TriangleMode::FilledBarycentric,
                    Some("scanline") => TriangleMode::FilledScanline,
                    _ => {
                        return Err(io::Error::other(
                            "Use --mode wireframe|barycentric|scanline",
                        ));
                    }
                });
            }
            "--help" | "-h" => {
                println!(
                    "Laboratorio 5: rellenar triangulos\n\
                    cargo run --release -- [opciones]\n\
                    --triangle              Triangulo de clase en vez del OBJ del laboratorio 4\n\
                    --mode MODO             wireframe, barycentric o scanline\n\
                    --export                Guarda los tres modos en docs/ sin abrir ventana\n\
                    --verify-triangles      Compara posiciones sin crear archivos\n\
                    --benchmark-triangles   Mide 7 series de 100 rasterizaciones sin ventana\n\
                    Ventana: 1 contorno, 2 baricentricas, 3 scanline, S guardar, Esc salir"
                );
                return Ok(None);
            }
            _ => {
                return Err(io::Error::other(format!(
                    "Argumento desconocido: {argument}. Use --help"
                )));
            }
        }
    }
    Ok(Some(options))
}

fn show_window(
    vertices: &[Vertex],
    uniforms: &Uniforms,
    mut mode: TriangleMode,
    scene: &str,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    // El builder de raylib-rs 5.5.1 no expone HIGHDPI. La bandera se configura
    // en el hilo principal antes de crear el contexto grafico.
    unsafe { raylib::ffi::SetConfigFlags(ConfigFlags::FLAG_WINDOW_HIGHDPI as u32) };
    let (mut window, thread) = raylib::init()
        .size(1000, 800)
        .resizable()
        .title("Laboratorio 05 - Rellenar triangulos")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    window.set_window_min_size(640, 520);
    window.set_target_fps(60);
    let interface = ui::Interface::new(&mut window, &thread);
    let mut images: [Option<Image>; 3] = std::array::from_fn(|_| None);
    let mut texture: Option<Texture2D> = None;
    let mut redraw = true;
    let mut slot = 0;
    let mut saved_until = 0.0;
    while !window.window_should_close() {
        let layout = ui::Layout::new(window.get_screen_width(), window.get_screen_height());
        let mouse = window.get_mouse_position();
        window.set_mouse_cursor(layout.cursor(mouse));
        if let Some(selected) = layout
            .selection(&window)
            .filter(|&selected| selected != mode)
        {
            mode = selected;
            redraw = true;
        }
        if redraw {
            slot = match mode {
                TriangleMode::Wireframe => 0,
                TriangleMode::FilledBarycentric => 1,
                TriangleMode::FilledScanline => 2,
            };
            let image = images[slot]
                .get_or_insert_with(|| presentation::export_image(vertices, uniforms, mode));
            let uploaded = window.load_texture_from_image(&thread, image)?;
            uploaded.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            texture = Some(uploaded);
            println!("Modo: {}", mode.name());
            redraw = false;
        }
        if window.is_key_pressed(KeyboardKey::KEY_S) {
            let path = root.join(format!("docs/{scene}_{}.png", mode.name()));
            if let Some(image) = &images[slot] {
                save_image(image, &path)?;
            }
            saved_until = window.get_time() + 2.0;
        }
        let saved = window.get_time() < saved_until;
        let mut drawing = window.begin_drawing(&thread);
        drawing.clear_background(Color::BLACK);
        if let Some(texture) = &texture {
            drawing.draw_texture_pro(
                texture,
                Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
                layout.image,
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }
        interface.draw(&mut drawing, &layout, mode, mouse, scene, saved);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let Some(options) = parse_options()? else {
        return Ok(());
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if options.verify {
        validation::verify(root)?;
    }
    // La verificacion no crea archivos ni abre una ventana.
    if options.verify && !options.export && !options.benchmark {
        return Ok(());
    }
    let (vertices, uniforms, scene) = if options.triangle {
        let positions = [(300.0, 200.0), (400.0, 400.0), (500.0, 200.0)];
        (
            positions
                .map(|(x, y)| Vertex::new_with_color(Vector3::new(x, y, 0.0), Vector3::one()))
                .to_vec(),
            Uniforms {
                model_matrix: Matrix::identity(),
            },
            "triangulo",
        )
    } else {
        let path = root.join("assets/nave_kylo2.obj");
        let model = load_obj(
            path.to_str()
                .ok_or_else(|| io::Error::other("Ruta OBJ invalida"))?,
        )?;
        println!(
            "Vertices OBJ: {} | Triangulos: {}",
            model.vertices.len(),
            model.indices.len() / 3
        );
        (model.get_vertex_array(), model_uniforms(&model), "nave")
    };
    if options.benchmark {
        let transformed: Vec<Vertex> = vertices
            .iter()
            .map(|v| vertex_shader(v, &uniforms))
            .collect();
        let triangles = primitive_assembly(&transformed);
        benchmark::run(&triangles, options.mode, scene)?;
    }
    if options.export {
        for mode in TriangleMode::ALL {
            let image = presentation::export_image(&vertices, &uniforms, mode);
            let path = root.join(format!("docs/{scene}_{}.png", mode.name()));
            save_image(&image, &path)?;
            println!("Imagen guardada: {}", path.display());
        }
    }
    if !options.export && !options.benchmark && !options.verify {
        show_window(
            &vertices,
            &uniforms,
            options.mode.unwrap_or(TriangleMode::Wireframe),
            scene,
            root,
        )?;
    }
    Ok(())
}
