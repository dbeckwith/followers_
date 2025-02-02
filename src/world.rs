use crate::{
    color::Color,
    image::Image,
    math::{lerp, Vec2},
};
use anyhow::{ensure, Result};
use log::info;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::{f32::consts::PI, fmt, ops::Range};

// enough for a minute of 1000 particles
const HISTORY_MEMORY_CAP: usize = 3600 * 1000 * size_of::<Vec2>();

pub struct World {
    idxs: Range<usize>,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    partners: Vec<[usize; 2]>,
    colors: Vec<Color>,
    history: Vec<Vec<Vec2>>,
    acc_limit: i32,
}

#[derive(Debug, Clone)]
pub struct SimParams {
    pub seed: Seed,
    pub particle_count: usize,
    pub acc_limit: i32,
}

#[derive(Debug, Clone)]
pub struct DisplayParams {
    pub particle_color_hue_mid: f32,
    pub particle_color_hue_spread: f32,
    pub particle_color_saturation_mid: f32,
    pub particle_color_saturation_spread: f32,
    pub particle_color_value: f32,
    pub particle_color_alpha: f32,
}

impl World {
    pub fn new(
        sim_params: &SimParams,
        display_params: &DisplayParams,
    ) -> Result<Self> {
        let SimParams {
            seed,
            particle_count,
            acc_limit,
        } = sim_params;
        let DisplayParams {
            particle_color_hue_mid,
            particle_color_hue_spread,
            particle_color_saturation_mid,
            particle_color_saturation_spread,
            particle_color_value,
            particle_color_alpha,
        } = display_params;
        ensure!(*particle_count >= 3);
        info!(
            "world init - {}:{particle_count}:2^{acc_limit}",
            seed.fmt_hash()
        );

        let idxs = 0..*particle_count;

        let mut seeds = ChaCha20Rng::seed_from_u64(seed.as_hash())
            .sample_iter(rand::distributions::Standard);

        macro_rules! with_rng {
            (| $rng:ident | $body:expr) => {{
                #[allow(unused, unused_mut)]
                let mut $rng =
                    ChaCha20Rng::seed_from_u64(seeds.next().unwrap());
                $body
            }};
        }

        let positions = with_rng!(|rng| idxs
            .clone()
            .map(|idx| {
                let t = lerp(
                    idx as f32,
                    0.0,
                    *particle_count as f32,
                    0.0,
                    2.0 * PI,
                );
                let r = rng.gen_range(9.0..=10.0);
                Vec2::new(r * t.cos(), r * t.sin())
            })
            .collect::<Vec<_>>());

        let velocities = with_rng!(|rng| idxs
            .clone()
            .map(|_idx| Vec2::new(0.0, 0.0))
            .collect::<Vec<_>>());

        let partners = with_rng!(|rng| idxs
            .clone()
            .map(|idx| {
                let i = idx;
                let mut j = rng.gen_range(idxs.clone());
                while j == i {
                    j = rng.gen_range(idxs.clone());
                }
                let mut k = rng.gen_range(idxs.clone());
                while k == i || k == j {
                    k = rng.gen_range(idxs.clone());
                }
                [j, k]
            })
            .collect::<Vec<_>>());

        let colors = with_rng!(|rng| idxs
            .clone()
            .map(|_idx| {
                Color::hsva(
                    rng.gen_range(
                        particle_color_hue_mid - particle_color_hue_spread / 2.0
                            ..=particle_color_hue_mid
                                + particle_color_hue_spread / 2.0,
                    ),
                    rng.gen_range(
                        particle_color_saturation_mid
                            - particle_color_saturation_spread / 2.0
                            ..=particle_color_saturation_mid
                                + particle_color_saturation_spread / 2.0,
                    ),
                    *particle_color_value,
                    *particle_color_alpha,
                )
            })
            .collect::<Vec<_>>());

        let history = vec![positions.clone()];

        Ok(Self {
            idxs,
            positions,
            velocities,
            partners,
            colors,
            history,
            acc_limit: *acc_limit,
        })
    }

    pub fn update(&mut self) {
        let Self {
            idxs,
            positions,
            velocities,
            partners,
            colors: _,
            history,
            acc_limit,
        } = self;

        let acc_limit = (*acc_limit as f32).exp2();

        for idx in idxs.clone() {
            let pos = positions[idx];
            let [p1, p2] = partners[idx];
            let p1 = positions[p1];
            let p2 = positions[p2];
            let vel = &mut velocities[idx];

            let p_dist_sq = p2.distance_squared(p1);
            let t = if p_dist_sq == 0.0 {
                1.0
            } else {
                ((pos - p1).dot(p2 - p1) / p_dist_sq).max(1.0)
            };
            let target_pos = p2 * t + p1 * (1.0 - t);

            let acc = target_pos - pos;
            let acc = acc.clamp_length_max(acc_limit);
            *vel += acc;
            *vel = vel.clamp_length_max(1.0);
        }

        for idx in idxs.clone() {
            positions[idx] += velocities[idx];
        }

        if (history.len() + 1) * idxs.len() * size_of::<Vec2>()
            > HISTORY_MEMORY_CAP
        {
            // pop oldest
            // SAFETY: history is never empty since it starts off containing the
            // initial positions
            history.swap_remove(0);
            history.rotate_left(1);
        }
        history.push(positions.clone());
    }

