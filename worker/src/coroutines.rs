use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

struct StateHolder {
    world: *mut u8,
}

#[derive(Clone)]
struct CoAccess {
    state: Rc<RefCell<StateHolder>>,
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
    fn add_future<Fut, F>(&mut self, f: F)
    where
        Fut: Future<Output = ()> + 'static,
        F: FnOnce(CoAccess) -> Fut,
    {
        let acc = self.access.clone();
        let p = Box::pin(f(acc));
        self.routines.push(p);
    }

    fn run_complete(&mut self) {
        let mut cx = Context::from_waker(Waker::noop());
        while self.routines.len() > 0 {
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
        }
    }
}

#[cfg(test)]
mod test {
    use std::task::Waker;

    use super::*;

    async fn foobar(a: CoAccess) {
        println!("foo");
        sleep_ticks(3).await;
        unsafe {
            *a.state.borrow_mut().world = 0;
        }
        println!("bar");
    }

    #[test]
    fn test_basic() {
        let mut cx = Context::from_waker(Waker::noop());

        let mut world = 42;
        let state = Rc::new(RefCell::new(StateHolder { world: &raw mut world }));
        let acc = CoAccess { state: state.clone() };
        let mut future = Box::pin(foobar(acc));
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        println!("{world}");
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Ready(()));
        println!("{world}");
    }

    #[test]
    fn test_runtime() {
        let mut world = 42;
        let holder = StateHolder { world: &raw mut world };
        let state = Rc::new(RefCell::new(holder));
        let access = CoAccess { state: state.clone() };

        let mut rt = CoroutineRuntime {
            routines: Vec::new(),
            access,
        };
        rt.add_future(foobar);
        println!("{world}");
        rt.run_complete();
        println!("{world}");
    }
}
