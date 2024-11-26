use crate::db;
use crate::migrations;
use crate::warehouse;
use futures::stream::StreamExt;
use libp2p::development_transport;
use libp2p::floodsub::Floodsub;
use libp2p::floodsub::FloodsubEvent;
use libp2p::floodsub::Topic;
use libp2p::identity::PublicKey;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::KademliaEvent;
use libp2p::kad::{Kademlia, KademliaConfig};
use libp2p::mdns::MdnsEvent;
use libp2p::mdns::{Mdns, MdnsConfig};
use libp2p::ping::{Ping, PingConfig, PingEvent};
use libp2p::swarm::SwarmBuilder;
use libp2p::swarm::SwarmEvent;
use libp2p::NetworkBehaviour;
use libp2p::{identity, Multiaddr, PeerId};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub message_type: String,
    pub content: String,
    pub timestamp: u64,
    pub sender: String,
}

pub enum KMBehaviourEvent {
    Mdns(MdnsEvent),
    Kademlia(KademliaEvent),
    Ping(PingEvent),
    Floodsub(FloodsubEvent),
}

// 为 KMBehaviourEvent ��现 From 特征
impl From<FloodsubEvent> for KMBehaviourEvent {
    fn from(event: FloodsubEvent) -> Self {
        KMBehaviourEvent::Floodsub(event)
    }
}

impl From<PingEvent> for KMBehaviourEvent {
    fn from(event: PingEvent) -> Self {
        KMBehaviourEvent::Ping(event)
    }
}

impl From<MdnsEvent> for KMBehaviourEvent {
    fn from(event: MdnsEvent) -> Self {
        KMBehaviourEvent::Mdns(event)
    }
}

impl From<KademliaEvent> for KMBehaviourEvent {
    fn from(event: KademliaEvent) -> Self {
        KMBehaviourEvent::Kademlia(event)
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "KMBehaviourEvent")]
pub struct KMBehaviour {
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: Mdns,
    pub ping: Ping,
    pub floodsub: Floodsub,
}

impl KMBehaviour {
    // 初始化订阅主题
    pub fn init_subscriptions(&mut self, topics: Vec<&str>) {
        for topic_str in topics {
            let topic = Topic::new(topic_str);
            self.floodsub.subscribe(topic);
        }
    }

