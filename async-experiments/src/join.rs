use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub fn join2<F1, F2>(fut1: F1, fut2: F2) -> Join2<F1, F2>
where
    F1: Future,
    F2: Future,
{
    Join2(JoinState::Pending(fut1), JoinState::Pending(fut2))
}

pub struct Join2<F1: Future, F2: Future>(JoinState<F1>, JoinState<F2>);

#[derive(Debug)]
enum JoinState<F: Future> {
    Pending(F),
    Ready(F::Output),
    Stolen,
}
impl<F: Future> JoinState<F> {
    fn steal(&mut self) -> F::Output {
        match std::mem::replace(self, JoinState::Stolen) {
            JoinState::Ready(output) => output,
            _ => unreachable!("tried to take output of non-ready join state"),
        }
    }
}

impl<F1: Future, F2: Future> Future for Join2<F1, F2> {
    type Output = (F1::Output, F2::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        fn make_progress<F: Future>(field: &mut JoinState<F>, cx: &mut Context<'_>) {
            match field {
                JoinState::Pending(fut) => match unsafe { Pin::new_unchecked(fut) }.poll(cx) {
                    Poll::Ready(result) => {
                        *field = JoinState::Ready(result);
                    }
                    Poll::Pending => {}
                },

                JoinState::Ready(_) => {}
                JoinState::Stolen => unreachable!("future polled after completion"),
            }
        }

        make_progress(&mut this.0, cx);
        make_progress(&mut this.1, cx);

        if let (JoinState::Ready(_), JoinState::Ready(_)) = (&this.0, &this.1) {
            return Poll::Ready((this.0.steal(), this.1.steal()));
        }

        Poll::Pending
    }
}

impl<F1: Future + Debug, F2: Future + Debug> Debug for Join2<F1, F2>
where
    F1::Output: Debug,
    F2::Output: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Join2")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}
