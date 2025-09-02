use crate::state::input::Input;
use std::sync::Arc;
use winit::monitor::MonitorHandle;
use winit::window::{Fullscreen, Window};

pub struct MonitorChangeSystem {}

impl MonitorChangeSystem {
    // TODO how does this work on web? should i just handle events there? is this code not relevant for web at all?
    pub fn update_monitor(input: &Input, window: &Arc<Window>) {
        if input.m_pressed.is_toggled_on() {
            let monitors: Vec<MonitorHandle> = window.available_monitors().collect();
            if monitors.len() == 1 {
                return;
            }

            let current_monitor_handle = window
                .current_monitor()
                .expect("Game should be on a monitor");
            let current_index = monitors
                .iter()
                .position(|m| m.name() == current_monitor_handle.name())
                .expect("Current monitor should be in available monitors");

            let next_index = (current_index + 1) % monitors.len();
            let next_monitor = monitors[next_index].clone();
            window.set_fullscreen(Some(Fullscreen::Borderless(Some(next_monitor))));
        }
    }
}
