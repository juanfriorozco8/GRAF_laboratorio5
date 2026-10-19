use raylib::prelude::{Color, Image, Vector3};
use std::{fs, io, path::Path};

pub struct Framebuffer {
    pub image: Image,
    width: i32,
    height: i32,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            image: Image::gen_image_color(width, height, Color::BLACK),
            width,
            height,
        }
    }

    pub fn point(&mut self, x: i32, y: i32, color: Vector3) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        self.image.draw_pixel(
            x,
            y,
            Color::new(channel(color.x), channel(color.y), channel(color.z), 255),
        );
    }

    // Promedia bloques de muestras: cada gris del borde representa cobertura,
    // sin desenfocar el interior ni modificar la geometria del triangulo.
    pub fn resolve(&self, samples: i32) -> Image {
        assert!(samples > 0 && self.width % samples == 0 && self.height % samples == 0);
        let width = self.width / samples;
        let height = self.height / samples;
        let source = self.image.get_image_data();
        let mut image = Image::gen_image_color(width, height, Color::BLACK);
        let count = (samples * samples) as u32;
        for y in 0..height {
            for x in 0..width {
                let mut total = [0_u32; 3];
                for dy in 0..samples {
                    let row = ((y * samples + dy) * self.width + x * samples) as usize;
                    for dx in 0..samples {
                        let color = source[row + dx as usize];
                        total[0] += u32::from(color.r);
                        total[1] += u32::from(color.g);
                        total[2] += u32::from(color.b);
                    }
                }
                let [r, g, b] = total.map(|sum| ((sum + count / 2) / count) as u8);
                image.draw_pixel(x, y, Color::new(r, g, b, 255));
            }
        }
        image
    }
}

pub fn save_image(image: &Image, path: &Path) -> io::Result<()> {
    let filename = path
        .to_str()
        .ok_or_else(|| io::Error::other("Ruta PNG invalida"))?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    image.export_image(filename);
    if fs::metadata(path)?.len() == 0 {
        return Err(io::Error::other("No se pudo guardar la imagen PNG"));
    }
    Ok(())
}
