use std::time::Duration;

pub struct UpdateTickHandler {
    target_tick_time_nano_seconds: u32,
    last_update_time_nanos: u64,
    accumulated_time_nanos: u32,
}

// Time probably needs to be retrieved from server in order to match ticks
impl UpdateTickHandler {
    pub fn new() -> Self {
        let performance = web_sys::window()
            .expect("window should exist")
            .performance()
            .expect("performance should be available");
        UpdateTickHandler {
            target_tick_time_nano_seconds: Duration::from_secs(1).div_f64(60.0).as_nanos() as u32,
            last_update_time_nanos: (performance.now() * 1_000_000.0) as u64,
            accumulated_time_nanos: 0,
        }
    }

    pub fn should_update(&mut self) -> bool {
        let performance = web_sys::window()
            .expect("window should exist")
            .performance()
            .expect("performance should be available");
        let now_nanos = (performance.now() * 1_000_000.0) as u64;
        let elapsed_nanos = now_nanos - self.last_update_time_nanos;
        self.accumulated_time_nanos += elapsed_nanos as u32;

        self.last_update_time_nanos = now_nanos;
        self.accumulated_time_nanos > self.target_tick_time_nano_seconds
    }

    pub fn updated(&mut self) {
        self.accumulated_time_nanos -= self.target_tick_time_nano_seconds;
    }
}
