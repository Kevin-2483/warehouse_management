#[macro_use]
extern crate rocket;

use rocket::serde::json::Json;
use rocket_sync_db_pools::{database, diesel};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::figment::{Figment, providers::{Env, Serialized}};
use rocket::{Build, Rocket, Config};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use rand::Rng;
use rand::distributions::Alphanumeric;
use crate::models::*;
use crate::schema::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::collections::HashMap;
use rocket::fairing::AdHoc;
use log::{info, error};

use serde::Deserialize;
use chrono::Utc;
use uuid::Uuid;



mod models;
mod schema;



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
        warehouses.load::<Warehouse>(c).map(Json).expect("Error loading warehouses")
    }).await
}

#[post("/warehouses", format = "json", data = "<new_warehouse>")]
async fn create_warehouse(new_warehouse: Json<NewWarehouse>, conn: DbConn) -> Result<Json<Warehouse>, String> {

    let db = conn;

    let new_warehouse = db.run(move |c| {
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
    }).await?;
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

            let admin_count: i64 = administrators.count().get_result(c).expect("Error counting admins");

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
        }).await;

        Ok(rocket)
    }
}

async fn run_db_migrations(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    let db = DbConn::get_one(&rocket).await.expect("database connection");

    match db.run(|conn| {
        conn.run_pending_migrations(MIGRATIONS)
            .map(|versions| {
                // Log each version of the migration
                versions.into_iter().map(|v| {
                    let version_str = v.to_string();
                    info!("Successfully applied migration: {}", version_str);
                    version_str
                }).collect::<Vec<_>>()
            })
    }).await {
        Ok(versions) => {
            info!("All pending migrations were run successfully: {:?}", versions);
            Ok(rocket)
        },
        Err(e) => {
            error!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    }
}

#[tokio::main]
async fn main() {
    let rocket = rocket().await;
    rocket.launch().await.unwrap();
}

async fn rocket() -> Rocket<Build> {
    // Initialize the logger
    env_logger::init();

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
        .attach(AdHoc::try_on_ignite("Database Migrations", run_db_migrations))
        .attach(AdminInit)
        // 挂载路由
        .mount("/", routes![index])
        .mount("/api", routes![get_warehouses, create_warehouse]);

    rocket
}

