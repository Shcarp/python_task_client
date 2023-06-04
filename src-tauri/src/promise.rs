use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::sync::mpsc::{self, Sender, Receiver};

#[derive(Debug)]
pub enum PromiseResult<T> {
    Resolved(T),
    Rejected,
}

#[derive(Debug)]
pub struct Promise<T: Send + Sync> {
    sender: Sender<PromiseResult<T>>,
    receiver: Receiver<PromiseResult<T>>,
    state: Option<T>,
}

unsafe impl<T: Send + Sync> Send for Promise<T> {}
unsafe impl<T: Send + Sync> Sync for Promise<T> {}
impl<T: Send + Sync> Unpin for Promise<T>{}


impl<T: Send + Sync > Promise<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Promise { sender, receiver, state: None }
    }
        
    pub fn resolve(&self, value: T) {
        self.sender.send(PromiseResult::Resolved(value)).unwrap();
    }

    pub fn reject(&self) {
        self.sender.send(PromiseResult::Rejected).unwrap();
    }

    pub fn value(&mut self) -> Option<T> {
        self.state.take()
    }
}

impl<T: Send + Sync + Clone> Future for Promise<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.try_recv() {
            Ok(PromiseResult::Resolved(value)) => {
                println!("Promise::poll: Ok");
                self.state = Some(value.clone());
                Poll::Ready(value)
            },
            Ok(PromiseResult::Rejected) => {
                Poll::Ready(self.state.take().unwrap())
            },
            Err(_) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

