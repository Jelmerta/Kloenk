//TODO wondering if this needs access to renderer / FIFO/mailbox information
// TODO is last() an expensive operation? store last frame time separate?

// TODO use start time + accumulated time
use std::time::{Duration, Instant};

// TODO What about Ticks?
pub struct FramerateHandler {
    // frame_times: Vec<Instant>, // maybe some other thing works better? FIFO? deque?
    // target_frame_time_nano_seconds: Option<u64>,
    target_tick_time_nano_seconds: u32,
    last_update_time: Instant,
    accumulated_time_nanos: u32,
}

impl FramerateHandler {
    // pub fn new(monitor_capable_millihertz: Option<u32>, present_mode: PresentMode) -> Self {
    pub fn new() -> Self {
        // pub fn new(monitor_capable_millihertz: Option<u32>) -> Self {
        // let used_millihertz = match present_mode {
        //     PresentMode::AutoNoVsync => monitor_capable_millihertz,
        //     PresentMode::Mailbox => monitor_capable_millihertz,
        //     _ => panic!("Present mode not supported"),
        // };

        // let used_millihertz = Some(120000);
        // let used_millihertz = Some(2 * monitor_capable_millihertz.unwrap());
        // let used_millihertz: Option<u32> = None;
        // let used_millihertz = monitor_capable_millihertz;
        // let target_frame_time_nano_seconds = if let Some(mhz) = used_millihertz {
        //     Some(1_000_000_000_000 / mhz as u64) // TODO Note: Rounded down by division
        // } else {
        //     None
        // };

        FramerateHandler {
            // frame_times: Vec::with_capacity(200),
            target_tick_time_nano_seconds: Duration::from_secs(1).div_f64(60.0).as_nanos() as u32, //,Duration::div_duration_f64() ,//1.0/60.0,
            last_update_time: Instant::now(),
            accumulated_time_nanos: 0,
        }
    }

    // TODO should_update?
    pub fn should_update(&mut self) -> bool {
        let time_to_add_nanos = self.last_update_time.elapsed().as_nanos() as u32;
        self.accumulated_time_nanos += time_to_add_nanos;
        self.last_update_time = Instant::now();

        // }

        // TODO maybe make sure accumulated time is not greater than two times the target time? in order not to render too many frames?
        // if let Some(target_nanos) = self.target_frame_time_nano_seconds {
        //     self.accumulated_time_nanos > target_nanos
        // } else {
        //     true
        // #[cfg(feature = "debug-logging")]
        // log::debug!("should accumulated_time_nanos {:?}", self.accumulated_time_nanos);
        self.accumulated_time_nanos > self.target_tick_time_nano_seconds
    }

    // pub fn rendered_frame(&mut self) {
    pub fn updated(&mut self) {
        // if self.frame_times.len() == self.frame_times.capacity() {
        //     let last_frame_time = self.frame_times.last().expect("frame times are filled").clone();
        //     self.frame_times = Vec::with_capacity(200);
        //     self.frame_times.push(last_frame_time)
        // }

        // self.frame_times.push(Instant::now());
        // if let Some(target_nanos) = self.target_frame_time_nano_seconds {
        //     self.accumulated_time_nanos = self.accumulated_time_nanos % target_nanos; // We don't want to have to render multiple times // TODO might still queue multiple request_draws? should we do something during update?
        // }

        self.accumulated_time_nanos -= self.target_tick_time_nano_seconds;
        // #[cfg(feature = "debug-logging")]
        // log::debug!("updated accumulated_time_nanos {:?}", self.accumulated_time_nanos);
    }
    //
    // // TODO could just call this once a second? or only debug?
    // pub fn fps_last_second_average(&mut self) -> Option<f32> {
    //     let mut total_frame_time = 0.0;
    //     if self.frame_times.len() < 60 { // todo 60 doesnt make sense
    //         return None;
    //     } else {
    //         for i in 0..59 { // todo 60 doesnt make sense
    //             let frame_time_before = self.frame_times.get(i).expect("Frames are filled");
    //             let frame_time_after = self.frame_times.get(i + 1).expect("Frames are filled");
    //             let diff = frame_time_after.duration_since(*frame_time_before).as_secs_f32();
    //             total_frame_time += diff;
    //         }
    //     }
    //     Some(total_frame_time / 59.0) // todo 60 doesnt make sense
    // }
    //
    // pub fn fps_last_second_slowest(&mut self) -> Option<f32> {
    //     let mut slowest = None;
    //     if self.frame_times.len() < 60 { // todo 60 doesnt make sense
    //         return None;
    //     } else {
    //         for i in 0..59 { // todo 60 doesnt make sense
    //             let frame_time_before = self.frame_times.get(i).expect("Frames are filled");
    //             let frame_time_after = self.frame_times.get(i + 1).expect("Frames are filled");
    //             let diff = frame_time_after.duration_since(*frame_time_before).as_secs_f32();
    //             if slowest.is_none() || diff < slowest.expect("Slowest is Some") {
    //                 slowest = Some(diff);
    //             }
    //         }
    //     }
    //     slowest
    // }

    // TODO maybe some stat like standard deviation -> finding outliers, stuttering
}
