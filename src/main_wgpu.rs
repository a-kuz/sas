use sas::*;
mod app;
mod bot_handler;
mod game_loop;
mod hud_scoreboard;
mod menu;
mod weapon_handler;

use app::App;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::sync::Arc;
use pollster::FutureExt;

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
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("SAS III - Still Alive Somehow??")
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            .build(&event_loop)
            .unwrap(),
    );

    let mut renderer = sas::wgpu_renderer::WgpuRenderer::new(window.clone()).block_on()
        .expect("Failed to create wgpu renderer");

    let mut app = App::new();
    let mut app_initialized = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        renderer.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                if !app_initialized {
                    app.initialize().block_on();
                    app_initialized = true;
                }

                if let Some(frame) = renderer.begin_frame() {
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = renderer
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                    {
                        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
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

                window.request_redraw();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

