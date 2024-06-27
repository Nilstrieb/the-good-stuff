use std::{
    fmt::Debug,
    future::Future,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

#[derive(Debug)]
pub struct JoinHandle<T> {
    inner: Arc<Inner<T>>,
}

struct Inner<T> {
    result: Mutex<Option<T>>,
    waker: Mutex<Option<Waker>>,
}

pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    R: Send + 'static,
    F: Send + FnOnce() -> R + 'static,
{
    let inner = Arc::new(Inner {
        result: Mutex::new(None),
        waker: Mutex::new(None),
    });
    let inner2 = inner.clone();
    std::thread::spawn(move || {
        let result = f();
        *inner2.result.lock().unwrap() = Some(result);
        if let Some(waker) = inner2.waker.lock().unwrap().take() {
            waker.wake();
        }
    });

    JoinHandle { inner }
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut result = self.inner.result.lock().unwrap();
        match result.take() {
            Some(result) => Poll::Ready(result),
            None => {
                *self.inner.waker.lock().unwrap() = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<T: Debug> Debug for Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner")
            .field("result", &self.result)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use crate::Executor;

    #[test]
    fn spawn_value() {
        let executor = Executor::new();

        let result = executor.block_on(super::spawn_blocking(|| 1 + 1));
        assert_eq!(result, 2);
    }
}
