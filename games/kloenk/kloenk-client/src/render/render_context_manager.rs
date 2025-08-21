use crate::render::camera_manager::CameraManager;
use crate::render::color_manager::ColorManager;
use crate::render::instance::InstanceRaw;
use crate::render::material_manager::TextureManager;
use crate::render::model::Vertex;
use crate::render::{model, texture};
use wgpu::{Device, PipelineCompilationOptions, RenderPipeline, SurfaceConfiguration};

pub struct RenderContextManager {
    pub render_pipeline: RenderPipeline,
}

impl RenderContextManager {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        color_manager: &ColorManager,
        camera_manager: &CameraManager,
        material_manager: &TextureManager,
    ) -> Self {
        let render_pipeline = Self::setup_textured_context(
            device,
            config,
            color_manager,
            camera_manager,
            material_manager,
        );
        RenderContextManager { render_pipeline }
    }

    fn setup_textured_context(
        device: &Device,
        config: &SurfaceConfiguration,
        color_manager: &ColorManager,
        camera_manager: &CameraManager,
        texture_manager: &TextureManager,
    ) -> RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/color_texture_shader.wgsl").into(),
            ),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &color_manager.bind_group_layout,
                    &texture_manager.bind_group_layout,
                    &camera_manager.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[model::ColorTextureVertex::layout(), InstanceRaw::layout()],
                compilation_options: PipelineCompilationOptions::default(),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Depth::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format.add_srgb_suffix(), // Set view format to srgb, required for WebGPU otherwise shows bland colours could add this only if wasm32
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        })
    }
}
