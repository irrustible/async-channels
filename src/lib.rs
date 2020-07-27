pub mod waker_queue;
pub mod mpsc;
pub mod spmc;
pub mod spsc;
pub use concurrent_queue::{PopError, PushError};

pub trait AsyncSender<T> : private::AsyncQueue<T> {}
pub trait AsyncReceiver<T> : private::AsyncQueue<T> {}

pub trait Sender<T> : private::AsyncQueue<T> {}
pub trait Receiver<T> : private::AsyncQueue<T> {}

pub fn try_send<T, S>(s: S, value: T) -> Result<(), PushError<T>>
where T: 'static + Send, S: AsyncSender<T> {
    s.queue().try_push(value)
}

pub fn try_recv<T, S>(s: S) -> Result<T, PopError>
where T: 'static + Send, S: AsyncSender<T> {
    s.queue().try_pop()
}

pub async fn send<T, S>(s: S, value: T) -> Result<(), PushError<T>>
where T: 'static + Send, S: AsyncSender<T> {
    s.queue().push(value).await
}

pub async fn recv<T, S>(s: S) -> Result<T, PopError>
where T: 'static + Send, S: AsyncSender<T> {
    s.queue().pop().await
}

mod private {
    pub trait AsyncQueue<T> {
        fn queue(&self) -> &crate::waker_queue::WakerQueue<T>;
    }
}
