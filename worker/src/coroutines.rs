use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use froql::world::World;

#[derive(Default)]
struct StateHolder {
    world: Option<NonNull<World>>,
}

#[derive(Clone)]
pub struct CoAccess {
    state: Rc<RefCell<StateHolder>>,
}

// can't actually Send this between threads
// but now we can make the futures require Send
// which gives us compile time checks on if we are holding a reference accross await points
//
// to make sure nothing untoward happens
// access must go through the CosyncRuntime and can only happen singlethreaded, so we are good
unsafe impl Send for CoAccess {}

impl CoAccess {
    pub fn get(&mut self) -> &mut World {
        unsafe { &mut *self.state.borrow_mut().world.unwrap().as_ptr() }
    }

    unsafe fn fill_with(&mut self, world: &mut World) {
        self.state.borrow_mut().world =
            Some(unsafe { NonNull::new_unchecked(world as *mut World) });
    }

    fn empty_out(&mut self) {
        self.state.borrow_mut().world = None;
    }

    fn new() -> Self {
        Self { state: Default::default() }
    }
}

pub fn sleep_ticks(ticks: usize) -> SleepForTick {
    SleepForTick::new(ticks)
}

#[derive(Clone, Copy, Debug)]
pub struct SleepForTick(pub usize);

impl SleepForTick {
    pub fn new(ticks: usize) -> Self {
        Self(ticks)
    }
}

impl Future for SleepForTick {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 == 0 {
            Poll::Ready(())
        } else {
            self.0 -= 1;
            Poll::Pending
        }
    }
}

pub struct CoroutineRuntime {
    routines: Vec<Pin<Box<dyn Future<Output = ()>>>>,
    access: CoAccess,
    cx: Context<'static>,
}

impl CoroutineRuntime {
    pub fn new() -> Self {
        Self {
            routines: Vec::new(),
            access: CoAccess::new(),
            cx: Context::from_waker(Waker::noop()),
        }
    }

    pub fn add_future<Fut, F>(&mut self, f: F)
    where
        // Send bound is so that &mut World references can't be held across suspend points
        Fut: Future<Output = ()> + 'static + Send,
        F: FnOnce(CoAccess) -> Fut,
    {
        let acc = self.access.clone();
        let p = Box::pin(f(acc));
        self.routines.push(p);
    }

    pub fn run_completing(&mut self, world: &mut World) {
        while self.routines.len() > 0 {
            self.run_step(world);
        }
    }

    pub fn run_step(&mut self, world: &mut World) {
        unsafe { self.access.fill_with(world) };
        let mut i = 0;
        while i < self.routines.len() {
            let c = &mut self.routines[i];
            let remove = match c.as_mut().poll(&mut self.cx) {
                Poll::Ready(()) => true,
                Poll::Pending => false,
            };
            if remove {
                let _ = self.routines.remove(i);
            } else {
                i += 1;
            }
        }
        self.access.empty_out();
    }

    pub fn is_empty(&self) -> bool {
        self.routines.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    async fn foobar(mut a: CoAccess) {
        println!("foo");
        sleep_ticks(3).await;
        let world = a.get();
        *world.singleton_mut::<i32>() = 0;
        println!("bar");
    }

    #[test]
    fn test_runtime() {
        let mut world = World::new();
        world.singleton_add(42);
        let mut rt = CoroutineRuntime::new();
        assert_eq!(*world.singleton::<i32>(), 42);
        rt.add_future(foobar);
        rt.run_completing(&mut world);
        assert_eq!(*world.singleton::<i32>(), 0);
    }
}
