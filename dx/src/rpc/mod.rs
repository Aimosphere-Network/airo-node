use core::marker::PhantomData;
use std::sync::Arc;

pub use crate::rpc::{service::Service, worker::Worker};
use crate::{
    network::{DxEvent, DxNetworkProvider},
    RecordKey,
};
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
};
use sp_core::Bytes;

mod service;
mod worker;

#[cfg(test)]
mod tests;

#[rpc(client, server)]
pub trait DataExchangeApi<Key> {
    #[method(name = "dx_upload")]
    async fn upload(&self, key: Key, data: Bytes) -> RpcResult<()>;

    #[method(name = "dx_download")]
    async fn download(&self, key: Key) -> RpcResult<Option<Bytes>>;
}

/// Provides RPC methods to exchange data offchain.
pub struct DataExchange<Key> {
    service: Service,
    _phantom: PhantomData<Key>,
}

impl<Key> DataExchange<Key> {
    pub fn new(service: Service) -> Self {
        Self { service, _phantom: Default::default() }
    }
}

#[async_trait]
impl<Key> DataExchangeApiServer<Key> for DataExchange<Key>
where
    Key: AsRef<[u8]> + Send + Sync + 'static,
{
    async fn upload(&self, key: Key, data: Bytes) -> RpcResult<()> {
        self.service.clone().put_data(RecordKey::new(&key), data.0).await;
        Ok(())
    }

    async fn download(&self, key: Key) -> RpcResult<Option<Bytes>> {
        Ok(self.service.clone().get_data(RecordKey::new(&key)).await.map(Bytes))
    }
}

/// Message send from the [`Service`] to the [`Worker`].
pub(crate) enum ServiceMsg {
    PutData { key: RecordKey, data: Vec<u8> },
    GetData { key: RecordKey, sender: oneshot::Sender<Option<Vec<u8>>> },
}

/// Create a new data exchange [`Worker`] and [`Service`].
pub fn new_worker_and_service<DxEventStream>(
    network_service: Arc<dyn DxNetworkProvider + Send + Sync>,
    dht_event_rx: DxEventStream,
) -> (Worker<DxEventStream>, Service)
where
    DxEventStream: Stream<Item = DxEvent> + Unpin,
{
    let (to_worker, from_service) = mpsc::channel(0);

    let worker = Worker::new(from_service, network_service, dht_event_rx);
    let service = Service::new(to_worker);

    (worker, service)
}
