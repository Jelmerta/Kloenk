

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