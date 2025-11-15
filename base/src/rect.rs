use crate::{Color, ContextTrait, FPos, FVec};

/// x and y are in the top left
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn new_wh(w: f32, h: f32) -> Self {
        Rect { x: 0.0, y: 0.0, w, h }
    }

    pub fn new_dim(FVec { x: w, y: h }: FVec) -> Self {
        Rect { x: 0.0, y: 0.0, w, h }
    }

    pub fn with_dim(mut self, FVec { x: w, y: h }: FVec) -> Self {
        self.w = w;
        self.h = h;
        self
    }

    pub fn new_center_wh(FPos { x, y }: FPos, w: f32, h: f32) -> Self {
        Rect { x: x - w / 2.0, y: y - h / 2.0, w, h }
    }

    pub fn translate(&self, FVec { x: dx, y: dy }: FVec) -> Self {
        let Rect { x, y, w, h } = self;
        Rect { x: x + dx, y: y + dy, w: *w, h: *h }
    }

    pub fn take_left(&self, amount: f32) -> Self {
        Rect { x: self.x, y: self.y, w: self.w.min(amount), h: self.h }
    }

    pub fn take_top(&self, amount: f32) -> Self {
        Rect { x: self.x, y: self.y, w: self.w, h: self.h.min(amount) }
    }

    pub fn take_right(&self, amount: f32) -> Self {
        Rect {
            x: self.x.max(self.x + self.w - amount),
            y: self.y,
            w: self.w.min(amount),
            h: self.h,
        }
    }

    pub fn take_bot(&self, amount: f32) -> Self {
        Rect {
            x: self.x,
            y: self.y.max(self.y + self.h - amount),
            w: self.w,
            h: self.h.min(amount),
        }
    }

    pub fn cut_top(&mut self, amount: f32) -> Self {
        let r = self.take_top(amount);
        self.y += r.h;
        self.h -= r.h;
        r
    }

    pub fn cut_left(&mut self, amount: f32) -> Self {
        let r = self.take_left(amount);
        self.x += r.w;
        self.w -= r.w;
        r
    }

    /// the rect without specified amount of space on the left side
    pub fn skip_left(&self, amount: f32) -> Self {
        Rect { x: self.x + amount, y: self.y, w: self.w - amount, h: self.h }
    }

    /// the rect without specified amount of space on the top side
    pub fn skip_top(&self, amount: f32) -> Self {
        Rect { x: self.x, y: self.y + amount, w: self.w, h: self.h - amount }
    }

    /// the rect without specified amount of space on the right side
    pub fn skip_right(&self, amount: f32) -> Self {
        Rect { x: self.x, y: self.y, w: self.w - amount, h: self.h }
    }

    /// the rect without specified amount of space on the bottom side
    pub fn skip_bot(&self, amount: f32) -> Self {
        Rect { x: self.x, y: self.y, w: self.w, h: self.h - amount }
    }

    pub fn skip_all(&self, amount: f32) -> Self {
        Rect {
            x: self.x + amount,
            y: self.y + amount,
            w: self.w - 2. * amount,
            h: self.h - 2. * amount,
        }
    }

    pub fn grow_all(&self, amount: f32) -> Self {
        Rect {
            x: self.x - amount,
            y: self.y - amount,
            w: self.w + 2. * amount,
            h: self.h + 2. * amount,
        }
    }

    /// resulting rect contains both components
    pub fn fuse(&self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);

        // right end x coordinate
        let rx = (self.x + self.w).max(other.x + other.w);
        // bottom end y coordinate
        let by = (self.y + self.h).max(other.y + other.h);

        let w = rx - x;
        let h = by - y;

        Rect { x, y, w, h }
    }

    pub fn scale(&self, factor: f32) -> Self {
        Rect { x: self.x, y: self.y, w: self.w * factor, h: self.h * factor }
    }

    pub fn contains(&self, pos: FPos) -> bool {
        self.x <= pos.x
            && pos.x < self.x + self.w
            && self.y <= pos.y
            && pos.y < self.y + self.h
    }

    pub fn center(&self) -> FPos {
        FPos { x: self.x + self.w / 2.0, y: self.y + self.h / 2.0 }
    }

    pub fn move_by(&self, x: f32, y: f32) -> Self {
        Rect { x: self.x + x, y: self.y + y, w: self.w, h: self.h }
    }

    pub fn move_by_pos(&self, pos: FPos) -> Rect {
        self.move_by(pos.x, pos.y)
    }

    pub fn origin(&self) -> FPos {
        FPos { x: self.x, y: self.y }
    }

    /// dimensions: width & height
    pub fn dim(&self) -> FVec {
        FVec { x: self.w, y: self.h }
    }

    pub fn draw(self, c: &mut dyn ContextTrait, color: Color, z_level: i32) -> Self {
        c.draw_rect(self, color, z_level);
        self
    }

    pub fn draw_lines(
        self,
        c: &mut dyn ContextTrait,
        thickness: f32,
        color: Color,
        z_level: i32,
    ) {
        c.draw_rect_lines(self, thickness, color, z_level);
    }
}