    pub fn render(&self, image: &mut Image) {
        let Self {
            idxs,
            positions,
            velocities: _,
            partners: _,
            colors,
            history: _,
            acc_limit: _,
        } = self;

        let hw = (image.width() as f32) / 2.0;
        let hh = (image.height() as f32) / 2.0;
        for idx in idxs.clone() {
            let pos = positions[idx];
            let x = pos.x + hw;
            let y = pos.y + hh;
            let color = colors[idx];
            image.draw_particle(x, y, color);
        }
    }

    pub fn generate_svg(&self, background_color: Color) -> String {
        use std::fmt::Write;

        let Self {
            idxs,
            positions: _,
            velocities: _,
            partners: _,
            colors,
            history,
            acc_limit: _,
        } = self;

        let mut s = String::new();

        macro_rules! w {
            ($($args:tt)*) => (write!(&mut s, $($args)*).unwrap())
        }
        macro_rules! wln {
            ($($args:tt)*) => (writeln!(&mut s, $($args)*).unwrap())
        }

        let (x, y, x1, y1) = history
            .iter()
            .flatten()
            .copied()
            .fold(None, |max, Vec2 { x, y }| {
                Some(max.map_or(
                    (x, y, x, y),
                    |(min_x, min_y, max_x, max_y)| {
                        (x.min(min_x), y.min(min_y), x.max(max_x), y.max(max_y))
                    },
                ))
            })
            .unwrap_or((0.0, 0.0, 0.0, 0.0));
        let w = x1 - x;
        let h = y1 - y;
        let bg = background_color.fmt_hex();
        wln!(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        w!(r#"<svg"#);
        w!(r#" xmlns="http://www.w3.org/2000/svg""#);
        w!(r#" width="{w}""#);
        w!(r#" height="{h}""#);
        w!(r#" viewBox="{x} {y} {w} {h}""#);
        w!(r#" style="background: #{bg};""#);
        wln!(r#">"#);
        for idx in idxs.clone() {
            let color = colors[idx].fmt_hex();
            w!(r#"  <path"#);
            w!(r#" fill="none""#);
            w!(r##" stroke="#{color}""##);
            w!(r#" stroke-linejoin="round""#);
            w!(r#" d=""#);
            let mut cmd = 'M';
            for step in history {
                let Vec2 { x, y } = step[idx];
                w!(r#" {cmd} {x} {y}"#);
                cmd = 'L';
            }
            w!(r#"""#);
            wln!(r#" />"#);
        }
        wln!(r#"</svg>"#);

        s
    }
}

impl SimParams {
    pub fn file_name(&self, ext: &str) -> String {
        let Self {
            seed,
            particle_count,
            acc_limit,
        } = self;
        let seed = seed.fmt_hash();
        format!("{particle_count}-2_{acc_limit}-{seed}.{ext}")
    }
}

#[derive(Debug, Clone)]
pub struct Seed {
    s: String,
    n: u64,
}

impl Seed {
    pub fn from_str(seed: String) -> Self {
        let n = hash_seed(&seed);
        Self { s: seed, n }
    }

    pub fn from_hash(hash: u64) -> Self {
        let s = unhash_seed(hash);
        Self { s, n: hash }
    }

    pub fn as_str(&self) -> &str {
        self.s.as_str()
    }

    pub fn as_hash(&self) -> u64 {
        self.n
    }

    pub fn fmt_hash(&self) -> SeedHash {
        SeedHash(self.n)
    }
}

fn hash_seed(seed: &str) -> u64 {
    seed.strip_prefix("0x")
        .filter(|seed| seed.len() == 16)
        .and_then(|seed| u64::from_str_radix(seed, 16).ok())
        .unwrap_or_else(|| {
            let md5::Digest([b0, b1, b2, b3, b4, b5, b6, b7, ..]) =
                md5::compute(seed);
            u64::from_le_bytes([b0, b1, b2, b3, b4, b5, b6, b7])
        })
}

fn unhash_seed(hash: u64) -> String {
    SeedHash(hash).to_string()
}

pub struct SeedHash(u64);

impl fmt::Display for SeedHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(hash) = self;
        write!(f, "0x{hash:016x}")
    }
}
