use super::map::Map;
use super::nav_graph::{NavEdgeType, NavGraph, NavNodeType};
use std::collections::HashSet;

const TILE_WIDTH: f32 = 32.0;
const TILE_HEIGHT: f32 = 16.0;
const GRAVITY: f32 = 0.056;
const JUMP_FORCE: f32 = -2.9;
const MAX_SPEED_GROUND: f32 = 5.0;
const MAX_SPEED_AIR: f32 = 6.0;
const MAX_FALL_SPEED: f32 = 5.0;
const GRID_SIZE: f32 = 32.0;
const PLAYER_HITBOX_WIDTH: f32 = 56.0;
const PLAYER_HITBOX_HEIGHT: f32 = 96.0;
const EDGE_DIRECTION_EPS: f32 = 4.0;
const LINE_CHECK_STEP: f32 = 8.0;

pub struct NavGraphGenerator {
    map: Map,
    graph: NavGraph,
    ground_nodes: Vec<(f32, f32)>,
    jump_pad_node_indices: Vec<usize>,
    teleporter_node_indices: Vec<usize>,
}

impl NavGraphGenerator {
    pub fn new(map: Map) -> Self {
        Self {
            map,
            graph: NavGraph::new(),
            ground_nodes: Vec::new(),
            jump_pad_node_indices: Vec::new(),
            teleporter_node_indices: Vec::new(),
        }
    }

    pub fn generate(mut self) -> NavGraph {
        self.generate_ground_nodes();
        self.generate_jump_pad_nodes();
        self.generate_teleporter_nodes();
        self.generate_item_nodes();
        
        println!("Generated {} nodes", self.graph.nodes.len());
        
        self.connect_item_nodes();
        self.generate_walk_edges();
        self.generate_jump_edges();
        self.generate_fall_off_edges();
        self.generate_jump_down_edges();
        self.generate_jump_gap_edges();
        self.generate_air_control_edges();
        self.generate_jump_pad_edges();
        self.generate_teleporter_edges();
        self.generate_rocket_jump_edges();
        
        println!("Generated {} edges", self.graph.edges.len());
        
        let components = self.graph.find_connected_components();
        println!("Found {} connected components", components.len());
        for (i, component) in components.iter().enumerate() {
            println!("  Component {}: {} nodes", i, component.len());
        }
        
        if components.len() > 1 {
            println!("Warning: Graph has {} disconnected components!", components.len());
        }
        
        self.graph
    }

    fn update_best_target(best: &mut Option<(usize, f32)>, candidate: usize, distance: f32) {
        if let Some((_, best_distance)) = best {
            if distance >= *best_distance - 0.001 {
                return;
            }
        }
        *best = Some((candidate, distance));
    }

    fn add_selected_edges(
        &mut self,
        from: usize,
        edge_type: NavEdgeType,
        selections: [Option<(usize, f32)>; 3],
    ) -> usize {
        let mut added = 0;
        let mut used = HashSet::new();
        for (target, _) in selections.into_iter().flatten() {
            if used.insert(target) {
                let edge_kind = edge_type.clone();
                self.graph.add_edge(from, target, edge_kind);
                added += 1;
            }
        }
        added
    }

    fn has_clear_edge(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist <= 0.0 {
            return true;
        }
        let steps = (dist / LINE_CHECK_STEP).ceil() as i32;
        if steps <= 1 {
            return true;
        }
        for step in 1..steps {
            let t = step as f32 / steps as f32;
            let x = x1 + dx * t;
            let y = y1 + dy * t;
            if self.is_solid_at(x, y) || self.is_solid_at(x, y - PLAYER_HITBOX_HEIGHT * 0.5) {
                return false;
            }
        }
        true
    }

    fn generate_ground_nodes(&mut self) {
        let mut visited = HashSet::new();

        for x in 0..self.map.width {
            for y in 0..self.map.height {
                if self.is_ground_tile(x, y) {
                    let world_x = x as f32 * TILE_WIDTH + TILE_WIDTH / 2.0;
                    let world_y = y as f32 * TILE_HEIGHT;

                    let key = (x, y);

                    if !visited.contains(&key) {
                        visited.insert(key);
                        self.graph.add_node(world_x, world_y, NavNodeType::Ground);
                        self.ground_nodes.push((world_x, world_y));
                    }
                }
            }
        }
    }

