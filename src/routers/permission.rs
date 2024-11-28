use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, put, delete}; 

use crate::models::{Permission, NewPermission, DbConn};
use crate::schema::permissions;
use crate::token::TokenGuard;

#[get("/permissions")]
pub async fn list_permissions(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<Permission>>, Status> {
    conn.run(|c| {
        permissions::table
            .select(Permission::as_select())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/permissions/<id>")]
pub async fn get_permission(conn: DbConn, _token: TokenGuard, id: i32) -> Result<Json<Permission>, Status> {
    conn.run(move |c| {
        permissions::table
            .find(id)
            .select(Permission::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[post("/permissions", data = "<permission>")]
pub async fn create_permission(conn: DbConn, _token: TokenGuard, permission: Json<NewPermission>) -> Result<Status, Status> {
    conn.run(|c| {
        diesel::insert_into(permissions::table)
            .values(permission.into_inner())
            .execute(c)
    }).await
    .map(|_| Status::Created)
    .map_err(|_| Status::InternalServerError)
}

#[put("/permissions/<id>", data = "<permission>")]
pub async fn update_permission(
    conn: DbConn,
    _token: TokenGuard,
    id: i32,
    permission: Json<NewPermission>
) -> Result<Status, Status> {
    conn.run(move |c| {
        diesel::update(permissions::table.find(id))
            .set((
                permissions::permission_name.eq(&permission.permission_name),
                permissions::description.eq(&permission.description),
            ))
            .execute(c)
    }).await
    .map(|_| Status::Ok)
    .map_err(|_| Status::InternalServerError)
}

#[delete("/permissions/<id>")]
pub async fn delete_permission(conn: DbConn, _token: TokenGuard, id: i32) -> Result<Status, Status> {
    conn.run(move |c| {
        diesel::delete(permissions::table.find(id))
            .execute(c)
    }).await
    .map(|_| Status::NoContent)
    .map_err(|_| Status::InternalServerError)
}
