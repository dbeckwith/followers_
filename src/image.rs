use crate::color::Color;
use zerocopy::IntoBytes;

pub struct Image {
    width: usize,
    height: usize,
    background: Color,
    pixels: Vec<Color>,
}

impl Image {
    pub fn new(width: usize, height: usize, background: Color) -> Self {
        Self {
            width,
            height,
            background,
            pixels: vec![background; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[x + y * self.width] = color;
    }

    pub fn blend_pixel(&mut self, x: usize, y: usize, color: Color) {
        let p = &mut self.pixels[x + y * self.width];
        *p = p.blend(color);
    }

    pub fn draw_particle(&mut self, x: f32, y: f32, color: Color) {
        macro_rules! calc {
            ($x:expr, $w:expr) => {{
                let w = $w;
                let mut x = $x;
                let wf = w as f32;
                x -= 0.5;
                if x <= -1.0 || x >= wf {
                    (None, None)
                } else if x < 0.0 {
                    let xf = 1.0 + x;
                    (None, Some((0, xf)))
                } else if x >= wf - 1.0 {
                    let xf = x.fract();
                    (Some((w - 1, (1.0 - xf))), None)
                } else {
                    let xf = x.fract();
                    let x = x as usize;
                    (Some((x, (1.0 - xf))), Some((x + 1, xf)))
                }
            }};
        }
        macro_rules! write {
            ($x:expr, $y:expr) => {
                if let (Some((x, xf)), Some((y, yf))) = ($x, $y) {
                    let c = color.fade(xf * yf);
                    self.blend_pixel(x, y, c);
                }
            };
        }
        let (x0, x1) = calc!(x, self.width);
        let (y0, y1) = calc!(y, self.height);
        write!(x0, y0);
        write!(x0, y1);
        write!(x1, y0);
        write!(x1, y1);
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
        // resize the image
        // preserve its contents in the center of the new image
        use std::cmp::Ordering::*;
        let bg = self.background;
        let w1 = self.width;
        let h1 = self.height;
        let w2 = width;
        let h2 = height;
        match (w2.cmp(&w1), h2.cmp(&h1)) {
            (Less, Less | Equal) => {
                let mx = (w1 - w2) / 2;
                let my = (h1 - h2) / 2;
                let p1 = &self.pixels;
                let mut p2 = vec![bg; w2 * h2];
                for y2 in 0..h2 {
                    let y1 = y2 - my;
                    let i1 = mx + w1 * y1;
                    let i2 = w2 * y2;
                    p2[i2..i2 + w2].copy_from_slice(&p1[i1..i1 + w2]);
                }
                self.pixels = p2;
                self.width = width;
                self.height = height;
            },
            (Less, Greater) => {
                let mx = (w1 - w2) / 2;
                let my = (h2 - h1) / 2;
                let p1 = &self.pixels;
                let mut p2 = vec![bg; w2 * h2];
                for y1 in 0..h1 {
                    let y2 = y1 + my;
                    let i1 = mx + w1 * y1;
                    let i2 = w2 * y2;
                    p2[i2..i2 + w2].copy_from_slice(&p1[i1..i1 + w2]);
                }
                self.pixels = p2;
                self.width = width;
                self.height = height;
            },
            (Equal, Less) => {
                let my = (h1 - h2) / 2;
                let p1 = &self.pixels;
                let p2 = p1[w1 * my..w1 * (my + h2)].to_vec();
                self.pixels = p2;
                self.width = width;
                self.height = height;
            },
            (Equal, Equal) => {},
            (Equal, Greater) => {
                let my = (h2 - h1) / 2;
                let p1 = &self.pixels;
                let mut p2 = vec![bg; w2 * h2];
                p2[w2 * my..w2 * (my + h1)].copy_from_slice(p1);
                self.pixels = p2;
                self.width = width;
                self.height = height;
            },
            (Greater, Less | Equal) => {
                let mx = (w2 - w1) / 2;
                let my = (h1 - h2) / 2;
                let p1 = &self.pixels;
                let mut p2 = vec![bg; w2 * h2];
                for y2 in 0..h2 {
                    let y1 = y2 + my;
                    let i1 = w1 * y1;
                    let i2 = mx + w2 * y2;
                    p2[i2..i2 + w1].copy_from_slice(&p1[i1..i1 + w1]);
                }
                self.pixels = p2;
                self.width = width;
                self.height = height;
            },
            (Greater, Greater) => {
                let mx = (w2 - w1) / 2;
                let my = (h2 - h1) / 2;
                let p1 = &self.pixels;
                let mut p2 = vec![bg; w2 * h2];
                for y1 in 0..h1 {
                    let y2 = y1 + my;
                    let i1 = w1 * y1;
                    let i2 = mx + w2 * y2;
                    p2[i2..i2 + w1].copy_from_slice(&p1[i1..i1 + w1]);
                }
                self.pixels = p2;
                self.width = width;
                self.height = height;
            },
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(self.background);
    }
}
