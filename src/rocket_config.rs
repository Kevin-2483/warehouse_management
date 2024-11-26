use rocket::{Build, Rocket};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::collections::{HashMap, HashSet};
use rocket::figment::{providers::{Env, Serialized}, Figment};
use rocket::Config;
use crate::models::DbConn;
use rocket::http::Method;
use crate::admin_init::AdminInit;
use rocket::routes;

// 将 rocket 函数移到这里
pub async fn rocket() -> Rocket<Build> {
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
        .attach(AdminInit) // 使用 AdminInit
        // 挂载路由
        .mount("/", routes![])
        .mount(
            "/api",
            routes![
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