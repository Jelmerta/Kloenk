use cgmath::{Matrix4, Point3, SquareMatrix, Vector3, Zero};
use std::sync::Arc;
use winit::window::Window;

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

    // TODO only for 3d and only maybe upon update?
    pub fn update_view_projection_matrix(&mut self, window: &Arc<Window>) {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);

        let scale = 1.0; //window.inner_size().height as f32 / DEFAULT_RESOLUTION_HEIGHT;
        let resolution = window.inner_size().width as f32 / window.inner_size().height as f32;
        let height = scale;
        let width = scale * resolution;
        let isometric_projection =
            Self::ortho_wgpu(-width, width, -height, height, self.z_near, self.z_far);
        self.view_projection_matrix = isometric_projection * view;
    }

    pub fn update_inverse_matrix(&mut self) {
        self.view_projection_matrix_inverted = self.view_projection_matrix.invert().unwrap();
    }

    // Refer to wgpu_ortho_projection.png for the matrices multiplied to get this. Based on the matrix provided in sotrh tutorial
    fn ortho_wgpu(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Matrix4<f32> {
        let width_reciprocal = 1.0 / (right - left);
        let height_reciprocal = 1.0 / (top - bottom);
        let depth_reciprocal = 1.0 / (far - near);

        let c0r0 = 2.0 * width_reciprocal;

        let c1r1 = 2.0 * height_reciprocal;

        let c2r2 = -depth_reciprocal;

        let c3r0 = -(right + left) * width_reciprocal;
        let c3r1 = -(top + bottom) * height_reciprocal;
        let c3r2 = -(far + near) * (0.5 * depth_reciprocal);

        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4::new(
            c0r0, 0.0, 0.0, 0.0,
            0.0, c1r1, 0.0, 0.0,
            0.0, 0.0, c2r2, c2r2,
            c3r0, c3r1, c3r2, 1.0 + c3r2,
        )
    }
}
