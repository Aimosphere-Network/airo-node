use futures::{
    channel::{mpsc, oneshot},
    SinkExt,
};
use sc_network::KademliaKey;

use crate::ServiceMsg;

/// Service to interact with the [`crate::Worker`].
#[derive(Clone)]
pub struct Service {
    to_worker: mpsc::Sender<ServiceMsg>,
}

impl Service {
    pub(crate) fn new(to_worker: mpsc::Sender<ServiceMsg>) -> Self {
        Self { to_worker }
    }

    /// Put data into the DHT.
    pub async fn put_data(&mut self, key: KademliaKey, data: Vec<u8>) {
        // TODO: handle error
        let _ = self.to_worker.send(ServiceMsg::PutData { key, data }).await;
    }

    /// Get data from the DHT.
    pub async fn get_data(&mut self, key: KademliaKey) -> Option<Vec<u8>> {
        let (sender, receiver) = oneshot::channel();

        // TODO: handle error
        self.to_worker.send(ServiceMsg::GetData { key, sender }).await.ok()?;
        receiver.await.ok().flatten()
    }
}
