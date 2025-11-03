use std::cell::RefCell;

use base::*;

use macroquad::prelude::*;
#[cfg(feature = "hotreload")]
use quicksilver::reflections::ValueReflection;

use crate::{
    camera::{CameraWrapper, screen_camera},
    draw::kf_draw_texture,
    material::SpriteShader,
    text::Texter,
    util::texture_store::TextureStore,
};

pub struct Context {
    draw_buffer: RefCell<Vec<DrawCommand>>,
    pub camera: CameraWrapper,
    pub textures: TextureStore,
    pub loading: Vec<(String, String)>,
    pub inner: ContextInner,
    #[cfg(feature = "hotreload")]
    pub egui_ctx: Option<&'static mut egui::Ui>,
    #[cfg(feature = "hotreload")]
    pub egui_drawn: bool,
}

pub struct ContextInner {
    pub texter: Texter,
    pub sprite_shader: SpriteShader,
}

impl ContextTrait for Context {
    /// time since program start
    fn time(&self) -> f64 {
        get_time()
    }

    fn delta(&self) -> f32 {
        get_frame_time()
    }

    fn fps(&self) -> f32 {
        get_fps() as f32
    }

    fn draw_rect(&mut self, rect: base::Rect, c: base::Color, z_level: i32) {
        let color = macroquad::prelude::Color { r: c.r, g: c.g, b: c.b, a: c.a };

        let command = move |_: &mut ContextInner| {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
        };
        self.draw_buffer
            .borrow_mut()
            .push(DrawCommand { z_level, command: Box::new(command) });
    }

