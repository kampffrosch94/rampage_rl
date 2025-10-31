#[cfg(not(target_arch = "wasm32"))]
use std::{ffi::c_void, panic::AssertUnwindSafe};

#[cfg(not(target_arch = "wasm32"))]
use base::{ContextTrait, PersistWrapper};
pub use game::update_inner;
pub use persistent::PersistentState;
mod animation;
mod dijstra;
mod ecs_setup;
mod game;
mod persistent;
mod quicksilver_glue;
mod rand;
mod util;

pub const GRIDSIZE: f32 = 16.;

#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub extern "C" fn create_worker_state() -> PersistWrapper {
    // println!("Create state");
    let state = PersistentState::new();
    let size = size_of_val(&state);
    let align = align_of_val(&state);
    let boxed = Box::new(state);
    let ptr = Box::into_raw(boxed) as *mut c_void;
    PersistWrapper { ptr, size, align }
}

#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub extern "C" fn after_reload(pers: &mut PersistWrapper) {
    // println!("After reload.");
    let state = &mut pers.ref_mut::<PersistentState>();
    state.hot_load();
}

#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub extern "C" fn before_reload(pers: &mut PersistWrapper) {
    // println!("Before reload.");
    let state = &mut pers.ref_mut::<PersistentState>();
    state.world.as_mut().unwrap().process();
    state.hot_save();
}

#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn update(c: &mut dyn ContextTrait, persistent_state: &mut PersistWrapper) {
    _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
        if persistent_state.align != align_of::<PersistentState>()
            || persistent_state.size != size_of::<PersistentState>()
        {
            println!("Reinit persistent state.");
            *persistent_state = create_worker_state();
        }
        let s: &mut PersistentState = persistent_state.ref_mut();
        update_inner(c, s);
    }));
}
