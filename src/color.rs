use zerocopy::{Immutable, IntoBytes};

#[derive(Debug, Clone, Copy, IntoBytes, Immutable)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

const BYTE_MAX_FLOAT: f32 = 0xff as f32;

impl Color {
    pub const fn transparent() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    pub fn hex(rgba: u32) -> Self {
        let r = ((rgba & 0xff000000) >> 24) as u8;
        let g = ((rgba & 0x00ff0000) >> 16) as u8;
        let b = ((rgba & 0x0000ff00) >> 8) as u8;
        let a = (rgba & 0x000000ff) as u8;
        Self { r, g, b, a }
    }

    pub fn hsva(mut h: f32, mut s: f32, mut v: f32, mut a: f32) -> Self {
        h = h.rem_euclid(360.0);
        s = s.clamp(0.0, 100.0);
        v = v.clamp(0.0, 100.0);
        a = a.clamp(0.0, 100.0);
        h /= 60.0;
        s /= 100.0;
        v /= 100.0;
        a /= 100.0;
        let c = v * s;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let (r, g, b) = if h < 1.0 {
            (c, x, 0.0)
        } else if h < 2.0 {
            (x, c, 0.0)
        } else if h < 3.0 {
            (0.0, c, x)
        } else if h < 4.0 {
            (0.0, x, c)
        } else if h < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        let m = v - c;
        let r = r + m;
        let g = g + m;
        let b = b + m;
        fn srgb_from_linear(x: f32) -> f32 {
            if x <= 0.0031308 {
                x * 12.92
            } else {
                x.powf(1.0 / 2.4) * 1.055 - 0.055
            }
        }
        let r = srgb_from_linear(r);
        let g = srgb_from_linear(g);
        let b = srgb_from_linear(b);
        let r = (r * BYTE_MAX_FLOAT) as u8;
        let g = (g * BYTE_MAX_FLOAT) as u8;
        let b = (b * BYTE_MAX_FLOAT) as u8;
        let a = (a * BYTE_MAX_FLOAT) as u8;
        Self { r, g, b, a }
    }

    pub fn blend(self, other: Color) -> Self {
        let Self {
            r: top_r,
            g: top_g,
            b: top_b,
            a: top_a,
        } = other;
        let Self {
            r: bot_r,
            g: bot_g,
            b: bot_b,
            a: bot_a,
        } = self;

        let top_r = top_r as f32 / BYTE_MAX_FLOAT;
        let top_g = top_g as f32 / BYTE_MAX_FLOAT;
        let top_b = top_b as f32 / BYTE_MAX_FLOAT;
        let top_a = top_a as f32 / BYTE_MAX_FLOAT;
        let bot_r = bot_r as f32 / BYTE_MAX_FLOAT;
        let bot_g = bot_g as f32 / BYTE_MAX_FLOAT;
        let bot_b = bot_b as f32 / BYTE_MAX_FLOAT;
        let bot_a = bot_a as f32 / BYTE_MAX_FLOAT;

        let top_a_inv = 1.0 - top_a;
        let a = top_a + bot_a * top_a_inv;
        let r = (top_r * top_a + bot_r * bot_a * top_a_inv) / a;
        let g = (top_g * top_a + bot_g * bot_a * top_a_inv) / a;
        let b = (top_b * top_a + bot_b * bot_a * top_a_inv) / a;

        let r = (r * BYTE_MAX_FLOAT) as u8;
        let g = (g * BYTE_MAX_FLOAT) as u8;
        let b = (b * BYTE_MAX_FLOAT) as u8;
        let a = (a * BYTE_MAX_FLOAT) as u8;

        Self { r, g, b, a }
    }
}
