use crate::{rpc::ServiceMsg, RecordKey};
use futures::{
    channel::{mpsc, oneshot},
    SinkExt,
};

/// Service to interact with the [`crate::rpc::Worker`].
#[derive(Clone)]
pub struct Service {
    to_worker: mpsc::Sender<ServiceMsg>,
}

impl Service {
    pub(crate) fn new(to_worker: mpsc::Sender<ServiceMsg>) -> Self {
        Self { to_worker }
    }

    /// Put data into the Dx network.
    pub async fn put_data(&mut self, key: RecordKey, data: Vec<u8>) {
        // TODO: handle error
        let _ = self.to_worker.send(ServiceMsg::PutData { key, data }).await;
    }

    /// Get data from the Dx network.
    pub async fn get_data(&mut self, key: RecordKey) -> Option<Vec<u8>> {
        let (sender, receiver) = oneshot::channel();

        // TODO: handle error
        self.to_worker.send(ServiceMsg::GetData { key, sender }).await.ok()?;
        receiver.await.ok().flatten()
    }
}
