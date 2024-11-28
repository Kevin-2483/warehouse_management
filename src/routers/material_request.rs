use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post};
use chrono::Utc;

use crate::models::{MaterialRequest, NewMaterialRequest, DbConn};
use crate::schema::material_requests;
use crate::token::TokenGuard;

#[get("/material_requests")]
pub async fn list_material_requests(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<MaterialRequest>>, Status> {
    conn.run(|c| {
        material_requests::table
            .select(MaterialRequest::as_select())
            .order(material_requests::request_date.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/material_requests/<request_id>")]
pub async fn get_material_request(
    conn: DbConn,
    _token: TokenGuard,
    request_id: i32
) -> Result<Json<MaterialRequest>, Status> {
    conn.run(move |c| {
        material_requests::table
            .find(request_id)
            .select(MaterialRequest::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[get("/material_requests/by_material/<material_id>")]
pub async fn get_requests_by_material(
    conn: DbConn,
    _token: TokenGuard,
    material_id: i32
) -> Result<Json<Vec<MaterialRequest>>, Status> {
    conn.run(move |c| {
        material_requests::table
            .filter(material_requests::material_id.eq(material_id))
            .select(MaterialRequest::as_select())
            .order(material_requests::request_date.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/material_requests/by_warehouse/<warehouse_id>")]
pub async fn get_requests_by_warehouse(
    conn: DbConn,
    _token: TokenGuard,
    warehouse_id: i32
) -> Result<Json<Vec<MaterialRequest>>, Status> {
    conn.run(move |c| {
        material_requests::table
            .filter(material_requests::warehouse_id.eq(warehouse_id))
            .select(MaterialRequest::as_select())
            .order(material_requests::request_date.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/material_requests/by_status/<status>")]
pub async fn get_requests_by_status(
    conn: DbConn,
    _token: TokenGuard,
    status: String
) -> Result<Json<Vec<MaterialRequest>>, Status> {
    conn.run(move |c| {
        material_requests::table
            .filter(material_requests::status.eq(status))
            .select(MaterialRequest::as_select())
            .order(material_requests::request_date.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/material_requests/by_requester/<requested_by>")]
pub async fn get_requests_by_requester(
    conn: DbConn,
    _token: TokenGuard,
    requested_by: i32
) -> Result<Json<Vec<MaterialRequest>>, Status> {
    conn.run(move |c| {
        material_requests::table
            .filter(material_requests::requested_by.eq(requested_by))
            .select(MaterialRequest::as_select())
            .order(material_requests::request_date.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[post("/material_requests", data = "<request>")]
pub async fn create_material_request(
    conn: DbConn,
    _token: TokenGuard,
    request: Json<NewMaterialRequest>
) -> Result<Json<MaterialRequest>, Status> {
    let request_with_date = (
        material_requests::material_id.eq(request.material_id),
        material_requests::quantity.eq(request.quantity),
        material_requests::requested_by.eq(request.requested_by),
        material_requests::warehouse_id.eq(request.warehouse_id),
        material_requests::status.eq(&request.status),
        material_requests::request_date.eq(Utc::now().naive_utc()),
    );

    conn.run(move |c| {
        diesel::insert_into(material_requests::table)
            .values(request_with_date)
            .get_result(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

// 搜索材料请求
#[get("/material_requests/search?<material_id>&<warehouse_id>&<status>&<start_date>&<end_date>")]
pub async fn search_material_requests(
    conn: DbConn,
    _token: TokenGuard,
    material_id: Option<i32>,
    warehouse_id: Option<i32>,
    status: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>
) -> Result<Json<Vec<MaterialRequest>>, Status> {
    use chrono::NaiveDateTime;

    conn.run(move |c| {
        let mut query_builder = material_requests::table
            .into_boxed();

        if let Some(mid) = material_id {
            query_builder = query_builder.filter(
                material_requests::material_id.eq(mid)
            );
        }

        if let Some(wid) = warehouse_id {
            query_builder = query_builder.filter(
                material_requests::warehouse_id.eq(wid)
            );
        }

        if let Some(s) = status {
            query_builder = query_builder.filter(
                material_requests::status.eq(s)
            );
        }

        if let Some(start) = start_date {
            if let Ok(start_dt) = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", start), "%Y-%m-%d %H:%M:%S") {
                query_builder = query_builder.filter(
                    material_requests::request_date.ge(start_dt)
                );
            }
        }

        if let Some(end) = end_date {
            if let Ok(end_dt) = NaiveDateTime::parse_from_str(&format!("{} 23:59:59", end), "%Y-%m-%d %H:%M:%S") {
                query_builder = query_builder.filter(
                    material_requests::request_date.le(end_dt)
                );
            }
        }

        query_builder
            .order(material_requests::request_date.desc())
            .select(MaterialRequest::as_select())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}
