use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, delete};

use crate::models::{UserRole, NewUserRole, DbConn};
use crate::schema::user_roles;
use crate::token::TokenGuard;

#[get("/user_roles")]
pub async fn list_user_roles(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<UserRole>>, Status> {
    conn.run(|c| {
        user_roles::table
            .select(UserRole::as_select())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/user_roles/by_user/<user_id>")]
pub async fn get_user_roles(conn: DbConn, _token: TokenGuard, user_id: i32) -> Result<Json<Vec<UserRole>>, Status> {
    conn.run(move |c| {
        user_roles::table
            .filter(user_roles::user_id.eq(user_id))
            .select(UserRole::as_select())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/user_roles/by_role/<role_id>")]
pub async fn get_role_users(conn: DbConn, _token: TokenGuard, role_id: i32) -> Result<Json<Vec<UserRole>>, Status> {
    conn.run(move |c| {
        user_roles::table
            .filter(user_roles::role_id.eq(role_id))
            .select(UserRole::as_select())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[post("/user_roles", data = "<user_role>")]
pub async fn create_user_role(conn: DbConn, _token: TokenGuard, user_role: Json<NewUserRole>) -> Result<Status, Status> {
    // 首先检查是否已存在相同的用户角色关联
    let exists = conn.run(move |c| {
        user_roles::table
            .filter(user_roles::user_id.eq(user_role.user_id))
            .filter(user_roles::role_id.eq(user_role.role_id))
            .count()
            .get_result::<i64>(c)
    }).await;

    match exists {
        Ok(count) if count > 0 => {
            return Err(Status::Conflict); // 返回409表示已存在
        }
        Ok(_) => {
            // 不存在，继续创建
            conn.run(move |c| {
                diesel::insert_into(user_roles::table)
                    .values(user_role.into_inner())
                    .execute(c)
            }).await
            .map(|_| Status::Created)
            .map_err(|_| Status::InternalServerError)
        }
        Err(_) => Err(Status::InternalServerError)
    }
}

#[delete("/user_roles/<user_id>/<role_id>")]
pub async fn delete_user_role(conn: DbConn, _token: TokenGuard, user_id: i32, role_id: i32) -> Result<Status, Status> {
    conn.run(move |c| {
        diesel::delete(
            user_roles::table
                .filter(user_roles::user_id.eq(user_id))
                .filter(user_roles::role_id.eq(role_id))
        )
        .execute(c)
    }).await
    .map(|affected| {
        if affected > 0 {
            Status::NoContent
        } else {
            Status::NotFound
        }
    })
    .map_err(|_| Status::InternalServerError)
}

// 批量设置用户角色（替换现有的所有角色）
#[post("/user_roles/batch/<user_id>", data = "<role_ids>")]
pub async fn set_user_roles(
    conn: DbConn,
    _token: TokenGuard,
    user_id: i32,
    role_ids: Json<Vec<i32>>
) -> Result<Status, Status> {
    conn.run(move |c| {
        c.transaction(|c| {
            // 删除用户现有的所有角色
            diesel::delete(user_roles::table.filter(user_roles::user_id.eq(user_id)))
                .execute(c)?;

            // 插入新的角色
            let new_user_roles: Vec<NewUserRole> = role_ids
                .iter()
                .map(|&role_id| NewUserRole {
                    user_id,
                    role_id,
                })
                .collect();

            for user_role in new_user_roles {
                diesel::insert_into(user_roles::table)
                    .values(user_role)
                    .execute(c)?;
            }

            Ok::<_, diesel::result::Error>(())
        })
    }).await
    .map(|_| Status::Ok)
    .map_err(|_| Status::InternalServerError)
}
