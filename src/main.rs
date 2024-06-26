#[macro_use]
extern crate rocket;

extern crate base64;

// use rocket::fairing::AdHoc;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::figment::{
    providers::{Env, Serialized},
    Figment,
};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
// use rocket::request;
// use rocket::response::status;
use rocket::http::Method;
use rocket::http::{Cookie, CookieJar};
// use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::Request;
use rocket::{Build, Config, Rocket};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use rocket_sync_db_pools::{database, diesel};

use crate::models::*;
use crate::schema::*;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::Integer;
use diesel::sql_types::Nullable;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
// use std::io::Cursor;
use std::result::Result as StdResult; // 为了避免名称冲突，使用别名
                                      // use std::time::Duration;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
// use std::clone;

use chrono::Local;
use chrono::NaiveDateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
// use uuid::Uuid;

use libp2p::floodsub::{Floodsub, FloodsubEvent, Topic};
// use libp2p::floodsub;
use libp2p::futures::StreamExt;
use libp2p::identity::{self, ed25519, PublicKey};
// use libp2p::identity::Keypair;
use libp2p::kad::{record::store::MemoryStore, Kademlia, KademliaConfig, KademliaEvent};

use libp2p::mdns::{Mdns, MdnsConfig, MdnsEvent};
use libp2p::ping::{Ping, PingConfig, PingEvent};
// use libp2p::request_response::{
//     ProtocolName, RequestResponse, RequestResponseCodec, RequestResponseEvent,
//     RequestResponseMessage,
// };
use libp2p::swarm::{Swarm, SwarmBuilder, SwarmEvent};

// use libp2p::swarm::{
//     NetworkBehaviour, NetworkBehaviourEventProcess
// };
use libp2p::{development_transport, Multiaddr, NetworkBehaviour, PeerId};

use tokio;
// use tokio::io::{ AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::io;
use tokio::signal;
use tokio::task;
use tokio_util::compat::TokioAsyncReadCompatExt;
// use tokio_util::compat::FuturesAsyncReadCompatExt;
// use libp2p::dns::DnsConfig;
// use libp2p::tcp::GenTcpConfig;

use log::LevelFilter;
use log::{error, info, warn};
// use log::debug;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::Config as LogConfig;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::init_config;

use base64::{engine::general_purpose, Engine as _};
use futures::prelude::*;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

mod models;
mod schema;

#[derive(Debug)]
struct AdminInit;
#[derive(Debug)]
pub struct JwtToken(pub String);

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomResponder {
    status: Status,
    message: String,
}

#[derive(FromForm, Deserialize)]
struct LoginCredentials {
    username: String,
    password: String,
}

impl Claims {
    fn new(username: &str) -> Self {
        let now = SystemTime::now();
        let exp = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 1800; // Token expires in 30 minutes

        Self {
            sub: username.to_owned(),
            exp: exp as usize,
        }
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "KMBehaviourEvent")]
struct KMBehaviour {
    kademlia: Kademlia<MemoryStore>,
    mdns: Mdns,
    ping: Ping,
    floodsub: Floodsub,
}

enum KMBehaviourEvent {
    Kademlia(KademliaEvent),
    Mdns(MdnsEvent),
    Ping(PingEvent),
    Floodsub(FloodsubEvent),
}

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

impl From<PingEvent> for KMBehaviourEvent {
    fn from(event: PingEvent) -> Self {
        KMBehaviourEvent::Ping(event)
    }
}

