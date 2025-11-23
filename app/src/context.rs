use std::cell::RefCell;

use base::{text::Label, *};

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
    #[cfg(feature = "hotreload")]
    /// used for Window IDs and the like
    pub egui_id: u32,
}

pub struct ContextInner {
    pub texter: Texter,
    #[expect(unused)]
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
        zone!("Context::draw_rect");

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
        zone!("Context::draw_rect_lines");
        let color = macroquad::prelude::Color { r: c.r, g: c.g, b: c.b, a: c.a };

        let command = move |_: &mut ContextInner| {
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, thickness, color);
        };
        self.draw_buffer
            .borrow_mut()
            .push(DrawCommand { z_level, command: Box::new(command) });
    }

    fn draw_circle(&mut self, circle: base::Circle, c: base::Color, z_level: i32) {
        zone!("Context::draw_circle");
        let color = macroquad::prelude::Color { r: c.r, g: c.g, b: c.b, a: c.a };

        let command = move |_: &mut ContextInner| {
            draw_circle(circle.pos.x, circle.pos.y, circle.radius, color);
        };
        self.draw_buffer
            .borrow_mut()
            .push(DrawCommand { z_level, command: Box::new(command) });
    }

    fn draw_texture(&mut self, name: &str, x: f32, y: f32, z_level: i32) {
        zone!("Context::draw_texture");
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
        zone!("Context::draw_texture_part");
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
        zone!("Context::draw_texture_part_scaled");
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
        zone!("Context::load_texture");
        self.loading.push((name.to_string(), path.to_string()));
    }

    fn texture_dimensions(&mut self, name: &str) -> base::Rect {
        zone!("Context::texture_dimensions");
        self.textures
            .get(name)
            .map(|t| base::Rect { x: 0., y: 0., w: t.width(), h: t.width() })
            .unwrap_or(base::Rect { x: 0., y: 0., w: 0., h: 0. })
    }

    fn is_pressed(&self, input: Input) -> bool {
        zone!("Context::is_pressed");
        match input {
            Input::MouseLeft => is_mouse_button_pressed(MouseButton::Left),
            Input::MouseMiddle => is_mouse_button_pressed(MouseButton::Middle),
            Input::MouseRight => is_mouse_button_pressed(MouseButton::Right),
            Input::RestartGame => is_key_pressed(KeyCode::F2),
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
            Input::Confirm => {
                is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::KpEnter)
            }
            Input::Cancel => is_key_pressed(KeyCode::Escape),
            Input::Inventory => is_key_pressed(KeyCode::I),
            Input::DebugSlowdown => is_key_pressed(KeyCode::F12),
            Input::Inspect => is_key_pressed(KeyCode::X),
            Input::MouseMoveCamera => {
                is_mouse_button_down(MouseButton::Middle)
                    || is_mouse_button_down(MouseButton::Right)
            }
            Input::DebugToggle => is_key_pressed(KeyCode::F1),
            Input::Ability1 => is_key_pressed(KeyCode::Key1),
            Input::Ability2 => is_key_pressed(KeyCode::Key2),
            Input::Ability3 => is_key_pressed(KeyCode::Key3),
            Input::Ability4 => is_key_pressed(KeyCode::Key4),
            Input::Ability5 => is_key_pressed(KeyCode::Key5),
            Input::Test => is_key_pressed(KeyCode::Period),
        }
    }

    fn mouse_screen(&self) -> FPos {
        zone!("Context::mouse_screen");
        let m = mouse_position();
        FPos { x: m.0, y: m.1 }
    }

    fn mouse_world(&self) -> FPos {
        zone!("Context::mouse_world");
        let m = self.camera.mouse_world();
        FPos { x: m.x, y: m.y }
    }

    fn camera_set_shake(&mut self, offset: FVec) {
        zone!("Context::camera_set_shake");
        self.camera.shake_offset.x = offset.x;
        self.camera.shake_offset.y = offset.y;
    }

    fn camera_move_rel(&mut self, offset: FVec) {
        zone!("Context::camera_move_rel");
        self.camera.move_camera_relativ(offset);
    }

    fn text(&mut self, FVec { x: w, y: h }: FVec, text: &[(&str, TextProperty)]) -> Label {
        zone!("Context::text");
        self.inner.texter.set_text(w, h, text)
    }

    fn draw_text(&mut self, handle: u128, FPos { x, y }: FPos, z_level: i32) {
        zone!("Context::draw_text");
        let command = move |inner: &mut ContextInner| {
            if inner.texter.draw_text(handle, x, y).is_none() {
                let rect = macroquad::prelude::Rect::new(x, y, 300., 300.);
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, PINK);
            }
        };
        self.draw_buffer
            .borrow_mut()
            .push(DrawCommand { z_level, command: Box::new(command) });
    }

    fn screen_rect(&self) -> base::Rect {
        zone!("Context::screen_rect");
        base::Rect::new(0., 0., screen_width(), screen_height())
    }

    #[cfg(feature = "hotreload")]
    fn inspect(&mut self, val: &mut ValueReflection) {
        zone!("Context::inspect");
        self.egui_drawn = true;
        use crate::egui_inspector::draw_value;
        if let Some(ref mut ui) = self.egui_ctx {
            draw_value(ui, val);
            ui.separator();
        }
    }

    #[cfg(feature = "hotreload")]
    fn inspect_str(&mut self, s: &str) {
        zone!("Context::inspect_str");
        self.egui_drawn = true;
        if let Some(ref mut ui) = self.egui_ctx {
            let id = egui::Id::new(self.egui_id);
            self.egui_id += 1;
            egui::Window::new("Adhoc Debug").id(id).show(ui.ctx(), |ui| {
                ui.label(s);
            });
        }
    }

    fn mouse_wheel(&self) -> f32 {
        zone!("Context::mouse_wheel");
        let (_x, y) = mouse_wheel();
        y
    }

    fn camera_zoom(&mut self, change: i32) {
        zone!("Context::mouse_wheel");
        self.camera.zoom(change);
    }

    fn screen_rect_world(&self) -> base::Rect {
        zone!("Context::screen_rect_world");
        let base::Rect { x, y, w, h } = self.screen_rect();
        let FPos { x: x_new, y: y_new } = self.camera.screen_to_world(FPos { x, y });
        let FPos { x: xe, y: ye } = self.camera.screen_to_world(FPos { x: x + w, y: y + h });
        base::Rect { x: x_new, y: y_new, w: xe - x_new, h: ye - y_new }
    }

    fn avy_label(&self, choice: u32) -> &'static str {
        zone!("Context::avy_label");
        if let Some((label, _key)) = AVY_KEYS.get(choice as usize) {
            label
        } else {
            println!("Missing avy label for choice {choice}");
            "todo:avy"
        }
    }

    fn avy_is_key_pressed(&self) -> Option<u32> {
        zone!("Context::avy_is_key_pressed");
        for (i, (_label, key)) in AVY_KEYS.iter().enumerate() {
            if is_key_pressed(*key) {
                return Some(i as u32);
            }
        }
        None
    }

    fn camera_world_to_screen(&mut self, pos: FPos) -> FPos {
        zone!("Context::world_to_screen");
        self.camera.world_to_screen(pos)
    }

    fn camera_screen_to_world(&mut self, pos: FPos) -> FPos {
        zone!("Context::screen_to_world");
        self.camera.screen_to_world(pos)
    }
}

const AVY_KEYS: [(&str, KeyCode); 12] = [
    ("a", KeyCode::A),
    ("s", KeyCode::S),
    ("d", KeyCode::D),
    ("f", KeyCode::F),
    ("q", KeyCode::Q),
    ("w", KeyCode::W),
    ("e", KeyCode::E),
    ("r", KeyCode::R),
    ("y", KeyCode::Y),
    ("x", KeyCode::X),
    ("c", KeyCode::C),
    ("v", KeyCode::V),
];

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
            #[cfg(feature = "hotreload")]
            egui_id: 0,
        }
    }

    /// executes deferred drawing, should be called once per frame
    pub async fn process(&mut self) {
        #[cfg(feature = "profile")]
        tracy::zone!("Context::process");

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

        self.inner.texter.collect_garbage();
        #[cfg(feature = "hotreload")]
        {
            self.egui_id = 0;
        }
    }
}

struct DrawCommand {
    z_level: i32,
    command: Box<dyn FnOnce(&mut ContextInner) -> ()>,
}
