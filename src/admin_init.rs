use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Build, Rocket};
use diesel::prelude::*;
use log::info;

use rand::distributions::Alphanumeric;
use rand::Rng;


use crate::schema::administrators;
use crate::models::DbConn;

pub struct AdminInit;

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