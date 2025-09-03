use std::time::{Duration, Instant};

pub struct UpdateTickHandler {
    target_tick_time_nano_seconds: u32,
    last_update_time: Instant,
    accumulated_time_nanos: u32,
}

// Time probably needs to be retrieved from server in order to match ticks
impl UpdateTickHandler {
    pub fn new() -> Self {
        UpdateTickHandler {
            target_tick_time_nano_seconds: Duration::from_secs(1).div_f64(60.0).as_nanos() as u32,
            last_update_time: Instant::now(),
            accumulated_time_nanos: 0,
        }
    }

    pub fn should_update(&mut self) -> bool {
        let now = Instant::now();
        let elapsed_nanos = now.duration_since(self.last_update_time).as_nanos() as u32;
        self.accumulated_time_nanos += elapsed_nanos;
        self.last_update_time = now;
        self.accumulated_time_nanos > self.target_tick_time_nano_seconds
    }

    pub fn updated(&mut self) {
        self.accumulated_time_nanos -= self.target_tick_time_nano_seconds;
    }
}
