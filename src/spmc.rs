use crate::waker_queue::*;
use crate::private::AsyncQueue;
use std::sync::Arc;

pub fn bounded<T: 'static + Send>(size: usize) -> (Sender<T>, Receiver<T>) {
    let queue = Arc::new(WakerQueue::bounded(size));
    (Sender { queue: queue.clone() }, Receiver { queue })
}

pub fn unbounded<T: 'static + Send>() -> (Sender<T>, Receiver<T>) {
    let queue = Arc::new(WakerQueue::unbounded());
    (Sender { queue: queue.clone() }, Receiver { queue })
}

pub struct Sender<T: 'static + Send> {
    queue: Arc<WakerQueue<T>>,
}

impl<T: 'static + Send> AsyncQueue<T> for Sender<T> {
    fn queue(&self) -> &WakerQueue<T>  { &*self.queue }
}

impl<T: 'static + Send> crate::Sender<T> for Sender<T> {}

impl<T: 'static + Send> crate::AsyncSender<T> for Sender<T> {}


#[derive(Clone)]
pub struct Receiver<T: 'static + Send> {
    queue: Arc<WakerQueue<T>>,
}

impl<T: 'static + Send> AsyncQueue<T> for Receiver<T> {
    fn queue(&self) -> &WakerQueue<T>  { &*self.queue }
}

impl<T: 'static + Send> crate::Receiver<T> for Receiver<T> {}
