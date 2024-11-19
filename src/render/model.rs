use std::ops::Range;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)] // Not sure what this effectively does here
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)] // Read up more about bytemuck, to cast our VERTICES as a &[u8]
pub struct ColoredVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex for ColoredVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ColoredVertex>() as wgpu::BufferAddress,
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
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

impl Vertex for TexVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<TexVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    // pub materials: Vec<Material>, // TODO I think materials should be sharable across models right for memory efficiency and stuff
}

pub struct Mesh {
    // pub name: String,
    pub vertex_type: VertexType,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    // pub material: usize,
}

pub enum VertexType {
    Texture { material_id: String },
    Color { color: [f32; 3] },
}

// TODO needs to be stored somewhere
pub struct Material {
    // pub name: String,
    // pub diffuse_texture: texture::Texture,
    pub texture_bind_group: wgpu::BindGroup,
}

// pub enum Material {
//     // pub name: String,
//     // pub diffuse_texture: texture::Texture,
//     Texture { texture_bind_group: wgpu::BindGroup },
//     Color { color: Vector3<f32> },
// }

// sotrh decides to implement a trait on renderpass
pub trait Draw<'a> {
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: Range<u32>);
}

impl<'a, 'b> Draw<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, instances: Range<u32>) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}
