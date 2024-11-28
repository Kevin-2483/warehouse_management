use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post};
use chrono::Utc;

use crate::models::{ProductionTask, NewProductionTask, DbConn};
use crate::schema::production_tasks;
use crate::token::TokenGuard;

#[get("/production_tasks")]
pub async fn list_production_tasks(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<ProductionTask>>, Status> {
    conn.run(|c| {
        production_tasks::table
            .select(ProductionTask::as_select())
            .order(production_tasks::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/production_tasks/<task_id>")]
pub async fn get_production_task(conn: DbConn, _token: TokenGuard, task_id: i32) -> Result<Json<ProductionTask>, Status> {
    conn.run(move |c| {
        production_tasks::table
            .find(task_id)
            .select(ProductionTask::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[get("/production_tasks/by_product/<product_id>")]
pub async fn get_tasks_by_product(conn: DbConn, _token: TokenGuard, product_id: i32) -> Result<Json<Vec<ProductionTask>>, Status> {
    conn.run(move |c| {
        production_tasks::table
            .filter(production_tasks::product_id.eq(product_id))
            .select(ProductionTask::as_select())
            .order(production_tasks::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/production_tasks/by_status/<status>")]
pub async fn get_tasks_by_status(conn: DbConn, _token: TokenGuard, status: String) -> Result<Json<Vec<ProductionTask>>, Status> {
    conn.run(move |c| {
        production_tasks::table
            .filter(production_tasks::status.eq(status))
            .select(ProductionTask::as_select())
            .order(production_tasks::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[post("/production_tasks", data = "<task>")]
pub async fn create_production_task(conn: DbConn, _token: TokenGuard, task: Json<NewProductionTask>) -> Result<Status, Status> {
    let task_with_timestamp = (
        production_tasks::product_id.eq(task.product_id),
        production_tasks::quantity.eq(task.quantity),
        production_tasks::due_date.eq(task.due_date),
        production_tasks::created_by.eq(task.created_by),
        production_tasks::created_at.eq(Utc::now().naive_utc()),
        production_tasks::status.eq("pending"), // 默认状态为 pending
    );

    conn.run(move |c| {
        diesel::insert_into(production_tasks::table)
            .values(task_with_timestamp)
            .execute(c)
    }).await
    .map(|_| Status::Created)
    .map_err(|_| Status::InternalServerError)
}
