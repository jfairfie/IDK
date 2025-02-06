mod actions;

use std::{fs, iter};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use egui::{FontDefinitions, Ui};
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform::{Platform, PlatformDescriptor};
use rfd::FileDialog;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use IDK::FileState;

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

    let mut search = String::new();
    let mut text_body = String::new();
    let mut open_modal = false;
    let mut search_open = false;
    let mut pressed_timer: u8 = 0;

    let mut file_state = Mutex::new(FileState::new());

    let mut pressed_keys: HashSet<VirtualKeyCode> = HashSet::new();

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

                if pressed_keys.len() > 1 && pressed_timer == 4 {
                    process_multiple_keys(&mut pressed_keys, &mut file_state, &mut text_body, &mut search_open);
                    pressed_timer = 0;
                } else if pressed_keys.len() > 1 {
                    pressed_timer += 1;
                } else if pressed_keys {
                    pressed_timer = 0;
                }

                egui::CentralPanel::default().show(&context, |ui| {
                    if search_open {
                        ui.label("Search");
                        ui.text_edit_singleline(&mut search);

                        if search != "" && text_body != "" {
                            if text_body.contains(search.as_str()) {

                            }
                        }
                    }

                    ui.add_space(20.0);
                    ui.text_edit_multiline(&mut text_body);
                });

                if open_modal {
                    egui::Window::new("Open File")
                        .collapsible(false)
                        .resizable(false)
                        .show(&context, |ui| {
                            open_file_modal(ui, &mut open_modal, &mut file_state, &mut text_body);
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
                                state: ElementState::Released,
                                virtual_keycode: Some(key),
                                ..
                            } => {
                                match key {
                                    VirtualKeyCode::S => {
                                        pressed_keys.remove(&VirtualKeyCode::S);
                                    }
                                    VirtualKeyCode::F => {
                                        pressed_keys.remove(&VirtualKeyCode::F);
                                    }
                                    VirtualKeyCode::LShift => {
                                        pressed_keys.remove(&VirtualKeyCode::LShift);
                                    }
                                    VirtualKeyCode::LWin => {
                                        pressed_keys.remove(&VirtualKeyCode::LWin);
                                    }
                                    _ => {}
                                }
                            }

                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(key),
                                ..
                            } => {
                                match key {
                                    VirtualKeyCode::A => {
                                    }
                                    VirtualKeyCode::F => {
                                        pressed_keys.insert(VirtualKeyCode::F);
                                    }
                                    VirtualKeyCode::S => {
                                        pressed_keys.insert(VirtualKeyCode::S);
                                    }
                                    VirtualKeyCode::LShift => {
                                        pressed_keys.insert(VirtualKeyCode::LShift);
                                    }
                                    VirtualKeyCode::D => {
                                    }
                                    VirtualKeyCode::W => {
                                    }
                                    VirtualKeyCode::LWin => {
                                        pressed_keys.insert(VirtualKeyCode::LWin);
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

fn process_multiple_keys(pressed_keys: &mut HashSet<VirtualKeyCode>, file_state: &mut Mutex<FileState>, text_body: &mut String, search_open: &mut bool) {
    dbg!("Processing");
    if pressed_keys.contains(&VirtualKeyCode::LWin) && pressed_keys.contains(&VirtualKeyCode::S) {
        save_current_file(pressed_keys, file_state, text_body);
    } else if pressed_keys.contains(&VirtualKeyCode::LWin) && pressed_keys.contains(&VirtualKeyCode::LShift) && pressed_keys.contains(&VirtualKeyCode::F) {
        dbg!("Searching through text");
    } else if pressed_keys.contains(&VirtualKeyCode::LWin) && pressed_keys.contains(&VirtualKeyCode::F)  {
        *search_open = if *search_open { false } else { true };
        // dbg!("Searching within file");
    }

    pressed_keys.clear();
}

fn save_current_file(pressed_keys: &mut HashSet<VirtualKeyCode>, file_state: &mut Mutex<FileState>, text_body: &mut String) {
    dbg!("Saving");
    {
        let state = file_state.lock().unwrap();

        for file in state.open_files.clone() {
            if file.current_file {
                let file = File::create(file.file_path);

                match file {
                    Ok(mut file) => {
                        match file.write_all(text_body.as_bytes()) {
                            Ok(_) => { }
                            Err(err) => { eprintln!("Error saving file {}", err)}
                        }
                    }
                    Err(err) => { eprintln!("Error opening file {}", err); }
                }
            }
        }
    }
}

fn open_file_modal(ui: &mut Ui, show_modal: &mut bool, file_state: &mut Mutex<FileState>, text_body: &mut String) {
    ui.label("Select file to open...");

    if ui.button("Browser...").clicked() {
        if let Some(path) = FileDialog::new().pick_file() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    {
                        let mut file_state = file_state.lock().unwrap();
                        file_state.insert_file(path.to_str().unwrap().to_string());
                    }

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
