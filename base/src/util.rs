use std::hash::{Hash, Hasher};

pub struct F32Helper(pub f32);

impl F32Helper {
    pub fn eq(a: &f32, b: &f32) -> bool {
        F32Helper(*a) == F32Helper(*b)
    }
}

impl PartialEq for F32Helper {
    fn eq(&self, other: &Self) -> bool {
        (self.0 * 1024.0) as u32 == (other.0 * 1024.0) as u32
    }
}

impl Hash for F32Helper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ((self.0 * 1024.0) as u32).hash(state);
    }
}
