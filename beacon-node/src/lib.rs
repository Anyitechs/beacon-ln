use ldk_node::bitcoin::Network;
use ldk_node::bitcoin::secp256k1::PublicKey;
use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::lightning::util::ser::Hostname;
use ldk_node::{Builder, ChannelDetails, Node, NodeError, NodeStatus, UserChannelId};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

pub mod types;

/// Handles creating a single node per time. Intended to be
/// mostly internal and shouldn't talk to the UI directly.
pub struct BeaconNode {
    node: Arc<Node>,
}

/// Config objects for Beacon
#[derive(Debug, Clone)]
pub struct BeaconNodeConfig {
    pub network: Network,
    pub chain_source: Option<String>,
    pub gossip_source: Option<String>,
    pub node_alias: Option<String>,
}

impl BeaconNode {
    /// Creates a *NEW* BeaconNode (read: Lightning Node).
    pub fn new(
        config: BeaconNodeConfig,
        storage_dir_path: String,
        listening_addresses: SocketAddress,
    ) -> Result<Self, BeaconNodeError> {
        let mut builder = Builder::new();
        builder.set_network(config.network);
        builder.set_storage_dir_path(storage_dir_path);
        builder.set_listening_addresses(vec![listening_addresses])?;

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
            Some(node_alias) => builder.set_node_alias(node_alias)?,
            None => builder.set_node_alias("beaconln-node".to_string())?,
        };

        let node = builder.build()?;
        node.start()
            .map_err(|e| BeaconNodeError::StartNodeError(e.to_string()))?;

        log::info!("Beacon Node started successfully");

        Ok(Self {
            node: Arc::new(node),
        })
    }

    /// This returns the complete status of the Node and we can go ahead
    /// to extract specific details from the returned object.
    pub fn get_node_status(&self) -> NodeStatus {
        self.node.status()
    }

    /// Returns the Public key ID of the Node
    pub fn get_node_id(&self) -> PublicKey {
        self.node.node_id()
    }

    /// Gets the funding address for the node.
    pub fn get_node_funding_address(&self) -> String {
        self.node
            .onchain_payment()
            .new_address()
            .expect("Failed to get new onchain address")
            .to_string()
    }

    /// Connects to a node and open a new unannounced channel.
    pub fn open_unannounced_channel(
        &self,
        node_id: PublicKey,
        address: SocketAddress,
        channel_amount_sats: u64,
        push_to_counterparty_msat: Option<u64>,
    ) -> Result<UserChannelId, NodeError> {
        self.node.open_channel(
            node_id,
            address,
            channel_amount_sats,
            push_to_counterparty_msat,
            None,
        )
    }

    /// Connects to a node and opens a new announced channel.
    pub fn open_announced_channel(
        &self,
        node_id: PublicKey,
        address: SocketAddress,
        channel_amount_sats: u64,
        push_to_counterparty_msat: Option<u64>,
    ) -> Result<UserChannelId, NodeError> {
        self.node.open_announced_channel(
            node_id,
            address,
            channel_amount_sats,
            push_to_counterparty_msat,
            None,
        )
    }

    /// List channels for a particular node.
    pub fn list_node_channels(&self) -> Vec<ChannelDetails> {
        self.node.list_channels()
    }
}

#[derive(Debug)]
pub enum BeaconNodeError {
    BuildNodeError(ldk_node::BuildError),
    StartNodeError(String),
    DuplicateNodeNameError(String),
}

impl std::fmt::Display for BeaconNodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BeaconNodeError::BuildNodeError(e) => write!(f, "Failed to build node: {}", e),
            BeaconNodeError::StartNodeError(e) => write!(f, "Failed to start node: {}", e),
            BeaconNodeError::DuplicateNodeNameError(e) => {
                write!(f, "Duplicate node name error: {}", e)
            }
        }
    }
}

impl From<ldk_node::BuildError> for BeaconNodeError {
    fn from(value: ldk_node::BuildError) -> Self {
        Self::BuildNodeError(value)
    }
}

/// This manages the lifecycle and resources of all nodes. It's the
/// heart of the node clustering feature as it's responsible for creating,
/// tracking, listing nodes on the UI, etc.
pub struct BeaconNodeManager {
    base_storage_dir: PathBuf,
    running_nodes: HashMap<String, BeaconNode>,
}

impl BeaconNodeManager {
    pub fn new(base_storage_dir: PathBuf) -> Self {
        let mut manager = Self {
            base_storage_dir,
            running_nodes: HashMap::new(),
        };
        manager.discover_and_restart_nodes();
        manager
    }

