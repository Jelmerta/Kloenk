use cgmath::*;

#[derive(Debug)]
pub struct Camera {
    target: Point3<f32>,
    pub eye: Point3<f32>, // Or: position
    up: Vector3<f32>,
    z_near: f32,
    z_far: f32,
}

// This is just used to convert OpenGL's coordinate system to WGPUs (as used in Metal/DX)
#[rustfmt::skip] // ? just for formatting as 4x4?
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

impl Camera {
    pub fn new() -> Self {
        Self {
            target: (0.0, 0.0, 0.0).into(),
            eye: (0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            z_near: 0.1,
            z_far: 100.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> 
    {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // let projection = cgmath::perspective(cgmath::Deg(self.fov_y_degrees), self.aspect, self.z_near, self.z_far); // Perspective projection
        let projection = cgmath::ortho(-1., 1., -1., 1., self.z_near, self.z_far); // Isometric projection. I don't really grok near and far yet
        return OPENGL_TO_WGPU_MATRIX * projection * view;
    }
}
