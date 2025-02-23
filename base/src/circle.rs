use crate::{Circle, FPos, Rect};

impl Circle {
    pub fn new(x: f32, y: f32, radius: f32) -> Self {
        Circle { pos: FPos { x, y }, radius }
    }

    pub fn overlaps_rect(&self, rect: &Rect) -> bool {
        let dist_x = (self.pos.x - rect.center().x).abs();
        let dist_y = (self.pos.y - rect.center().y).abs();
        if dist_x > rect.w / 2.0 + self.radius || dist_y > rect.h / 2.0 + self.radius {
            return false;
        }
        if dist_x <= rect.w / 2.0 || dist_y <= rect.h / 2.0 {
            return true;
        }
        let lhs = dist_x - rect.w / 2.0;
        let rhs = dist_y - rect.h / 2.0;
        let dist_sq = (lhs * lhs) + (rhs * rhs);
        return dist_sq <= self.radius * self.radius;
    }

    pub fn contains(&self, pos: FPos) -> bool {
        return pos.distance(self.pos) < self.radius;
    }
}
