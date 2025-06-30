use fps_counter::FPSCounter;
#[cfg(not(feature = "staticlink"))]
use hotreload::WorkerReloader;
use macroquad::prelude::*;
mod camera;
mod context;
mod draw;
#[cfg(feature = "hotreload")]
mod egui_inspector;
#[cfg(feature = "hotreload")]
mod egui_macroquad;
mod fps_counter;
mod material;
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __cxa_thread_atexit_impl() {}

fn window_conf() -> Conf {
    Conf {
        window_title: "RampageRL".to_owned(),
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
        .load_texture(format!("{prefix}/assets/32rogues/tiles.png"), "tiles", false)
        .await
        .unwrap();

    ctx.textures
        .load_texture(format!("{prefix}/assets/32rogues/rogues.png"), "rogues", false)
        .await
        .unwrap();

    ctx.textures
        .load_texture(format!("{prefix}/assets/pixeltest.png"), "test", false)
        .await
        .unwrap();

    ctx.textures
        .load_texture(format!("{prefix}/assets/32rogues/monsters.png"), "monsters", false)
        .await
        .unwrap();

    ctx.textures
        .load_texture(format!("{prefix}/assets/32rogues/items.png"), "items", false)
        .await
        .unwrap();
    ctx.textures
        .load_texture(format!("{prefix}/assets/32rogues/animals.png"), "animals", false)
        .await
        .unwrap();
    ctx.textures
        .load_texture(format!("{prefix}/assets/32rogues/autotiles.png"), "autotiles", false)
        .await
        .unwrap();
    ctx.textures
        .load_texture(
            format!("{prefix}/assets/32rogues/animated-tiles.png"),
            "animated-tiles",
            false,
        )
        .await
        .unwrap();
    ctx.textures
        .load_texture(
            format!("{prefix}/assets/32rogues/32rogues-palette.png"),
            "palette",
            false,
        )
        .await
        .unwrap();
    ctx.textures
        .load_texture(
            format!("{prefix}/assets/32rogues/items-palette-swaps.png"),
            "items-swaps",
            false,
        )
        .await
        .unwrap();

    let mut fps_counter = FPSCounter::new();

    #[cfg(feature = "hotreload")]
    let mut egui = egui_macroquad::Egui::new();

    let mut egui_drawn_before = false;

    loop {
        #[cfg(feature = "hotreload")]
        egui_inspector::reset_id();

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

        // set_camera(&Camera2D::from_display_rect(Rect {
        //     x: 0.,
        //     y: screen_height(),
        //     w: screen_width(),
        //     h: -screen_height(),
        // }));
        ctx.camera.process();

        fps_counter.update();
        let fps = fps_counter.get_fps();
        let dpi = screen_dpi_scale();

        let w = screen_width();
        let h = screen_height();
        ctx.camera.set();
        let s = format!("FPS: {fps} DPI: {dpi} Screen: {w} x {h}");
        draw_text(&s, 20.0, -20.0, 30.0, WHITE);

        if egui_drawn_before {
            #[cfg(feature = "hotreload")]
            egui.ui(|_, egui_ctx| {
                egui_ctx.set_pixels_per_point(1.5);

                egui::SidePanel::left("my_left_panel").show(egui_ctx, |ui| {
                    ctx.egui_ctx = Some(unsafe { std::mem::transmute(ui) });
                    worker.update(ctx);
                    ctx.egui_ctx = None;
                });
            });
        } else {
            worker.update(ctx);
        }

        ctx.process().await;

        #[cfg(feature = "hotreload")]
        {
            if egui_drawn_before {
                egui.draw();
            }
            egui_drawn_before = ctx.egui_drawn;
            ctx.egui_drawn = false;
        }

        /*
        egui_macroquad::ui(|egui_ctx| {
            egui_macroquad::egui::Window::new("egui ❤ macroquad")
                .show(egui_ctx, |ui| {
                    ui.label("Test");
                });
        });

        // Draw things before egui

        egui_macroquad::draw();
        */

        gl_use_default_material();
        next_frame().await;
    }
}
