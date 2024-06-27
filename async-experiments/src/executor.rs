use std::{
    future::Future,
    pin::pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

pub struct Executor {}

impl Executor {
    pub fn new() -> Self {
        Executor {}
    }

    pub fn block_on<F: Future>(&self, fut: F) -> F::Output {
        let mut fut = pin!(fut);
        let this_thread = std::thread::current();
        let waker = Waker::from(Arc::new(WakeFn(move || {
            this_thread.unpark();
        })));
        let mut ctx = Context::from_waker(&waker);

        loop {
            let result = fut.as_mut().poll(&mut ctx);
            match result {
                Poll::Ready(output) => return output,
                Poll::Pending => std::thread::park(),
            }
        }
    }
}

struct WakeFn<F>(F);
impl<F: Fn()> Wake for WakeFn<F> {
    fn wake(self: std::sync::Arc<Self>) {
        (self.0)()
    }
}
