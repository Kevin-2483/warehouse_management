use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, put, delete}; 
use diesel::prelude::*;
use crate::models::{Role, NewRole, DbConn};
use crate::schema::roles;
use crate::token::TokenGuard;

#[get("/roles")]
pub async fn get_roles(conn: DbConn) -> Result<Json<Vec<Role>>, Status> {
    let roles = conn.run(|c| {
        roles::table
            .select(Role::as_select())
            .load(c)
    }).await;

    match roles {
        Ok(roles_list) => Ok(Json(roles_list)),
        Err(_) => Err(Status::InternalServerError)
    }
}

#[get("/role/<role_id>")]
pub async fn get_role(conn: DbConn, role_id: i32) -> Result<Json<Role>, Status> {
    let role = conn.run(move |c| {
        roles::table
            .find(role_id)
            .select(Role::as_select())
            .first(c)
    }).await;

    match role {
        Ok(role_data) => Ok(Json(role_data)),
        Err(_) => Err(Status::NotFound)
    }
}

#[post("/role", data = "<role>")]
pub async fn create_role(
    conn: DbConn,
    role: Json<NewRole>,
    token: TokenGuard,
) -> Result<Status, Status> {
    let result = conn.run(move |c| {
        diesel::insert_into(roles::table)
            .values(&role.into_inner())
            .execute(c)
    }).await;

    match result {
        Ok(_) => Ok(Status::Created),
        Err(_) => Err(Status::InternalServerError)
    }
}

#[put("/role/<role_id>", data = "<role>")]
pub async fn update_role(
    conn: DbConn,
    role_id: i32,
    role: Json<NewRole>,
    token: TokenGuard,
) -> Result<Status, Status> {
    let result = conn.run(move |c| {
        diesel::update(roles::table.find(role_id))
            .set((
                roles::role_name.eq(role.role_name.to_owned()),
                roles::description.eq(role.description.to_owned()),
            ))
            .execute(c)
    }).await;

    match result {
        Ok(1) => Ok(Status::Ok),
        Ok(0) => Err(Status::NotFound),
        _ => Err(Status::InternalServerError)
    }
}

#[delete("/role/<role_id>")]
pub async fn delete_role(
    conn: DbConn,
    role_id: i32,
    token: TokenGuard,
) -> Result<Status, Status> {
    let result = conn.run(move |c| {
        diesel::delete(roles::table.find(role_id))
            .execute(c)
    }).await;

    match result {
        Ok(1) => Ok(Status::Ok),
        Ok(0) => Err(Status::NotFound),
        _ => Err(Status::InternalServerError)
    }
}
