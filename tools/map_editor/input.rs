use macroquad::prelude::*;
use super::tools::{EditorTool, SelectedObject};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputContext {
    Global,
    DrawTool,
    EraseTool,
    LightTool,
    BackgroundTool,
    ObjectSelected,
}

pub struct InputHandler {
    pub zoom_changed: bool,
    pub texture_scale_changed: bool,
    pub bg_scale_changed: bool,
    pub bg_alpha_changed: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            zoom_changed: false,
            texture_scale_changed: false,
            bg_scale_changed: false,
            bg_alpha_changed: false,
        }
    }
    
    pub fn reset_flags(&mut self) {
        self.zoom_changed = false;
        self.texture_scale_changed = false;
        self.bg_scale_changed = false;
        self.bg_alpha_changed = false;
    }
    
    pub fn handle_zoom(&mut self, zoom: &mut f32) -> bool {
        let is_ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        let mouse_wheel_y = mouse_wheel().1;
        
        if is_ctrl {
            if mouse_wheel_y != 0.0 {
                let zoom_factor = 1.1;
                if mouse_wheel_y > 0.0 {
                    *zoom *= zoom_factor;
                } else {
                    *zoom /= zoom_factor;
                }
                *zoom = zoom.max(0.1).min(5.0);
                self.zoom_changed = true;
                return true;
            }
            
            if is_key_pressed(KeyCode::Equal) {
                *zoom = (*zoom * 1.1).min(5.0);
                self.zoom_changed = true;
                return true;
            }
            if is_key_pressed(KeyCode::Minus) {
                *zoom = (*zoom / 1.1).max(0.1);
                self.zoom_changed = true;
                return true;
            }
            if is_key_pressed(KeyCode::Key0) {
                *zoom = 1.0;
                self.zoom_changed = true;
                return true;
            }
        }
        false
    }
    
    pub fn handle_texture_scale(&mut self, texture_scale: &mut f32, context: InputContext) -> bool {
        if context != InputContext::DrawTool {
            return false;
        }
        
        let is_shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        
        if is_shift {
            if is_key_pressed(KeyCode::LeftBracket) {
                *texture_scale = (*texture_scale - 0.1).max(0.1);
                self.texture_scale_changed = true;
                return true;
            }
            if is_key_pressed(KeyCode::RightBracket) {
                *texture_scale = (*texture_scale + 0.1).min(10.0);
                self.texture_scale_changed = true;
                return true;
            }
        }
        false
    }
    
    pub fn handle_background_controls(&mut self, 
        bg_alpha: &mut f32, 
        bg_scale: &mut f32, 
        context: InputContext
    ) -> bool {
        if context != InputContext::BackgroundTool {
            return false;
        }
        
        let is_ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        
        if !is_ctrl {
            if is_key_pressed(KeyCode::Equal) {
                *bg_scale = (*bg_scale + 0.1).min(10.0);
                self.bg_scale_changed = true;
                return true;
            }
            if is_key_pressed(KeyCode::Minus) {
                *bg_scale = (*bg_scale - 0.1).max(0.1);
                self.bg_scale_changed = true;
                return true;
            }
            
            if is_key_pressed(KeyCode::LeftBracket) {
                *bg_alpha = (*bg_alpha - 0.1).max(0.0);
                self.bg_alpha_changed = true;
                return true;
            }
            if is_key_pressed(KeyCode::RightBracket) {
                *bg_alpha = (*bg_alpha + 0.1).min(1.0);
                self.bg_alpha_changed = true;
                return true;
            }
        }
        false
    }
    
    pub fn handle_light_radius_scroll(&self, 
        lights: &mut Vec<crate::map::LightSource>,
        world_x: f32,
        world_y: f32,
        context: InputContext
    ) -> bool {
        if context != InputContext::LightTool {
            return false;
        }
        
        let is_ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        let scroll = if !is_ctrl { mouse_wheel().1 } else { 0.0 };
        
        if scroll != 0.0 {
            let mut changed = false;
            for light in lights {
                let dx = light.x - world_x;
                let dy = light.y - world_y;
                if dx * dx + dy * dy < 400.0 {
                    light.radius = (light.radius + scroll * 20.0).max(50.0).min(2000.0);
                    changed = true;
                }
            }
            return changed;
        }
        false
    }
}

pub fn get_current_context(
    current_tool: EditorTool,
    selected_object: &Option<SelectedObject>
) -> InputContext {
    if selected_object.is_some() {
        return InputContext::ObjectSelected;
    }
    
    match current_tool {
        EditorTool::Draw => InputContext::DrawTool,
        EditorTool::Erase => InputContext::EraseTool,
        EditorTool::Light => InputContext::LightTool,
        EditorTool::Background => InputContext::BackgroundTool,
        _ => InputContext::Global,
    }
}
