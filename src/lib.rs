mod hooks;

use crate::hooks::use_element;
use anyhow::{ensure, Result};
use dioxus::prelude::*;
use log::{debug, info};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::{
    cell::RefCell,
    f32::consts::PI,
    ops::{Add, AddAssign, Mul, Sub},
    rc::Rc,
};
use wasm_bindgen::prelude::*;
use zerocopy::{Immutable, IntoBytes};

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    info!("wasm start");

    let window = web_sys::window()
        .ok_or_else(|| JsError::new("failed to get window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsError::new("failed to get document of window"))?;
    let body = document
        .body()
        .ok_or_else(|| JsError::new("failed to get body of document"))?;

    dioxus::web::launch::launch_cfg(
        App,
        dioxus::web::Config::new().rootelement(body.into()),
    );

    Ok(())
}

#[component]
fn App() -> Element {
    let world = use_signal(World::new);
    let mut world_renderer = use_signal(|| None::<WorldRenderer>);

    let (canvas_element, on_canvas_mounted) =
        use_element::<web_sys::HtmlCanvasElement>();

    use_effect(move || {
        let canvas_element = canvas_element.read();
        let canvas_element = &*canvas_element;
        if let Some(canvas_element) = canvas_element {
            let renderer = WorldRenderer::new(canvas_element, world);
            world_renderer.set(Some(renderer));
        }
    });

    // TODO: size canvas to window

    // TODO: update world renderer on canvas resize
    // don't just create a brand new renderer; preserve image data from old one

    // TODO: display params

    rsx! {
        canvas {
            width: 1920,
            height: 1080,
            onmounted: on_canvas_mounted,
        }
    }
}

struct World {
    params: Params,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    partners: Vec<[usize; 2]>,
    colors: Vec<Color>,
}

struct Params {
    particle_count: usize,
    seed: u64,
}

impl Params {
    fn new(particle_count: usize, seed: u64) -> Result<Self> {
        ensure!(particle_count > 2);
        Ok(Self {
            particle_count,
            seed,
        })
    }

    fn idxs(&self) -> std::ops::Range<usize> {
        0..self.particle_count
    }
}

impl World {
    fn new() -> Self {
        let seed: u64 = thread_rng().gen();
        let seed: u64 = 0x27e3771584a46455;
        info!("SEED: 0x{seed:016x}");

        let params = Params::new(1000, seed).unwrap();

        let mut seeds = ChaCha20Rng::seed_from_u64(params.seed)
            .sample_iter(rand::distributions::Standard);

        macro_rules! with_rng {
            (| $rng:ident | $body:expr) => {{
                #[allow(unused, unused_mut)]
                let mut $rng =
                    ChaCha20Rng::seed_from_u64(seeds.next().unwrap());
                $body
            }};
        }

        let positions = with_rng!(|rng| params
            .idxs()
            .map(|idx| {
                let t = lerp(
                    idx as f32,
                    0.0,
                    (params.particle_count - 1) as f32,
                    0.0,
                    2.0 * PI,
                );
                let r = rng.gen_range(9.0..=10.0);
                Vec2::new(r * t.cos(), r * t.sin())
            })
            .collect::<Vec<_>>());

        let velocities = with_rng!(|rng| params
            .idxs()
            .map(|_idx| Vec2::new(0.0, 0.0))
            .collect::<Vec<_>>());

        let partners = with_rng!(|rng| params
            .idxs()
            .map(|idx| {
                let i = idx;
                let mut j = rng.gen_range(params.idxs());
                while j == i {
                    j = rng.gen_range(params.idxs());
                }
                let mut k = rng.gen_range(params.idxs());
                while k == i || k == j {
                    k = rng.gen_range(params.idxs());
                }
                [j, k]
            })
            .collect::<Vec<_>>());

        let colors = with_rng!(|rng| params
            .idxs()
            .map(|_idx| {
                Color::hsva(
                    rng.gen_range(0.0..=240.0),
                    rng.gen_range(20.0..=40.0),
                    80.0,
                    6.0,
                )
            })
            .collect::<Vec<_>>());

        Self {
            params,
            positions,
            velocities,
            partners,
            colors,
        }
    }

    fn update(&mut self, image: &mut Image) {
        let Self {
            params,
            positions,
            velocities,
            partners,
            colors,
        } = self;

        for idx in params.idxs() {
            let pos = positions[idx];
            let [p1, p2] = partners[idx];
            let p1 = positions[p1];
            let p2 = positions[p2];
            let vel = &mut velocities[idx];

            let t = (pos - p1).dot(p2 - p1) / p2.distance_squared(p1);
            let t = t.max(1.0);
            let target_pos = p2 * t + p1 * (1.0 - t);

            let acc = target_pos - pos;
            let acc = acc.clamp_length_max(0.5);
            *vel += acc;
            *vel = vel.clamp_length_max(1.0);
        }

        for idx in params.idxs() {
            positions[idx] += velocities[idx];
        }

        let w = image.width() as f32;
        let h = image.height() as f32;
        for idx in params.idxs() {
            let pos = positions[idx];
            let x = pos.x + w / 2.0;
            let y = pos.y + h / 2.0;
            if x < 0.0 || x >= w || y < 0.0 || y >= h {
                return;
            }
            let x = x as usize;
            let y = y as usize;

            let color = colors[idx];

            image.blend_pixel(x, y, color);
        }
    }
}

struct WorldRenderer {
    #[allow(clippy::type_complexity)]
    _closure_handle: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
}

impl WorldRenderer {
    fn new(
        canvas: &web_sys::HtmlCanvasElement,
        mut world: Signal<World>,
    ) -> WorldRenderer {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let width = canvas.width() as usize;
        let height = canvas.height() as usize;
        let mut image = Image::new(width, height);
        debug!("start render {}x{}", width, height);

        let window = canvas.owner_document().unwrap().default_view().unwrap();

        let closure_handle =
            Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
        let closure = Closure::new({
            let window = window.clone();
            let closure_handle = Rc::clone(&closure_handle);
            move || {
                debug!("update");
                world.write().update(&mut image);
                let image_data = image.to_image_data();
                context.put_image_data(&image_data, 0.0, 0.0).unwrap();
                window
                    .request_animation_frame(
                        closure_handle
                            .borrow()
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unchecked_ref(),
                    )
                    .unwrap();
            }
        });
        window
            .request_animation_frame(closure.as_ref().unchecked_ref())
            .unwrap();
        *closure_handle.borrow_mut() = Some(closure);

        WorldRenderer {
            _closure_handle: closure_handle,
        }
    }
}

struct Image {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl Image {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::hsva(0.0, 0.0, 0.0, 100.0); width * height],
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn blend_pixel(&mut self, x: usize, y: usize, color: Color) {
        let p = &mut self.pixels[x + y * self.width];
        *p = p.blend(color);
    }

    fn to_image_data(&self) -> web_sys::ImageData {
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
}

#[derive(Debug, Clone, Copy, IntoBytes, Immutable)]
#[repr(C)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

const BYTE_MAX_FLOAT: f32 = 0xff as f32;

impl Color {
    fn hsva(mut h: f32, mut s: f32, mut v: f32, mut a: f32) -> Self {
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

    fn blend(self, other: Color) -> Self {
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

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, scale: f32) -> Self::Output {
        Self {
            x: self.x * scale,
            y: self.y * scale,
        }
    }
}

impl Vec2 {
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    fn length_squared(self) -> f32 {
        self.dot(self)
    }

    fn distance_squared(self, other: Self) -> f32 {
        (self - other).length_squared()
    }

    fn clamp_length_max(self, max_length: f32) -> Self {
        let max_length_sq = max_length * max_length;
        let length_sq = self.x * self.x + self.y * self.y;
        if length_sq > max_length_sq {
            self * (max_length / length_sq.sqrt())
        } else {
            self
        }
    }
}

fn lerp(x: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
    (x - old_min) / (old_max - old_min) * (new_max - new_min) + new_min
}
