use std::fmt::Debug;
use std::future::{Future};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::mpsc::{self, Sender, Receiver};

#[derive(Debug)]
pub enum PromiseResult<T> {
    Resolved(T),
    Rejected(T),
}

unsafe impl<T: Send> Send for PromiseResult<T> {}
unsafe impl<T: Sync> Sync for PromiseResult<T> {}

#[derive(Debug)]
pub struct Promise<T: Send + Sync> {
    receiver: Receiver<PromiseResult<T>>,
}

unsafe impl<T: Send + Sync> Send for Promise<T> {}
unsafe impl<T: Send + Sync> Sync for Promise<T> {}
impl<T: Send + Sync> Unpin for Promise<T>{}


impl<T: Send + Sync + Debug > Promise<T> {
    pub fn new() -> (Self, Arc<AsyncMutex<Sender<PromiseResult<T>>>>) {
        let (sender, receiver) = mpsc::channel(2048);
        (Promise { receiver }, Arc::new(AsyncMutex::new(sender)))
    }
        
    pub async fn resolve(sender: Arc<AsyncMutex<Sender<PromiseResult<T>>>>, value: T) {
        match sender.lock().await.send(PromiseResult::Resolved(value)).await {
            Ok(_) => {
                println!("resolve success");
            },
            Err(error) => {
                println!("resolve error: {:?}", error);
            },
        };
    }

    pub async fn reject(sender: Arc<AsyncMutex<Sender<PromiseResult<T>>>>, value: T) {
        sender.lock().await.send(PromiseResult::Rejected(value)).await.unwrap();
    }
}

impl<T: Send + Sync + Clone> Future for Promise<T> {
    type Output = PromiseResult<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.try_recv() {
            Ok(value) => {
                Poll::Ready(value)
            },
            Err(_) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
