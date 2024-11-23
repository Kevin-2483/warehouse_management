use std::collections::HashSet;
use std::env;
use std::error::Error;
use libp2p::floodsub::Floodsub;
use libp2p::floodsub::Topic;
use libp2p::kad::{Kademlia, KademliaConfig};
use libp2p::kad::store::MemoryStore;
use libp2p::mdns::{Mdns, MdnsConfig};
use libp2p::ping::{Ping, PingConfig, PingEvent};
use libp2p::{identity, Multiaddr, PeerId};
use libp2p::floodsub::FloodsubEvent;
use libp2p::kad::KademliaEvent;
use libp2p::mdns::MdnsEvent;
use libp2p::swarm::SwarmEvent;
use log::{error, info, warn};
use crate::migrations;
use crate::db;
use crate::warehouse;
use libp2p::NetworkBehaviour;
use libp2p::identity::PublicKey;
use libp2p::development_transport;
use libp2p::swarm::SwarmBuilder;
use futures::stream::StreamExt;

pub enum KMBehaviourEvent {
    Mdns(MdnsEvent),
    Kademlia(KademliaEvent),
    Ping(PingEvent),
    Floodsub(FloodsubEvent),
}

// 为 KMBehaviourEvent 实现 From 特征
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
    // 添加发送消息的函数
    pub fn send_message(&mut self, topic: Topic, message: String) {
        // 直接使用 publish 方法发布消息
        // floodsub.publish() 接受 Topic 和消息内容
        self.floodsub.publish(topic, message.as_bytes());
    }
}

pub type SwarmType = libp2p::swarm::Swarm<KMBehaviour>; // 定义 SwarmType 为 libp2p::swarm::Swarm<KMBehaviour>

pub async fn setup_network() -> Result<SwarmType, Box<dyn Error>>  {
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

pub async fn run_swarm(swarm: &mut SwarmType) -> Result<(), Box<dyn Error  + Send + 'static>> {
    let mut discovered_peers = HashSet::new();
    loop {
        let event = swarm.select_next_some().await;
        match event {
            SwarmEvent::Behaviour(KMBehaviourEvent::Mdns(MdnsEvent::Discovered(peers))) => {
                for (peer_id, _) in peers {
                    if discovered_peers.insert(peer_id.clone()) {
                        info!("Discovered peer via mDNS: {:?}", peer_id);
                        if let Err(e) = swarm.dial(peer_id.clone()) {
                            info!("Failed to dial discovered peer: {:?}", e);
                        }
                    }
                }
            }
            SwarmEvent::Behaviour(KMBehaviourEvent::Mdns(MdnsEvent::Expired(peers))) => {
                for (peer_id, _) in peers {
                    info!("Expired peer via mDNS: {:?}", peer_id);
                }
            }
            SwarmEvent::Behaviour(KMBehaviourEvent::Kademlia(KademliaEvent::RoutingUpdated { peer, .. })) => {
                if discovered_peers.insert(peer.clone()) {
                    info!("Discovered peer via Kademlia: {:?}", peer);
                    if let Err(e) = swarm.dial(peer) {
                        info!("Failed to dial discovered peer: {:?}", e);
                    }
                }
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to peer: {:?}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                if let Some(err) = cause {
                    info!("Connection to peer {:?} closed with error: {:?}", peer_id, err);
                } else {
                    info!("Connection to peer {:?} closed.", peer_id);
                }
            }
            SwarmEvent::Behaviour(KMBehaviourEvent::Ping(event)) => {
                info!("Ping: {:?}", event);
            }
            SwarmEvent::Behaviour(KMBehaviourEvent::Floodsub(FloodsubEvent::Message(message))) => {
                info!("Received: '{:?}' from {:?}", String::from_utf8_lossy(&message.data), message.source);
            }
            _ => {}
        }
    }
}
