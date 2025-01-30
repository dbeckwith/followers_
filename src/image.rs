use crate::color::Color;
use zerocopy::IntoBytes;

pub struct Image {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::hsva(0.0, 0.0, 0.0, 100.0); width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn blend_pixel(&mut self, x: usize, y: usize, color: Color) {
        let p = &mut self.pixels[x + y * self.width];
        *p = p.blend(color);
    }

    pub fn to_image_data(&self) -> web_sys::ImageData {
        let data = self.pixels.as_bytes();
        let sw = self.width as u32;
        let sh = self.height as u32;
        web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(data),
            sw,
            sh,
        )
        .unwrap()
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        // TODO: preserve image content on resize
        *self = Self::new(width, height);
    }
}
