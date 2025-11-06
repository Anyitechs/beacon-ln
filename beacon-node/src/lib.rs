use ldk_node::bitcoin::Network;
use ldk_node::{Builder, Node};
use std::sync::Arc;

pub struct BeaconNode {
    node: Arc<Node>,
}

// Config objects for Beacon
pub struct BeaconNodeConfig {
    network: Network,
    chain_source: Option<String>,
    gossip_source: Option<String>,
    storage_dir_path: Option<String>,
}

impl BeaconNode {
    pub fn new(config: BeaconNodeConfig) -> Result<Self, ldk_node::BuildError> {
        let mut builder = Builder::new();
        builder.set_network(config.network);

        match config.chain_source {
            Some(chain_source) => builder.set_chain_source_esplora(chain_source, None),
            None => builder
                .set_chain_source_esplora("https://blockstream.info/testnet/api".to_string(), None),
        };

        match config.gossip_source {
            Some(gossip_source) => builder.set_gossip_source_rgs(gossip_source),
            None => builder.set_gossip_source_rgs(
                "https://rapidsync.lightningdevkit.org/testnet/snapshot".to_string(),
            ),
        };

        match config.storage_dir_path {
            Some(storage_dir_path) => builder.set_storage_dir_path(storage_dir_path),
            None => builder.set_storage_dir_path("beacon.sqlite".to_string()),
        };

        // TODO: Handle build and start error gracefully
        let node = builder.build().unwrap();
        node.start().unwrap();

        Ok(Self {
            node: Arc::new(node),
        })
    }
}
