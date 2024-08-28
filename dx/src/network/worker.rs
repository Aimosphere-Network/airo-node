use crate::network::{
    behaviour::{Behaviour, BehaviourEvent, DataRequest, DataResponse},
    config::DxConfig,
    service::NetworkService,
    ServiceMsg,
};
use async_channel::Sender;
use futures::StreamExt;
use hex::ToHex;
use libp2p::{
    identity::{ed25519, Keypair},
    kad,
    kad::{AddProviderError, AddProviderOk, GetProvidersError, GetProvidersOk},
    multiaddr::Protocol,
    noise, request_response,
    request_response::OutboundRequestId,
    swarm::SwarmEvent,
    tcp, yamux, Swarm,
};
pub use libp2p::{kad::RecordKey, PeerId};
use sc_telemetry::log;
use sc_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver};
use sp_runtime::traits::Block as BlockT;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

const DX_DIR: &str = "dx/data/";

#[derive(Clone)]
pub enum Event {
    StartProviding { key: RecordKey },
    StartProvidingFailed { key: RecordKey },
    FoundProviders { key: RecordKey, providers: HashSet<PeerId> },
    FoundProvidersFailed { key: RecordKey },
    DataReceived { key: RecordKey, peer: PeerId, data: Vec<u8> },
    DataReceivedFailed { key: RecordKey, peer: PeerId },
}

pub struct NetworkWorker {
    local_peer_id: PeerId,
    service: Arc<NetworkService>,
    from_service: TracingUnboundedReceiver<ServiceMsg>,
    network: Swarm<Behaviour>,
    event_sender: Vec<Sender<Event>>,
    pending_request_file: HashMap<OutboundRequestId, (RecordKey, PeerId)>,
    path: PathBuf, // TODO. Abstract storage.
}

impl NetworkWorker {
    pub fn new<Block: BlockT>(config: DxConfig<Block>) -> Self {
        let local_identity = config.node_key.into_keypair().unwrap(); // TODO. Remove it
        let local_identity: ed25519::Keypair = local_identity.into();
        let local_keypair: Keypair = local_identity.into();
        let local_peer_id = local_keypair.public().to_peer_id();

        let mut network = libp2p::SwarmBuilder::with_existing_identity(local_keypair)
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)
            .unwrap() // TODO. Remove it
            .with_behaviour(|_key| {
                Behaviour::new(
                    config.genesis_hash,
                    local_peer_id,
                    config
                        .known_addresses
                        .into_iter()
                        .map(|peer| (peer.peer_id.into(), peer.multiaddr.into())),
                )
            })
            .unwrap() // TODO. Remove it
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Listen on addresses.
        for addr in &config.listen_addresses {
            if let Err(err) = network.listen_on(addr.clone().into()) {
                log::warn!(target: "dx-libp2p", "👻 Can't listen on {} because: {:?}", addr, err)
            }
        }

        // Add external addresses.
        for addr in &config.public_addresses {
            network.add_external_address(addr.clone().into());
        }

        let (to_worker, from_service) = tracing_unbounded("mpsc_dx_msg", 10_000);
        let service = Arc::new(NetworkService { to_worker });

        let mut path = config.path;
        path.push(DX_DIR);
        fs::create_dir_all(path.clone()).expect("The Dx directory should be created; qed");
        log::info!(target: "dx-libp2p", "👻 Storage path: {:?}", path);

