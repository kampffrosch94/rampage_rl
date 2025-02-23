use std::{
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
};

use base::*;

use macroquad::prelude::*;
use notify::{Event, INotifyWatcher, RecursiveMode, Watcher};
mod camera;
mod context;
mod util;

/// this makes it possible to unload shared libraries even if they use
/// thread local storage with destructors
/// it leaks (a little bit) of memory though
///
/// see this post for details:
/// https://fasterthanli.me/articles/so-you-want-to-live-reload-rust
#[no_mangle]
pub unsafe extern "C" fn __cxa_thread_atexit_impl() {}

#[macroquad::main("Froql Reloaded")]
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

struct WorkerReloader {
    worker: Option<WorkerWrapper>,
    receiver: Receiver<Result<Event, notify::Error>>,
    path: PathBuf,
    #[allow(unused)]
    watcher: INotifyWatcher,
    persist_state: PersistWrapper,
}

#[allow(improper_ctypes_definitions)]
type UpdateFuncT =
    extern "C" fn(&mut dyn ContextTrait, &mut PersistWrapper, &mut PersistWrapper) -> ();

#[allow(improper_ctypes_definitions)]
type FleetingStateCreateFuncT = extern "C" fn() -> PersistWrapper;

#[allow(improper_ctypes_definitions)]
type PermanentStateCreateFuncT = extern "C" fn() -> PersistWrapper;

#[allow(improper_ctypes_definitions)]
type FleetingStateDisposeFuncT = extern "C" fn(&mut PersistWrapper, PersistWrapper);

struct WorkerWrapper {
    #[allow(unused)]
    lib: libloading::Library,
    #[allow(improper_ctypes_definitions)]
    update: UpdateFuncT,
    /// renewed on hotreload
    fleeting_state: PersistWrapper,
}

impl WorkerReloader {
    fn new(path: &str) -> Self {
        let path = PathBuf::from(path);
        let worker = Self::create_worker(&path);

        let (tx, receiver) = std::sync::mpsc::channel();

        let mut watcher = notify::recommended_watcher(tx).unwrap();
        watcher.watch(path.parent().unwrap(), RecursiveMode::NonRecursive).unwrap();

        let create: libloading::Symbol<PermanentStateCreateFuncT> =
            unsafe { worker.lib.get(b"permanent_state").unwrap() };
        let persist_state = create();

        let worker = Some(worker);
        Self { worker, watcher, receiver, path, persist_state }
    }

    fn create_worker(path: &Path) -> WorkerWrapper {
        unsafe {
            let lib = libloading::Library::new(path).unwrap();

            let symb: libloading::Symbol<UpdateFuncT> = lib.get(b"update").unwrap();
            let update: UpdateFuncT = std::mem::transmute(symb.into_raw());

            let fleeting_state_create: libloading::Symbol<FleetingStateCreateFuncT> =
                lib.get(b"fleeting_state_create").unwrap();
            let fleeting_state = fleeting_state_create();

            WorkerWrapper { lib, update, fleeting_state }
        }
    }

    fn update(&mut self, ctx: &mut dyn ContextTrait) {
        let mut modified = false; // debounce reloading twice on multiple events
        while let Ok(event) = self.receiver.try_recv() {
            if let Ok(e) = event {
                if e.kind.is_create()
                   && e.paths.iter().any(|p| p.file_name() == self.path.file_name())
                {
                    dbg!(&e);
                    modified = true;
                }
            }
        }

        if modified && Path::new(&self.path).exists() {
            // need to unload before we can reload
            {
                let worker = self.worker.take().unwrap();
                let fleeting_state_dispose: libloading::Symbol<FleetingStateDisposeFuncT> =
                    unsafe { worker.lib.get(b"fleeting_state_dispose").unwrap() };
                fleeting_state_dispose(&mut self.persist_state, worker.fleeting_state);
            }
            println!("Reloading!");
            self.worker = Some(Self::create_worker(&self.path));
        }

        let worker = self.worker.as_mut().unwrap();
        let update = worker.update;
        let fleeting_state = &mut worker.fleeting_state;
        let ps = &mut self.persist_state;

        update(ctx, ps, fleeting_state);
    }
}
