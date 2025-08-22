//TODO wondering if this needs access to renderer / FIFO/mailbox information

use std::time::Instant;
use wgpu::PresentMode;

pub struct FramerateHandler {
    frame_times: Vec<Instant>,
    target_fps: Option<u32>,
    delta_time: Duration,
}

impl FramerateHandler {
    pub fn new(present_mode: PresentMode) -> Self {
        FramerateHandler {
            frame_times: Vec::with_capacity(60),
            target_fps:,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let previous_time = self.frame_times.last().or().expect();
        let delta_time = now.duration_since(*previous_time);

        self.delta_time = delta_time;
    }
}