impl From<FloodsubEvent> for KMBehaviourEvent {
    fn from(event: FloodsubEvent) -> Self {
        KMBehaviourEvent::Floodsub(event)
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for JwtToken {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let cookies: &CookieJar<'_> = request.cookies();
        if let Some(cookie) = cookies.get("token") {
            let token = cookie.value();
            match decode_token(token) {
                Some(username) => Outcome::Success(JwtToken(username)),
                None => Outcome::Error((Status::Unauthorized, ())),
            }
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

// 生成 JWT
fn generate_token(username: &str) -> String {
    let claims = Claims::new(username);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("your_secret_key".as_ref()), // 替换为你的密钥
    )
    .unwrap()
}

fn decode_token(token: &str) -> Option<String> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret("your_secret_key".as_ref()),
        &Validation::default(),
    );

    match token_data {
        Ok(token) => {
            if token.claims.exp
                >= SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as usize
            {
                Some(token.claims.sub)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

// impl NetworkBehaviourEventProcess<KademliaEvent> for KMBehaviour {
//     fn inject_event(&mut self, event: KademliaEvent) {
//         // 处理 Kademlia 事件
//         match event {
//             KademliaEvent::RoutingUpdated { peer, .. } => {
//                 info!("Kademlia RoutingUpdated: {:?}", peer);
//             }
//             KademliaEvent::UnroutablePeer { peer } => {
//                 info!("Kademlia UnroutablePeer: {:?}", peer);
//             }
//             KademliaEvent::RoutablePeer { peer, .. } => {
//                 info!("Kademlia RoutablePeer: {:?}", peer);
//             }
//             KademliaEvent::PendingRoutablePeer { peer, .. } => {
//                 info!("Kademlia PendingRoutablePeer: {:?}", peer);
//             }
//             _ => {
//                 info!("Unhandled Kademlia event: {:?}", event);
//             }
//         }
//     }
// }

// impl NetworkBehaviourEventProcess<MdnsEvent> for KMBehaviour {
//     fn inject_event(&mut self, event: MdnsEvent) {
//         // 处理 mDNS 事件
//         match event {
//             MdnsEvent::Discovered(peers) => {
//                 for (peer_id, _) in peers {
//                     info!("mDNS discovered: {:?}", peer_id);
//                 }
//             }
//             MdnsEvent::Expired(peers) => {
//                 for (peer_id, _) in peers {
//                     info!("mDNS expired: {:?}", peer_id);
//                 }
//             }
//         }
//     }
// }

// impl NetworkBehaviourEventProcess<PingEvent> for KMBehaviour {
//     fn inject_event(&mut self, event: PingEvent) {
//         // 处理 Ping 事件
//         info!("Ping event: {:?}", event);
//     }
// }

// impl NetworkBehaviourEventProcess<FloodsubEvent> for KMBehaviour {
//     fn inject_event(&mut self, event: FloodsubEvent) {
//         // 处理 Floodsub 事件
//         match event {
//             FloodsubEvent::Message(message) => {
//                 info!("Received: '{}' from {:?}", String::from_utf8_lossy(&message.data), message.source);
//             }
//             _ => {
//                 info!("Unhandled Floodsub event: {:?}", event);
//             }
//         }
//     }
// }

#[database("sqlite_db")]
pub struct DbConn(SqliteConnection);

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = warehouses)]
pub struct NewWarehouse {
    pub id: String,
    pub name: String,
    pub location: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = categories)]
pub struct NewCategory {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = administrators)]
pub struct NewAdministrator {
    pub username: String,
    pub password: String,
    pub superuser: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
#[derive(Queryable, Serialize, Deserialize)]
#[diesel(table_name = warehouses)]
pub struct GetWarehouses {
    pub id: String,
    pub name: String,
    pub location: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[get("/warehouses")]
async fn get_warehouses(
    conn: DbConn,
    // token: JwtToken
) -> Result<Json<Vec<GetWarehouses>>, Json<CustomResponder>> {
    // use schema::administrators::dsl::*;
    conn.run(move |c| {
        // let admin = administrators
        //     .filter(username.eq(token.0.clone()))
        //     .first::<Administrator>(c)
        //     .optional()
        //     .map_err(|_| Json(CustomResponder{ status:rocket::http::Status::InternalServerError, message: "Error checking for administrator".to_string()}))?;
        // if admin.is_none() {
        //     return Err(Json(CustomResponder{ status:rocket::http::Status::Unauthorized, message: "Permission Denied".to_string()}));
        // }
        use schema::warehouses::dsl::*;

        let result = warehouses
            .select((id, name, location, created_at, updated_at)) // 选择除去localkey之外的列
            .load::<GetWarehouses>(c)
            .map(Json)
            .expect("Error loading warehouses");
        Ok(result)
    })
    .await
}

#[post("/products", format = "application/json", data = "<new_product>")]
async fn create_product(
    conn: DbConn,
    new_product: Json<NewProduct>,
    token: JwtToken,
) -> Result<Json<NewProduct>, Json<CustomResponder>> {
    let db = conn;
    use schema::administrators::dsl::*;
    let new_product = db
        .run(move |c| {
            let admin = administrators
                .filter(username.eq(token.0.clone()))
                .first::<Administrator>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for administrator".to_string(),
                    })
                })?;

