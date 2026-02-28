use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

pub struct ChannelQueue<T> {
    sender: mpsc::UnboundedSender<T>,
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<T>>>,
}

impl<T: Send + 'static> ChannelQueue<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn send(&self, item: T) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
        self.sender.send(item)
    }

    pub async fn recv(&self) -> Option<T> {
        let mut receiver = self.receiver.lock().await;
        receiver.recv().await
    }
}

impl<T: Send + 'static> Default for ChannelQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}
