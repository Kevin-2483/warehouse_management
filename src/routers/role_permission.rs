use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, delete};

use crate::models::{RolePermission, NewRolePermission, DbConn};
use crate::schema::role_permissions;
use crate::token::TokenGuard;

#[get("/role_permissions")]
pub async fn list_role_permissions(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<RolePermission>>, Status> {
    conn.run(|c| {
        role_permissions::table
            .select(RolePermission::as_select())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/role_permissions/by_role/<role_id>")]
pub async fn get_role_permissions(conn: DbConn, _token: TokenGuard, role_id: i32) -> Result<Json<Vec<RolePermission>>, Status> {
    conn.run(move |c| {
        role_permissions::table
            .filter(role_permissions::role_id.eq(role_id))
            .select(RolePermission::as_select())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/role_permissions/by_permission/<permission_id>")]
pub async fn get_permission_roles(conn: DbConn, _token: TokenGuard, permission_id: i32) -> Result<Json<Vec<RolePermission>>, Status> {
    conn.run(move |c| {
        role_permissions::table
            .filter(role_permissions::permission_id.eq(permission_id))
            .select(RolePermission::as_select())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[post("/role_permissions", data = "<role_permission>")]
pub async fn create_role_permission(conn: DbConn, _token: TokenGuard, role_permission: Json<NewRolePermission>) -> Result<Status, Status> {
    // 首先检查是否已存在相同的角色权限关联
    let exists = conn.run(move |c| {
        role_permissions::table
            .filter(role_permissions::role_id.eq(role_permission.role_id))
            .filter(role_permissions::permission_id.eq(role_permission.permission_id))
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
                diesel::insert_into(role_permissions::table)
                    .values(role_permission.into_inner())
                    .execute(c)
            }).await
            .map(|_| Status::Created)
            .map_err(|_| Status::InternalServerError)
        }
        Err(_) => Err(Status::InternalServerError)
    }
}

#[delete("/role_permissions/<role_id>/<permission_id>")]
pub async fn delete_role_permission(
    conn: DbConn,
    _token: TokenGuard,
    role_id: i32,
    permission_id: i32
) -> Result<Status, Status> {
    conn.run(move |c| {
        diesel::delete(
            role_permissions::table
                .filter(role_permissions::role_id.eq(role_id))
                .filter(role_permissions::permission_id.eq(permission_id))
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

// 批量设置角色权限（替换现有的所有权限）
#[post("/role_permissions/batch/<role_id>", data = "<permission_ids>")]
pub async fn set_role_permissions(
    conn: DbConn,
    _token: TokenGuard,
    role_id: i32,
    permission_ids: Json<Vec<i32>>
) -> Result<Status, Status> {
    conn.run(move |c| {
        c.transaction(|c| {
            // 删除角色现有的所有权限
            diesel::delete(role_permissions::table.filter(role_permissions::role_id.eq(role_id)))
                .execute(c)?;

            // 插入新的权限
            let new_role_permissions: Vec<NewRolePermission> = permission_ids
                .iter()
                .map(|&permission_id| NewRolePermission {
                    role_id,
                    permission_id,
                })
                .collect();

            for role_permission in new_role_permissions {
                diesel::insert_into(role_permissions::table)
                    .values(role_permission)
                    .execute(c)?;
            }

            Ok::<_, diesel::result::Error>(())
        })
    }).await
    .map(|_| Status::Ok)
    .map_err(|_| Status::InternalServerError)
}

// 检查角色是否具有特定权限
#[get("/role_permissions/check/<role_id>/<permission_id>")]
pub async fn check_role_permission(
    conn: DbConn,
    _token: TokenGuard,
    role_id: i32,
    permission_id: i32
) -> Result<Status, Status> {
    conn.run(move |c| {
        role_permissions::table
            .filter(role_permissions::role_id.eq(role_id))
            .filter(role_permissions::permission_id.eq(permission_id))
            .count()
            .get_result::<i64>(c)
    }).await
    .map(|count| {
        if count > 0 {
            Status::Ok
        } else {
            Status::NotFound
        }
    })
    .map_err(|_| Status::InternalServerError)
}
