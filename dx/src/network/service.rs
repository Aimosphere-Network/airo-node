use crate::{
    network::{DxEvent, DxNetworkEventStream, DxNetworkProvider, ServiceMsg},
    PeerId, RecordKey,
};
use futures::Stream;
use sc_utils::mpsc::TracingUnboundedSender;
use std::pin::Pin;

pub struct NetworkService {
    pub to_worker: TracingUnboundedSender<ServiceMsg>,
}

impl DxNetworkProvider for NetworkService {
    fn start_providing(&self, key: RecordKey, data: Vec<u8>) {
        // TODO. Handle error
        let _ = self.to_worker.unbounded_send(ServiceMsg::StartProviding { key, data });
    }

    fn find_first_provider(&self, key: RecordKey) {
        // TODO. Handle error
        let _ = self.to_worker.unbounded_send(ServiceMsg::FindFirstProvider { key });
    }

    fn get_data(&self, key: RecordKey, peer: PeerId) {
        // TODO. Handle error
        let _ = self.to_worker.unbounded_send(ServiceMsg::GetData { key, peer });
    }
}

impl DxNetworkEventStream for NetworkService {
    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = DxEvent> + Send>> {
        let (tx, rx) = async_channel::unbounded();
        let _ = self.to_worker.unbounded_send(ServiceMsg::EventStream { sender: tx });
        Box::pin(rx)
    }
}
