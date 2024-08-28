use sc_network::{
    config::{MultiaddrWithPeerId, NodeKeyConfig},
    Multiaddr,
};
use sp_runtime::traits::Block as BlockT;
use std::path::PathBuf;

/// Data exchange configuration
pub struct DxConfig<Block: BlockT> {
    /// Genesis hash of the chain
    pub genesis_hash: Block::Hash,

    /// Multiaddresses to listen for incoming connections.
    pub listen_addresses: Vec<Multiaddr>,

    /// Multiaddresses to advertise.
    pub public_addresses: Vec<Multiaddr>,

    /// The node key configuration, which determines the node's network identity keypair.
    pub node_key: NodeKeyConfig,

    /// List of initial node addresses
    pub known_addresses: Vec<MultiaddrWithPeerId>,

    /// The path to the files directory
    pub path: PathBuf,
}
