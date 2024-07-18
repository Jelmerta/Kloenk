use cgmath::*;
use winit::{event::*, dpi::PhysicalPosition};

#[derive(Debug)]
pub struct Camera {
    target: Point3<f32>,
    eye: Point3<f32>, // Or: position
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

pub const TOTAL_DISTANCE: f32 = 200000.; // Verify naming, probbaly not total distance
// pub const STARTING_CAMERA_DISTANCE: f32 = f32::sqrt(TOTAL_DISTANCE / 3.0);

impl Camera {
    pub fn new() -> Self {
        let STARTING_CAMERA_DISTANCE: f32 = f32::sqrt(TOTAL_DISTANCE / 3.0);

        Self {
            target: (0.0, 0.0, 0.0).into(),
            eye: (STARTING_CAMERA_DISTANCE, STARTING_CAMERA_DISTANCE, STARTING_CAMERA_DISTANCE).into(),
            up: Vector3::unit_y(),
            z_near: 0.1,
            z_far: 100.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // let projection = cgmath::perspective(cgmath::Deg(self.fov_y_degrees), self.aspect, self.z_near, self.z_far); // Perspective projection
        let projection = cgmath::ortho(-1., 1., -1., 1., self.z_near, self.z_far); // Isometric projection. I don't really grok near and far yet
        return OPENGL_TO_WGPU_MATRIX * projection * view;
    }
}

#[derive(Debug)]
pub struct CameraController {
    scroll: f32,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            scroll: 0.0,
        }
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) =>  -scroll * 0.5, // i dont understand what
            // this does, is this assigning anything?
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll, ..
            }) => -*scroll as f32,
        };

    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        camera.eye = (camera.eye.x + self.scroll * 10.0, camera.eye.y + self.scroll * 10.0, camera.eye.z + self.scroll * 10.0).into();
        self.scroll = 0.0;
    }
}

