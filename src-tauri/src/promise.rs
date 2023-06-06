use std::fmt::Debug;
use std::future::{Future};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc::{self, Sender, Receiver};

#[derive(Debug)]
pub enum PromiseResult<T> {
    Resolved(T),
    Rejected,
}

unsafe impl<T: Send> Send for PromiseResult<T> {}
unsafe impl<T: Sync> Sync for PromiseResult<T> {}

#[derive(Debug)]
pub struct Promise<T: Send + Sync> {
    receiver: Receiver<PromiseResult<T>>,
    state: Option<T>,
}

unsafe impl<T: Send + Sync> Send for Promise<T> {}
unsafe impl<T: Send + Sync> Sync for Promise<T> {}
impl<T: Send + Sync> Unpin for Promise<T>{}


impl<T: Send + Sync + Debug > Promise<T> {
    pub fn new() -> (Self, Sender<PromiseResult<T>>) {
        let (sender, receiver) = mpsc::channel(1024);
        (Promise { receiver, state: None }, sender)
    }
        
    pub async fn resolve(sender: Sender<PromiseResult<T>>, value: T) {
        sender.send(PromiseResult::Resolved(value)).await.unwrap();
    }

    pub async fn reject(sender: Sender<PromiseResult<T>>) {
        sender.send(PromiseResult::Rejected).await.unwrap();
    }
}

impl<T: Send + Sync + Clone> Future for Promise<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.try_recv() {
            Ok(PromiseResult::Resolved(value)) => {
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