            if admin.is_none() {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::Unauthorized,
                    message: "Permission Denied".to_string(),
                }));
            }

            use crate::schema::products::dsl::*;
            let new_product = NewProduct {
                name: new_product.name.clone(),
                description: new_product.description.clone(),
                category_id: new_product.category_id.clone(),
                created_at: Some(Utc::now().naive_utc()),
                updated_at: Some(Utc::now().naive_utc()),
            };

            let existing_product = products
                .filter(name.eq(&new_product.name))
                .first::<Product>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for product".to_string(),
                    })
                })?;

            if let Some(_) = existing_product {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::BadRequest,
                    message: "Product with this name already exists".to_string(),
                }));
            }

            diesel::insert_into(products)
                .values(&new_product)
                .execute(c)
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error inserting new product".to_string(),
                    })
                })?;
            Ok(Json(new_product))
        })
        .await?;
    Ok(new_product)
}

#[post("/categories", format = "application/json", data = "<new_category>")]
async fn create_category(
    conn: DbConn,
    new_category: Json<NewCategory>,
    token: JwtToken,
) -> Result<Json<NewCategory>, Json<CustomResponder>> {
    let db = conn;
    use schema::administrators::dsl::*;
    let new_category = db
        .run(move |c| {
            let admin = administrators
                .filter(username.eq(token.0.clone()))
                .first::<Administrator>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for administrator".to_string(),
                    })
                })?;

            if admin.is_none() {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::Unauthorized,
                    message: "Permission Denied".to_string(),
                }));
            }
            use crate::schema::categories::dsl::*;
            let new_category = NewCategory {
                name: new_category.name.clone(),
                description: new_category.description.clone(),
            };

            let existing_category = categories
                .filter(name.eq(&new_category.name))
                .first::<Category>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for category".to_string(),
                    })
                })?;

            if let Some(_) = existing_category {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::BadRequest,
                    message: "Category with this name already exists".to_string(),
                }));
            }

            diesel::insert_into(categories)
                .values(&new_category)
                .execute(c)
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error inserting new category".to_string(),
                    })
                })?;
            Ok(Json(new_category))
        })
        .await?;
    Ok(new_category)
}

#[post("/warehouses", format = "json", data = "<new_warehouse>")]
async fn create_warehouse(
    new_warehouse: Json<NewWarehouse>,
    conn: DbConn,
    token: JwtToken,
) -> Result<Json<Warehouse>, Json<CustomResponder>> {
    let db = conn;
    use schema::administrators::dsl::*;
    let new_warehouse = db
        .run(move |c| {
            let admin = administrators
                .filter(username.eq(token.0.clone()))
                .first::<Administrator>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for administrator".to_string(),
                    })
                })?;

            if admin.is_none() {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::Unauthorized,
                    message: "Permission Denied".to_string(),
                }));
            }

            use self::schema::warehouses::dsl::*;

            let existing_warehouse = warehouses
                .filter(location.eq(&new_warehouse.location))
                .first::<Warehouse>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for warehouses".to_string(),
                    })
                })?;

            if let Some(_) = existing_warehouse {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::BadRequest,
                    message: "Warehouse with this location already exists".to_string(),
                }));
            }

            let new_warehouse = Warehouse {
                id: new_warehouse.id.clone(),
                localkey: Some("".to_string()),
                name: new_warehouse.name.clone(),
                location: new_warehouse.location.clone(),
                created_at: Some(Utc::now().naive_utc()),
                updated_at: Some(Utc::now().naive_utc()),
            };

            diesel::insert_into(warehouses)
                .values(&new_warehouse)
                .execute(c)
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error inserting new warehouse".to_string(),
                    })
                })?;

            Ok(Json(new_warehouse))
        })
        .await?;
    Ok(new_warehouse)
}

