use cgmath::{ortho, Matrix4, Point3, SquareMatrix, Vector3, Zero};

// TODO kind of weird having all this functionality in this component. should be separated.
#[derive(Debug)]
pub struct Camera {
    pub target: Point3<f32>,
    pub eye: Point3<f32>,
    pub up: Vector3<f32>,
    pub z_near: f32,
    pub z_far: f32,

    pub view_projection_matrix: Matrix4<f32>,

    pub view_projection_matrix_inverted: Matrix4<f32>,
}

// This is just used to convert OpenGL's coordinate system to WGPUs (as used in Metal/DX)
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
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

            view_projection_matrix: Matrix4::zero(),

            view_projection_matrix_inverted: Matrix4::zero(),
        }
    }

    pub fn update_view_projection_matrix(&mut self, window_width: u32, window_height: u32) {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);

        let isometric_projection = ortho(
            -(window_width as f32) / window_height as f32,
            window_width as f32 / window_height as f32,
            -1.,
            1.,
            self.z_near,
            self.z_far,
        );
        self.view_projection_matrix = OPENGL_TO_WGPU_MATRIX * isometric_projection * view;
    }

    pub fn update_inverse_matrix(&mut self) {
        self.view_projection_matrix_inverted = self.view_projection_matrix.invert().unwrap();
    }
}