    // 发送消息的增强版本
    pub fn send_message(&mut self, topic: Topic, content: String) -> Result<(), Box<dyn Error>> {
        let mut connection = db::establish_connection()?;
        let local_key = warehouse::get_warehouse_id(&mut connection)?;
        let local_peer_id = PeerId::from(PublicKey::Ed25519(local_key.public()));
        let message = NetworkMessage {
            message_type: "generic".to_string(),
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            sender: local_peer_id.to_string(),
        };
        // 序列化消息
        match serde_json::to_vec(&message) {
            Ok(message_bytes) => {
                self.floodsub.publish(topic, message_bytes);
                Ok(())
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn handle_message(&self, message: &[u8], source: &PeerId) -> Result<(), Box<dyn Error>> {
        let mut connection = db::establish_connection()?;
        let local_key = warehouse::get_warehouse_id(&mut connection)?;
        let local_peer_id = PeerId::from(PublicKey::Ed25519(local_key.public()));
        // 如果是自己发送的消息，则忽略
        if source == &local_peer_id {
            debug!("Ignoring self-sent message");
            return Ok(());
        }

        match serde_json::from_slice::<NetworkMessage>(message) {
            Ok(network_message) => {
                info!(
                    "Received message from peer {}: {:?}",
                    source, network_message
                );

                match network_message.message_type.as_str() {
                    "generic" => {
                        info!("Generic message content: {}", network_message.content);
                        Ok(())
                    }
                    "discovery" => {
                        info!("Discovery message from peer: {}", source);
                        // 这里可以添加对等点发现逻辑
                        Ok(())
                    }
                    _ => {
                        warn!("Unknown message type: {}", network_message.message_type);
                        Ok(())
                    }
                }
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    // 添加节点发现广播
    pub fn broadcast_discovery(&mut self) -> Result<(), Box<dyn Error>> {
        let mut connection = db::establish_connection()?;
        let local_key = warehouse::get_warehouse_id(&mut connection)?;
        let local_peer_id = PeerId::from(PublicKey::Ed25519(local_key.public()));
        let message = NetworkMessage {
            message_type: "discovery".to_string(),
            content: "Node discovery broadcast".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            sender: local_peer_id.to_string(),
        };

        let topic = Topic::new("discovery");
        if let Ok(message_bytes) = serde_json::to_vec(&message) {
            self.floodsub.publish(topic, message_bytes);
            info!("Sent discovery broadcast");
        }
        Ok(())
    }
}

pub type SwarmType = libp2p::swarm::Swarm<KMBehaviour>; // 定义 SwarmType 为 libp2p::swarm::Swarm<KMBehaviour>

pub async fn setup_network() -> Result<SwarmType, Box<dyn Error>> {
 // 创建本地PeerId
    let mut connection = db::establish_connection()?;
    migrations::run_db_migrations(&mut connection).await;
    let local_key = match warehouse::get_warehouse_id(&mut connection) {
        Ok(id) => id,
        Err(err) => {
            error!("Failed to get warehouse local key: {}", err);
            warehouse::generate_and_insert_new_local_key(&mut connection)
        }
    };
    let local_peer_id = PeerId::from(PublicKey::Ed25519(local_key.public()));
    info!("Local peer id: {:?}", local_peer_id);
    let floodsub = Floodsub::new(local_peer_id.clone());
    // 创建传输层
    let transport = development_transport(libp2p::identity::Keypair::Ed25519(local_key)).await?;

    // 创建Kademlia DHT
    let store = MemoryStore::new(local_peer_id.clone());
    let kademlia_config = KademliaConfig::default();
    let kademlia = Kademlia::with_config(local_peer_id.clone(), store, kademlia_config);

    // 创建mDNS
    let mdns = Mdns::new(MdnsConfig::default()).await?;

    let ping = Ping::new(PingConfig::new().with_keep_alive(true));

    // 创建网络行为
    let behaviour = KMBehaviour {
        kademlia,
        mdns,
        ping,
        floodsub,
    };

    // 构建Swarm
    let mut swarm = SwarmBuilder::new(transport, behaviour, local_peer_id)
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    // 获取环境变量中的 bootstrap_peer_id
    let bootstrap_peer_id_str = match env::var("BOOTSTRAP_PEER_ID") {
        Ok(val) => {
            info!("Find BOOTSTRAP_PEER_ID {:?}", val);
            val
        }
        Err(_) => {
            warn!("Warning: BOOTSTRAP_PEER_ID environment variable is not set");
            PeerId::from_public_key(&libp2p::identity::PublicKey::Ed25519(
                identity::ed25519::PublicKey::decode(&[0u8; 32]).unwrap(),
            ))
            .to_string()
        }
    };

    // 获取环境变量中的 bootstrap_addr
    let bootstrap_addr_str = match env::var("BOOTSTRAP_ADDR") {
        Ok(val) => {
            info!("Find BOOTSTRAP_ADDR {:?}", val);
            val
        }
        Err(_) => {
            warn!("Warning: BOOTSTRAP_ADDR environment variable is not set");
            "/ip4/127.0.0.1/tcp/8080".to_string()
        }
    };
    // 获取环境变量中的 listening_addr_str
    let listening_addr_str = match env::var("LISTENING_ADDR") {
        Ok(val) => {
            info!("Find LISTENING_ADDR {:?}", val);
            val
        }
        Err(_) => {
            warn!("Warning: LISTENING_ADDR environment variable is not set, using default value:/ip4/0.0.0.0/tcp/12345");
            "/ip4/0.0.0.0/tcp/12345".to_string()
        }
    };
    // 添加引导节点
    let bootstrap_peer_id = bootstrap_peer_id_str.parse::<PeerId>()?;
    let bootstrap_addr: Multiaddr = bootstrap_addr_str.parse().unwrap();
    match env::var_os("BOOTSTRAP_ADDR") {
        Some(addr_value) => {
            info!("find BOOTSTRAP_ADDR {:?}", addr_value);
            match env::var_os("BOOTSTRAP_PEER_ID") {
                Some(id_value) => {
                    info!(
                        "via {:?} discover node which peer id is {:?}",
                        addr_value, id_value
                    );
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&bootstrap_peer_id, bootstrap_addr);
                }
                None => {
                    warn!("BOOTSTRAP_PEER_ID environment variable is not set,run as bootstrap node")
                }
            }
        }
        None => warn!("BOOTSTRAP_ADDR environment variable is not set,run as bootstrap node"),
    }
    // 设置监听地址
    let listen_addr: Multiaddr = env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| listening_addr_str.to_string())
        .parse()
        .unwrap();
    swarm.listen_on(listen_addr)?;
    Ok(swarm) // 返回 Swarm 实例
}

pub async fn run_swarm(swarm: &mut SwarmType) -> Result<(), Box<dyn Error + Send + 'static>> {
    let mut discovered_peers = HashSet::new();
    let mut discovery_interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = discovery_interval.tick() => {
                if let Err(e) = swarm.behaviour_mut().broadcast_discovery() {
                    error!("Failed to broadcast discovery: {:?}", e);
                }
            }
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(KMBehaviourEvent::Mdns(MdnsEvent::Discovered(peers))) => {
                        for (peer_id, _) in peers {
                            if discovered_peers.insert(peer_id.clone()) {
                                info!("🔍 Discovered new peer via mDNS: {:?}", peer_id);
                                if let Err(e) = swarm.dial(peer_id.clone()) {
                                    error!("❌ Failed to dial discovered peer: {:?}", e);
                                }
                            }
                        }
                    }
                    SwarmEvent::Behaviour(KMBehaviourEvent::Mdns(MdnsEvent::Expired(peers))) => {
                        for (peer_id, _) in peers {
                            info!("Expired peer via mDNS: {:?}", peer_id);
                        }
                    }
                    SwarmEvent::Behaviour(KMBehaviourEvent::Kademlia(KademliaEvent::RoutingUpdated {
                        peer,
                        ..
                    })) => {
                        if discovered_peers.insert(peer.clone()) {
                            info!("Discovered peer via Kademlia: {:?}", peer);
                            if let Err(e) = swarm.dial(peer) {
                                info!("Failed to dial discovered peer: {:?}", e);
                            }
                        }
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("🔗 Connected to peer: {:?}", peer_id);
                    }
                    SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        info!("❌ Connection closed with peer {:?}: {:?}", peer_id, cause);
                    }
                    SwarmEvent::Behaviour(KMBehaviourEvent::Ping(event)) => {
                        info!("Ping: {:?}", event);
                    }
                    SwarmEvent::Behaviour(KMBehaviourEvent::Floodsub(FloodsubEvent::Message(message))) => {
                        if let Err(e) = swarm.behaviour().handle_message(&message.data, &message.source) {
                            error!("Error handling message: {:?}", e);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
