use ldk_node::bitcoin::Network;
use ldk_node::bitcoin::secp256k1::PublicKey;
use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::lightning::util::ser::Hostname;
use ldk_node::{Builder, Node, NodeStatus};
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
    // Name for the node, which may be used when displaying
    // the node in a graph.
    node_alias: Option<String>,
    listening_addresses: Option<Vec<SocketAddress>>,
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

        match config.node_alias {
            Some(node_alias) => builder.set_node_alias(node_alias).unwrap(),
            None => builder.set_node_alias("beaconln-node".to_string()).unwrap(),
        };

        match config.listening_addresses {
            Some(listening_addresses) => builder
                .set_listening_addresses(listening_addresses)
                .unwrap(),
            None => {
                let socket_address = SocketAddress::Hostname {
                    hostname: Hostname::try_from(String::from("127.0.0.1")).unwrap(),
                    port: 3000,
                };

                builder
                    .set_listening_addresses(vec![socket_address])
                    .unwrap()
            }
        };

        // TODO: Handle build and start error gracefully
        let node = builder.build().unwrap();
        node.start().unwrap();

        Ok(Self {
            node: Arc::new(node),
        })
    }

    // This returns the complete status of the Node and we can go ahead
    // to extract specific details from the returned object.
    pub fn get_node_status(&self) -> NodeStatus {
        self.node.status()
    }

    // Returns the Public key ID of the Node
    pub fn get_node_id(&self) -> PublicKey {
        self.node.node_id()
    }

    // Returns the Node alias
    // pub fn get_node_alias(&self) -> String {
    //     self.node.node_alias().unwrap()
    // }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn set_test_config() -> BeaconNodeConfig {
        let socket_address = SocketAddress::Hostname {
            hostname: Hostname::try_from(String::from("127.0.0.1")).unwrap(),
            port: 3000,
        };

        BeaconNodeConfig {
            network: Network::Testnet,
            chain_source: None,
            gossip_source: None,
            storage_dir_path: None,
            node_alias: Some("beacon-node".to_string()),
            listening_addresses: Some(vec![socket_address]),
        }
    }

    #[test]
    fn test_start_node() {
        let config = set_test_config();
        let node = BeaconNode::new(config).unwrap();

        let node_status = node.get_node_status();

        assert!(node_status.is_running)
    }
}
