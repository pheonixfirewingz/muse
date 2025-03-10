use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake};
use std::thread;
struct SimpleWaker;
impl Wake for SimpleWaker {
    /// Does nothing, as we're not waiting on anything.
    ///
    /// This is only needed to satisfy the `Wake` trait, which is itself
    /// required to use the `block_on` function in this module.
    fn wake(self: Arc<Self>) {}
}

/// Blocks the current thread until the provided future has resolved.
///
/// This function is intended for use in single-threaded, synchronous code.
/// It is not intended for use in asynchronous code, and should not be used
/// in conjunction with an async runtime.
///
/// The function takes a future as an argument, blocks the current thread,
/// and then returns the output of the future once it has resolved.
pub fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);
    let waker = Arc::new(SimpleWaker).into();
    let mut context = Context::from_waker(&waker);

    loop {
        match Future::poll(future.as_mut(), &mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::yield_now(),
        }
    }
}