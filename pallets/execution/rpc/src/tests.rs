use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::{channel::mpsc, executor::LocalPool, task::LocalSpawn};
use sc_network::{DhtEvent, KademliaKey, NetworkDHTProvider};

use crate::new_worker_and_service;

struct TestNetwork {
    dht: Arc<Mutex<HashMap<KademliaKey, Vec<u8>>>>,
    event_sender: mpsc::UnboundedSender<DhtEvent>,
}

impl TestNetwork {
    fn new(event_sender: mpsc::UnboundedSender<DhtEvent>) -> Self {
        Self { dht: Default::default(), event_sender }
    }
}

impl NetworkDHTProvider for TestNetwork {
    fn get_value(&self, key: &KademliaKey) {
        let event = match self.dht.lock().unwrap().get(key) {
            Some(value) => DhtEvent::ValueFound(vec![(key.clone(), value.clone())]),
            None => DhtEvent::ValueNotFound(key.clone()),
        };

        self.event_sender.clone().unbounded_send(event).unwrap();
    }

    fn put_value(&self, key: KademliaKey, value: Vec<u8>) {
        self.dht.lock().unwrap().insert(key.clone(), value);
        self.event_sender.clone().unbounded_send(DhtEvent::ValuePut(key)).unwrap();
    }
}

#[test]
fn get_data() {
    let (dht_event_tx, dht_event_rx) = mpsc::unbounded();
    let network = Arc::new(TestNetwork::new(dht_event_tx));

    let (worker, mut service) = new_worker_and_service(network, dht_event_rx);

    let mut pool = LocalPool::new();
    pool.spawner().spawn_local_obj(Box::pin(worker.run()).into()).unwrap();

    let key = KademliaKey::new(&"key");
    let value = Vec::from("value");

    pool.run_until(async {
        service.put_data(key.clone(), value.clone()).await;
        assert_eq!(Some(value), service.get_data(key).await,);
    });
}
