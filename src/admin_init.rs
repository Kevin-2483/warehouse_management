use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Build, Rocket};
use diesel::prelude::*;
use log::info;

use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::schema::administrators; // 根据需要导入 administrators 模块
use crate::models::DbConn; // 修改为正确的导入路径

pub struct AdminInit; // 定义 AdminInit 结构体

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