    fn draw_rect_lines(
        &mut self,
        rect: base::Rect,
        thickness: f32,
        c: base::Color,
        z_level: i32,
    ) {
        let color = macroquad::prelude::Color { r: c.r, g: c.g, b: c.b, a: c.a };

        let command = move |_: &mut ContextInner| {
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, thickness, color);
        };
        self.draw_buffer
            .borrow_mut()
            .push(DrawCommand { z_level, command: Box::new(command) });
    }

    fn draw_circle(&mut self, circle: base::Circle, c: base::Color, z_level: i32) {
        let color = macroquad::prelude::Color { r: c.r, g: c.g, b: c.b, a: c.a };

        let command = move |_: &mut ContextInner| {
            draw_circle(circle.pos.x, circle.pos.y, circle.radius, color);
        };
        self.draw_buffer
            .borrow_mut()
            .push(DrawCommand { z_level, command: Box::new(command) });
    }

    fn draw_texture(&mut self, name: &str, x: f32, y: f32, z_level: i32) {
        // load if not in texture store
        // then add to draw buffer
        if let Some(texture) = self.textures.get(name) {
            let source = None;
            let params = DrawTextureParams { source, ..Default::default() };
            let command = move |_i: &mut ContextInner| {
                // i.sprite_shader.set(&texture);
                kf_draw_texture(&texture, x, y, WHITE, params);
            };
            self.draw_buffer
                .borrow_mut()
                .push(DrawCommand { z_level, command: Box::new(command) });
        } else {
            self.draw_rect(base::Rect::new(x, y, 300., 300.), base::Color::PINK, 9999);
        }
    }

    fn draw_texture_part(
        &mut self,
        name: &str,
        src: base::Rect,
        x: f32,
        y: f32,
        z_level: i32,
    ) {
        // load if not in texture store
        // then add to draw buffer
        if let Some(texture) = self.textures.get(name) {
            let source =
                Some(macroquad::math::Rect { x: src.x, y: src.y, w: src.w, h: src.h });
            let params = DrawTextureParams { source, ..Default::default() };
            let command = move |_i: &mut ContextInner| {
                // i.sprite_shader.set(&texture);
                kf_draw_texture(&texture, x, y, WHITE, params);
            };
            self.draw_buffer
                .borrow_mut()
                .push(DrawCommand { z_level, command: Box::new(command) });
        } else {
            self.draw_rect(base::Rect::new(x, y, 300., 300.), base::Color::PINK, 9999);
        }
    }

    fn draw_texture_part_scaled(
        &mut self,
        name: &str,
        src: base::Rect,
        target: base::Rect,
        z_level: i32,
    ) {
        // load if not in texture store
        // then add to draw buffer
        if let Some(texture) = self.textures.get(name) {
            let source =
                Some(macroquad::math::Rect { x: src.x, y: src.y, w: src.w, h: src.h });
            let dest_size = Some(vec2(target.w, target.h));
            let params = DrawTextureParams { source, dest_size, ..Default::default() };
            let command = move |_i: &mut ContextInner| {
                // i.sprite_shader.set(&texture);
                kf_draw_texture(&texture, target.x, target.y, WHITE, params);
            };
            self.draw_buffer
                .borrow_mut()
                .push(DrawCommand { z_level, command: Box::new(command) });
        } else {
            self.draw_rect(target, base::Color::PINK, 9999);
        }
    }

    fn load_texture(&mut self, name: &str, path: &str) {
        self.loading.push((name.to_string(), path.to_string()));
    }

    fn texture_dimensions(&mut self, name: &str) -> base::Rect {
        self.textures
            .get(name)
            .map(|t| base::Rect { x: 0., y: 0., w: t.width(), h: t.width() })
            .unwrap_or(base::Rect { x: 0., y: 0., w: 0., h: 0. })
    }

    fn is_pressed(&self, input: Input) -> bool {
        match input {
            Input::MouseLeft => is_mouse_button_pressed(MouseButton::Left),
            Input::MouseMiddle => is_mouse_button_pressed(MouseButton::Middle),
            Input::MouseRight => is_mouse_button_pressed(MouseButton::Right),
            Input::RestartGame => is_key_pressed(KeyCode::F1),
            Input::Save => is_key_pressed(KeyCode::F5),
            Input::Load => is_key_pressed(KeyCode::F9),
            Input::MoveSW => is_key_pressed(KeyCode::Kp1),
            Input::MoveS => is_key_pressed(KeyCode::Kp2) || is_key_pressed(KeyCode::Down),
            Input::MoveSE => is_key_pressed(KeyCode::Kp3),
            Input::MoveW => is_key_pressed(KeyCode::Kp4) || is_key_pressed(KeyCode::Left),
            Input::MoveSkip => is_key_pressed(KeyCode::Kp5),
            Input::MoveE => is_key_pressed(KeyCode::Kp6) || is_key_pressed(KeyCode::Right),
            Input::MoveNW => is_key_pressed(KeyCode::Kp7),
            Input::MoveN => is_key_pressed(KeyCode::Kp8) || is_key_pressed(KeyCode::Up),
            Input::MoveNE => is_key_pressed(KeyCode::Kp9),
            Input::Confirm => is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space),
            Input::Cancel => is_key_pressed(KeyCode::Escape),
            Input::Inventory => is_key_pressed(KeyCode::I),
            Input::DebugSlowdown => is_key_pressed(KeyCode::F12),
            Input::MouseMoveCamera => {
                is_mouse_button_down(MouseButton::Middle)
                    || is_mouse_button_down(MouseButton::Right)
            }
        }
    }

    fn mouse_screen(&self) -> FPos {
        let m = mouse_position();
        FPos { x: m.0, y: m.1 }
    }

    fn mouse_world(&self) -> FPos {
        let m = self.camera.mouse_world();
        FPos { x: m.x, y: m.y }
    }

    fn camera_set_shake(&mut self, offset: FVec) {
        self.camera.shake_offset.x = offset.x;
        self.camera.shake_offset.y = offset.y;
    }

    fn camera_move_rel(&mut self, offset: FVec, time: f32) {
        self.camera.move_camera_relativ(offset, time);
    }

    fn set_text(
        &mut self,
        key: u64,
        w: f32,
        h: f32,
        text: &[(&str, TextProperty)],
    ) -> base::Rect {
        self.inner.texter.set_text(key, w, h, text)
    }

    fn draw_text(&mut self, key: u64, x: f32, y: f32, z_level: i32) {
        let command = move |inner: &mut ContextInner| {
            if inner.texter.draw_text(key, x, y).is_none() {
                let rect = macroquad::prelude::Rect::new(x, y, 300., 300.);
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, PINK);
            }
        };
        self.draw_buffer
            .borrow_mut()
            .push(DrawCommand { z_level, command: Box::new(command) });
    }

    fn screen_rect(&self) -> base::Rect {
        base::Rect::new(0., 0., screen_width(), screen_height())
    }

    #[cfg(feature = "hotreload")]
    fn inspect(&mut self, val: &mut ValueReflection) {
        self.egui_drawn = true;
        use crate::egui_inspector::draw_value;
        if let Some(ref mut ui) = self.egui_ctx {
            draw_value(ui, val);
            ui.separator();
        }
    }

    fn mouse_wheel(&self) -> f32 {
        let (_x, y) = mouse_wheel();
        y
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            draw_buffer: Default::default(),
            camera: Default::default(),
            textures: Default::default(),
            loading: Default::default(),
            inner: ContextInner { texter: Texter::new(), sprite_shader: SpriteShader::new() },
            #[cfg(feature = "hotreload")]
            egui_ctx: None,
            #[cfg(feature = "hotreload")]
            egui_drawn: false,
        }
    }

    /// executes deferred drawing, should be called once per frame
    pub async fn process(&mut self) {
        for (name, path) in self.loading.drain(..) {
            if let Err(_err) = self.textures.load_texture(&path, name, false).await {
                println!("Error loading {}", &path);
            }
        }

        let buffer = &mut self.draw_buffer.borrow_mut();
        buffer.sort_by_key(|it| it.z_level);
        let mut camera_switched = false;
        self.camera.set();
        for draw in buffer.drain(..) {
            if !camera_switched && draw.z_level >= 1000 {
                camera_switched = true;
                set_camera(&screen_camera());
                gl_use_default_material();
            }
            (draw.command)(&mut self.inner);
        }
    }
}

struct DrawCommand {
    z_level: i32,
    command: Box<dyn FnOnce(&mut ContextInner) -> ()>,
}
