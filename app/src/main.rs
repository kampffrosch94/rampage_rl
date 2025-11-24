#[macro_use]
mod util;
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

/// this makes it possible to unload shared libraries even if they use
/// thread local storage with destructors
/// it leaks (a little bit) of memory though
///
/// see this post for details:
/// https://fasterthanli.me/articles/so-you-want-to-live-reload-rust
#[cfg(not(feature = "staticlink"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __cxa_thread_atexit_impl() {}

#[cfg(feature = "profile")]
#[global_allocator]
static ALLOC: base::tracy::alloc::GlobalAllocator = base::tracy::alloc::GlobalAllocator::new();

fn window_conf() -> Conf {
    Conf {
        window_title: "RampageRL".to_owned(),
        fullscreen: false,
        high_dpi: false,
        //sample_count: 1,
        ..Default::default()
    }
}

fn main() {
    println!("Starting");
    macroquad::Window::from_config(window_conf(), inner_main());
}

async fn inner_main() {
    #[cfg(not(feature = "staticlink"))]
    let mut worker = WorkerReloader::new("../target/debug/libworker.so");
    #[cfg(feature = "staticlink")]
    let mut worker = static_worker::StaticWorker::new();

    let ctx = &mut context::Context::new();

    #[cfg(not(feature = "staticlink"))]
    let prefix = "..";
    #[cfg(feature = "staticlink")]
    let prefix = ".";

    let asset_paths = [
        ("assets/32rogues/extruded-tileset.png", "tiles", false),
        ("assets/32rogues/rogues.png", "rogues", false),
        ("assets/pixeltest.png", "test", false),
        ("assets/32rogues/monsters.png", "monsters", false),
        ("assets/32rogues/items.png", "items", false),
        ("assets/32rogues/animals.png", "animals", false),
        ("assets/32rogues/autotiles.png", "autotiles", false),
        ("assets/32rogues/32rogues-palette.png", "palette", false),
        ("assets/32rogues/items-palette-swaps.png", "items-swaps", false),
        ("assets/32rogues/animated-tiles.png", "animated-tiles", false),
    ];

    for (path, shorthand, aa) in asset_paths {
        ctx.textures.load_texture(format!("{prefix}/{path}"), shorthand, aa).await.unwrap();
    }

    println!("Assets loaded.");

    #[cfg(feature = "hotreload")]
    let mut egui = egui_macroquad::Egui::new();
    #[allow(unused_mut)]
    let mut egui_drawn_before = false;

    loop {
        {
            base::zone!("Main loop");
            #[cfg(feature = "hotreload")]
            egui_inspector::reset_id();

            clear_background(BLACK);

            ctx.camera.process();

            if egui_drawn_before {
                #[cfg(feature = "hotreload")]
                egui.ui(|_, egui_ctx| {
                    egui_ctx.set_pixels_per_point(1.5);
                    egui::SidePanel::left("my_left_panel").show(egui_ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ctx.egui_ctx = Some(unsafe { std::mem::transmute(ui) });
                            worker.update(ctx);
                            ctx.egui_ctx = None;
                        });
                    });
                });
            } else {
                worker.update(ctx);
            }

            ctx.process().await;

            gl_use_default_material();

            #[cfg(feature = "hotreload")]
            {
                if egui_drawn_before {
                    egui.draw();
                }
                egui_drawn_before = ctx.egui_drawn;
                ctx.egui_drawn = false;
            }
        } // extra scope for tracy zone

        next_frame().await;

        base::frame!();
    }
}
