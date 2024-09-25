use crate::{
    network::{DxEvent, DxNetworkProvider},
    rpc::ServiceMsg,
    PeerId, RecordKey,
};
use futures::{
    channel::{mpsc, oneshot},
    stream::Fuse,
    FutureExt, Stream, StreamExt,
};
use jsonrpsee::tracing::log;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub struct Worker<DxEventStream> {
    /// Channel receiver for messages send by a [`crate::Service`].
    from_service: Fuse<mpsc::Receiver<ServiceMsg>>,

    network_service: Arc<dyn DxNetworkProvider + Send + Sync>,

    /// Channel we receive Dht events on.
    dx_event_rx: DxEventStream,

    /// Lookups currently in progress.
    pending_lookups: HashMap<RecordKey, Vec<oneshot::Sender<Option<Vec<u8>>>>>,

    /// Peers providing data.
    lookup_peers: HashMap<RecordKey, HashSet<PeerId>>,
}

impl<DxEventStream> Worker<DxEventStream>
where
    DxEventStream: Stream<Item = DxEvent> + Unpin,
{
    /// Construct a [`Worker`].
    pub(crate) fn new(
        from_service: mpsc::Receiver<ServiceMsg>,
        network_service: Arc<dyn DxNetworkProvider + Send + Sync>,
        dx_event_rx: DxEventStream,
    ) -> Self {
        Self {
            from_service: from_service.fuse(),
            network_service,
            dx_event_rx,
            pending_lookups: HashMap::new(),
            lookup_peers: HashMap::new(),
        }
    }

    /// Start the worker
    pub async fn run(mut self) {
        loop {
            futures::select! {
                msg = self.from_service.select_next_some() => {
                    // Handle messages from [`Service`]. Ignore if sender side is closed.
                    self.handle_message(msg);
                }
                event = self.dx_event_rx.next().fuse() => {
                    if let Some(event) = event {
                        self.handle_dx_event(event).await;
                    } else {
                        // This point is reached if the network has shut down, at which point there is not
                        // much else to do than to shut down the authority discovery as well.
                        return;
                    }
                }
            }
        }
    }

    fn handle_message(&mut self, msg: ServiceMsg) {
        match msg {
            ServiceMsg::PutData { key, data } => self.network_service.start_providing(key, data),
            ServiceMsg::GetData { key, sender } => {
                self.network_service.find_first_provider(key.clone());
                self.pending_lookups.entry(key).or_default().push(sender);
            },
        }
    }

    async fn handle_dx_event(&mut self, event: DxEvent) {
        match event {
            DxEvent::FoundProviders { key, providers } => {
                if providers.is_empty() {
                    if let Some(senders) = self.pending_lookups.remove(&key) {
                        for sender in senders {
                            let _ = sender.send(None);
                        }
                    }
                } else {
                    for provider in providers.clone() {
                        self.network_service.get_data(key.clone(), provider);
                    }
                    self.lookup_peers.insert(key, providers);
                }
            },
            DxEvent::FoundProvidersFailed { key } => {
                log::warn!(target: "dx", "👻 Failed to find providers for {:?}", key);
                if let Some(senders) = self.pending_lookups.remove(&key) {
                    for sender in senders {
                        let _ = sender.send(None);
                    }
                }
            },
            DxEvent::DataReceived { key, data, .. } => {
                // TODO: Refactor this to validate all the entries and find a proper one (e.g.
                // signature, hash match). A proper validation mechanism should probably be injected
                // at type construction.

                self.lookup_peers.remove(&key);
                if let Some(senders) = self.pending_lookups.remove(&key) {
                    for sender in senders {
                        let _ = sender.send(Some(data.clone()));
                    }
                }
            },
            DxEvent::DataReceivedFailed { key, peer } => {
                if let Some(peers) = self.lookup_peers.get_mut(&key) {
                    peers.remove(&peer);

                    if !peers.is_empty() {
                        return;
                    }
                }

                if let Some(senders) = self.pending_lookups.remove(&key) {
                    for sender in senders {
                        let _ = sender.send(None);
                    }
                }
            },
            _ => {
                //TODO: Add logging and metrics
            },
        }
    }
}
