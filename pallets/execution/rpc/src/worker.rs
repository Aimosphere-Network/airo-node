use std::{collections::HashMap, sync::Arc};

use futures::{
    channel::{mpsc, oneshot},
    stream::Fuse,
    FutureExt, Stream, StreamExt,
};
use sc_network::{DhtEvent, KademliaKey, NetworkDHTProvider};

use crate::ServiceMsg;

/// NetworkProvider provides [`Worker`] with all necessary hooks into the
/// underlying Substrate networking. Using this trait abstraction instead of
/// `sc_network::NetworkService` directly is necessary to unit test [`Worker`].
pub trait NetworkProvider: NetworkDHTProvider + Send + Sync {}

impl<T> NetworkProvider for T where T: NetworkDHTProvider + Send + Sync {}

pub struct Worker<DhtEventStream> {
    /// Channel receiver for messages send by a [`crate::Service`].
    from_service: Fuse<mpsc::Receiver<ServiceMsg>>,

    network: Arc<dyn NetworkProvider>,

    /// Channel we receive Dht events on.
    dht_event_rx: DhtEventStream,

    /// Lookups currently in progress.
    pending_lookups: HashMap<KademliaKey, Vec<oneshot::Sender<Option<Vec<u8>>>>>,
}

impl<DhtEventStream> Worker<DhtEventStream>
where
    DhtEventStream: Stream<Item = DhtEvent> + Unpin,
{
    /// Construct a [`Worker`].
    pub(crate) fn new(
        from_service: mpsc::Receiver<ServiceMsg>,
        network: Arc<dyn NetworkProvider>,
        dht_event_rx: DhtEventStream,
    ) -> Self {
        Self {
            from_service: from_service.fuse(),
            network,
            dht_event_rx,
            pending_lookups: HashMap::new(),
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
                event = self.dht_event_rx.next().fuse() => {
                    if let Some(event) = event {
                        self.handle_dht_event(event).await;
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
            ServiceMsg::PutData { key, data } => self.network.put_value(key, data),
            ServiceMsg::GetData { key, sender } => {
                self.network.get_value(&key);
                self.pending_lookups.entry(key).or_default().push(sender);
            },
        }
    }

    async fn handle_dht_event(&mut self, event: DhtEvent) {
        match event {
            DhtEvent::ValueFound(found) => {
                // TODO: Refactor this to validate all the entries and find a proper one (e.g.
                // signature, hash match). A proper validation mechanism should probably be injected
                // at type construction.
                let (key, value) = {
                    if let Some(entry) = found.into_iter().next() {
                        entry
                    } else {
                        return;
                    }
                };

                if let Some(senders) = self.pending_lookups.remove(&key) {
                    for sender in senders {
                        let _ = sender.send(Some(value.clone()));
                    }
                }
            },
            DhtEvent::ValueNotFound(key) => {
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
