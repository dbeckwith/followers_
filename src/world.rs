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

// enough for a minute of 1000 particles
const HISTORY_MEMORY_CAP: usize = 3600 * 1000 * size_of::<Vec2>();

pub struct World {
    params: Params,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    partners: Vec<[usize; 2]>,
    colors: Vec<Color>,
    history: Vec<Vec<Vec2>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub seed: u64,
    pub particle_count: usize,
    pub particle_color_hue_mid: f32,
    pub particle_color_hue_spread: f32,
    pub particle_color_saturation_mid: f32,
    pub particle_color_saturation_spread: f32,
    pub particle_color_value: f32,
    pub particle_color_alpha: f32,
    pub acc_limit: f32,
}

impl Params {
    fn check(&self) -> Result<()> {
        ensure!(self.particle_count > 2);
        Ok(())
    }

    fn idxs(&self) -> std::ops::Range<usize> {
        0..self.particle_count
    }
}

impl World {
    pub fn new(params: Params) -> Result<Self> {
        params.check()?;

        let Params {
            seed,
            particle_count,
            particle_color_hue_mid,
            particle_color_hue_spread,
            particle_color_saturation_mid,
            particle_color_saturation_spread,
            particle_color_value,
            particle_color_alpha,
            acc_limit,
        } = params;
        info!("world init - 0x{seed:016x}:{particle_count}:2^{acc_limit}");

        let mut seeds = ChaCha20Rng::seed_from_u64(seed)
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
                    (particle_count - 1) as f32,
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
                    particle_color_value,
                    particle_color_alpha,
                )
            })
            .collect::<Vec<_>>());

        let history = vec![positions.clone()];

        Ok(Self {
            params,
            positions,
            velocities,
            partners,
            colors,
            history,
        })
    }

    pub fn update(&mut self) {
        let Self {
            params,
            positions,
            velocities,
            partners,
            colors: _,
            history,
        } = self;
        let Params {
            seed: _,
            particle_count,
            particle_color_hue_mid: _,
            particle_color_hue_spread: _,
            particle_color_saturation_mid: _,
            particle_color_saturation_spread: _,
            particle_color_value: _,
            particle_color_alpha: _,
            acc_limit,
        } = *params;

        let acc_limit = acc_limit.exp2();

        for idx in params.idxs() {
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

        for idx in params.idxs() {
            positions[idx] += velocities[idx];
        }

        if (history.len() + 1) * particle_count * size_of::<Vec2>()
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
            params,
            positions,
            velocities: _,
            partners: _,
            colors,
            history: _,
        } = self;

        let hw = (image.width() as f32) / 2.0;
        let hh = (image.height() as f32) / 2.0;
        for idx in params.idxs() {
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
            params,
            positions: _,
            velocities: _,
            partners: _,
            colors,
            history,
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
        for idx in params.idxs() {
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
