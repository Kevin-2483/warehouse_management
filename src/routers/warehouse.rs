use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, put, delete}; 
use diesel::prelude::*;
use chrono::Utc;
use crate::models::{Warehouse, NewWarehouse, DbConn};
use crate::schema::warehouses;
use crate::token;
use serde::Deserialize;

// Get all warehouses
#[get("/warehouses")]
pub async fn get_warehouses(conn: DbConn, token: String) -> Result<Json<Vec<Warehouse>>, Status> {
    // Verify token
    if token::decode_token(&token).is_none() {
        return Err(Status::Unauthorized);
    }

    let warehouses = conn.run(|c| {
        warehouses::table
            .select(Warehouse::as_select())
            .load(c)
    }).await;

    match warehouses {
        Ok(warehouse_list) => Ok(Json(warehouse_list)),
        Err(_) => Err(Status::InternalServerError)
    }
}

// Get single warehouse by ID
#[get("/warehouse/<warehouse_id>")]
pub async fn get_warehouse(conn: DbConn, warehouse_id: i32, token: String) -> Result<Json<Warehouse>, Status> {
    // Verify token
    if token::decode_token(&token).is_none() {
        return Err(Status::Unauthorized);
    }

    let warehouse = conn.run(move |c| {
        warehouses::table
            .find(warehouse_id)
            .select(Warehouse::as_select())
            .first(c)
    }).await;

    match warehouse {
        Ok(warehouse_data) => Ok(Json(warehouse_data)),
        Err(_) => Err(Status::NotFound)
    }
}

// Create new warehouse
#[post("/warehouse", data = "<warehouse>")]
pub async fn create_warehouse(
    conn: DbConn,
    warehouse: Json<NewWarehouse>,
    token: String,
) -> Result<Status, Status> {
    // Verify token
    if token::decode_token(&token).is_none() {
        return Err(Status::Unauthorized);
    }

    let result = conn.run(move |c| {
        diesel::insert_into(warehouses::table)
            .values(&*warehouse)
            .execute(c)
    }).await;

    match result {
        Ok(_) => Ok(Status::Created),
        Err(_) => Err(Status::InternalServerError)
    }
}

// Update warehouse data structure
#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = warehouses)]
pub struct UpdateWarehouse {
    pub warehouse_name: Option<String>,
    pub location: Option<String>,
    pub capacity: Option<i32>,
    pub current_stock: Option<i32>,
}

// Update warehouse
#[put("/warehouse/<warehouse_id>", data = "<warehouse>")]
pub async fn update_warehouse(
    conn: DbConn,
    warehouse_id: i32,
    warehouse: Json<UpdateWarehouse>,
    token: String,
) -> Result<Status, Status> {
    // Verify token
    if token::decode_token(&token).is_none() {
        return Err(Status::Unauthorized);
    }

    let result = conn.run(move |c| {
        diesel::update(warehouses::table.find(warehouse_id))
            .set((
                &*warehouse,
                warehouses::last_updated.eq(Some(Utc::now().naive_utc()))
            ))
            .execute(c)
    }).await;

    match result {
        Ok(rows) if rows > 0 => Ok(Status::Ok),
        Ok(_) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError)
    }
}

// Delete warehouse
#[delete("/warehouse/<warehouse_id>")]
pub async fn delete_warehouse(
    conn: DbConn,
    warehouse_id: i32,
    token: String,
) -> Result<Status, Status> {
    // Verify token
    if token::decode_token(&token).is_none() {
        return Err(Status::Unauthorized);
    }

    let result = conn.run(move |c| {
        diesel::delete(warehouses::table.find(warehouse_id))
            .execute(c)
    }).await;

    match result {
        Ok(rows) if rows > 0 => Ok(Status::NoContent),
        Ok(_) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError)
    }
}