use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NavGraph {
    pub nodes: Vec<NavNode>,
    pub edges: Vec<NavEdge>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NavNode {
    pub id: usize,
    pub x: f32,
    pub y: f32,
    pub node_type: NavNodeType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NavNodeType {
    Ground,
    Platform,
    JumpPad,
    Teleporter,
    ItemLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NavEdge {
    pub from: usize,
    pub to: usize,
    pub edge_type: NavEdgeType,
    pub cost: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NavEdgeType {
    Walk,
    Jump,
    JumpDown,
    FallOff,
    JumpGap,
    AirControl,
    JumpPad,
    Teleport,
    RocketJump,
}

impl NavGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, x: f32, y: f32, node_type: NavNodeType) -> usize {
        let id = self.nodes.len();
        self.nodes.push(NavNode {
            id,
            x,
            y,
            node_type,
        });
        id
    }

    pub fn add_edge(&mut self, from: usize, to: usize, edge_type: NavEdgeType) {
        let from_node = &self.nodes[from];
        let to_node = &self.nodes[to];
        let dx = to_node.x - from_node.x;
        let dy = to_node.y - from_node.y;
        let distance = (dx * dx + dy * dy).sqrt();

        let cost = match edge_type {
            NavEdgeType::Walk => distance,
            NavEdgeType::Jump => distance * 1.2,
            NavEdgeType::JumpDown => distance * 0.8,
            NavEdgeType::FallOff => distance * 0.6,
            NavEdgeType::JumpGap => distance * 1.5,
            NavEdgeType::AirControl => distance * 1.3,
            NavEdgeType::JumpPad => distance * 0.5,
            NavEdgeType::Teleport => 10.0,
            NavEdgeType::RocketJump => distance * 2.0,
        };

        self.edges.push(NavEdge {
            from,
            to,
            edge_type,
            cost,
        });
    }

    pub fn find_path(&self, start_node: usize, goal_node: usize) -> Option<Vec<usize>> {
        if start_node >= self.nodes.len() || goal_node >= self.nodes.len() {
            return None;
        }

        let mut open_set = vec![start_node];
        let mut came_from: HashMap<usize, usize> = HashMap::new();
        let mut g_score: HashMap<usize, f32> = HashMap::new();
        let mut f_score: HashMap<usize, f32> = HashMap::new();

        g_score.insert(start_node, 0.0);
        f_score.insert(start_node, self.heuristic(start_node, goal_node));

        while !open_set.is_empty() {
            let current_idx = open_set
                .iter()
                .enumerate()
                .min_by(|(_, &a), (_, &b)| {
                    f_score
                        .get(&a)
                        .unwrap_or(&f32::MAX)
                        .partial_cmp(f_score.get(&b).unwrap_or(&f32::MAX))
                        .unwrap()
                })
                .map(|(idx, _)| idx)
                .unwrap();

            let current = open_set.remove(current_idx);

            if current == goal_node {
                return Some(self.reconstruct_path(came_from, current));
            }

            for edge in &self.edges {
                if edge.from != current {
                    continue;
                }

                let neighbor = edge.to;
                let tentative_g_score = g_score.get(&current).unwrap_or(&f32::MAX) + edge.cost;

                if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&f32::MAX) {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g_score);
                    f_score.insert(
                        neighbor,
                        tentative_g_score + self.heuristic(neighbor, goal_node),
                    );

                    if !open_set.contains(&neighbor) {
                        open_set.push(neighbor);
                    }
                }
            }
        }

        None
    }

    fn heuristic(&self, from: usize, to: usize) -> f32 {
        let from_node = &self.nodes[from];
        let to_node = &self.nodes[to];
        let dx = to_node.x - from_node.x;
        let dy = to_node.y - from_node.y;
        (dx * dx + dy * dy).sqrt()
    }

    fn reconstruct_path(&self, came_from: HashMap<usize, usize>, current: usize) -> Vec<usize> {
        let mut path = vec![current];
        let mut current = current;

        while let Some(&prev) = came_from.get(&current) {
            path.push(prev);
            current = prev;
        }

        path.reverse();
        path
    }

    pub fn find_nearest_node(&self, x: f32, y: f32) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = (a.x - x).powi(2) + (a.y - y).powi(2);
                let dist_b = (b.x - x).powi(2) + (b.y - y).powi(2);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(idx, _)| idx)
    }

    pub fn render(&self, camera_x: f32, camera_y: f32, show_edges: bool) {
        if show_edges {
            for edge in &self.edges {
                let from = &self.nodes[edge.from];
                let to = &self.nodes[edge.to];

                let color = match edge.edge_type {
                    NavEdgeType::Walk => Color::from_rgba(0, 255, 0, 100),
                    NavEdgeType::Jump => Color::from_rgba(255, 255, 0, 100),
                    NavEdgeType::JumpDown => Color::from_rgba(0, 255, 255, 100),
                    NavEdgeType::FallOff => Color::from_rgba(100, 200, 255, 100),
                    NavEdgeType::JumpGap => Color::from_rgba(255, 128, 0, 100),
                    NavEdgeType::AirControl => Color::from_rgba(128, 0, 255, 100),
                    NavEdgeType::JumpPad => Color::from_rgba(0, 128, 255, 100),
                    NavEdgeType::Teleport => Color::from_rgba(255, 0, 255, 100),
                    NavEdgeType::RocketJump => Color::from_rgba(255, 0, 0, 100),
                };

                draw_line(
                    from.x - camera_x,
                    from.y - camera_y,
                    to.x - camera_x,
                    to.y - camera_y,
                    2.0,
                    color,
                );
            }
        }

        for node in &self.nodes {
            let color = match node.node_type {
                NavNodeType::Ground => Color::from_rgba(0, 255, 0, 200),
                NavNodeType::Platform => Color::from_rgba(0, 200, 255, 200),
                NavNodeType::JumpPad => Color::from_rgba(255, 128, 0, 200),
                NavNodeType::Teleporter => Color::from_rgba(255, 0, 255, 200),
                NavNodeType::ItemLocation => Color::from_rgba(255, 255, 0, 200),
            };

            draw_circle(node.x - camera_x, node.y - camera_y, 4.0, color);
        }
    }

    pub fn render_path(&self, path: &[usize], camera_x: f32, camera_y: f32) {
        for i in 0..path.len().saturating_sub(1) {
            let from = &self.nodes[path[i]];
            let to = &self.nodes[path[i + 1]];

            draw_line(
                from.x - camera_x,
                from.y - camera_y,
                to.x - camera_x,
                to.y - camera_y,
                3.0,
                Color::from_rgba(255, 255, 255, 255),
            );
        }

        for &node_id in path {
            let node = &self.nodes[node_id];
            draw_circle(
                node.x - camera_x,
                node.y - camera_y,
                5.0,
                Color::from_rgba(255, 255, 255, 255),
            );
        }
    }
    
    pub fn find_connected_components(&self) -> Vec<Vec<usize>> {
        let mut visited = vec![false; self.nodes.len()];
        let mut components = Vec::new();
        
        for start_node in 0..self.nodes.len() {
            if visited[start_node] {
                continue;
            }
            
            let mut component = Vec::new();
            let mut queue = vec![start_node];
            visited[start_node] = true;
            
            while let Some(current) = queue.pop() {
                component.push(current);
                
                for edge in &self.edges {
                    if edge.from == current && !visited[edge.to] {
                        visited[edge.to] = true;
                        queue.push(edge.to);
                    }
                    if edge.to == current && !visited[edge.from] {
                        visited[edge.from] = true;
                        queue.push(edge.from);
                    }
                }
            }
            
            components.push(component);
        }
        
        components
    }
}