    fn discover_and_restart_nodes(&mut self) {
        if let Ok(entries) = std::fs::read_dir(&self.base_storage_dir) {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                    let node_name = entry.file_name().to_string_lossy().to_string();
                    log::info!("Discovered existing node: {}", node_name);

                    let config = BeaconNodeConfig {
                        // This is ignored and loaded from the persisted config
                        network: Network::Testnet,
                        chain_source: None,
                        gossip_source: None,
                        node_alias: Some(node_name.clone()),
                    };

                    // Attempts to restart the node. If it fails, we log the
                    // error and continue.
                    match self.create_node(node_name.clone(), config) {
                        Ok(_) => log::info!("Successfully restarted node '{}'", node_name),
                        Err(e) => {
                            // We ignore DuplicateNodeNameError as it shouldn't happen here.
                            if !matches!(e, BeaconNodeError::DuplicateNodeNameError(_)) {
                                log::error!("Failed to restart node '{}': {}", node_name, e);
                            }
                        }
                    }
                }
            }
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
        std::fs::create_dir_all(&node_storage_path).map_err(|e| {
            BeaconNodeError::StartNodeError(format!("Failed to create storage dir: {}", e))
        })?;

        let storage_path_str = node_storage_path
            .to_str()
            .ok_or_else(|| {
                BeaconNodeError::StartNodeError("Storage path is not valid UTF-8".to_string())
            })?
            .to_string();

        let port = 9735 + self.running_nodes.len() as u16;
        let listening_address = SocketAddress::Hostname {
            hostname: Hostname::try_from(String::from("127.0.0.1")).map_err(|e| {
                BeaconNodeError::StartNodeError(format!("Invalid hostname: {:?}", e))
            })?,
            port,
        };

        log::info!("Attempting to start node on port {}", port);

        let node = BeaconNode::new(config, storage_path_str, listening_address)?;

        let entry = self.running_nodes.entry(node_name);
        let beacon_node = entry.or_insert(node);

        Ok(beacon_node)
    }

    pub fn list_nodes(&self) -> Vec<NodeDetails> {
        let mut running_nodes: Vec<NodeDetails> = Vec::new();

        for node in self.running_nodes.iter() {
            let name = node.0.clone();
            let status = node.1.get_node_status();
            let is_online = status.is_running;
            let channels_active = node.1.list_node_channels().len();
            // TODO: Calculate the actual uptime of the node
            let uptime_hours = 0;

            let nodes = NodeDetails {
                name,
                is_online,
                channels_active: channels_active as u32,
                uptime_hours,
            };

            running_nodes.push(nodes);
        }
        running_nodes
    }
}

/// Commands the UI will have to send to the NodeManager actor
#[derive(Debug)]
pub enum BeaconNodeManagerCommand {
    CreateNode {
        node_name: String,
        config: BeaconNodeConfig,
    },
    ListNodes,
}

#[derive(Debug, Clone)]
pub struct NodeDetails {
    pub name: String,
    pub is_online: bool,
    pub channels_active: u32,
    pub uptime_hours: u32,
}

/// Events the manager will send back to the UI
#[derive(Debug, Clone)]
pub enum BeaconNodeManagerEvent {
    NodeCreated(String),
    NodeListUpdated(Vec<NodeDetails>),
    Error(String),
}

/// The NodeManager Handle for the UI
pub struct BeaconNodeManagerHandle {
    pub command_sender: mpsc::Sender<BeaconNodeManagerCommand>,
    pub event_receiver: broadcast::Receiver<BeaconNodeManagerEvent>,
}

impl Clone for BeaconNodeManagerHandle {
    fn clone(&self) -> Self {
        Self {
            command_sender: self.command_sender.clone(),
            event_receiver: self.event_receiver.resubscribe(),
        }
    }
}

/// This is the "main" function for the BeaconNode library.
/// The UI will call this *once* on startup.
pub fn start_manager_actor(base_storage_dir: PathBuf) -> BeaconNodeManagerHandle {
    let (command_sender, mut command_receiver) = mpsc::channel(128);
    let (event_sender, event_receiver) = broadcast::channel(128);

    let mut beacon_node_manager = BeaconNodeManager::new(base_storage_dir);

    // This task runs in the background and processes commands one by one,
    // so the UI doesn't get blocked
    tokio::spawn(async move {
        log::info!("NodeManager actor started");

        while let Some(command) = command_receiver.recv().await {
            log::info!("NodeManager received command: {:?}", command);
            match command {
                BeaconNodeManagerCommand::CreateNode { node_name, config } => {
                    log::info!("Manager received command to create node: {}", node_name);
                    match beacon_node_manager.create_node(node_name.clone(), config) {
                        Ok(_) => {
                            event_sender
                                .send(BeaconNodeManagerEvent::NodeCreated(node_name))
                                .ok();
                            let nodes = beacon_node_manager.list_nodes();
                            event_sender
                                .send(BeaconNodeManagerEvent::NodeListUpdated(nodes))
                                .ok();
                        }
                        Err(e) => {
                            event_sender
                                .send(BeaconNodeManagerEvent::Error(e.to_string()))
                                .ok();
                        }
                    }
                }
                BeaconNodeManagerCommand::ListNodes => {
                    let nodes = beacon_node_manager.list_nodes();
                    event_sender
                        .send(BeaconNodeManagerEvent::NodeListUpdated(nodes))
                        .ok();
                }
            }
        }
    });

    BeaconNodeManagerHandle {
        command_sender,
        event_receiver,
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
