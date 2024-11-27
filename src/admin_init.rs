use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Build, Rocket};
use diesel::prelude::*;
use log::info;

use rand::distributions::Alphanumeric;
use rand::Rng;

use bcrypt;

use crate::schema::users;
use crate::models::{DbConn, NewUser};

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
            use self::users::dsl::*;

            let admin_count: i64 = users
                .count()
                .get_result(c)
                .expect("Error counting users");

            if admin_count == 0 {
                // Generate a random password for initial admin
                let password: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(12)
                    .map(char::from)
                    .collect();

                let new_admin = NewUser {
                    username: "admin".to_string(),
                    password_hash: bcrypt::hash(password.as_bytes(), bcrypt::DEFAULT_COST)
                        .expect("Failed to hash password"),
                    full_name: Some("System Administrator".to_string()),
                    position: Some("Administrator".to_string()),
                    contact_info: None,
                    status: Some("active".to_string()),
                };

                diesel::insert_into(users)
                    .values(&new_admin)
                    .execute(c)
                    .expect("Error saving admin user");

                info!("Created initial admin user with username: 'admin' and password: '{}'", password);
            }

            Ok(rocket)
        }).await
    }
}