use std::path::Path;

use raylib::prelude::*;

use crate::{HEIGHT, WIDTH, triangle::TriangleMode};

const LABELS: [&str; 3] = ["01  Contorno", "02  Baricentricas", "03  Scanline"];

pub struct Layout {
    pub image: Rectangle,
    tabs: [Rectangle; 3],
    width: f32,
    height: f32,
    margin: f32,
}

impl Layout {
    pub fn new(width: i32, height: i32) -> Self {
        let width = width as f32;
        let height = height as f32;
        let margin = if width < 800.0 { 24.0 } else { 36.0 };
        let available_width = width - margin * 2.0;
        let available_height = height - 184.0;
        let scale = (available_width / WIDTH as f32).min(available_height / HEIGHT as f32);
        let image = Rectangle::new(
            (width - WIDTH as f32 * scale) * 0.5,
            76.0 + (available_height - HEIGHT as f32 * scale) * 0.5,
            WIDTH as f32 * scale,
            HEIGHT as f32 * scale,
        );
        let widths = [124.0, 170.0, 124.0];
        let gap = 20.0;
        let mut x = (width - widths.iter().sum::<f32>() - gap * 2.0) * 0.5;
        let tabs = widths.map(|tab_width| {
            let tab = Rectangle::new(x, height - 91.0, tab_width, 34.0);
            x += tab_width + gap;
            tab
        });
        Self {
            image,
            tabs,
            width,
            height,
            margin,
        }
    }

    fn hovered(&self, mouse: Vector2) -> Option<usize> {
        self.tabs.iter().position(|tab| {
            mouse.x >= tab.x
                && mouse.x < tab.x + tab.width
                && mouse.y >= tab.y
                && mouse.y < tab.y + tab.height
        })
    }

    pub fn selection(&self, window: &RaylibHandle) -> Option<TriangleMode> {
        let keys = [
            KeyboardKey::KEY_ONE,
            KeyboardKey::KEY_TWO,
            KeyboardKey::KEY_THREE,
        ];
        if let Some(index) = keys.iter().position(|&key| window.is_key_pressed(key)) {
            return Some(TriangleMode::ALL[index]);
        }
        if window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            return self
                .hovered(window.get_mouse_position())
                .map(|index| TriangleMode::ALL[index]);
        }
        None
    }

    pub fn cursor(&self, mouse: Vector2) -> MouseCursor {
        if self.hovered(mouse).is_some() {
            MouseCursor::MOUSE_CURSOR_POINTING_HAND
        } else {
            MouseCursor::MOUSE_CURSOR_DEFAULT
        }
    }
}

pub struct Interface {
    font: Option<Font>,
    fallback: WeakFont,
}

impl Interface {
    pub fn new(window: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        // Usa una fuente del sistema sin redistribuirla; Raylib ofrece el fallback.
        let candidates = [
            "/System/Library/Fonts/SFNSMono.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "C:/Windows/Fonts/consola.ttf",
        ];
        let font = candidates
            .iter()
            .filter(|path| Path::new(path).is_file())
            .find_map(|path| window.load_font_ex(thread, path, 48, None).ok());
        if let Some(font) = &font {
            font.texture()
                .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        }
        Self {
            font,
            fallback: window.get_font_default(),
        }
    }

    fn text(
        &self,
        drawing: &mut impl RaylibDraw,
        text: &str,
        position: Vector2,
        size: f32,
        alpha: u8,
    ) {
        let color = Color::new(255, 255, 255, alpha);
        match &self.font {
            Some(font) => drawing.draw_text_ex(font, text, position, size, 0.6, color),
            None => drawing.draw_text_ex(&self.fallback, text, position, size, 0.6, color),
        }
    }

    fn text_width(&self, text: &str, size: f32) -> f32 {
        match &self.font {
            Some(font) => font.measure_text(text, size, 0.6).x,
            None => self.fallback.measure_text(text, size, 0.6).x,
        }
    }

    pub fn draw(
        &self,
        drawing: &mut impl RaylibDraw,
        layout: &Layout,
        mode: TriangleMode,
        mouse: Vector2,
        scene: &str,
        saved: bool,
    ) {
        self.text(
            drawing,
            "LABORATORIO 05",
            Vector2::new(layout.margin, 29.0),
            11.0,
            255,
        );
        let scene = if scene == "nave" {
            "NAVE DE KYLO REN"
        } else {
            "TRIANGULO"
        };
        self.text(
            drawing,
            scene,
            Vector2::new(
                layout.width - layout.margin - self.text_width(scene, 11.0),
                29.0,
            ),
            11.0,
            150,
        );
        drawing.draw_rectangle(
            layout.margin as i32,
            57,
            (layout.width - layout.margin * 2.0) as i32,
            1,
            Color::new(255, 255, 255, 30),
        );
        for (index, (label, tab)) in LABELS.iter().zip(&layout.tabs).enumerate() {
            let active = TriangleMode::ALL[index] == mode;
            let hovered = layout.hovered(mouse) == Some(index);
            let text_width = self.text_width(label, 12.0);
            let x = tab.x + (tab.width - text_width) * 0.5;
            self.text(
                drawing,
                label,
                Vector2::new(x, tab.y + 4.0),
                12.0,
                if active || hovered { 255 } else { 130 },
            );
            if active {
                drawing.draw_rectangle(
                    x.round() as i32,
                    (tab.y + 28.0) as i32,
                    text_width.ceil() as i32,
                    1,
                    Color::WHITE,
                );
            }
        }
        self.text(
            drawing,
            "1 / 2 / 3   Cambiar vista",
            Vector2::new(layout.margin, layout.height - 29.0),
            10.0,
            130,
        );
        let footer = if saved {
            "Imagen guardada"
        } else {
            "S  Guardar    Esc  Salir"
        };
        self.text(
            drawing,
            footer,
            Vector2::new(
                layout.width - layout.margin - self.text_width(footer, 10.0),
                layout.height - 29.0,
            ),
            10.0,
            if saved { 255 } else { 130 },
        );
    }
}
