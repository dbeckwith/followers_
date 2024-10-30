#![warn(rust_2018_idioms, clippy::all)]
#![deny(clippy::correctness)]

use nannou::{
    image::{DynamicImage, GenericImage, GenericImageView},
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
    particles: Vec<Particle>,
    image: DynamicImage,
}

const NUM_PARTNERS: usize = 2;

#[derive(Debug)]
struct Particle {
    idx: usize,
    partners: [usize; NUM_PARTNERS],
    pos: Vec2,
    vel: Vec2,
    color: Hsv,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .title("FOLLOWERS")
        .size(800, 500)
        .view(view)
        .raw_event(raw_event)
        .event(event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    // let seed: u64 = thread_rng().gen();
    let seed: u64 = 0x10f5bb69e5e71738;
    eprintln!("SEED: 0x{seed:016x}");
    let mut rng = ChaCha20Rng::seed_from_u64(0);
    let num_particles = 1000;
    assert!(num_particles > NUM_PARTNERS);
    let particles = (0..num_particles)
        .map(|idx| Particle {
            idx,
            partners: {
                let i = idx;
                let mut j = rng.gen_range(0..num_particles);
                while j == i {
                    j = rng.gen_range(0..num_particles);
                }
                let mut k = rng.gen_range(0..num_particles);
                while k == i || k == j {
                    k = rng.gen_range(0..num_particles);
                }
                [j, k]
            },
            pos: {
                let t = map_range(idx, 0, num_particles - 1, 0.0, 2.0 * PI);
                let r = rng.gen_range(0.0..=100.0);
                Vec2::new(r * t.cos(), r * t.sin())
            },
            vel: Vec2::new(0.0, 0.0),
            color: hsv(
                rng.gen_range(0.0 / 360.0..=60.0 / 360.0),
                rng.gen_range(0.20..=0.40),
                0.60,
            ),
        })
        .collect::<Vec<_>>();

    let image = DynamicImage::new_rgba8(
        app.main_window().rect().w() as u32,
        app.main_window().rect().h() as u32,
    );

    Model {
        window_id,
        egui,
        particles,
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
    app: &App,
    Model {
        window_id,
        egui,
        particles,
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
            // TODO: save screenshot
            // app.main_window().capture_frame(path);
        },
        event => {},
    }
}

fn update(
    app: &App,
    Model {
        window_id,
        egui,
        particles,
        image,
    }: &mut Model,
    update: Update,
) {
    egui.set_elapsed_time(update.since_start);
    let gui = egui.begin_frame();

    for idx in 0..particles.len() {
        let [p1, p2] = particles[idx].partners;
        // SAFETY: particles are initialized with valid partner indexes that are
        // distinct from each other and the main particle's index
        let [p, p1, p2] = unsafe {
            get_many_unchecked_mut(particles.as_mut_slice(), [idx, p1, p2])
        };
        let p1 = &*p1;
        let p2 = &*p2;

        let t = (p.pos - p1.pos).dot(p2.pos - p1.pos)
            / p2.pos.distance_squared(p1.pos);
        let t = t.max(1.0);
        let target_pos = p2.pos * t + p1.pos * (1.0 - t);

        let acc = target_pos - p.pos;
        let acc = acc.clamp_length_max(0.5);
        p.vel += acc;
        p.vel = p.vel.clamp_length_max(1.0);
    }

    for p in particles.iter_mut() {
        p.pos += p.vel;
    }

    for particle in particles {
        particle.render(image);
    }
}

impl Particle {
    fn render(&self, image: &mut DynamicImage) {
        let w = image.width();
        let h = image.height();
        let x = self.pos.x + w as f32 / 2.0;
        let y = self.pos.y + h as f32 / 2.0;
        if x < 0.0 || x >= w as f32 || y < 0.0 || y >= h as f32 {
            return;
        }
        let x = x as u32;
        let y = y as u32;
        let (r, g, b, a) = Rgba::from(Hsva::new(
            self.color.hue,
            self.color.saturation,
            self.color.value,
            0.06,
        ))
        .into_components();
        image.blend_pixel(
            x,
            y,
            nannou::image::Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                (a * 255.0) as u8,
            ]),
        );
    }
}

fn view(
    app: &App,
    Model {
        window_id,
        egui,
        particles,
        image,
    }: &Model,
    frame: Frame<'_>,
) {
    let draw = app.draw();

    draw.background().hsv(0.0 / 360.0, 0.00, 1.00);
    draw.texture(&wgpu::Texture::from_image(app, image));

    draw.to_frame(app, &frame).unwrap();
    egui.draw_to_frame(&frame).unwrap();
}

/// safety: indexes must be in-bounds and distinct
// based on https://github.com/rust-lang/rust/blob/1.79.0/library/core/src/slice/mod.rs#L4460
unsafe fn get_many_unchecked_mut<const N: usize, T>(
    self_: &mut [T],
    indices: [usize; N],
) -> [&mut T; N] {
    use std::mem::MaybeUninit;
    let slice: *mut T = self_.as_mut_ptr();
    let mut arr: MaybeUninit<[&mut T; N]> = MaybeUninit::uninit();
    let arr_ptr = arr.as_mut_ptr();

    // SAFETY: We expect `indices` to contain disjunct values that are
    // in bounds of `self`.
    unsafe {
        for i in 0..N {
            let idx = *indices.get_unchecked(i);
            *(*arr_ptr).get_unchecked_mut(i) = &mut *slice.add(idx);
        }
        arr.assume_init()
    }
}
