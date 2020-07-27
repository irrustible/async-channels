use atomic_waker::AtomicWaker;
use concurrent_queue::{ConcurrentQueue, PopError, PushError};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

pub struct WakerQueue<T> {
    queue: ConcurrentQueue<T>,
    waker: AtomicWaker,
}

impl<T: 'static + Send> WakerQueue<T> {
    pub fn bounded(size: usize) -> WakerQueue<T> {
        WakerQueue {
            queue: ConcurrentQueue::bounded(size),
            waker: AtomicWaker::new(),
        }
    }

    pub fn unbounded() -> WakerQueue<T> {
        WakerQueue {
            queue: ConcurrentQueue::unbounded(),
            waker: AtomicWaker::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn capacity(&self) -> Option<usize> {
        self.queue.capacity()
    }

    pub fn close(&self) {
        self.queue.close();
    }

    pub fn is_closed(&self) -> bool {
        self.queue.is_closed()
    }

    pub fn try_push(&self, value: T) -> Result<(), PushError<T>> {
        self.queue.push(value)
    }

    pub fn try_push_wake(&self, value: T) -> Result<(), PushError<T>> {
        let was_empty = self.queue.is_empty();
        match self.queue.push(value) {
            Ok(()) => {
                if was_empty { self.waker.wake(); }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
    
    pub fn try_pop(&self) -> Result<T, PopError> {
        self.queue.pop()
    }

    pub fn try_pop_wake(&self) -> Result<T, PopError> {
        let was_full = self.queue.is_full();
        match self.queue.pop() {
            Ok(val) => {
                if was_full { self.wake(); }
                Ok(val)
            }
            Err(e) => Err(e),
        }
    }

    pub fn push<'a>(&'a self, value:T) -> Push<'a, T> {
        Push::new(self, value)
    }

    pub fn pop<'a>(&'a self) -> Pop<'a, T> {
        Pop::new(self)
    }

    pub fn register(&self, waker: &Waker) {
        self.waker.register(waker);
    }

    pub fn wake(&self) {
        self.waker.wake();
    }
}

unsafe impl<T: 'static + Send> Send for WakerQueue<T> {}
unsafe impl<T: 'static + Send> Sync for WakerQueue<T> {}

pin_project! {
    pub struct Push<'a, T> {
        queue: &'a WakerQueue<T>,
        value: Option<T>,
    }
}

impl<'a, T: 'static + Send> Push<'a, T> {
    fn new(queue: &'a WakerQueue<T>, value: T) -> Push<'a, T> {
        Push { queue, value: Some(value) }
    }
}

impl<'a, T: 'static + Send> Future for Push<'a, T> {
    type Output = Result<(), PushError<T>>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        let value = this.value.take().expect("Do not poll futures after completion.");
        match this.queue.try_push(value) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(PushError::Closed(value)) => Poll::Ready(Err(PushError::Closed(value))),
            Err(PushError::Full(value)) => {
                this.queue.register(ctx.waker());
                *this.value = Some(value);
                Poll::Pending
            }
        }
    }
}

pin_project! {
    pub struct Pop<'a, T> {
        queue: &'a WakerQueue<T>,
    }
}

impl<'a, T: 'static + Send> Pop<'a, T> {
    fn new(queue: &'a WakerQueue<T>) -> Pop<'a, T> {
        Pop { queue }
    }
}

impl<'a, T: 'static + Send> Future for Pop<'a, T> {
    type Output = Result<T, PopError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Result<T, PopError>> {
        let this = self.project();
        match this.queue.try_pop() {
            Ok(val) => Poll::Ready(Ok(val)),
            Err(PopError::Closed) => Poll::Ready(Err(PopError::Closed)),
            Err(PopError::Empty) => {
                this.queue.register(ctx.waker());
                Poll::Pending
            }
        }
    }
}
