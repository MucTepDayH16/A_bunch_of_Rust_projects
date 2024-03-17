use std::{
    future::Future,
    pin::pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::*,
    thread::{self, Thread},
};

#[derive(Debug)]
struct MyWaker {
    th: Thread,
    parked: AtomicBool,
}

impl Wake for MyWaker {
    fn wake(self: Arc<Self>) {
        if !self.parked.swap(false, Ordering::Release) {
            self.th.unpark();
        }
    }
}

pub fn block_on<Fut: Future>(fut: Fut) -> Fut::Output {
    static ENTER: AtomicBool = AtomicBool::new(false);

    ENTER
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .expect("Someone already entered");

    let mut fut = pin!(fut);

    let th = thread::current();
    let parked = AtomicBool::new(false);
    let waker = Arc::new(MyWaker { th, parked });
    let waker_clone = waker.clone().into();
    let mut cx = Context::from_waker(&waker_clone);

    loop {
        if let Poll::Ready(output) = fut.as_mut().poll(&mut cx) {
            ENTER.fetch_and(false, Ordering::Release);
            return output;
        }

        while waker.parked.swap(true, Ordering::Acquire) {
            thread::park();
        }
    }
}

fn main() {}
