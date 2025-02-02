use std::{fs, iter};
use egui::{FontDefinitions, Ui};
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform::{Platform, PlatformDescriptor};
use rfd::FileDialog;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let event_loop = EventLoop::new();
    env_logger::init();

    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("IDK IDE")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: 1920,
            height: 1080
        })
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false
    })).unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
            label: None
        },
        None
    )).unwrap();

    let size = window.inner_size();
    let surface_format = surface.get_supported_formats(&adapter)[0];
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &surface_config);

    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width,
        physical_height: size.height,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
    });

    let mut egui_rpass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);

    let mut name = String::new();
    let mut text_body = String::new();
    let mut open_modal = false;

    let _ = event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        platform.handle_event(&event);

        match event {
            Event::RedrawRequested(_) => {
                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => {
                        return;
                    }
                    Err(e) => {
                        eprint!("Dropped frame with error {}", e);
                        return;
                    }
                };

                let output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                platform.begin_frame();

                let context = platform.context();

                egui::CentralPanel::default().show(&context, |ui| {
                    ui.label("Enter your name");
                    ui.text_edit_singleline(&mut name);

                    if ui.button("Greet me!").clicked() {
                        println!("Clicked");
                    }

                    ui.text_edit_multiline(&mut text_body);
                });

                if open_modal {
                    egui::Window::new("Open File")
                        .collapsible(false)
                        .resizable(false)
                        .show(&context, |ui| {
                            open_file_modal(ui, &mut open_modal, &mut text_body);
                        });

                }

                egui::TopBottomPanel::top("menu_bar").show(&context, |ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Open File").clicked() {
                                open_modal = true;
                                ui.close_menu();
                            }
                            if ui.button("Organize windows").clicked() {
                                ui.ctx().memory().reset_areas();
                                ui.close_menu();
                            }
                            if ui
                                .button("Reset egui memory")
                                .on_hover_text("Forget scroll, positions, sizes etc")
                                .clicked()
                            {
                                *ui.ctx().memory() = Default::default();
                                ui.close_menu();
                            }
                        });
                    });
                });

                let full_output = platform.end_frame(Some(&window));
                let paint_jobs = platform.context().tessellate(full_output.shapes);

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder")
                });

                let screen_descriptor = ScreenDescriptor {
                    physical_width: surface_config.width,
                    physical_height: surface_config.height,
                    scale_factor: window.scale_factor() as f32,
                };

                let tdelta = full_output.textures_delta;

                egui_rpass
                    .add_textures(&device, &queue, &tdelta)
                    .expect("Add texture ok");
                egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

                egui_rpass
                    .execute(
                        &mut encoder,
                        &output_view,
                        &paint_jobs,
                        &screen_descriptor,
                        Some(wgpu::Color::BLACK),
                    )
                    .unwrap();

                queue.submit(iter::once(encoder.finish()));

                output_frame.present();

                egui_rpass
                    .remove_textures(tdelta).expect("remove texture ok");
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            Event::WindowEvent { event, window_id } => {
                match event {
                    WindowEvent::KeyboardInput { device_id, input, is_synthetic} => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(key),
                                ..
                            } => {
                                match key {
                                    VirtualKeyCode::A => {

                                    }
                                    VirtualKeyCode::S => {

                                    }
                                    VirtualKeyCode::D => {

                                    }
                                    VirtualKeyCode::W => {

                                    }
                                    VirtualKeyCode::Escape => {
                                        *control_flow = ControlFlow::Exit;
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    },
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            _ => {
            }
        }
    });
}

fn open_file_modal(ui: &mut Ui, show_modal: &mut bool, text_body: &mut String) {
    ui.label("Select file to open...");

    if ui.button("Browser...").clicked() {
        if let Some(path) = FileDialog::new().pick_file() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    *text_body = content;
                }
                Err(err) => {
                    eprintln!("Failed to read from file {}", err);
                }
            }
        }

        *show_modal = false;
    }

    if ui.button("Cancel").clicked() {
        *show_modal = false;
    }
}
