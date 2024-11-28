use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, put, delete};
use chrono::Utc;

use crate::models::{ProductionCost, NewProductionCost, DbConn};
use crate::schema::production_costs;
use crate::token::TokenGuard;

#[get("/production_costs")]
pub async fn list_production_costs(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<ProductionCost>>, Status> {
    conn.run(|c| {
        production_costs::table
            .select(ProductionCost::as_select())
            .order(production_costs::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/production_costs/<cost_id>")]
pub async fn get_production_cost(conn: DbConn, _token: TokenGuard, cost_id: i32) -> Result<Json<ProductionCost>, Status> {
    conn.run(move |c| {
        production_costs::table
            .find(cost_id)
            .select(ProductionCost::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[get("/production_costs/by_process/<process_type>")]
pub async fn get_costs_by_process(conn: DbConn, _token: TokenGuard, process_type: String) -> Result<Json<Vec<ProductionCost>>, Status> {
    conn.run(move |c| {
        production_costs::table
            .filter(production_costs::process_type.eq(process_type))
            .select(ProductionCost::as_select())
            .order(production_costs::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[post("/production_costs", data = "<cost>")]
pub async fn create_production_cost(conn: DbConn, _token: TokenGuard, cost: Json<NewProductionCost>) -> Result<Json<ProductionCost>, Status> {
    let cost_with_timestamp = (
        production_costs::process_type.eq(&cost.process_type),
        production_costs::cost_per_unit.eq(cost.cost_per_unit),
        production_costs::created_by.eq(cost.created_by),
        production_costs::created_at.eq(Utc::now().naive_utc()),
    );

    conn.run(move |c| {
        diesel::insert_into(production_costs::table)
            .values(cost_with_timestamp)
            .get_result(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[put("/production_costs/<cost_id>", data = "<cost>")]
pub async fn update_production_cost(
    conn: DbConn,
    _token: TokenGuard,
    cost_id: i32,
    cost: Json<NewProductionCost>
) -> Result<Json<ProductionCost>, Status> {
    conn.run(move |c| {
        diesel::update(production_costs::table.find(cost_id))
            .set((
                production_costs::process_type.eq(&cost.process_type),
                production_costs::cost_per_unit.eq(cost.cost_per_unit),
                production_costs::created_by.eq(cost.created_by),
            ))
            .get_result(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[delete("/production_costs/<cost_id>")]
pub async fn delete_production_cost(conn: DbConn, _token: TokenGuard, cost_id: i32) -> Result<Status, Status> {
    conn.run(move |c| {
        diesel::delete(production_costs::table.find(cost_id))
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

// 获取最新的生产成本记录
#[get("/production_costs/latest/<process_type>")]
pub async fn get_latest_cost(conn: DbConn, _token: TokenGuard, process_type: String) -> Result<Json<ProductionCost>, Status> {
    conn.run(move |c| {
        production_costs::table
            .filter(production_costs::process_type.eq(process_type))
            .order(production_costs::created_at.desc())
            .select(ProductionCost::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}
