use std::ops::Sub;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct IVec {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct FPos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct FVec {
    pub x: f32,
    pub y: f32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Sub<Pos> for Pos {
    type Output = IVec;

    fn sub(self, rhs: Pos) -> Self::Output {
        IVec { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl FPos {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn lerp(self, rhs: Self, s: f32) -> Self {
        let x = self.x + ((rhs.x - self.x) * s);
        let y = self.y + ((rhs.y - self.y) * s);
        Self { x, y }
    }

    pub fn distance(self, rhs: Self) -> f32 {
        (self - rhs).length()
    }
}

impl Sub for FPos {
    type Output = FVec;

    fn sub(self, rhs: Self) -> Self::Output {
        FVec { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl FVec {
    pub fn length(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
