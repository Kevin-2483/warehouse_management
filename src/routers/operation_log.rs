use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use chrono::Utc;
use rocket::{get, post};
use crate::models::{OperationLog, NewOperationLog, DbConn};
use crate::schema::operation_logs;
use crate::token::TokenGuard;

#[get("/operation_logs")]
pub async fn list_operation_logs(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<OperationLog>>, Status> {
    conn.run(|c| {
        operation_logs::table
            .select(OperationLog::as_select())
            .order(operation_logs::timestamp.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/operation_logs/<user_id>")]
pub async fn get_user_operation_logs(conn: DbConn, _token: TokenGuard, user_id: i32) -> Result<Json<Vec<OperationLog>>, Status> {
    conn.run(move |c| {
        operation_logs::table
            .filter(operation_logs::user_id.eq(user_id))
            .select(OperationLog::as_select())
            .order(operation_logs::timestamp.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[post("/operation_logs", data = "<log>")]
pub async fn create_operation_log(conn: DbConn, _token: TokenGuard, log: Json<NewOperationLog>) -> Result<Status, Status> {
    // Insert with current timestamp
    let log_with_timestamp = (
        operation_logs::user_id.eq(log.user_id),
        operation_logs::action.eq(&log.action),
        operation_logs::timestamp.eq(Utc::now().naive_utc()),
    );

    conn.run(move |c| {
        diesel::insert_into(operation_logs::table)
            .values(log_with_timestamp)
            .execute(c)
    }).await
    .map(|_| Status::Created)
    .map_err(|_| Status::InternalServerError)
}
