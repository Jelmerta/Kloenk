// Model without textures but with color. Like OSRS
// #[repr(C)] // Not sure what this effectively does here
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // Read up more about bytemuck, to cast our VERTICES as a &[u8]
// struct Vertex {
//     position: [f32; 3],
//     color: [f32; 3],
// }
//
use std::ops::Range;

use crate::texture;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)] // Not sure what this effectively does here
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // Read up more about bytemuck, to cast our VERTICES as a &[u8]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub color: [f32;3],
}

#[repr(C)] // Not sure what this effectively does here
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // Read up more about bytemuck, to cast our VERTICES as a &[u8]
pub struct TexVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    // Vertices
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    // Color
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}


impl Vertex for TexVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TexVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

// sotrh decides to implement a trait on renderpass
pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh);
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
    );

    fn draw_model(&mut self, model: &'a Model);
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh) {
        self.draw_mesh_instanced(mesh, 0..1);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(&mut self, model: &'b Model) {
        self.draw_model_instanced(model, 0..1);
    }

    fn draw_model_instanced(
            &mut self,
            model: &'b Model,
            instances: Range<u32>,
    ) {
        for mesh in &model.meshes {
            self.draw_mesh_instanced(mesh, instances.clone());
        }
    }
}