    fn is_ground_tile(&self, x: usize, y: usize) -> bool {
        if x >= self.map.width || y >= self.map.height {
            return false;
        }

        if y >= self.map.height - 1 || y == 0 {
            return false;
        }

        if !self.map.tiles[x][y].solid {
            return false;
        }

        if self.map.tiles[x][y - 1].solid {
            return false;
        }

        true
    }

    fn generate_jump_pad_nodes(&mut self) {
        for jump_pad in &self.map.jumppads {
            let node_id = self.graph
                .add_node(jump_pad.x + jump_pad.width / 2.0, jump_pad.y, NavNodeType::JumpPad);
            self.jump_pad_node_indices.push(node_id);
        }
    }

    fn generate_teleporter_nodes(&mut self) {
        for teleporter in &self.map.teleporters {
            let node_id = self.graph
                .add_node(teleporter.x + teleporter.width / 2.0, teleporter.y + teleporter.height / 2.0, NavNodeType::Teleporter);
            self.teleporter_node_indices.push(node_id);
        }
    }

    fn generate_item_nodes(&mut self) {
        for item in &self.map.items {
            self.graph
                .add_node(item.x, item.y, NavNodeType::ItemLocation);
        }
    }
    
    fn connect_item_nodes(&mut self) {
        let num_nodes = self.graph.nodes.len();
        let mut edges_to_add = Vec::new();
        
        for i in 0..num_nodes {
            let node_i = &self.graph.nodes[i];
            if node_i.node_type != NavNodeType::ItemLocation {
                continue;
            }
            
            let item_x = node_i.x;
            let item_y = node_i.y;
            
            let mut _found_close = false;
            
            for j in 0..num_nodes {
                if i == j {
                    continue;
                }
                
                let node_j = &self.graph.nodes[j];
                
                if node_j.node_type != NavNodeType::Ground && node_j.node_type != NavNodeType::Platform {
                    continue;
                }
                
                let dx = (item_x - node_j.x).abs();
                let dy = (item_y - node_j.y).abs();
                
                if dx < GRID_SIZE * 2.0 && dy < TILE_HEIGHT * 2.0 {
                    edges_to_add.push((i, j, NavEdgeType::Walk));
                    edges_to_add.push((j, i, NavEdgeType::Walk));
                    _found_close = true;
                }
            }
            
        }
        
        for (from, to, edge_type) in edges_to_add {
            self.graph.add_edge(from, to, edge_type);
        }
    }

    fn generate_walk_edges(&mut self) {
        for i in 0..self.graph.nodes.len() {
            for j in 0..self.graph.nodes.len() {
                if i == j {
                    continue;
                }

                let node_a = &self.graph.nodes[i];
                let node_b = &self.graph.nodes[j];

                if node_a.node_type != NavNodeType::Ground
                    && node_a.node_type != NavNodeType::Platform
                {
                    continue;
                }
                if node_b.node_type != NavNodeType::Ground
                    && node_b.node_type != NavNodeType::Platform
                {
                    continue;
                }

                let dx = (node_b.x - node_a.x).abs();
                let dy = (node_b.y - node_a.y).abs();

                if dx <= GRID_SIZE * 2.0 && dy <= TILE_HEIGHT {
                    if self.can_walk_between(node_a.x, node_a.y, node_b.x, node_b.y) {
                        self.graph.add_edge(i, j, NavEdgeType::Walk);
                    }
                }
            }
        }
    }

