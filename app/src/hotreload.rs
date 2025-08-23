use std::{
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
};

use base::*;

use notify::{Event, INotifyWatcher, RecursiveMode, Watcher};
pub struct WorkerReloader {
    worker: Option<WorkerWrapper>,
    receiver: Receiver<Result<Event, notify::Error>>,
    path: PathBuf,
    #[allow(unused)]
    watcher: INotifyWatcher,
    persist_state: PersistWrapper,
}

type PermanentStateCreateFuncT = extern "C" fn() -> PersistWrapper;

#[allow(improper_ctypes_definitions)]
type UpdateFuncT = extern "C" fn(&mut dyn ContextTrait, &mut PersistWrapper) -> ();

type BeforeReloadFuncT = extern "C" fn(&mut PersistWrapper);

type AfterReloadFuncT = extern "C" fn(&mut PersistWrapper);

struct WorkerWrapper {
    #[allow(unused)]
    lib: libloading::Library,
    update: UpdateFuncT,
}

impl WorkerReloader {
    pub fn new(path: &str) -> Self {
        let path = PathBuf::from(path);
        let worker = Self::create_worker(&path);

        let (tx, receiver) = std::sync::mpsc::channel();

        let mut watcher = notify::recommended_watcher(tx).unwrap();
        watcher.watch(path.parent().unwrap(), RecursiveMode::NonRecursive).unwrap();

        let create: libloading::Symbol<PermanentStateCreateFuncT> =
            unsafe { worker.lib.get(b"create_worker_state").unwrap() };
        let persist_state = create();

        let worker = Some(worker);
        Self { worker, watcher, receiver, path, persist_state }
    }

    fn create_worker(path: &Path) -> WorkerWrapper {
        unsafe {
            let lib = libloading::Library::new(path).unwrap();

            let symb: libloading::Symbol<UpdateFuncT> = lib.get(b"update").unwrap();
            let update: UpdateFuncT = std::mem::transmute(symb.into_raw());

            WorkerWrapper { lib, update }
        }
    }

    pub fn update(&mut self, ctx: &mut dyn ContextTrait) {
        let mut modified = false; // debounce reloading twice on multiple events
        while let Ok(event) = self.receiver.try_recv() {
            if let Ok(e) = event {
                if e.kind.is_create()
                    && e.paths.iter().any(|p| p.file_name() == self.path.file_name())
                {
                    // dbg!(&e);
                    modified = true;
                }
            }
        }

        if modified && Path::new(&self.path).exists() {
            // need to unload before we can reload
            {
                let worker = self.worker.take().unwrap();
                let before_reload: libloading::Symbol<BeforeReloadFuncT> =
                    unsafe { worker.lib.get(b"before_reload").unwrap() };
                before_reload(&mut self.persist_state);
            }
            println!("Reloading!");
            let worker = Self::create_worker(&self.path);
            {
                let after_reload: libloading::Symbol<AfterReloadFuncT> =
                    unsafe { worker.lib.get(b"after_reload").unwrap() };
                after_reload(&mut self.persist_state);
            }
            self.worker = Some(worker);
        }

        let worker = self.worker.as_mut().unwrap();
        let update = worker.update;
        let ps = &mut self.persist_state;

        update(ctx, ps);
    }
}