#[get("/products?<start>&<end>&<categories_name>")]
async fn get_products(
    conn: DbConn,
    start: String,
    end: String,
    categories_name: Option<String>,
    token: JwtToken,
) -> Result<Json<Vec<Product>>, Json<CustomResponder>> {
    use chrono::NaiveDateTime;
    use diesel::prelude::*;
    use schema::administrators::dsl::*;
    use schema::categories::dsl::{categories, id as cat_id, name as cat_name};
    use schema::products::dsl::{category_id, created_at, products};

    // 解析start和end为NaiveDateTime
    let start_dt = NaiveDateTime::parse_from_str(&start, "%Y-%m-%d %H:%M:%S").unwrap();
    let end_dt = NaiveDateTime::parse_from_str(&end, "%Y-%m-%d %H:%M:%S").unwrap();

    let results = conn
        .run(move |c| {
            let admin = administrators
                .filter(username.eq(token.0.clone()))
                .first::<Administrator>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for administrator".to_string(),
                    })
                })?;

            if admin.is_none() {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::Unauthorized,
                    message: "Permission Denied".to_string(),
                }));
            }

            let mut query = products
                .filter(created_at.between(start_dt, end_dt))
                .into_boxed();

            // 如果categories_name有值，添加分类名过滤条件
            if let Some(cat_name_filter) = categories_name {
                query = query.filter(
                    category_id.eq_any(
                        categories
                            .filter(cat_name.eq(cat_name_filter))
                            .select(cat_id),
                    ),
                );
            }

            Ok(query.load::<Product>(c).expect("Error loading warehouses")) // 如果加载或转换过程中出现错误，抛出一个错误
        })
        .await;

    results.map(Json)
}

#[get("/inventory?<start>&<end>&<product_name>")]
async fn get_inventory(
    conn: DbConn,
    start: String,
    end: String,
    product_name: Option<String>,
    token: JwtToken,
) -> Result<Json<Vec<Inventory>>, Json<CustomResponder>> {
    use chrono::NaiveDateTime;
    use diesel::prelude::*;
    use schema::administrators::dsl::*;

    use schema::inventory::dsl::{created_at,inventory, product_id};
    use schema::products::dsl::{name as pro_name, products};

    // 解析start和end为NaiveDateTime
    let start_dt = NaiveDateTime::parse_from_str(&start, "%Y-%m-%d %H:%M:%S").unwrap();
    let end_dt = NaiveDateTime::parse_from_str(&end, "%Y-%m-%d %H:%M:%S").unwrap();

    let results = conn
        .run(move |c| {
            let admin = administrators
                .filter(username.eq(token.0.clone()))
                .first::<Administrator>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for administrator".to_string(),
                    })
                })?;

            if admin.is_none() {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::Unauthorized,
                    message: "Permission Denied".to_string(),
                }));
            }

            let mut query = inventory
                .filter(created_at.between(start_dt, end_dt))
                .into_boxed();

            if let Some(product_name_filter) = &product_name {
                // 直接将 pro_id 的类型声明为 Integer 而不是 Nullable<Integer>
                let subquery = products
                    .filter(pro_name.eq(product_name_filter))
                    .select(sql::<Integer>("COALESCE(pro_id, 0)"));
                query = query.filter(product_id.eq_any(subquery));
            }
            

            Ok(query
                .load::<Inventory>(c)
                .expect("Error loading warehouses")) // 如果加载或转换过程中出现错误，抛出一个错误
        })
        .await;

    results.map(Json)
}

// 登录接口
#[post("/login", data = "<credentials>")]
async fn login(
    credentials: Json<LoginCredentials>,
    conn: DbConn,
    cookies: &CookieJar<'_>,
) -> Result<Json<CustomResponder>, Json<CustomResponder>> {
    use schema::administrators::dsl::*;

    let input_username = credentials.username.clone(); // 提取用户名
    let input_password = credentials.password.clone(); // 提取密码，并用不同的变量名

    let result = conn
        .run(move |c| {
            administrators
                .filter(username.eq(&input_username))
                .first::<Administrator>(c)
                .optional()
        })
        .await;

    match result {
        Ok(Some(administrator)) if administrator.password == input_password => {
            let token = generate_token(&administrator.username);
            // 将 token 存入 Cookie 中
            let cookie: Cookie = Cookie::build(("token", token.clone()))
                .path("/")
                .secure(false)
                .http_only(true)
                .build();
            cookies.add(cookie);
            Ok(Json(CustomResponder {
                status: rocket::http::Status::Ok,
                message: "Success".to_string(),
            }))
        }
        _ => Err(Json(CustomResponder {
            status: rocket::http::Status::InternalServerError,
            message: "Error checking for administrator".to_string(),
        })),
    }
}

