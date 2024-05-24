use core::marker::PhantomData;
use std::sync::Arc;

use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
};
use sc_network::{DhtEvent, KademliaKey};
use sp_core::Bytes;

pub use crate::{
    service::Service,
    worker::{NetworkProvider, Worker},
};

mod service;
mod worker;

#[cfg(test)]
mod tests;

#[rpc(client, server)]
pub trait DataExchangeApi<Key> {
    #[method(name = "exchange_upload")]
    async fn upload(&self, key: Key, data: Bytes) -> RpcResult<()>;

    #[method(name = "exchange_download")]
    async fn download(&self, key: Key) -> RpcResult<Option<Vec<u8>>>;
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
        self.service.clone().put_data(KademliaKey::new(&key), data.0).await;
        Ok(())
    }

    async fn download(&self, key: Key) -> RpcResult<Option<Vec<u8>>> {
        Ok(self.service.clone().get_data(KademliaKey::new(&key)).await)
    }
}

/// Message send from the [`Service`] to the [`Worker`].
pub(crate) enum ServiceMsg {
    PutData { key: KademliaKey, data: Vec<u8> },
    GetData { key: KademliaKey, sender: oneshot::Sender<Option<Vec<u8>>> },
}

/// Create a new data exchange [`Worker`] and [`Service`].
pub fn new_worker_and_service<DhtEventStream>(
    network: Arc<dyn NetworkProvider>,
    dht_event_rx: DhtEventStream,
) -> (Worker<DhtEventStream>, Service)
where
    DhtEventStream: Stream<Item = DhtEvent> + Unpin,
{
    let (to_worker, from_service) = mpsc::channel(0);

    let worker = Worker::new(from_service, network, dht_event_rx);
    let service = Service::new(to_worker);

    (worker, service)
}
