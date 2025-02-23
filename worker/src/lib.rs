use std::{ffi::c_void, panic::AssertUnwindSafe};

use base::{ContextTrait, PersistWrapper};
pub use fleeting::FleetingState;
pub use game::update_inner;
pub use persistent::PersistentState;
mod fleeting;
mod game;
mod genarena;
mod persistent;
mod util;

pub const GRIDSIZE: f32 = 16.;

#[no_mangle]
pub extern "C" fn permanent_state() -> PersistWrapper {
    let state = PersistentState::new();
    let size = size_of_val(&state);
    let align = align_of_val(&state);
    let boxed = Box::new(state);
    let ptr = Box::into_raw(boxed) as *mut c_void;
    PersistWrapper { ptr, size, align }
}

#[no_mangle]
pub extern "C" fn fleeting_state_create() -> PersistWrapper {
    let state = FleetingState::new();
    let size = size_of_val(&state);
    let align = align_of_val(&state);
    let boxed: Box<FleetingState> = Box::new(state);
    let ptr = Box::into_raw(boxed) as *mut c_void;
    PersistWrapper { ptr, size, align }
}

#[no_mangle]
pub extern "C" fn fleeting_state_dispose(pers: &mut PersistWrapper, fleet: PersistWrapper) {
    let ptr = fleet.ptr as *mut FleetingState;
    // put state into a box which gets dropped at the end of this method
    let mut boxed: Box<FleetingState> = unsafe { Box::from_raw(ptr) };
    boxed.co.run_blocking(pers.ref_mut());
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn update(
    c: &mut dyn ContextTrait,
    persistent_state: &mut PersistWrapper,
    fleeting_state: &mut PersistWrapper,
) {
    _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
        if persistent_state.align != align_of::<PersistentState>()
            || persistent_state.size != size_of::<PersistentState>()
        {
            println!("Reinit persistent state.");
            *persistent_state = permanent_state();
        }
        let s: &mut PersistentState = persistent_state.ref_mut();
        update_inner(c, s, fleeting_state.ref_mut());
    }));
}
