#![warn(rust_2018_idioms, clippy::all)]
#![deny(clippy::correctness)]

use nannou::{
    image::{DynamicImage, GenericImage, GenericImageView, Pixel},
    prelude::*,
};
use nannou_egui::Egui;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    window_id: WindowId,
    egui: Egui,
    params: Params,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    partners: Vec<[usize; 2]>,
    colors: Vec<Hsv>,
    image: DynamicImage,
}

struct Params {
    particle_count: usize,
    seed: u64,
}

impl Params {
    fn check(&self) {
        assert!(self.particle_count > 2);
    }

    fn idxs(&self) -> std::ops::Range<usize> {
        0..self.particle_count
    }
}

fn model(app: &App) -> Model {
    let seed: u64 = thread_rng().gen();
    let seed: u64 = 0xb0ddde9d83a31516;
    eprintln!("SEED: 0x{seed:016x}");

    let params = Params {
        particle_count: 1000,
        seed,
    };
    params.check();

    let window_id = app
        .new_window()
        .title("FOLLOWERS")
        .size(1920, 1080)
        .view(view)
        .raw_event(raw_event)
        .event(event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    let mut seeds = ChaCha20Rng::seed_from_u64(seed)
        .sample_iter(rand::distributions::Standard);

    macro_rules! with_rng {
        (| $rng:ident | $body:expr) => {{
            #[allow(unused, unused_mut)]
            let mut $rng = ChaCha20Rng::seed_from_u64(seeds.next().unwrap());
            $body
        }};
    }

    let positions = with_rng!(|rng| params
        .idxs()
        .map(|idx| {
            let t = map_range(idx, 0, params.particle_count - 1, 0.0, 2.0 * PI);
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
            hsv(
                rng.gen_range(0.0 / 360.0..=240.0 / 360.0),
                rng.gen_range(0.20..=0.40),
                0.80,
            )
        })
        .collect::<Vec<_>>());

    let image = DynamicImage::new_rgba8(
        window.rect().w() as u32,
        window.rect().h() as u32,
    );

    Model {
        window_id,
        egui,
        params,
        positions,
        velocities,
        partners,
        colors,
        image,
    }
}

fn raw_event(
    _app: &App,
    Model { egui, .. }: &mut Model,
    event: &nannou::winit::event::WindowEvent<'_>,
) {
    egui.handle_raw_event(event);
}

fn event(
    _app: &App,
    Model {
        window_id: _,
        egui,
        params,
        positions: _,
        velocities: _,
        partners: _,
        colors: _,
        image,
    }: &mut Model,
    event: WindowEvent,
) {
    let gui = egui.ctx();
    if gui.wants_pointer_input() {
        match &event {
            WindowEvent::MouseMoved(_)
            | WindowEvent::MousePressed(_)
            | WindowEvent::MouseReleased(_)
            | WindowEvent::MouseEntered
            | WindowEvent::MouseExited
            | WindowEvent::MouseWheel(..)
            | WindowEvent::Touch(_)
            | WindowEvent::TouchPressure(_) => return,
            _ => {},
        }
    }
    if gui.wants_keyboard_input() {
        match &event {
            WindowEvent::KeyPressed(_)
            | WindowEvent::KeyReleased(_)
            | WindowEvent::ReceivedCharacter(_) => return,
            _ => {},
        }
    }
    match event {
        WindowEvent::KeyPressed(Key::Space) => {
            let Params {
                particle_count,
                seed,
            } = params;
            let path = format!("out/{particle_count}-0x{seed:016x}.png");
            eprintln!("saving image to {path}");
            let mut image = image.to_rgba8();
            let background = Hsva::new(0.0 / 360.0, 0.00, 0.00, 1.00);
            for px in image.pixels_mut() {
                let px_ = *px;
                *px = hsva_to_image_rgba(background);
                px.blend(&px_);
            }
            if let Err(error) = image.save(&path) {
                eprintln!("error saving image to {path}: {error}");
            }
        },
        event => {},
    }
}

fn update(
    _app: &App,
    Model {
        window_id: _,
        egui,
        params,
        positions,
        velocities,
        partners,
        colors,
        image,
    }: &mut Model,
    update: Update,
) {
    egui.set_elapsed_time(update.since_start);
    let gui = egui.begin_frame();

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

    for idx in params.idxs() {
        let pos = positions[idx];
        let w = image.width();
        let h = image.height();
        let x = pos.x + w as f32 / 2.0;
        let y = pos.y + h as f32 / 2.0;
        if x < 0.0 || x >= w as f32 || y < 0.0 || y >= h as f32 {
            return;
        }
        let x = x as u32;
        let y = y as u32;

        let color = colors[idx];
        let color = Hsva::new(color.hue, color.saturation, color.value, 0.06);

        image.blend_pixel(x, y, hsva_to_image_rgba(color));
    }
}

fn view(
    app: &App,
    Model {
        window_id: _,
        egui,
        params: _,
        positions: _,
        velocities: _,
        partners: _,
        colors: _,
        image,
    }: &Model,
    frame: Frame<'_>,
) {
    let draw = app.draw();

    draw.background().hsv(0.0 / 360.0, 0.00, 0.00);
    draw.texture(&wgpu::Texture::from_image(app, image));

    draw.to_frame(app, &frame).unwrap();
    egui.draw_to_frame(&frame).unwrap();
}

fn hsva_to_image_rgba(hsva: Hsva) -> nannou::image::Rgba<u8> {
    let (r, g, b, a) = Rgba::from(hsva).into_components();
    nannou::image::Rgba([
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    ])
}
