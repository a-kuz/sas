use crate::game::map::Map;
use crate::game::tile_shader::TileShaderRenderer;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct TileRenderer {
    pub shader_renderer: TileShaderRenderer,
    pub tile_textures: HashMap<u16, Texture2D>,
    pub texture_scale: f32,
    pub texture_offset_x: f32,
    pub texture_offset_y: f32,
}

impl TileRenderer {
    pub fn new() -> Self {
        Self {
            shader_renderer: TileShaderRenderer::new(),
            tile_textures: HashMap::new(),
            texture_scale: 1.0,
            texture_offset_x: 0.0,
            texture_offset_y: 0.0,
        }
    }

    pub fn render_tiles(&self, map: &Map, camera_x: f32, camera_y: f32, time: f32) {
        const TILE_WIDTH: f32 = 32.0;
        const TILE_HEIGHT: f32 = 16.0;

        let start_x = ((camera_x / TILE_WIDTH).floor() as i32).max(0);
        let end_x =
            (((camera_x + screen_width()) / TILE_WIDTH).ceil() as i32).min(map.width as i32);
        let start_y = ((camera_y / TILE_HEIGHT).floor() as i32).max(0);
        let end_y =
            (((camera_y + screen_height()) / TILE_HEIGHT).ceil() as i32).min(map.height as i32);

        let mut visited = vec![vec![false; map.height]; map.width];

        for x in start_x..end_x {
            for y in start_y..end_y {
                let tile = &map.tiles[x as usize][y as usize];

                if tile.solid && !visited[x as usize][y as usize] {
                    let tex_id = tile.texture_id;
                    let shader_name = tile.shader_name.as_ref();

                    let mut block_width = 1;
                    let mut block_height = 1;

                    while x + block_width < end_x
                        && map.tiles[(x + block_width) as usize][y as usize].solid
                        && map.tiles[(x + block_width) as usize][y as usize].texture_id == tex_id
                        && map.tiles[(x + block_width) as usize][y as usize]
                            .shader_name
                            .as_ref()
                            == shader_name
                        && !visited[(x + block_width) as usize][y as usize]
                    {
                        block_width += 1;
                    }

                    let mut height_ok = true;
                    while y + block_height < end_y && height_ok {
                        for bx in 0..block_width {
                            if !map.tiles[(x + bx) as usize][(y + block_height) as usize].solid
                                || map.tiles[(x + bx) as usize][(y + block_height) as usize]
                                    .texture_id
                                    != tex_id
                                || map.tiles[(x + bx) as usize][(y + block_height) as usize]
                                    .shader_name
                                    .as_ref()
                                    != shader_name
                                || visited[(x + bx) as usize][(y + block_height) as usize]
                            {
                                height_ok = false;
                                break;
                            }
                        }
                        if height_ok {
                            block_height += 1;
                        }
                    }

                    for bx in 0..block_width {
                        for by in 0..block_height {
                            visited[(x + bx) as usize][(y + by) as usize] = true;
                        }
                    }

                    let screen_x = (x as f32 * TILE_WIDTH) - camera_x;
                    let screen_y = (y as f32 * TILE_HEIGHT) - camera_y;
                    let block_pixel_w = block_width as f32 * TILE_WIDTH;
                    let block_pixel_h = block_height as f32 * TILE_HEIGHT;

                    let world_x = x as f32 * TILE_WIDTH + self.texture_offset_x;
                    let world_y = y as f32 * TILE_HEIGHT + self.texture_offset_y;

                    if let Some(shader_name) = shader_name {
                        self.shader_renderer.render_tile_with_shader(
                            shader_name,
                            screen_x,
                            screen_y,
                            block_pixel_w,
                            block_pixel_h,
                            world_x,
                            world_y,
                            time,
                        );
                    } else if let Some(texture) = self.tile_textures.get(&tex_id) {
                        self.render_tiled_texture(
                            texture,
                            screen_x,
                            screen_y,
                            block_pixel_w,
                            block_pixel_h,
                            world_x,
                            world_y,
                        );
                    }
                }
            }
        }
    }

    fn render_tiled_texture(
        &self,
        texture: &Texture2D,
        screen_x: f32,
        screen_y: f32,
        block_pixel_w: f32,
        block_pixel_h: f32,
        world_x: f32,
        world_y: f32,
    ) {
        let base_tex_w = texture.width();
        let base_tex_h = texture.height();

        let scaled_tex_w = base_tex_w * self.texture_scale;
        let scaled_tex_h = base_tex_h * self.texture_scale;

        let start_offset_x = world_x % scaled_tex_w;
        let start_offset_y = world_y % scaled_tex_h;

        let tiles_x = ((block_pixel_w + start_offset_x) / scaled_tex_w).ceil() as i32 + 1;
        let tiles_y = ((block_pixel_h + start_offset_y) / scaled_tex_h).ceil() as i32 + 1;

        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let tile_x_world = world_x + tx as f32 * scaled_tex_w - start_offset_x;
                let tile_y_world = world_y + ty as f32 * scaled_tex_h - start_offset_y;

                let tile_x_screen = tile_x_world - world_x + screen_x;
                let tile_y_screen = tile_y_world - world_y + screen_y;

                let clip_left = (screen_x - tile_x_screen).max(0.0);
                let clip_top = (screen_y - tile_y_screen).max(0.0);
                let clip_right =
                    ((tile_x_screen + scaled_tex_w) - (screen_x + block_pixel_w)).max(0.0);
                let clip_bottom =
                    ((tile_y_screen + scaled_tex_h) - (screen_y + block_pixel_h)).max(0.0);

                let visible_w = scaled_tex_w - clip_left - clip_right;
                let visible_h = scaled_tex_h - clip_top - clip_bottom;

                if visible_w > 0.0 && visible_h > 0.0 {
                    let uv_x = clip_left / scaled_tex_w;
                    let uv_y = clip_top / scaled_tex_h;
                    let uv_w = visible_w / scaled_tex_w;
                    let uv_h = visible_h / scaled_tex_h;

                    draw_texture_ex(
                        texture,
                        tile_x_screen + clip_left,
                        tile_y_screen + clip_top,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(visible_w, visible_h)),
                            source: Some(Rect::new(
                                uv_x * base_tex_w,
                                uv_y * base_tex_h,
                                uv_w * base_tex_w,
                                uv_h * base_tex_h,
                            )),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }
}
