use std::collections::VecDeque;

use macroquad::time::get_frame_time;

pub struct FPSCounter {
    frametimes: VecDeque<f32>,
}

impl FPSCounter {
    pub fn new() -> Self {
        Self { frametimes: Default::default() }
    }

    pub fn update(&mut self) {
        self.frametimes.push_back(get_frame_time());
        while self.frametimes.len() > 60 {
            self.frametimes.pop_front();
        }
    }

    pub fn get_fps(&mut self) -> f32 {
        (1.0 / (self.frametimes.iter().copied().reduce(|a, b| a + b).unwrap_or(0.0)
            / self.frametimes.len() as f32))
            .round()
    }
}
