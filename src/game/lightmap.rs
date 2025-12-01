use super::map::{LightSource, Map};
use macroquad::prelude::*;

const LIGHTMAP_SCALE: usize = 2;

pub struct Lightmap {
    pub texture: Texture2D,
    pub width: usize,
    pub height: usize,
}

impl Lightmap {
    pub fn new(map: &Map, static_lights: &[LightSource], ambient: f32) -> Self {
        let tile_w = 32 * LIGHTMAP_SCALE;
        let tile_h = 16 * LIGHTMAP_SCALE;
        let width = map.width * tile_w;
        let height = map.height * tile_h;

        let mut pixels = vec![0u8; width * height * 4];

        for py in 0..height {
            for px in 0..width {
                let world_x = px as f32 / LIGHTMAP_SCALE as f32;
                let world_y = py as f32 / LIGHTMAP_SCALE as f32;

                let mut r_accum = 0.0f32;
                let mut g_accum = 0.0f32;
                let mut b_accum = 0.0f32;

                for light in static_lights {
                    let dx = world_x - light.x;
                    let dy = world_y - light.y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < light.radius {
                        let shadow = Self::calculate_shadow(world_x, world_y, light, map);

                        if shadow > 0.01 {
                            let attenuation = (1.0 - dist / light.radius).powf(1.6);
                            let contribution = attenuation * shadow * light.intensity;

                            r_accum += (light.r as f32 / 255.0) * contribution;
                            g_accum += (light.g as f32 / 255.0) * contribution;
                            b_accum += (light.b as f32 / 255.0) * contribution;
                        }
                    }
                }

                r_accum += ambient;
                g_accum += ambient;
                b_accum += ambient;

                let idx = (py * width + px) * 4;
                pixels[idx] = (r_accum.min(1.0) * 255.0) as u8;
                pixels[idx + 1] = (g_accum.min(1.0) * 255.0) as u8;
                pixels[idx + 2] = (b_accum.min(1.0) * 255.0) as u8;
                pixels[idx + 3] = 255;
            }
        }

        let image = Image {
            bytes: pixels,
            width: width as u16,
            height: height as u16,
        };
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Linear);

        Self {
            texture,
            width,
            height,
        }
    }

    fn calculate_shadow(world_x: f32, world_y: f32, light: &LightSource, map: &Map) -> f32 {
        let tile_w = 32.0;
        let tile_h = 16.0;
        let start_tx = world_x / tile_w;
        let start_ty = world_y / tile_h;
        let end_tx = light.x / tile_w;
        let end_ty = light.y / tile_h;

        let dir_tx = end_tx - start_tx;
        let dir_ty = end_ty - start_ty;
        let total_world_dx = light.x - world_x;
        let total_world_dy = light.y - world_y;
        let total_world_dist =
            (total_world_dx * total_world_dx + total_world_dy * total_world_dy).sqrt();
        if total_world_dist < 2.0 {
            return 1.0;
        }

        let step_x = if dir_tx > 0.0 { 1 } else { -1 };
        let step_y = if dir_ty > 0.0 { 1 } else { -1 };

        let inv_dx = if dir_tx.abs() < 1e-6 {
            f32::INFINITY
        } else {
            1.0 / dir_tx.abs()
        };
        let inv_dy = if dir_ty.abs() < 1e-6 {
            f32::INFINITY
        } else {
            1.0 / dir_ty.abs()
        };

        let mut cell_x = start_tx.floor() as i32;
        let mut cell_y = start_ty.floor() as i32;
        let frac_x = start_tx - start_tx.floor();
        let frac_y = start_ty - start_ty.floor();

        let mut t_max_x = if step_x > 0 {
            (1.0 - frac_x) * inv_dx
        } else {
            frac_x * inv_dx
        };
        let mut t_max_y = if step_y > 0 {
            (1.0 - frac_y) * inv_dy
        } else {
            frac_y * inv_dy
        };
        let t_delta_x = inv_dx;
        let t_delta_y = inv_dy;

        let mut t = 0.0f32;
        let max_t = 1.0f32;
        let mut iter = 0;
        while t <= max_t && iter < 8192 {
            if map.is_solid(cell_x, cell_y) {
                let traveled = t * total_world_dist;
                let softness = (traveled / 80.0).min(1.0);
                return 0.02 + softness * 0.08;
            }

            if t_max_x < t_max_y {
                cell_x += step_x;
                t = t_max_x;
                t_max_x += t_delta_x;
            } else {
                cell_y += step_y;
                t = t_max_y;
                t_max_y += t_delta_y;
            }
            iter += 1;
        }

        1.0
    }

    pub fn rebuild(&mut self, map: &Map, static_lights: &[LightSource], ambient: f32) {
        *self = Self::new(map, static_lights, ambient);
    }
}
