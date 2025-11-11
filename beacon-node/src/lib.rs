use ldk_node::bitcoin::Network;
use ldk_node::bitcoin::secp256k1::PublicKey;
use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::lightning::util::ser::Hostname;
use ldk_node::{Builder, Node, NodeStatus};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

// Handles creating a single node per time. Intended to be
// mostly internal and shouldn't talk to the UI directly.
pub struct BeaconNode {
    node: Arc<Node>,
}

// Config objects for Beacon
#[derive(Debug, Clone)]
pub struct BeaconNodeConfig {
    network: Network,
    chain_source: Option<String>,
    gossip_source: Option<String>,
    node_alias: Option<String>,
}

impl BeaconNode {
    // Creates a *NEW* BeaconNode (read: LDK Node).
    pub fn new(
        config: BeaconNodeConfig,
        storage_dir_path: String,
        listening_addresses: SocketAddress,
    ) -> Result<Self, ldk_node::BuildError> {
        let mut builder = Builder::new();
        builder.set_network(config.network);
        builder.set_storage_dir_path(storage_dir_path);
        builder
            .set_listening_addresses(vec![listening_addresses])
            .unwrap();

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

        match config.node_alias {
            Some(node_alias) => builder.set_node_alias(node_alias).unwrap(),
            None => builder.set_node_alias("beaconln-node".to_string()).unwrap(),
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
}

#[derive(Debug)]
pub enum BeaconNodeError {
    BuildNodeError(ldk_node::BuildError),
    StartNodeError(String),
    DuplicateNodeNameError(String),
}

impl From<ldk_node::BuildError> for BeaconNodeError {
    fn from(value: ldk_node::BuildError) -> Self {
        Self::BuildNodeError(value)
    }
}

// This manages the lifecycle and resources of all nodes. It's the
// heart of the node clustering feature as it's responsible for creating,
// tracking, listing nodes on the UI, etc.
pub struct BeaconNodeManager {
    base_storage_dir: PathBuf,
    running_nodes: HashMap<String, BeaconNode>,
}

impl BeaconNodeManager {
    pub fn new(base_storage_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&base_storage_dir).unwrap();
        Self {
            base_storage_dir,
            running_nodes: HashMap::new(),
        }
    }

    pub fn create_node(
        &mut self,
        node_name: String,
        config: BeaconNodeConfig,
    ) -> Result<&BeaconNode, BeaconNodeError> {
        if self.running_nodes.contains_key(&node_name) {
            return Err(BeaconNodeError::DuplicateNodeNameError(
                "Node already exists".to_string(),
            ));
        }

        let mut node_storage_path = self.base_storage_dir.clone();
        node_storage_path.push(&node_name);
        std::fs::create_dir_all(&node_storage_path).unwrap();
        let storage_path_str = node_storage_path.to_str().unwrap().to_string();

        let port = 9735 + self.running_nodes.len() as u16;
        let listening_address = SocketAddress::Hostname {
            hostname: Hostname::try_from(String::from("127.0.0.1")).unwrap(),
            port,
        };

        let node = BeaconNode::new(config, storage_path_str, listening_address)?;

        let entry = self.running_nodes.entry(node_name);
        let beacon_node = entry.or_insert(node);

        Ok(beacon_node)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tempfile::Builder;

    fn set_test_config() -> BeaconNodeConfig {
        BeaconNodeConfig {
            network: Network::Testnet,
            chain_source: None,
            gossip_source: None,
            node_alias: Some("beacon-node".to_string()),
        }
    }

    #[test]
    fn test_start_node() {
        let temp_dir = Builder::new().prefix("beacon-node-1").tempdir().unwrap();
        let storage_dir = temp_dir.path().to_str().unwrap().to_string();

        let listening_address = SocketAddress::Hostname {
            hostname: Hostname::try_from(String::from("127.0.0.1")).unwrap(),
            port: 3000,
        };

        let config = set_test_config();
        let node = BeaconNode::new(config, storage_dir, listening_address).unwrap();

        let node_status = node.get_node_status();

        assert!(node_status.is_running)
    }

    #[test]
    fn test_create_two_distinct_nodes() {
        let temp_dir = Builder::new().prefix("beacon-node").tempdir().unwrap();
        let storage_dir = temp_dir.path().to_str().unwrap().to_string();
        let config = set_test_config();
        let mut beacon_node_manager = BeaconNodeManager::new(storage_dir.into());

        beacon_node_manager
            .create_node("node1".to_string(), config.clone())
            .unwrap();
        beacon_node_manager
            .create_node("node2".to_string(), config)
            .unwrap();
        assert_eq!(beacon_node_manager.running_nodes.len(), 2);

        let node1 = beacon_node_manager.running_nodes.get("node1").unwrap();
        assert!(node1.get_node_status().is_running);

        let node2 = beacon_node_manager.running_nodes.get("node2").unwrap();
        assert!(node2.get_node_status().is_running);
    }
}
