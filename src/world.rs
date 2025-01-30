use crate::{
    color::Color,
    image::Image,
    math::{lerp, Vec2},
};
use anyhow::{ensure, Result};
use log::info;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::f32::consts::PI;

pub struct World {
    params: Params,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    partners: Vec<[usize; 2]>,
    colors: Vec<Color>,
}

pub struct Params {
    pub particle_count: usize,
    pub seed: u64,
}

impl Params {
    pub fn new(particle_count: usize, seed: u64) -> Result<Self> {
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
    pub fn new() -> Self {
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

    pub fn update(&mut self, image: &mut Image) {
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
