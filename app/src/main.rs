#[cfg(not(feature = "staticlink"))]
use hotreload::WorkerReloader;
use macroquad::prelude::*;
mod camera;
mod context;
mod draw;
mod text;

#[cfg(all(feature = "staticlink", feature = "hotreload"))]
compile_error!("features hotreload and staticlink can't be enabled at the same time");

#[cfg(not(feature = "staticlink"))]
mod hotreload;
#[cfg(feature = "staticlink")]
mod static_worker;
mod util;

/// this makes it possible to unload shared libraries even if they use
/// thread local storage with destructors
/// it leaks (a little bit) of memory though
///
/// see this post for details:
/// https://fasterthanli.me/articles/so-you-want-to-live-reload-rust
#[cfg(not(feature = "staticlink"))]
#[no_mangle]
pub unsafe extern "C" fn __cxa_thread_atexit_impl() {}

fn window_conf() -> Conf {
    Conf {
        window_title: "Roguelike".to_owned(),
        fullscreen: false,
        high_dpi: false,
        //sample_count: 1,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    #[cfg(not(feature = "staticlink"))]
    let mut worker = WorkerReloader::new("../target/debug/libworker.so");
    #[cfg(feature = "staticlink")]
    let mut worker = static_worker::StaticWorker::new();

    set_default_filter_mode(FilterMode::Nearest);

    let mut last_mouse_pos = mouse_position();
    let ctx = &mut context::Context::new();
    #[cfg(not(feature = "staticlink"))]
    let prefix = "..";
    #[cfg(feature = "staticlink")]
    let prefix = ".";
    ctx.textures
        .load_texture(format!("{prefix}/assets/tilemap/tilemap_packed.png"), "tiles", false)
        .await
        .unwrap();

    ctx.textures
        .load_texture(format!("{prefix}/assets/ui/rectangle_flat.png"), "ui_bg", true)
        .await
        .unwrap();

    ctx.textures
        .load_texture(format!("{prefix}/assets/pixeltest.png"), "test", true)
        .await
        .unwrap();

    loop {
        clear_background(Color { r: 0., g: 0., b: 0., a: 0. });

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

        // set_camera(&Camera2D::from_display_rect(Rect {
        //     x: 0.,
        //     y: screen_height(),
        //     w: screen_width(),
        //     h: -screen_height(),
        // }));
        ctx.camera.process();

        worker.update(ctx);

        ctx.process().await;

        let fps = get_fps();
        let fps = if fps > 55 && fps < 65 { 60 } else { fps };
        let dpi = screen_dpi_scale();

        let w = screen_width();
        let h = screen_height();
        let s = format!("FPS: {fps} DPI: {dpi} Screen: {w} x {h}");
        draw_text(&s, 20.0, 20.0, 30.0, WHITE);

        next_frame().await
    }
}
