use std::ffi::c_void;
use std::hash::Hash;

pub mod circle;
pub mod grids;
pub mod input;
pub mod pos;
pub mod rational;
pub mod rect;
pub mod shadowcasting;
pub mod text;
pub mod util;

pub use circle::Circle;
pub use input::Input;
pub use pos::{FPos, FVec, Pos};
pub use quicksilver;
use quicksilver::reflections::ValueReflection;
pub use rect::Rect;
use text::Label;
pub use text::TextProperty;
use util::F32Helper;

pub trait ContextTrait {
    /// time since program start
    fn time(&self) -> f64;

    /// frame delta time
    fn delta(&self) -> f32;

    /// frames per second
    fn fps(&self) -> f32;

    fn draw_rect(&mut self, rect: Rect, c: Color, z_level: i32);

    fn draw_rect_lines(&mut self, rect: Rect, thickness: f32, c: Color, z_level: i32);

    fn draw_circle(&mut self, circle: Circle, c: Color, z_level: i32);

    fn draw_texture(&mut self, name: &str, x: f32, y: f32, z_level: i32);

    fn draw_texture_part(&mut self, name: &str, src: Rect, x: f32, y: f32, z_level: i32);

    fn draw_texture_part_scaled(&mut self, name: &str, src: Rect, target: Rect, z_level: i32);

    fn load_texture(&mut self, name: &str, path: &str);

    fn texture_dimensions(&mut self, name: &str) -> Rect;

    fn is_pressed(&self, input: Input) -> bool;

    fn mouse_screen(&self) -> FPos;

    fn mouse_world(&self) -> FPos;

    fn mouse_wheel(&self) -> f32;

    fn camera_zoom(&mut self, change: i32);

    fn camera_set_shake(&mut self, offset: FVec);

    /// moves the camera center over time
    fn camera_move_rel(&mut self, offset: FVec);

    fn set_text(&mut self, w: f32, h: f32, text: &[(&str, TextProperty)]) -> Label;

    fn draw_text(&mut self, handle: u128, x: f32, y: f32, z_level: i32);

    /// Screen rect in screen coordinates
    fn screen_rect(&self) -> Rect;

    /// Screen rect, but translated to world coordinates
    fn screen_rect_world(&self) -> Rect;

    //fn inspect(&mut self, _val: ValueReflection) {}
    fn inspect(&mut self, _val: &mut ValueReflection) {}
}

/// Wrapper for state that is persisted between reloads
#[repr(C)]
pub struct PersistWrapper {
    pub ptr: *mut c_void,
    pub size: usize,
    pub align: usize,
}

impl PersistWrapper {
    pub fn ref_mut<T>(&mut self) -> &mut T {
        // TODO add checks for size and alignment matching
        let ptr = self.ptr as *mut T;
        unsafe { &mut *ptr }
    }
}

/// rgba values from 0.0 to 1.0
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.3, 0.3, 1.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const VIOLET: Color = Color::rgb(0.5, 0.0, 0.5);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const PINK: Color = Color::rgb(1.0, 0.75, 0.8);
    pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub const fn alpha(self, a: f32) -> Self {
        Color { r: self.r, g: self.g, b: self.b, a }
    }
}

impl Hash for Color {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Self { r, g, b, a } = self;
        F32Helper(*r).hash(state);
        F32Helper(*g).hash(state);
        F32Helper(*b).hash(state);
        F32Helper(*a).hash(state);
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        let Self { r, g, b, a } = self;
        let Self { r: or, g: og, b: ob, a: oa } = other;
        F32Helper::eq(r, or)
            && F32Helper::eq(g, og)
            && F32Helper::eq(b, ob)
            && F32Helper::eq(a, oa)
    }
}

impl Eq for Color {}
