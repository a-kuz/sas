use super::map::{LightSource, Map};
use std::collections::HashMap;

const GRID_CELL_SIZE: f32 = 64.0;

#[derive(Clone)]
pub struct LightGrid {
    pub cell_size: f32,
    pub grid_width: usize,
    pub grid_height: usize,
    pub cells: Vec<Vec<Vec<usize>>>,
    pub map_width_px: f32,
    pub map_height_px: f32,
}

impl LightGrid {
    pub fn new(map: &Map) -> Self {
        let map_width_px = map.width as f32 * 32.0;
        let map_height_px = map.height as f32 * 16.0;
        
        let grid_width = (map_width_px / GRID_CELL_SIZE).ceil() as usize;
        let grid_height = (map_height_px / GRID_CELL_SIZE).ceil() as usize;
        
        let cells = vec![vec![Vec::new(); grid_height]; grid_width];
        
        Self {
            cell_size: GRID_CELL_SIZE,
            grid_width,
            grid_height,
            cells,
            map_width_px,
            map_height_px,
        }
    }
    
    pub fn rebuild(&mut self, lights: &[LightSource], map: &Map) {
        for x in 0..self.grid_width {
            for y in 0..self.grid_height {
                self.cells[x][y].clear();
            }
        }
        
        for (light_idx, light) in lights.iter().enumerate() {
            let min_x = ((light.x - light.radius) / self.cell_size).floor().max(0.0) as usize;
            let max_x = ((light.x + light.radius) / self.cell_size).ceil().min(self.grid_width as f32) as usize;
            let min_y = ((light.y - light.radius) / self.cell_size).floor().max(0.0) as usize;
            let max_y = ((light.y + light.radius) / self.cell_size).ceil().min(self.grid_height as f32) as usize;
            
            for cx in min_x..max_x {
                for cy in min_y..max_y {
                    let cell_center_x = cx as f32 * self.cell_size + self.cell_size * 0.5;
                    let cell_center_y = cy as f32 * self.cell_size + self.cell_size * 0.5;
                    
                    let dx = cell_center_x - light.x;
                    let dy = cell_center_y - light.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    
                    let cell_diagonal = self.cell_size * 1.415;
                    
                    if dist < light.radius + cell_diagonal {
                        if !self.is_completely_occluded(light, cell_center_x, cell_center_y, map) {
                            self.cells[cx][cy].push(light_idx);
                        }
                    }
                }
            }
        }
    }
    
    fn is_completely_occluded(&self, light: &LightSource, cell_x: f32, cell_y: f32, map: &Map) -> bool {
        let dx = cell_x - light.x;
        let dy = cell_y - light.y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist < 10.0 {
            return false;
        }
        
        let steps = (dist / 32.0).min(16.0) as i32;
        let step_x = dx / steps as f32;
        let step_y = dy / steps as f32;
        
        let mut x = light.x;
        let mut y = light.y;
        
        for _ in 0..steps {
            x += step_x;
            y += step_y;
            
            let tile_x = (x / 32.0) as i32;
            let tile_y = (y / 16.0) as i32;
            
            if map.is_solid(tile_x, tile_y) {
                return true;
            }
        }
        
        false
    }
    
    pub fn get_lights_for_screen(&self, camera_x: f32, camera_y: f32, screen_w: f32, screen_h: f32, lights: &[LightSource]) -> Vec<usize> {
        let min_cx = ((camera_x / self.cell_size).floor().max(0.0) as usize).min(self.grid_width.saturating_sub(1));
        let max_cx = (((camera_x + screen_w) / self.cell_size).ceil() as usize).min(self.grid_width);
        let min_cy = ((camera_y / self.cell_size).floor().max(0.0) as usize).min(self.grid_height.saturating_sub(1));
        let max_cy = (((camera_y + screen_h) / self.cell_size).ceil() as usize).min(self.grid_height);
        
        let mut light_set: HashMap<usize, bool> = HashMap::new();
        
        for cx in min_cx..max_cx {
            for cy in min_cy..max_cy {
                for &light_idx in &self.cells[cx][cy] {
                    light_set.insert(light_idx, true);
                }
            }
        }
        
        let mut result: Vec<usize> = light_set.keys().copied().collect();
        
        result.sort_by(|&a, &b| {
            let light_a = &lights[a];
            let light_b = &lights[b];
            
            let center_x = camera_x + screen_w * 0.5;
            let center_y = camera_y + screen_h * 0.5;
            
            let dist_a = (light_a.x - center_x).powi(2) + (light_a.y - center_y).powi(2);
            let dist_b = (light_b.x - center_x).powi(2) + (light_b.y - center_y).powi(2);
            
            let score_a = light_a.intensity * light_a.radius / (1.0 + dist_a.sqrt());
            let score_b = light_b.intensity * light_b.radius / (1.0 + dist_b.sqrt());
            
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        result.truncate(8);
        result
    }
}


