use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, put, delete}; 
use diesel::prelude::*;
use crate::models::{User, NewUser, DbConn};
use crate::schema::users;
use crate::token;
use serde::Deserialize;
use rocket_dyn_templates::serde::Serialize;

#[get("/users")]
pub async fn get_users(conn: DbConn) -> Result<Json<Vec<User>>, Status> {
    let users = conn.run(|c| {
        users::table
            .select(User::as_select())
            .load(c)
    }).await;

    match users {
        Ok(users_list) => Ok(Json(users_list)),
        Err(_) => Err(Status::InternalServerError)
    }
}

#[get("/user/<user_id>")]
pub async fn get_user(conn: DbConn, user_id: i32) -> Result<Json<User>, Status> {
    let user = conn.run(move |c| {
        users::table
            .find(user_id)
            .select(User::as_select())
            .first(c)
    }).await;

    match user {
        Ok(user_data) => Ok(Json(user_data)),
        Err(_) => Err(Status::NotFound)
    }
}

#[post("/user", data = "<user>")]
pub async fn create_user(
    conn: DbConn,
    user: Json<NewUser>,
    token: String,
) -> Result<Status, Status> {
    // Verify token
    if token::decode_token(&token).is_none() {
        return Err(Status::Unauthorized);
    }

    let result = conn.run(move |c| {
        diesel::insert_into(users::table)
            .values(&user.into_inner())
            .execute(c)
    }).await;

    match result {
        Ok(_) => Ok(Status::Created),
        Err(_) => Err(Status::InternalServerError)
    }
}
#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub full_name: Option<String>,
    pub position: Option<String>,
    pub contact_info: Option<String>,
    pub status: Option<String>,
}

#[put("/user/<user_id>", data = "<user>")]
pub async fn update_user(
    conn: DbConn,
    user_id: i32,
    user: Json<UpdateUser>,
    token: String,
) -> Result<Status, Status> {
    // Verify token
    if token::decode_token(&token).is_none() {
        return Err(Status::Unauthorized);
    }

    let result = conn.run(move |c| {
        diesel::update(users::table.find(user_id))
            .set(&user.into_inner())
            .execute(c)
    }).await;

    match result {
        Ok(count) if count > 0 => Ok(Status::Ok),
        Ok(_) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError)
    }
}

#[delete("/user/<user_id>")]
pub async fn delete_user(
    conn: DbConn,
    user_id: i32,
    token: String,
) -> Result<Status, Status> {
    // Verify token
    if token::decode_token(&token).is_none() {
        return Err(Status::Unauthorized);
    }

    let result = conn.run(move |c| {
        diesel::delete(users::table.find(user_id))
            .execute(c)
    }).await;

    match result {
        Ok(count) if count > 0 => Ok(Status::Ok),
        Ok(_) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError)
    }
}
