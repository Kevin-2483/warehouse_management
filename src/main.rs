#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::figment::{
    providers::{Env, Serialized},
    Figment,
};
use rocket::serde::json::Json;
use rocket::{Build, Config, Rocket};
use rocket_sync_db_pools::{database, diesel};

use crate::models::*;
use crate::schema::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::result::Result as StdResult; // 为了避免名称冲突，使用别名
use std::io;

use chrono::Local;
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use libp2p::futures::StreamExt;
use libp2p::kad::{record::store::MemoryStore, Kademlia, KademliaConfig, KademliaEvent};
use libp2p::mdns::{Mdns, MdnsConfig, MdnsEvent};
use libp2p::swarm::{
    NetworkBehaviour, NetworkBehaviourEventProcess, Swarm, SwarmBuilder, SwarmEvent,
};
use libp2p::{development_transport, identity, Multiaddr, NetworkBehaviour, PeerId};
use libp2p::request_response::{RequestResponse, ProtocolName, RequestResponseCodec, RequestResponseMessage};


use tokio;
use tokio::signal;
use tokio::task;
// use libp2p::dns::DnsConfig;
// use libp2p::tcp::GenTcpConfig;

use log::LevelFilter;
use log::{debug, error, info};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::Config as LogConfig;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::init_config;

mod models;
mod schema;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "KMBehaviourEvent")]
struct KMBehaviour {
    kademlia: Kademlia<MemoryStore>,
    mdns: Mdns,
}

#[derive(Debug)]
enum KMBehaviourEvent {
    Kademlia(KademliaEvent),
    Mdns(MdnsEvent),
}

// 协议结构体
#[derive(Debug, Clone)]
struct SyncProtocol;
// 编解码器结构体
#[derive(Clone)]
struct SyncCodec;
// 请求和响应的结构体，都是简单的字符串包装器
#[derive(Debug, Clone, PartialEq, Eq)]
struct SyncRequest(String);

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyncResponse(String);
// 为SyncProtocol实现了ProtocolName特征，定义了协议的名称为"/sync/1.0.0"
impl ProtocolName for SyncProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/sync/1.0.0".as_bytes()
    }
}

// #[async_trait::async_trait]
// impl RequestResponseCodec for SyncCodec {
//     type Protocol = SyncProtocol;
//     type Request = SyncRequest;
//     type Response = SyncResponse;

//     async fn read_request<T>(&mut self, _: &SyncProtocol, io: &mut T) -> io::Result<Self::Request>
//     where
//         T: AsyncRead + Unpin + Send,
//     {
//         let mut buf = vec![0; 1024];
//         let n = io.read(&mut buf).await?;
//         Ok(SyncRequest(String::from_utf8_lossy(&buf[..n]).to_string()))
//     }

//     async fn read_response<T>(&mut self, _: &SyncProtocol, io: &mut T) -> io::Result<Self::Response>
//     where
//         T: AsyncRead + Unpin + Send,
//     {
//         let mut buf = vec![0; 1024];
//         let n = io.read(&mut buf).await?;
//         Ok(SyncResponse(String::from_utf8_lossy(&buf[..n]).to_string()))
//     }

//     async fn write_request<T>(&mut self, _: &SyncProtocol, io: &mut T, SyncRequest(data): SyncRequest) -> io::Result<()>
//     where
//         T: AsyncWrite + Unpin + Send,
//     {
//         io.write_all(data.as_bytes()).await
//     }

//     async fn write_response<T>(&mut self, _: &SyncProtocol, io: &mut T, SyncResponse(data): SyncResponse) -> io::Result<()>
//     where
//         T: AsyncWrite + Unpin + Send,
//     {
//         io.write_all(data.as_bytes()).await
//     }
// }

impl From<KademliaEvent> for KMBehaviourEvent {
    fn from(event: KademliaEvent) -> Self {
        KMBehaviourEvent::Kademlia(event)
    }
}

impl From<MdnsEvent> for KMBehaviourEvent {
    fn from(event: MdnsEvent) -> Self {
        KMBehaviourEvent::Mdns(event)
    }
}

impl NetworkBehaviourEventProcess<KademliaEvent> for KMBehaviour {
    fn inject_event(&mut self, event: KademliaEvent) {
        // 处理 Kademlia 事件
        match event {
            KademliaEvent::RoutingUpdated { peer, .. } => {
                info!("Kademlia RoutingUpdated: {:?}", peer);
            }
            KademliaEvent::UnroutablePeer { peer } => {
                info!("Kademlia UnroutablePeer: {:?}", peer);
            }
            KademliaEvent::RoutablePeer { peer, .. } => {
                info!("Kademlia RoutablePeer: {:?}", peer);
            }
            KademliaEvent::PendingRoutablePeer { peer, .. } => {
                info!("Kademlia PendingRoutablePeer: {:?}", peer);
            }
            _ => {
                info!("Unhandled Kademlia event: {:?}", event);
            }
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for KMBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        // 处理 mDNS 事件
        match event {
            MdnsEvent::Discovered(peers) => {
                for (peer_id, _) in peers {
                    info!("mDNS discovered: {:?}", peer_id);
                }
            }
            MdnsEvent::Expired(peers) => {
                for (peer_id, _) in peers {
                    info!("mDNS expired: {:?}", peer_id);
                }
            }
        }
    }
}

#[database("sqlite_db")]
pub struct DbConn(SqliteConnection);

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = warehouses)]
pub struct NewWarehouse {
    pub name: String,
    pub location: String,
}

