use sas::*;
mod app;
mod bot_handler;
mod game_loop;
mod hud_scoreboard;
mod menu;
mod weapon_handler;

use app::App;
use sas::input::winit_input::WinitInputState;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use std::sync::{Arc, Mutex};
use pollster::FutureExt;
use wgpu::{Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureViewDescriptor};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if !std::path::Path::new("q3-resources").exists() {
            let resource_base = sas::resource_path::get_resource_base_path();
            if resource_base.exists() && resource_base != std::path::PathBuf::from("q3-resources") {
                #[cfg(unix)]
                {
                    let _ = std::os::unix::fs::symlink(&resource_base, "q3-resources");
                }
                #[cfg(windows)]
                {
                    let _ = std::os::windows::fs::symlink_dir(&resource_base, "q3-resources");
                }
            }
        }
    }

    let event_loop = EventLoop::new().unwrap();
    
    #[cfg(target_os = "macos")]
    let window_attributes = winit::window::Window::default_attributes()
        .with_title("SAS III - Still Alive Somehow??")
        .with_visible(false);
    
    #[cfg(not(target_os = "macos"))]
    let window_attributes = winit::window::Window::default_attributes()
        .with_title("SAS III - Still Alive Somehow??")
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
    
    let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

    let wgpu_renderer = Arc::new(Mutex::new(sas::wgpu_renderer::WgpuRenderer::new(window.clone()).block_on().expect("Failed to create wgpu renderer")));
    let mut app = App::new();
    let mut app_initialized = false;
    let mut input_state = WinitInputState::new();
    let size = window.inner_size();
    input_state.window_size = (size.width, size.height);
    
    #[cfg(target_os = "macos")]
    {
        std::thread::sleep(std::time::Duration::from_millis(200));
        window.set_visible(true);
        window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
    }

    let wgpu_renderer_clone = wgpu_renderer.clone();
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                input_state.handle_window_event(event);
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        if let Some(ref mut gl) = app.game_loop {
                            if let Some(ref mut wgpu) = gl.wgpu_renderer {
                                wgpu.resize(*physical_size);
                            }
                        } else {
                            wgpu_renderer_clone.lock().unwrap().resize(*physical_size);
                        }
                        input_state.window_size = (physical_size.width, physical_size.height);
                    }
                    WindowEvent::RedrawRequested => {
                        if !app_initialized {
                            app.initialize().block_on();
                            app_initialized = true;
                            
                            if let Some(ref mut gl) = app.game_loop {
                                let size = window.inner_size();
                                let map_width = gl.game_state.map.width as f32 * 32.0;
                                let map_height = gl.game_state.map.height as f32 * 16.0;
                                
                                if let Some(local_player) = gl.game_state.players.first() {
                                    gl.camera.x = local_player.x - size.width as f32 * 0.5;
                                    gl.camera.y = local_player.y - size.height as f32 * 0.5;
                                    gl.camera.target_x = gl.camera.x;
                                    gl.camera.target_y = gl.camera.y;
                                } else {
                                    gl.camera.x = map_width * 0.5 - size.width as f32 * 0.5;
                                    gl.camera.y = map_height * 0.5 - size.height as f32 * 0.5;
                                    gl.camera.target_x = gl.camera.x;
                                    gl.camera.target_y = gl.camera.y;
                                }
                                println!("[CAMERA_INIT] Set camera to ({}, {})", gl.camera.x, gl.camera.y);
                            }
                        }

                        app.update(&mut input_state, &window);
                        
                        if let Some(ref mut gl) = app.game_loop {
                            let dt = 1.0 / 60.0;
                            gl.game_state.update(dt);
                            
                            let size = window.inner_size();
                            let map_width = gl.game_state.map.width as f32;
                            let map_height = gl.game_state.map.height as f32;
                            
                            if let Some(local_player) = gl.game_state.players.first() {
                                gl.camera.follow_with_size(local_player.x, local_player.y, size.width as f32, size.height as f32);
                            } else {
                                gl.camera.target_x = map_width * 0.5 - size.width as f32 * 0.5;
                                gl.camera.target_y = map_height * 0.5 - size.height as f32 * 0.5;
                            }
                            
                            gl.camera.update_with_size(dt, map_width, map_height, size.width as f32, size.height as f32);
                            if gl.wgpu_renderer.is_none() {
                                let new_renderer = sas::wgpu_renderer::WgpuRenderer::new(window.clone()).block_on().expect("Failed to create wgpu renderer");
                                gl.wgpu_renderer = Some(new_renderer);
                            }
                            if let Some(ref mut wgpu) = gl.wgpu_renderer {
                                if gl.game_state.wgpu_render_context.is_none() {
                                    gl.game_state.init_wgpu_renderer(wgpu);
                                    gl.game_state.load_tile_textures_to_wgpu(wgpu);
                                }
                                
                                gl.game_state.render_wgpu(
                                    wgpu,
                                    gl.camera.x,
                                    gl.camera.y,
                                    gl.camera.zoom,
                                );
                            }
                        } else {
                            if let Some(ref menu_state) = app.menu_state {
                                let mut renderer = wgpu_renderer_clone.lock().unwrap();
                                menu_state.render_wgpu(&mut *renderer);
                            } else {
                                let mut renderer = wgpu_renderer_clone.lock().unwrap();
                                if let Some(frame) = renderer.begin_frame() {
                                    let view = frame.texture.create_view(&TextureViewDescriptor::default());
                                    let mut encoder = renderer.device.create_command_encoder(&CommandEncoderDescriptor {
                                        label: Some("Menu Clear Encoder"),
                                    });
                                    
                                    {
                                        let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                                            label: Some("Menu Clear Pass"),
                                            color_attachments: &[Some(RenderPassColorAttachment {
                                                view: &view,
                                                resolve_target: None,
                                                ops: Operations {
                                                    load: LoadOp::Clear(Color {
                                                        r: 0.07,
                                                        g: 0.09,
                                                        b: 0.11,
                                                        a: 1.0,
                                                    }),
                                                    store: StoreOp::Store,
                                                },
                                            })],
                                            depth_stencil_attachment: None,
                                            occlusion_query_set: None,
                                            timestamp_writes: None,
                                        });
                                    }
                                    
                                    renderer.queue.submit(std::iter::once(encoder.finish()));
                                    renderer.end_frame(frame);
                                }
                            }
                        }
                        window.request_redraw();
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
