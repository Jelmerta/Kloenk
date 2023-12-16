use std::iter;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

#[repr(C)]  // Not sure what this effectively does here
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // Read up more about bytemuck, to cast our VERTICES as a &[u8]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1
                }
            ]
        }
    }
}



const TRIANGLE: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0]},
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0]},
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0]},
];

impl State {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(TRIANGLE),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let num_vertices = TRIANGLE.len() as u32;

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}



    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("kloenk-wasm")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    // UPDATED!
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}



// use std::default::Default;
// use winit::{
//     event::*,
//     event_loop::{EventLoop},
//     window::WindowBuilder,
// };
//
// use winit::dpi::PhysicalSize;
// use winit::keyboard::{Key, NamedKey};
//
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::*;
// use wgpu::ColorWrites;
// use wgpu::core::instance;
//
// #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
// pub async fn run() {
//     cfg_if::cfg_if! {
//         if #[cfg(target_arch = "wasm32")] {
//             std::panic::set_hook(Box::new(console_error_panic_hook::hook));
//             console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
//         } else {
//             env_logger::init();
//         }
//     }
//
//     // let event_loop = EventLoop::new();
//     let event_loop = EventLoop::new().unwrap();
//
//     // let window = WindowBuilder::new().build(&event_loop).unwrap();
//     let window = WindowBuilder::new()
//         .with_title("Kloenk")
//         .with_inner_size(PhysicalSize::new(800, 600))
//         .build(&event_loop)
//         .unwrap();
//     #[cfg(target_arch = "wasm32")]
//     {
//         use winit::platform::web::WindowExtWebSys;
//         web_sys::window()
//             .and_then(|window| window.document())
//             .and_then(|document| {
//                 let wasm_element = document.get_element_by_id("kloenk-wasm")?;
//                 let canvas = web_sys::Element::from(window.canvas().unwrap());
//                 wasm_element.append_child(&canvas).ok()?;
//                 Some(()) // What the hell does this do???
//             })
//             .expect("Couldn't append canvas to document body");
//     }
//
//     let mut state = State::new(window).await;
//
//     let mut close_requested = false;
//     event_loop.run(move |event, event_loop_window_target|
//         match event {
//             Event::WindowEvent {
//                 ref event,
//                 window_id,
//             } if window_id == state.window().id() => if !state.input(event) {
//                 match event { //if window_id == window.id() =>
//                     WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
//                         event:
//                         KeyEvent {
//                             logical_key: Key::Named(NamedKey::Escape),
//                             state: ElementState::Pressed,
//                             ..
//                         },
//                         ..
//                     } => close_requested = true,
//                     WindowEvent::RedrawRequested => {
//                         state.update();
//                         match state.render() {
//                             Ok(_) => {}
//                             Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
//                             Err(wgpu::SurfaceError::OutOfMemory) => close_requested = true,
//                             Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
//                             Err(e) => eprintln!("{:?}", e),
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//             Event::AboutToWait => {
//                 if close_requested {
//                     event_loop_window_target.exit();
//                 }
//             }
//             _ => {}
//         }
//     ).unwrap();
// }
//
// use winit::window::Window;
//
// struct State {
//     surface: wgpu::Surface,
//     device: wgpu::Device,
//     queue: wgpu::Queue,
//     config: wgpu::SurfaceConfiguration,
//     size: winit::dpi::PhysicalSize<u32>,
//     window: Window,
//     // render_pipeline: wgpu::RenderPipeline,
// }
//
// impl State {
//     async fn new(window: Window) -> Self {
//         let size = window.inner_size();
//
//         let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
//             backends: wgpu::Backends::all(),
//             ..Default::default()
//         });
//
//         let surface = unsafe { instance.create_surface(&window) }.unwrap();
//
//         let adapter = instance.request_adapter(
//             &wgpu::RequestAdapterOptions {
//                 power_preference: wgpu::PowerPreference::default(),
//                 compatible_surface: Some(&surface),
//                 force_fallback_adapter: false,
//             },
//         ).await.unwrap();
//
//         let (device, queue) = adapter.request_device(
//             &wgpu::DeviceDescriptor {
//                 features: wgpu::Features::empty(),
//                 limits: if cfg!(target_arch = "wasm32") {
//                     wgpu::Limits::downlevel_webgl2_defaults()
//                 } else {
//                     wgpu::Limits::default()
//                 },
//                 label: None,
//             },
//             None,
//         ).await.unwrap();
//
//         let surface_caps = surface.get_capabilities(&adapter);
//
//         let surface_format = surface_caps.formats.iter()
//             .copied()
//             .filter(|f| f.is_srgb())//.describe().srgb) ? deprecated maybe?
//             .next()
//             .unwrap_or(surface_caps.formats[0]);
//
//         let config = wgpu::SurfaceConfiguration {
//             usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//             format: surface_format,
//             width: size.width,//.max(1), // To make sure program does not crash when window has not loaded yet, we take a max of 1 for width and height. https://github.com/bevyengine/bevy/issues/170
//             height: size.height,//.max(1),
//             present_mode: surface_caps.present_modes[0],
//             alpha_mode: surface_caps.alpha_modes[0],
//             view_formats: vec![],
//         };
//
//         surface.configure(&device, &config);
//

//     }
//
//     pub fn window(&self) -> &Window {
//         &self.window
//     }
//
//     fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
//         todo!()
//     }
//
//     fn input(&mut self, event: &WindowEvent) -> bool {
//         false
//     }
//
//     fn update(&mut self) {
//         // todo!()
//     }
//
//     fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
//         let output = self.surface.get_current_texture()?;
//         let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
//         let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
//             label: Some("Render Encoder"),
//         });
//
//         {
//             let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                 label: Some("Render Pass"),
//                 color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                     view: &view,
//                     resolve_target: None,
//                     ops: wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(wgpu::Color {
//                             r: 0.5,
//                             g: 0.0,
//                             b: 0.5,
//                             a: 1.0,
//                         }),
//                         store: wgpu::StoreOp::Store,
//                     },
//                 })],
//                 depth_stencil_attachment: None,
//                 occlusion_query_set: None,
//                 timestamp_writes: None,
//             });
//         }
//
//         self.queue.submit(std::iter::once(encoder.finish()));
//         output.present();
//
//         Ok(()) //????
//     }
// }