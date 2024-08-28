use libp2p::{
    kad, request_response, request_response::ProtocolSupport, swarm::NetworkBehaviour, Multiaddr,
    PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRequest {
    pub key: Vec<u8>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DataResponse {
    pub data: Vec<u8>,
}

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub request_response: request_response::cbor::Behaviour<DataRequest, DataResponse>,
}

impl Behaviour {
    pub fn new<Hash, I>(genesis_hash: Hash, peer_id: PeerId, known_addresses: I) -> Self
    where
        Hash: AsRef<[u8]>,
        I: Iterator<Item = (PeerId, Multiaddr)>,
    {
        let store = kad::store::MemoryStore::new(peer_id);
        let config = kad::Config::new(protocol("kad", genesis_hash.as_ref()));
        let mut kademlia = kad::Behaviour::with_config(peer_id, store, config);
        kademlia.set_mode(Some(kad::Mode::Server));

        for (peer_id, addr) in known_addresses {
            kademlia.add_address(&peer_id, addr);
        }

        Self {
            kademlia,
            request_response: request_response::cbor::Behaviour::new(
                [(protocol("p2p", genesis_hash), ProtocolSupport::Full)],
                request_response::Config::default(),
            ),
        }
    }
}

fn protocol<Hash: AsRef<[u8]>>(name: &str, genesis_hash: Hash) -> StreamProtocol {
    let genesis_hash_hex = array_bytes::bytes2hex("", genesis_hash.as_ref());
    StreamProtocol::try_from_owned(format!("/dx/{genesis_hash_hex}/{name}"))
        .expect("protocol name is valid. qed")
}
