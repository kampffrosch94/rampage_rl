use std::{
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
};

use base::*;

use hotreload::WorkerReloader;
use macroquad::prelude::*;
mod camera;
mod context;
mod hotreload;
mod util;

/// this makes it possible to unload shared libraries even if they use
/// thread local storage with destructors
/// it leaks (a little bit) of memory though
///
/// see this post for details:
/// https://fasterthanli.me/articles/so-you-want-to-live-reload-rust
#[no_mangle]
pub unsafe extern "C" fn __cxa_thread_atexit_impl() {}

#[macroquad::main("Roguelike Template")]
async fn main() {
    let path = "../target/debug/libworker.so";
    let mut worker = WorkerReloader::new(path);

    let mut last_mouse_pos = mouse_position();
    let ctx = &mut context::Context::new();
    ctx.textures
       .load_texture("../assets/tilemap/tilemap_packed.png", "tiles", false)
       .await
       .unwrap();

    ctx.textures
       .load_texture("../assets/ui/rectangle_flat.png", "ui_bg", false)
       .await
       .unwrap();

    loop {
        clear_background(BLACK);

        if is_mouse_button_down(MouseButton::Middle) {
            ctx.camera.mouse_delta(last_mouse_pos, mouse_position());
        }

        last_mouse_pos = mouse_position();
        match mouse_wheel() {
            (_x, y) => {
                if y != 0. {
                    if y > 0. {
                        ctx.camera.zoom(1);
                    }
                    if y < 0. {
                        ctx.camera.zoom(-1);
                    }
                }
            }
        }

        ctx.camera.process();

        // let start = Instant::now();
        worker.update(ctx);
        // let duration = start.elapsed();
        // println!("Reload + Execution took: {:?}", duration)));

        ctx.process().await;

        let fps = get_fps();
        let s = format!("FPS: {}", if fps > 55 && fps < 65 { 60 } else { fps });
        draw_text(&s, 20.0, 20.0, 30.0, WHITE);

        next_frame().await
    }
}