#[post("/signup", data = "<new_administrator>")]
async fn signup(
    new_administrator: Json<NewAdministrator>,
    conn: DbConn,
    token: JwtToken,
) -> Result<Json<CustomResponder>, Json<CustomResponder>> {
    use schema::administrators::dsl::*;
    let input_username = new_administrator.username.clone(); // 提取用户名
    let input_password = new_administrator.password.clone(); // 提取密码，并用不同的变量名
    let input_superuser = new_administrator.superuser.clone();
    let result = conn
        .run(move |c| {
            let admin = administrators
                .filter(username.eq(token.0))
                .filter(superuser.eq(true))
                .first::<Administrator>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::InternalServerError,
                        message: "Error checking for administrator".to_string(),
                    })
                })?;
            if admin.is_none() {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::Unauthorized,
                    message: "Permission Denied".to_string(),
                }));
            }
            let existing_admin = administrators
                .filter(username.eq(input_username.clone()))
                .first::<Administrator>(c)
                .optional()
                .map_err(|_| {
                    Json(CustomResponder {
                        status: rocket::http::Status::Unauthorized,
                        message: "Error checking for administrator".to_string(),
                    })
                })?;
            if let Some(_) = existing_admin {
                return Err(Json(CustomResponder {
                    status: rocket::http::Status::BadRequest,
                    message: "Administrator with this username already exists".to_string(),
                }));
            }
            let new_administrator = NewAdministrator {
                username: input_username,
                password: input_password,
                superuser: input_superuser,
                created_at: Some(Utc::now().naive_utc()),
                updated_at: Some(Utc::now().naive_utc()),
            };
            diesel::insert_into(administrators)
                .values(&new_administrator)
                .execute(c)
                .expect("Error inserting admin");
            Ok(Json(CustomResponder {
                status: rocket::http::Status::Ok,
                message: "Administrator created".to_string(),
            }))
        })
        .await;
    result
}

#[rocket::options("/login")]
fn options_login() -> Status {
    Status::Ok // 返回一个允许的状态码，如 200 OK
}
#[rocket::options("/signup")]
fn options_signup() -> Status {
    Status::Ok // 返回一个允许的状态码，如 200 OK
}
// 受保护的路由
#[get("/protected")]
fn protected_route(token: JwtToken) -> String {
    format!("Hello, {}! This is a protected route.", token.0)
}

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
                let mut random_password = String::from("111");

                #[cfg(not(debug_assertions))]
                {
                    random_password = rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(12)
                        .map(char::from)
                        .collect();
                }

                info!("默认管理员已创建，用户名: admin, 密码: {}", random_password);
                diesel::insert_into(administrators)
                    .values((
                        username.eq("admin"),
                        password.eq(random_password),
                        superuser.eq(true),
                    ))
                    .execute(c)
                    .expect("Error inserting admin");
            }
        })
        .await;

        Ok(rocket)
    }
}

async fn run_db_migrations(conn: &mut SqliteConnection) {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    if let Err(err) = conn.run_pending_migrations(MIGRATIONS) {
        error!("Error running migrations: {}", err);
    } else {
        info!("Database migrations executed successfully");
    }
}

