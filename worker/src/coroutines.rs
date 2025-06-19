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

#[derive(Clone, Default)]
pub struct CoAccess {
    state: Rc<RefCell<StateHolder>>,
}

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
}

impl CoroutineRuntime {
    pub fn new() -> Self {
        Self { routines: Vec::new(), access: CoAccess::default() }
    }

    pub fn queue<Fut, F>(&mut self, f: F)
    where
        Fut: Future<Output = ()> + 'static,
        F: FnOnce(CoAccess) -> Fut,
    {
        let acc = self.access.clone();
        let p = Box::pin(f(acc));
        self.routines.push(p);
    }

    pub fn run_until_stall(&mut self, world: &mut World) {
        while self.routines.len() > 0 {
            self.run_blocking(world);
        }
    }

    pub fn run_blocking(&mut self, world: &mut World) {
        let mut cx = Context::from_waker(Waker::noop());
        unsafe { self.access.fill_with(world) };
        for i in 0..self.routines.len() {
            let c = &mut self.routines[i];
            let remove = match c.as_mut().poll(&mut cx) {
                Poll::Ready(()) => true,
                Poll::Pending => false,
            };
            if remove {
                let _ = self.routines.remove(i);
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

    async fn foobar(mut _a: CoAccess) {
        println!("foo");
        sleep_ticks(3).await;
        println!("bar");
    }

    #[test]
    fn test_runtime() {
        let mut world = World::new();
        let mut rt = CoroutineRuntime::new();
        rt.queue(foobar);
        rt.run_blocking(&mut world);
    }
}
