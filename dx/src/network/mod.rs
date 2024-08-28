use crate::{PeerId, RecordKey};
use async_channel::Sender;
use futures::Stream;
use std::{pin::Pin, sync::Arc};

mod behaviour;
mod config;
mod service;
mod worker;

pub use crate::network::{
    config::DxConfig,
    service::NetworkService,
    worker::{Event as DxEvent, NetworkWorker as DxNetworkWorker},
};

pub enum ServiceMsg {
    EventStream { sender: Sender<DxEvent> },
    StartProviding { key: RecordKey, data: Vec<u8> },
    FindFirstProvider { key: RecordKey },
    GetData { key: RecordKey, peer: PeerId },
}

pub trait DxNetworkProvider {
    fn start_providing(&self, key: RecordKey, data: Vec<u8>);
    fn find_first_provider(&self, key: RecordKey);
    fn get_data(&self, key: RecordKey, peer: PeerId);
}

impl<T> DxNetworkProvider for Arc<T>
where
    T: ?Sized,
    T: DxNetworkProvider,
{
    fn start_providing(&self, key: RecordKey, data: Vec<u8>) {
        T::start_providing(self, key, data)
    }

    fn find_first_provider(&self, key: RecordKey) {
        T::find_first_provider(self, key)
    }

    fn get_data(&self, key: RecordKey, peer: PeerId) {
        T::get_data(self, key, peer)
    }
}

pub trait DxNetworkEventStream {
    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = DxEvent> + Send>>;
}

impl<T> DxNetworkEventStream for Arc<T>
where
    T: ?Sized,
    T: DxNetworkEventStream,
{
    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = DxEvent> + Send>> {
        T::event_stream(self)
    }
}

pub trait DxNetworkService:
    DxNetworkProvider + DxNetworkEventStream + Send + Sync + 'static
{
}

impl<T> DxNetworkService for T where
    T: DxNetworkProvider + DxNetworkEventStream + Send + Sync + 'static
{
}