// 建立数据库连接
fn establish_connection() -> SqliteConnection {
    let database_url = "sqlite://./warehouse.db";
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

// 获取名为"ThisWarehouse"的仓库ID MpHCXo8e0RfSru1kQQKoJawUgEUs9oYmktHPF+bT26o=
fn get_warehouse_id(
    conn: &mut SqliteConnection,
) -> Result<ed25519::Keypair, diesel::result::Error> {
    use self::warehouses::dsl::*;
    let warehouse: Warehouse = warehouses.filter(localkey.is_not_null()).first(conn)?;
    info!("Found warehouse: {:?}", warehouse.localkey);
    let mut local_key_bytes = if let Some(local_key) = &warehouse.localkey {
        general_purpose::STANDARD
            .decode(local_key)
            .expect("Base64 decode error")
    } else {
        Vec::new() // 或者其他默认值的处理
    };
    let keypair =
        ed25519::Keypair::decode(local_key_bytes.as_mut_slice()).expect("Keypair decode error");
    Ok(keypair)
}

fn generate_and_insert_new_local_key(conn: &mut SqliteConnection) -> ed25519::Keypair {
    let local_key = ed25519::Keypair::generate();
    let local_key_base64 = general_purpose::STANDARD.encode(local_key.encode());
    info!(
        "Generated and inserted new key {:?} for warehouse ThisWarehouse",
        local_key_base64
    );
    let local_peer_id = PeerId::from(PublicKey::Ed25519(local_key.public()));
    let new_warehouse = Warehouse {
        id: local_peer_id.to_string(),
        localkey: Some(local_key_base64),
        name: "ThisWarehouse".to_string(),
        location: "/ip4/127.0.0.1/tcp/8080".to_string(),
        created_at: Some(Utc::now().naive_utc()),
        updated_at: Some(Utc::now().naive_utc()),
    };

    use self::warehouses::dsl::*;
    diesel::insert_into(warehouses)
        .values(&new_warehouse)
        .execute(conn)
        .expect("Error inserting new warehouse");

    local_key
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
    let mut connection = establish_connection();
    run_db_migrations(&mut connection).await;
    let local_key = match get_warehouse_id(&mut connection) {
        Ok(id) => id,
        Err(err) => {
            error!("Failed to get warehouse local key: {}", err);
            generate_and_insert_new_local_key(&mut connection)
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

    let topic = Topic::new("dev");
    // 获取环境变量中的 bootstrap_peer_id
    let bootstrap_peer_id_str = match env::var("BOOTSTRAP_PEER_ID") {
        Ok(val) => {
            info!("Find BOOTSTRAP_PEER_ID {:?}", val);
            val
        }
        Err(_) => {
            warn!("Warning: BOOTSTRAP_PEER_ID environment variable is not set");
            PeerId::from_public_key(&identity::PublicKey::Ed25519(
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
    Swarm::behaviour_mut(&mut swarm)
        .floodsub
        .subscribe(topic.clone());

    Swarm::listen_on(&mut swarm, listen_addr)?;

    let rocket_handle = task::spawn(async {
        let rocket = rocket().await;
        rocket.launch().await.unwrap();
    });

    let swarm_handle = task::spawn(async move {
        let mut discovered_peers = HashSet::new();
        let mut stdin = io::BufReader::new(io::stdin()).compat().lines();
        loop {
            tokio::select! {
                    line = stdin.next() => {
                        match line {
                            Some(Ok(line)) => {
                                let _ = swarm.behaviour_mut().floodsub.publish(topic.clone(), line.as_bytes());
                                info!("floodsub {:?} publish: {:?}", topic.clone(), line);
                            }
                            Some(Err(e)) => {
                                info!("Error reading from stdin: {:?}", e);
                            }
                            None => break, // EOF reached
                        }
                    }
                    event = swarm.next() => {
                        match event {
                            Some(event) => match event {
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
                                SwarmEvent::Behaviour(KMBehaviourEvent::Kademlia(
                                    KademliaEvent::RoutingUpdated { peer, .. },
                                )) => {
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
                                        info!(
                                            "Connection to peer {:?} closed with error: {:?}",
                                            peer_id, err
                                        );
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
                                _ => { }
                            },
                            None => break, // Swarm event stream ended
                        }
                    }
            }
        }
    });

    let mut rocket_handle = Some(rocket_handle);
    let mut swarm_handle = Some(swarm_handle);

    // 处理 SIGINT 信号
    tokio::select! {
        _ = signal::ctrl_c() => {
            warn!("Received shutdown signal, shutting down...");
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
        .merge(("port", 0))
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
    let mut rocket = rocket::custom(figment)
        // 附加数据库连接
        .attach(DbConn::fairing())
        .attach(AdminInit)
        // 挂载路由
        .mount("/", routes![index])
        .mount(
            "/api",
            routes![
                get_warehouses,
                create_warehouse,
                create_category,
                create_product,
                get_products,
                login,
                options_login,
                protected_route,
                signup,
                options_signup
            ],
        );

    // 只在 debug 模式下启用 CORS 配置
    #[cfg(debug_assertions)]
    {
        let cors = CorsOptions::default()
            .allowed_origins(AllowedOrigins::all())
            .allowed_methods(
                vec![
                    Method::Get,
                    Method::Post,
                    // 还可以添加其它允许的方法，如 Method::Put, Method::Delete 等
                ]
                .into_iter()
                .map(From::from)
                .collect::<HashSet<_>>(),
            )
            .allowed_headers(AllowedHeaders::all())
            .allow_credentials(true)
            .to_cors()
            .unwrap();
        rocket = rocket.attach(cors);
    }

    rocket
}