#[get("/warehouses")]
async fn get_warehouses(conn: DbConn) -> Json<Vec<Warehouse>> {
    conn.run(|c| {
        use schema::warehouses::dsl::*;
        warehouses
            .load::<Warehouse>(c)
            .map(Json)
            .expect("Error loading warehouses")
    })
    .await
}

#[post("/warehouses", format = "json", data = "<new_warehouse>")]
async fn create_warehouse(
    new_warehouse: Json<NewWarehouse>,
    conn: DbConn,
) -> Result<Json<Warehouse>, String> {
    let db = conn;

    let new_warehouse = db
        .run(move |c| {
            use self::schema::warehouses::dsl::*;

            let existing_warehouse = warehouses
                .filter(location.eq(&new_warehouse.location))
                .first::<Warehouse>(c)
                .optional()
                .map_err(|_| "Error checking for existing warehouse")?;

            if let Some(_) = existing_warehouse {
                return Err("Warehouse with this location already exists".to_string());
            }

            let new_id = Uuid::new_v4().as_u128() as i32;

            let new_warehouse = Warehouse {
                id: new_id,
                name: new_warehouse.name.clone(),
                location: new_warehouse.location.clone(),
                created_at: Some(Utc::now().naive_utc()),
                updated_at: Some(Utc::now().naive_utc()),
            };

            diesel::insert_into(warehouses)
                .values(&new_warehouse)
                .execute(c)
                .map_err(|_| "Error inserting new warehouse")?;

            Ok(Json(new_warehouse))
        })
        .await?;
    Ok(new_warehouse)
}

struct AdminInit;

#[rocket::async_trait]
impl Fairing for AdminInit {
    fn info(&self) -> Info {
        Info {
            name: "Admin Initialization",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
        let db = DbConn::get_one(&rocket).await.expect("database connection");

        db.run(|c| {
            use self::administrators::dsl::*;

            let admin_count: i64 = administrators
                .count()
                .get_result(c)
                .expect("Error counting admins");

            if admin_count == 0 {
                let random_password: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(12)
                    .map(char::from)
                    .collect();

                info!("默认管理员已创建，用户名: admin, 密码: {}", random_password);

                diesel::insert_into(administrators)
                    .values((username.eq("admin"), password.eq(random_password)))
                    .execute(c)
                    .expect("Error inserting admin");
            }
        })
        .await;

        Ok(rocket)
    }
}

async fn run_db_migrations(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    let db = DbConn::get_one(&rocket).await.expect("database connection");

    match db
        .run(|conn| {
            conn.run_pending_migrations(MIGRATIONS).map(|versions| {
                // Log each version of the migration
                versions
                    .into_iter()
                    .map(|v| {
                        let version_str = v.to_string();
                        info!("Successfully applied migration: {}", version_str);
                        version_str
                    })
                    .collect::<Vec<_>>()
            })
        })
        .await
    {
        Ok(versions) => {
            info!(
                "All pending migrations were run successfully: {:?}",
                versions
            );
            Ok(rocket)
        }
        Err(e) => {
            error!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    }
}

#[tokio::main]
async fn main() -> StdResult<(), Box<dyn Error>> {
    // 获取当前时间并格式化为文件名
    let now = Local::now();
    let log_file_name = format!("log/output_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));

    // 构建文件 appender
    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build(log_file_name)
        .unwrap();

    // 构建 console appender
    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build();

    // 创建 root 配置
    let logconfig = LogConfig::builder()
        .appender(Appender::builder().build("stdout", Box::new(console_appender)))
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Info),
        )
        .unwrap();

    // 初始化 log4rs 配置
    init_config(logconfig).unwrap();

    // 创建本地PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    info!("Local peer id: {:?}", local_peer_id);

    // 创建传输层
    let transport = development_transport(local_key.clone()).await?;

    // 创建Kademlia DHT
    let store = MemoryStore::new(local_peer_id.clone());
    let kademlia_config = KademliaConfig::default();
    let kademlia = Kademlia::with_config(local_peer_id.clone(), store, kademlia_config);

    // 创建mDNS
    let mdns = Mdns::new(MdnsConfig::default()).await?;

    // 创建网络行为
    let behaviour = KMBehaviour { kademlia, mdns };

    // 构建Swarm
    let mut swarm = SwarmBuilder::new(transport, behaviour, local_peer_id)
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    // 获取环境变量中的 bootstrap_peer_id
    let bootstrap_peer_id_str = match env::var("BOOTSTRAP_PEER_ID") {
        Ok(val) => val,
        Err(_) => {
            error!(
                "Warning: BOOTSTRAP_PEER_ID environment variable is not set, using radmon value"
            );
            PeerId::from_public_key(&identity::PublicKey::Ed25519(
                identity::ed25519::PublicKey::decode(&[0u8; 32]).unwrap(),
            ))
            .to_string()
        }
    };

    // 获取环境变量中的 bootstrap_addr
    let bootstrap_addr_str = match env::var("BOOTSTRAP_ADDR") {
        Ok(val) => val,
        Err(_) => {
            error!("Warning: BOOTSTRAP_ADDR environment variable is not set,using default value:/ip4/127.0.0.1/tcp/8080");
            "/ip4/127.0.0.1/tcp/8080".to_string()
        }
    };
    // 获取环境变量中的 listening_addr_str
    let listening_addr_str = match env::var("LISTENING_ADDR") {
        Ok(val) => val,
        Err(_) => {
            error!("Warning: LISTENING_ADDR environment variable is not set, using default value:/ip4/0.0.0.0/tcp/12345");
            "/ip4/0.0.0.0/tcp/12345".to_string()
        }
    };
    // 添加引导节点
    let bootstrap_peer_id = bootstrap_peer_id_str.parse::<PeerId>()?;
    let bootstrap_addr: Multiaddr = bootstrap_addr_str.parse().unwrap();
    swarm
        .behaviour_mut()
        .kademlia
        .add_address(&bootstrap_peer_id, bootstrap_addr);

    // 设置监听地址
    let listen_addr: Multiaddr = env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| listening_addr_str.to_string())
        .parse()
        .unwrap();
    Swarm::listen_on(&mut swarm, listen_addr)?;
    // 连接到其他节点
    // let remote_peer_id = PeerId::from_public_key(&identity::PublicKey::Ed25519(identity::ed25519::PublicKey::decode(&[1u8; 32]).unwrap()));
    // let remote_addr: Multiaddr = env::var("REMOTE_ADDR").unwrap_or_else(|_| "/ip4/127.0.0.1/tcp/8080".to_string()).parse().unwrap();
    // Swarm::dial(&mut swarm, remote_addr.clone()).expect("Failed to dial address");
    // swarm.behaviour_mut().kademlia.add_address(&remote_peer_id, remote_addr);
    // 并发运行 Rocket 和 Swarm

