use crate::{
    network::{DxEvent, DxNetworkProvider},
    rpc::new_worker_and_service,
    PeerId, RecordKey,
};
use futures::{channel::mpsc, executor::LocalPool, task::LocalSpawn};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

struct TestNetwork {
    dx_data: Arc<Mutex<HashMap<RecordKey, Vec<u8>>>>,
    dx_peers: Arc<Mutex<HashMap<RecordKey, PeerId>>>,
    event_sender: mpsc::UnboundedSender<DxEvent>,
}

impl TestNetwork {
    fn new(event_sender: mpsc::UnboundedSender<DxEvent>) -> Self {
        Self { dx_data: Default::default(), dx_peers: Default::default(), event_sender }
    }
}

impl DxNetworkProvider for TestNetwork {
    fn start_providing(&self, key: RecordKey, data: Vec<u8>) {
        self.dx_data.lock().unwrap().insert(key.clone(), data);
        self.dx_peers.lock().unwrap().insert(key.clone(), PeerId::random());
        self.event_sender
            .clone()
            .unbounded_send(DxEvent::StartProviding { key })
            .unwrap();
    }

    fn find_first_provider(&self, key: RecordKey) {
        let event = match self.dx_peers.lock().unwrap().get(&key) {
            None => DxEvent::FoundProvidersFailed { key },
            Some(peer) => {
                let providers = HashSet::from([*peer]);
                DxEvent::FoundProviders { key, providers }
            },
        };

        self.event_sender.clone().unbounded_send(event).unwrap()
    }

    fn get_data(&self, key: RecordKey, peer: PeerId) {
        let event = match self.dx_peers.lock().unwrap().get(&key) {
            Some(p) if peer.eq(p) => {
                let data = self.dx_data.lock().unwrap().get(&key).unwrap().clone();
                DxEvent::DataReceived { key, peer, data }
            },
            _ => DxEvent::DataReceivedFailed { key, peer },
        };

        self.event_sender.clone().unbounded_send(event).unwrap()
    }
}

#[test]
fn get_existing_data() {
    let (dx_event_tx, dx_event_rx) = mpsc::unbounded();
    let network_service = Arc::new(TestNetwork::new(dx_event_tx));

    let (worker, mut service) = new_worker_and_service(network_service, dx_event_rx);

    let mut pool = LocalPool::new();
    pool.spawner().spawn_local_obj(Box::pin(worker.run()).into()).unwrap();

    let key = RecordKey::new(&"key");
    let value = Vec::from("value");

    pool.run_until(async {
        service.put_data(key.clone(), value.clone()).await;
        assert_eq!(Some(value), service.get_data(key).await);
    });
}

#[test]
fn get_missing_data() {
    let (dx_event_tx, dx_event_rx) = mpsc::unbounded();
    let network_service = Arc::new(TestNetwork::new(dx_event_tx));

    let (worker, mut service) = new_worker_and_service(network_service, dx_event_rx);

    let mut pool = LocalPool::new();
    pool.spawner().spawn_local_obj(Box::pin(worker.run()).into()).unwrap();

    pool.run_until(async {
        assert_eq!(None, service.get_data(RecordKey::new(&"key")).await);
    });
}