    fn can_walk_between(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let dy = (y2 - y1).abs();
        if dy > TILE_HEIGHT {
            return false;
        }
        
        let y_check = y1.max(y2);
        let steps = 16;
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let x = x1 + (x2 - x1) * t;
            
            if self.is_solid_at(x, y_check - PLAYER_HITBOX_HEIGHT * 0.5) {
                return false;
            }
            
            let has_floor = self.is_solid_at(x, y_check + TILE_HEIGHT);
            if !has_floor {
                return false;
            }
        }
        true
    }
    
    fn has_ground_below(&self, x: f32, y: f32) -> bool {
        for check_y in 1..6 {
            let test_y = y + check_y as f32 * TILE_HEIGHT;
            if self.is_solid_at(x, test_y) {
                return true;
            }
        }
        false
    }

    fn generate_fall_off_edges(&mut self) {
        let mut falloff_count = 0;
        let mut checked = 0;
        for i in 0..self.graph.nodes.len() {
            let node_a = &self.graph.nodes[i];
            if node_a.node_type != NavNodeType::Ground && node_a.node_type != NavNodeType::Platform {
                continue;
            }
            
            let has_floor_ahead_left = self.is_solid_at(node_a.x - TILE_WIDTH, node_a.y + TILE_HEIGHT);
            let has_floor_ahead_right = self.is_solid_at(node_a.x + TILE_WIDTH, node_a.y + TILE_HEIGHT);
            
            let mut best_vertical = None;
            let mut best_left = None;
            let mut best_right = None;
            
            for j in 0..self.graph.nodes.len() {
                if i == j {
                    continue;
                }
                let node_b = &self.graph.nodes[j];
                let dx = node_b.x - node_a.x;
                let dy = node_b.y - node_a.y;
                if dy <= TILE_HEIGHT * 1.5 || dy > 800.0 {
                    continue;
                }
                if dx.abs() > 300.0 {
                    continue;
                }
                
                if dx < 0.0 && has_floor_ahead_left {
                    continue;
                }
                if dx > 0.0 && has_floor_ahead_right {
                    continue;
                }
                
                checked += 1;
                if self.can_fall_to_node(node_a.x, node_a.y, node_b.x, node_b.y) {
                    let distance = (dx * dx + dy * dy).sqrt();
                    if dx < -EDGE_DIRECTION_EPS {
                        Self::update_best_target(&mut best_left, j, distance);
                    } else if dx > EDGE_DIRECTION_EPS {
                        Self::update_best_target(&mut best_right, j, distance);
                    } else {
                        Self::update_best_target(&mut best_vertical, j, distance);
                    }
                }
            }
            falloff_count += self.add_selected_edges(
                i,
                NavEdgeType::FallOff,
                [best_vertical, best_left, best_right],
            );
        }
        println!(
            "Checked {} potential FallOff pairs, generated {} edges",
            checked,
            falloff_count
        );
    }

    fn can_fall_to_node(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let dx = x2 - x1;
        let dy = y2 - y1;
        
        if dy <= TILE_HEIGHT {
            return false;
        }
        
        let move_speed = if dx.abs() < 16.0 { 0.0 } else { MAX_SPEED_AIR.min(dx.abs() / 60.0) };
        let vel_x_dir = if dx.abs() < 1.0 { 0.0 } else { dx.signum() };
        
        let mut vel_y = 0.0;
        let mut pos_x = x1;
        let mut pos_y = y1;
        
        for _frame in 0..500 {
            pos_x += vel_x_dir * move_speed;
            
            let ground_below = self.is_solid_at(pos_x, pos_y + TILE_HEIGHT);
            
            if !ground_below {
                vel_y += GRAVITY;
                
                if vel_y > -1.0 && vel_y < 0.0 {
                    vel_y /= 1.0 + 0.11;
                }
                if vel_y > 0.0 && vel_y < 5.0 {
                    vel_y *= 1.0 + 0.1;
                }
                
                if vel_y > MAX_FALL_SPEED {
                    vel_y = MAX_FALL_SPEED;
                }
                
                pos_y += vel_y;
            }
            
            if self.is_solid_at(pos_x, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) {
                return false;
            }
            
            if _frame % 5 == 0 {
                let hitbox_left = pos_x - PLAYER_HITBOX_WIDTH / 2.0;
                let hitbox_right = pos_x + PLAYER_HITBOX_WIDTH / 2.0;
                
                if self.is_solid_at(hitbox_left, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) ||
                   self.is_solid_at(hitbox_right, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) {
                    return false;
                }
            }
            
            let tolerance_x = TILE_WIDTH * 2.0;
            let tolerance_y = TILE_HEIGHT * 2.0;
            
            if (pos_x - x2).abs() < tolerance_x && (pos_y - y2).abs() < tolerance_y {
                return true;
            }
            
            if pos_y > y2 + TILE_HEIGHT * 6.0 {
                return false;
            }
        }

        false
    }

    fn generate_jump_edges(&mut self) {
        for i in 0..self.graph.nodes.len() {
            let node_a = &self.graph.nodes[i];
            if node_a.node_type != NavNodeType::Ground && node_a.node_type != NavNodeType::Platform {
                continue;
            }
            let mut best_vertical = None;
            let mut best_left = None;
            let mut best_right = None;
            for j in 0..self.graph.nodes.len() {
                if i == j {
                    continue;
                }
                let node_b = &self.graph.nodes[j];
                let dy = node_a.y - node_b.y;
                if dy <= 0.0 {
                    continue;
                }
                if self.can_jump_to(node_a.x, node_a.y, node_b.x, node_b.y) {
                    let dx = node_b.x - node_a.x;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if dx < -EDGE_DIRECTION_EPS {
                        Self::update_best_target(&mut best_left, j, distance);
                    } else if dx > EDGE_DIRECTION_EPS {
                        Self::update_best_target(&mut best_right, j, distance);
                    } else {
                        Self::update_best_target(&mut best_vertical, j, distance);
                    }
                }
            }
            self.add_selected_edges(i, NavEdgeType::Jump, [best_vertical, best_left, best_right]);
        }
    }

    fn can_jump_to(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let dx = x2 - x1;
        let dy = y1 - y2;

        if dy < 0.0 {
            return false;
        }

        let move_speed = MAX_SPEED_AIR.min(dx.abs() / 40.0);
        let vel_x_dir = if dx == 0.0 { 0.0 } else { dx.signum() };
        
        let mut vel_y = JUMP_FORCE;
        let mut pos_x = x1;
        let mut pos_y = y1;
        
        for _frame in 0..300 {
            vel_y += GRAVITY;
            
            if vel_y > -1.0 && vel_y < 0.0 {
                vel_y /= 1.0 + 0.11;
            }
            if vel_y > 0.0 && vel_y < 5.0 {
                vel_y *= 1.0 + 0.1;
            }
            
            if vel_y > MAX_FALL_SPEED {
                vel_y = MAX_FALL_SPEED;
            }
            
            pos_x += vel_x_dir * move_speed;
            pos_y += vel_y;
            
            if self.is_solid_at(pos_x, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) {
                return false;
            }
            
            if _frame % 5 == 0 {
                let hitbox_left = pos_x - PLAYER_HITBOX_WIDTH / 2.0;
                let hitbox_right = pos_x + PLAYER_HITBOX_WIDTH / 2.0;
                
                if self.is_solid_at(hitbox_left, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) ||
                   self.is_solid_at(hitbox_right, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) {
                    return false;
                }
            }
            
            let tolerance_x = TILE_WIDTH * 0.75;
            let tolerance_y = TILE_HEIGHT * 1.25;
            
            if (pos_x - x2).abs() < tolerance_x && (pos_y - y2).abs() < tolerance_y {
                return true;
            }
            
            if pos_y > y2 + TILE_HEIGHT * 3.0 {
                return false;
            }
        }

        false
    }

    fn is_solid_at(&self, x: f32, y: f32) -> bool {
        let tile_x = (x / TILE_WIDTH).floor() as i32;
        let tile_y = (y / TILE_HEIGHT).floor() as i32;

        if tile_x < 0
            || tile_x >= self.map.width as i32
            || tile_y < 0
            || tile_y >= self.map.height as i32
        {
            return true;
        }

        self.map.tiles[tile_x as usize][tile_y as usize].solid
    }

    fn generate_jump_down_edges(&mut self) {
        for i in 0..self.graph.nodes.len() {
            let node_a = &self.graph.nodes[i];
            if node_a.node_type != NavNodeType::Ground && node_a.node_type != NavNodeType::Platform {
                continue;
            }
            let mut best_vertical = None;
            let mut best_left = None;
            let mut best_right = None;
            for j in 0..self.graph.nodes.len() {
                if i == j {
                    continue;
                }
                let node_b = &self.graph.nodes[j];
                let dx = node_b.x - node_a.x;
                let dy = node_b.y - node_a.y;
                if dy <= TILE_HEIGHT * 2.0 || dy > 600.0 {
                    continue;
                }
                if dx.abs() <= 64.0 || dx.abs() > 450.0 {
                    continue;
                }
                if self.can_jump_down_to(node_a.x, node_a.y, node_b.x, node_b.y) {
                    let distance = (dx * dx + dy * dy).sqrt();
                    if dx < -EDGE_DIRECTION_EPS {
                        Self::update_best_target(&mut best_left, j, distance);
                    } else if dx > EDGE_DIRECTION_EPS {
                        Self::update_best_target(&mut best_right, j, distance);
                    } else {
                        Self::update_best_target(&mut best_vertical, j, distance);
                    }
                }
            }
            self.add_selected_edges(i, NavEdgeType::JumpDown, [best_vertical, best_left, best_right]);
        }
    }

    fn can_jump_down_to(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let dx = x2 - x1;
        let move_speed = MAX_SPEED_AIR.min(dx.abs() / 50.0);
        let vel_x_dir = dx.signum();
        
        let mut vel_y = JUMP_FORCE * 0.5;
        let mut pos_x = x1;
        let mut pos_y = y1;
        
        for _frame in 0..400 {
            vel_y += GRAVITY;
            
            if vel_y > -1.0 && vel_y < 0.0 {
                vel_y /= 1.0 + 0.11;
            }
            if vel_y > 0.0 && vel_y < 5.0 {
                vel_y *= 1.0 + 0.1;
            }
            
            if vel_y > MAX_FALL_SPEED {
                vel_y = MAX_FALL_SPEED;
            }
            
            pos_x += vel_x_dir * move_speed;
            pos_y += vel_y;
            
            if self.is_solid_at(pos_x, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) {
                return false;
            }
            
            if _frame % 5 == 0 {
                let hitbox_left = pos_x - PLAYER_HITBOX_WIDTH / 2.0;
                let hitbox_right = pos_x + PLAYER_HITBOX_WIDTH / 2.0;
                
                if self.is_solid_at(hitbox_left, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) ||
                   self.is_solid_at(hitbox_right, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) {
                    return false;
                }
            }
            
            let tolerance_x = TILE_WIDTH;
            let tolerance_y = TILE_HEIGHT * 1.5;
            
            if (pos_x - x2).abs() < tolerance_x && (pos_y - y2).abs() < tolerance_y {
                return true;
            }
            
            if pos_y > y2 + TILE_HEIGHT * 4.0 {
                return false;
            }
        }

        false
    }

    fn generate_jump_gap_edges(&mut self) {
        let num_nodes = self.graph.nodes.len();
        let existing_edges: std::collections::HashSet<(usize, usize)> = self.graph.edges
            .iter()
            .filter(|e| e.edge_type == NavEdgeType::Walk)
            .map(|e| (e.from, e.to))
            .collect();
        
        for i in 0..num_nodes {
            let node_a = &self.graph.nodes[i];
            if node_a.node_type != NavNodeType::Ground && node_a.node_type != NavNodeType::Platform {
                continue;
            }
            let mut best_vertical = None;
            let mut best_left = None;
            let mut best_right = None;
            for j in 0..num_nodes {
                if i == j {
                    continue;
                }
                
                if existing_edges.contains(&(i, j)) {
                    continue;
                }
                
                let node_b = &self.graph.nodes[j];
                let dx = node_b.x - node_a.x;
                let dy = (node_b.y - node_a.y).abs();
                if dx.abs() > 200.0 {
                    continue;
                }
                if dy > TILE_HEIGHT * 2.0 {
                    continue;
                }
                
                let distance = (dx * dx + dy * dy).sqrt();
                if dx < -EDGE_DIRECTION_EPS {
                    Self::update_best_target(&mut best_left, j, distance);
                } else if dx > EDGE_DIRECTION_EPS {
                    Self::update_best_target(&mut best_right, j, distance);
                } else {
                    Self::update_best_target(&mut best_vertical, j, distance);
                }
            }
            self.add_selected_edges(i, NavEdgeType::JumpGap, [best_vertical, best_left, best_right]);
        }
    }

    fn can_jump_gap(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let dx = x2 - x1;
        let dy = y2 - y1;
        
        if dx.abs() < GRID_SIZE * 2.0 {
            return false;
        }
        
        if dx.abs() > 450.0 {
            return false;
        }
        
        if dy.abs() > TILE_HEIGHT * 2.0 {
            return false;
        }

        let move_speed = MAX_SPEED_AIR.min(dx.abs() / 40.0);
        let vel_x_dir = if dx == 0.0 { 0.0 } else { dx.signum() };
        
        let mut vel_y = JUMP_FORCE;
        let mut pos_x = x1;
        let mut pos_y = y1;
        
        for _frame in 0..300 {
            vel_y += GRAVITY;
            
            if vel_y > -1.0 && vel_y < 0.0 {
                vel_y /= 1.0 + 0.11;
            }
            if vel_y > 0.0 && vel_y < 5.0 {
                vel_y *= 1.0 + 0.1;
            }
            
            if vel_y > MAX_FALL_SPEED {
                vel_y = MAX_FALL_SPEED;
            }
            
            pos_x += vel_x_dir * move_speed;
            pos_y += vel_y;
            
            if self.is_solid_at(pos_x, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) {
                return false;
            }
            
            if _frame % 5 == 0 {
                let hitbox_left = pos_x - PLAYER_HITBOX_WIDTH / 2.0;
                let hitbox_right = pos_x + PLAYER_HITBOX_WIDTH / 2.0;
                
                if self.is_solid_at(hitbox_left, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) ||
                   self.is_solid_at(hitbox_right, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) {
                    return false;
                }
            }
            
            let tolerance_x = TILE_WIDTH * 0.75;
            let tolerance_y = TILE_HEIGHT * 1.25;
            
            if (pos_x - x2).abs() < tolerance_x && (pos_y - y2).abs() < tolerance_y {
                return true;
            }
            
            if pos_y > y2 + TILE_HEIGHT * 3.0 {
                return false;
            }
        }

        false
    }

    fn generate_air_control_edges(&mut self) {
        for i in 0..self.graph.nodes.len() {
            let node_a = &self.graph.nodes[i];
            if node_a.node_type != NavNodeType::Ground && node_a.node_type != NavNodeType::Platform {
                continue;
            }
            let mut best_vertical = None;
            let mut best_left = None;
            let mut best_right = None;
            for j in 0..self.graph.nodes.len() {
                if i == j {
                    continue;
                }
                let node_b = &self.graph.nodes[j];
                let dx = node_b.x - node_a.x;
                let dy = node_b.y - node_a.y;
                if dy <= 70.0 || dy > 600.0 {
                    continue;
                }
                if dx.abs() > 150.0 {
                    continue;
                }
                if self.can_air_control_to(node_a.x, node_a.y, node_b.x, node_b.y) {
                    let distance = (dx * dx + dy * dy).sqrt();
                    if dx < -EDGE_DIRECTION_EPS {
                        Self::update_best_target(&mut best_left, j, distance);
                    } else if dx > EDGE_DIRECTION_EPS {
                        Self::update_best_target(&mut best_right, j, distance);
                    } else {
                        Self::update_best_target(&mut best_vertical, j, distance);
                    }
                }
            }
            self.add_selected_edges(i, NavEdgeType::AirControl, [best_vertical, best_left, best_right]);
        }
    }

    fn can_air_control_to(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let dx = x2 - x1;
        let move_speed = (dx.abs() / 100.0).min(MAX_SPEED_AIR * 0.5);
        let vel_x_dir = if dx == 0.0 { 0.0 } else { dx.signum() };
        
        let mut vel_y = 0.0;
        let mut pos_x = x1;
        let mut pos_y = y1;
        
        for _frame in 0..400 {
            vel_y += GRAVITY;
            
            if vel_y > -1.0 && vel_y < 0.0 {
                vel_y /= 1.0 + 0.11;
            }
            if vel_y > 0.0 && vel_y < 5.0 {
                vel_y *= 1.0 + 0.1;
            }
            
            if vel_y > MAX_FALL_SPEED {
                vel_y = MAX_FALL_SPEED;
            }
            
            pos_x += vel_x_dir * move_speed;
            pos_y += vel_y;
            
            if self.is_solid_at(pos_x, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) {
                return false;
            }
            
            if _frame % 5 == 0 {
                let hitbox_left = pos_x - PLAYER_HITBOX_WIDTH / 2.0;
                let hitbox_right = pos_x + PLAYER_HITBOX_WIDTH / 2.0;
                
                if self.is_solid_at(hitbox_left, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) ||
                   self.is_solid_at(hitbox_right, pos_y - PLAYER_HITBOX_HEIGHT * 0.5) {
                    return false;
                }
            }
            
            let tolerance_x = TILE_WIDTH;
            let tolerance_y = TILE_HEIGHT * 1.5;
            
            if (pos_x - x2).abs() < tolerance_x && (pos_y - y2).abs() < tolerance_y {
                return true;
            }
            
            if pos_y > y2 + TILE_HEIGHT * 4.0 {
                return false;
            }
        }

        false
    }

    fn generate_jump_pad_edges(&mut self) {
        const GRAVITY: f32 = 0.056;
        const MAX_FALL_SPEED: f32 = 5.0;
        
        for (idx, jump_pad) in self.map.jumppads.iter().enumerate() {
            let jump_pad_node_id = self.jump_pad_node_indices[idx];

            for i in 0..self.graph.nodes.len() {
                if i == jump_pad_node_id {
                    continue;
                }

                let node = &self.graph.nodes[i];
                
                let close_x = (node.x - jump_pad.x - jump_pad.width / 2.0).abs() < jump_pad.width / 2.0 + PLAYER_HITBOX_WIDTH + GRID_SIZE;
                let close_y = (node.y - jump_pad.y).abs() < PLAYER_HITBOX_HEIGHT + TILE_HEIGHT * 2.0;

                if close_x && close_y {
                    self.graph.add_edge(i, jump_pad_node_id, NavEdgeType::Walk);
                }
            }

            let initial_vel_x = jump_pad.force_x;
            let initial_vel_y = jump_pad.force_y;
            let start_x = jump_pad.x + jump_pad.width / 2.0;
            let start_y = jump_pad.y;

            println!("Jump pad at ({:.0}, {:.0}) with force ({:.1}, {:.1})",
                     jump_pad.x, jump_pad.y, jump_pad.force_x, jump_pad.force_y);

            for target_node_idx in 0..self.graph.nodes.len() {
                if target_node_idx == jump_pad_node_id {
                    continue;
                }

                let target_node = &self.graph.nodes[target_node_idx];
                let dx_to_target = target_node.x - start_x;
                let dy_to_target = target_node.y - start_y;
                
                if dy_to_target > TILE_HEIGHT * 2.0 {
                    continue;
                }
                
                if dx_to_target.abs() > 400.0 {
                    continue;
                }
                
                let air_control = if dx_to_target.abs() > 32.0 {
                    dx_to_target.signum() * (dx_to_target.abs() / 80.0).min(MAX_SPEED_AIR * 0.7)
                } else {
                    0.0
                };
                
                let vel_x = initial_vel_x;
                let mut vel_y = initial_vel_y;
                let mut pos_x = start_x;
                let mut pos_y = start_y;
                
                for _frame in 0..500 {
                    vel_y += GRAVITY;
                    
                    if vel_y > -1.0 && vel_y < 0.0 {
                        vel_y /= 1.0 + 0.11;
                    }
                    if vel_y > 0.0 && vel_y < 5.0 {
                        vel_y *= 1.0 + 0.1;
                    }
                    
                    if vel_y > MAX_FALL_SPEED {
                        vel_y = MAX_FALL_SPEED;
                    }
                    
                    pos_x += vel_x + air_control;
                    pos_y += vel_y;
                    
                    let tolerance_x = TILE_WIDTH * 2.0;
                    let tolerance_y = TILE_HEIGHT * 2.0;
                    
                    if (pos_x - target_node.x).abs() < tolerance_x && (pos_y - target_node.y).abs() < tolerance_y {
                        self.graph.add_edge(jump_pad_node_id, target_node_idx, NavEdgeType::JumpPad);
                        break;
                    }
                    
                    if pos_y > start_y + TILE_HEIGHT * 10.0 {
                        break;
                    }
                }
            }
        }
    }

    fn generate_teleporter_edges(&mut self) {
        if self.map.teleporters.is_empty() {
            return;
        }

        for (idx, teleporter) in self.map.teleporters.iter().enumerate() {
            let teleporter_node_id = self.teleporter_node_indices[idx];

            for i in 0..self.graph.nodes.len() {
                if i == teleporter_node_id {
                    continue;
                }

                let node = &self.graph.nodes[i];
                
                let player_left = node.x - PLAYER_HITBOX_WIDTH / 2.0;
                let player_right = node.x + PLAYER_HITBOX_WIDTH / 2.0;
                let player_top = node.y - PLAYER_HITBOX_HEIGHT;
                let player_bottom = node.y;
                
                let tp_left = teleporter.x;
                let tp_right = teleporter.x + teleporter.width;
                let tp_top = teleporter.y;
                let tp_bottom = teleporter.y + teleporter.height;
                
                let _x_overlap = player_right >= tp_left && player_left <= tp_right;
                let _y_overlap = player_bottom >= tp_top && player_top <= tp_bottom;
                
                let close_x = (node.x - teleporter.x - teleporter.width / 2.0).abs() < teleporter.width / 2.0 + PLAYER_HITBOX_WIDTH + GRID_SIZE;
                let close_y = (node.y - teleporter.y - teleporter.height / 2.0).abs() < teleporter.height / 2.0 + PLAYER_HITBOX_HEIGHT + TILE_HEIGHT * 2.0;

                if close_x && close_y {
                    self.graph
                        .add_edge(i, teleporter_node_id, NavEdgeType::Walk);
                }
            }

            for i in 0..self.graph.nodes.len() {
                if i == teleporter_node_id {
                    continue;
                }

                let node = &self.graph.nodes[i];
                let dist = ((node.x - teleporter.dest_x).powi(2)
                    + (node.y - teleporter.dest_y).powi(2))
                    .sqrt();

                if dist < 96.0 {
                    self.graph
                        .add_edge(teleporter_node_id, i, NavEdgeType::Teleport);
                }
            }
        }
    }

    fn generate_rocket_jump_edges(&mut self) {
        for i in 0..self.graph.nodes.len() {
            let node_a = &self.graph.nodes[i];
            if node_a.node_type != NavNodeType::Ground && node_a.node_type != NavNodeType::Platform {
                continue;
            }
            let mut best_vertical = None;
            let mut best_left = None;
            let mut best_right = None;
            for j in 0..self.graph.nodes.len() {
                if i == j {
                    continue;
                }
                let node_b = &self.graph.nodes[j];
                let dy = node_a.y - node_b.y;
                if dy <= TILE_HEIGHT * 8.0 {
                    continue;
                }
                if self.can_rocket_jump_to(node_a.x, node_a.y, node_b.x, node_b.y) {
                    let dx = node_b.x - node_a.x;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if dx < -EDGE_DIRECTION_EPS {
                        Self::update_best_target(&mut best_left, j, distance);
                    } else if dx > EDGE_DIRECTION_EPS {
                        Self::update_best_target(&mut best_right, j, distance);
                    } else {
                        Self::update_best_target(&mut best_vertical, j, distance);
                    }
                }
            }
            self.add_selected_edges(i, NavEdgeType::RocketJump, [best_vertical, best_left, best_right]);
        }
    }

    fn can_rocket_jump_to(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let steps = 25;
        let dx = x2 - x1;
        let dy = y1 - y2;
        
        let rocket_jump_height = TILE_HEIGHT * 15.0;

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let x = x1 + dx * t;
            let y = y1 - (dy * t + rocket_jump_height * (t - t * t) * 4.0);

            if self.is_solid_at(x, y) {
                return false;
            }
        }

        true
    }
}