    let rocket_handle = task::spawn(async {
        let rocket = rocket().await;
        rocket.launch().await.unwrap();
    });

    let swarm_handle = task::spawn(async move {
        loop {
            match swarm.next().await.unwrap() {
                SwarmEvent::Behaviour(KMBehaviourEvent::Mdns(MdnsEvent::Discovered(peers))) => {
                    for (peer_id, _) in peers {
                        info!("Discovered peer via mDNS: {:?}", peer_id);
                    }
                }
                SwarmEvent::Behaviour(KMBehaviourEvent::Mdns(MdnsEvent::Expired(peers))) => {
                    for (peer_id, _) in peers {
                        info!("Expired peer via mDNS: {:?}", peer_id);
                    }
                }
                SwarmEvent::Behaviour(KMBehaviourEvent::Kademlia(
                    KademliaEvent::RoutingUpdated { peer, .. },
                )) => {
                    info!("Discovered peer via Kademlia: {:?}", peer);
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    info!("Connected to peer: {:?}", peer_id);
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    if let Some(err) = cause {
                        info!(
                            "Connection to peer {:?} closed with error: {:?}",
                            peer_id, err
                        );
                    } else {
                        info!("Connection to peer {:?} closed.", peer_id);
                    }
                }
                _ => {}
            }
        }
    });

    let mut rocket_handle = Some(rocket_handle);
    let mut swarm_handle = Some(swarm_handle);

    // 处理 SIGINT 信号
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, shutting down...");
        }
        _ = async { if let Some(handle) = rocket_handle.take() { handle.await.unwrap(); } } => {
            info!("Rocket task completed.");
        }
        _ = async { if let Some(handle) = swarm_handle.take() { handle.await.unwrap(); } } => {
            info!("Swarm task completed.");
        }
    }

    info!("Waiting for tasks to finish...");
    if let Some(handle) = rocket_handle {
        handle.await?;
    }
    if let Some(handle) = swarm_handle {
        handle.await?;
    }

    Ok(())
}

async fn rocket() -> Rocket<Build> {
    // 从默认配置创建 Figment 实例
    let figment = Figment::from(Config::default())
        // 合并自定义的数据库配置
        .merge(Serialized::default("databases", {
            let mut databases: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
            let mut db_config: HashMap<&str, &str> = HashMap::new();
            db_config.insert("url", "sqlite://./warehouse.db");
            databases.insert("sqlite_db", db_config);
            databases
        }))
        // 合并环境变量配置，前缀为 "APP_"
        .merge(Env::prefixed("APP_"));

    // 使用自定义的配置启动 Rocket 应用程序
    let rocket = rocket::custom(figment)
        // 附加数据库连接
        .attach(DbConn::fairing())
        // 添加数据库迁移 fairing
        .attach(AdHoc::try_on_ignite(
            "Database Migrations",
            run_db_migrations,
        ))
        .attach(AdminInit)
        // 挂载路由
        .mount("/", routes![index])
        .mount("/api", routes![get_warehouses, create_warehouse]);

    rocket
}