        Self {
            local_peer_id,
            service,
            from_service,
            network,
            event_sender: Vec::new(),
            pending_request_file: HashMap::new(),
            path,
        }
    }

    pub fn network_service(&self) -> Arc<NetworkService> {
        self.service.clone()
    }

    pub async fn run(mut self) {
        while self.next_action().await {}
    }

    pub async fn next_action(&mut self) -> bool {
        futures::select! {
            // Next message from the service.
            msg = self.from_service.next() => {
                if let Some(msg) = msg {
                    self.handle_worker_message(msg).await;
                } else {
                    return false
                }
            },
            // Next event from `Swarm` (the stream guaranteed to never terminate).
            event = self.network.select_next_some() => {
                self.handle_swarm_event(event).await;
            },
        }

        true
    }

    async fn handle_worker_message(&mut self, msg: ServiceMsg) {
        match msg {
            ServiceMsg::EventStream { sender } => {
                self.event_sender.push(sender);
            },
            ServiceMsg::StartProviding { key, data } => {
                // TODO. Handle errors
                let _ = self.save(&key, &data);
                let _ = self.network.behaviour_mut().kademlia.start_providing(key);
            },
            ServiceMsg::FindFirstProvider { key } => {
                self.network.behaviour_mut().kademlia.get_providers(key);
            },
            ServiceMsg::GetData { key, peer } => {
                if peer == self.local_peer_id {
                    // TODO. Handle errors
                    let data = self.load(&key).unwrap();
                    for sender in &self.event_sender {
                        let _ = sender
                            .send(Event::DataReceived {
                                key: key.clone(),
                                peer,
                                data: data.clone(),
                            })
                            .await;
                    }
                } else {
                    let request_id = self
                        .network
                        .behaviour_mut()
                        .request_response
                        .send_request(&peer, DataRequest { key: key.to_vec() });
                    self.pending_request_file.insert(request_id, (key, peer));
                }
            },
        }
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    result: kad::QueryResult::StartProviding(result),
                    ..
                },
            )) => {
                for sender in &self.event_sender {
                    // TODO. Handle errors
                    match result.clone() {
                        Ok(AddProviderOk { key }) => {
                            let _ = sender.send(Event::StartProviding { key }).await;
                        },
                        Err(AddProviderError::Timeout { key }) => {
                            let _ = sender.send(Event::StartProvidingFailed { key }).await;
                        },
                    }
                }
            },
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    id,
                    result: kad::QueryResult::GetProviders(result),
                    ..
                },
            )) => {
                for sender in &self.event_sender {
                    // TODO. Handle errors
                    match result.clone() {
                        Ok(GetProvidersOk::FoundProviders { key, providers }) => {
                            let _ = sender.send(Event::FoundProviders { key, providers }).await;
                            // Finish the query. We are only interested in the first result.
                            self.network
                                .behaviour_mut()
                                .kademlia
                                .query_mut(&id)
                                .map(|mut q| q.finish());
                        },
                        Err(GetProvidersError::Timeout { key, .. }) => {
                            let _ = sender.send(Event::FoundProvidersFailed { key }).await;
                        },
                        _ => {},
                    }
                }
            },
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(_)) => {},
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request { request, channel, .. } => {
                    // TODO. Handle errors
                    let data = self.load(&RecordKey::new(&request.key)).unwrap();
                    let _ = self
                        .network
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, DataResponse { data });
                },
                request_response::Message::Response { request_id, response } => {
                    // TODO. Handle error
                    let (key, peer) = self.pending_request_file.remove(&request_id).unwrap();

                    // TODO. Start providing the received data

                    for sender in &self.event_sender {
                        //TODO. Handle error
                        let _ = sender
                            .send(Event::DataReceived {
                                key: key.clone(),
                                peer,
                                data: response.data.clone(),
                            })
                            .await;
                    }
                },
            },
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure { request_id, error, .. },
            )) => {
                // TODO. Handle error
                let (key, peer) = self.pending_request_file.remove(&request_id).unwrap();

                log::error!(target: "dx-libp2p", "👻 p2p request to {} failed because: {:?}", peer, error);

                for sender in &self.event_sender {
                    // TODO. Handle error
                    let _ = sender.send(Event::DataReceivedFailed { key: key.clone(), peer }).await;
                }
            },
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::ResponseSent { .. },
            )) => {},
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.network.local_peer_id();
                log::info!(target: "dx-libp2p", "👻 Listening on {:?}", address.with(Protocol::P2p(local_peer_id)));
            },
            SwarmEvent::IncomingConnection { .. } => {},
            SwarmEvent::ConnectionEstablished { .. } => {},
            SwarmEvent::ConnectionClosed { .. } => {},
            SwarmEvent::OutgoingConnectionError { .. } => {},
            SwarmEvent::IncomingConnectionError { .. } => {},
            SwarmEvent::Dialing { .. } => {},
            e => {
                log::error!(target: "dx-libp2p", "👻 Unhandled event: {:?}", e);
            },
        }
    }

    fn save(&self, key: &RecordKey, data: &[u8]) -> std::io::Result<()> {
        let file_name = key.encode_hex::<String>();
        let path = self.path.join(file_name);
        fs::write(path, data)
    }

    fn load(&self, key: &RecordKey) -> std::io::Result<Vec<u8>> {
        let file_name = key.encode_hex::<String>();
        let path = self.path.join(file_name);
        fs::read(path)
    }
}
