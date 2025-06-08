#![allow(dead_code)]
// vendored from https://github.com/optozorax/egui-macroquad/tree/dfbdb967d6cf4e4726b84a568ec1b2bdc7e4f492

use egui_miniquad::EguiMq;
use macroquad::prelude::*;
use miniquad as mq;

pub struct Egui {
    egui_mq: EguiMq,
    input_subscriber_id: usize,
}

impl Egui {
    pub fn new() -> Self {
        Self {
            egui_mq: EguiMq::new(unsafe { get_internal_gl() }.quad_context),
            input_subscriber_id: macroquad::input::utils::register_input_subscriber(),
        }
    }

    pub fn ui<F>(&mut self, f: F)
    where
        F: FnMut(&mut dyn mq::RenderingBackend, &egui::Context),
    {
        let gl = unsafe { get_internal_gl() };
        macroquad::input::utils::repeat_all_miniquad_input(self, self.input_subscriber_id);
        self.egui_mq.run(gl.quad_context, f);
    }

    pub fn draw(&mut self) {
        let mut gl = unsafe { get_internal_gl() };
        // Ensure that macroquad's shapes are not goint to be lost, and draw them now
        gl.flush();
        self.egui_mq.draw(gl.quad_context);
    }
}

impl mq::EventHandler for Egui {
    fn update(&mut self) {}

    fn draw(&mut self) {}

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_down_event(mb, x, y);
    }

    fn mouse_button_up_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_up_event(mb, x, y);
    }

    fn char_event(&mut self, character: char, _keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.key_down_event(keycode, keymods);
    }

    fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}
