use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::{get, post, put, delete};
use chrono::Utc;

use crate::models::{ProductSpecification, NewProductSpecification, DbConn};
use crate::schema::product_specifications;
use crate::token::TokenGuard;

#[get("/product_specifications")]
pub async fn list_product_specifications(conn: DbConn, _token: TokenGuard) -> Result<Json<Vec<ProductSpecification>>, Status> {
    conn.run(|c| {
        product_specifications::table
            .select(ProductSpecification::as_select())
            .order(product_specifications::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/product_specifications/<product_id>")]
pub async fn get_product_specification(
    conn: DbConn,
    _token: TokenGuard,
    product_id: i32
) -> Result<Json<ProductSpecification>, Status> {
    conn.run(move |c| {
        product_specifications::table
            .find(product_id)
            .select(ProductSpecification::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[get("/product_specifications/by_name/<product_name>")]
pub async fn get_specification_by_name(
    conn: DbConn,
    _token: TokenGuard,
    product_name: String
) -> Result<Json<ProductSpecification>, Status> {
    conn.run(move |c| {
        product_specifications::table
            .filter(product_specifications::product_name.eq(product_name))
            .select(ProductSpecification::as_select())
            .first(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[get("/product_specifications/by_material/<material_type>")]
pub async fn get_specifications_by_material(
    conn: DbConn,
    _token: TokenGuard,
    material_type: String
) -> Result<Json<Vec<ProductSpecification>>, Status> {
    conn.run(move |c| {
        product_specifications::table
            .filter(product_specifications::material_type.eq(material_type))
            .select(ProductSpecification::as_select())
            .order(product_specifications::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[get("/product_specifications/by_model/<model>")]
pub async fn get_specifications_by_model(
    conn: DbConn,
    _token: TokenGuard,
    model: String
) -> Result<Json<Vec<ProductSpecification>>, Status> {
    conn.run(move |c| {
        product_specifications::table
            .filter(product_specifications::model.eq(model))
            .select(ProductSpecification::as_select())
            .order(product_specifications::created_at.desc())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[post("/product_specifications", data = "<specification>")]
pub async fn create_product_specification(
    conn: DbConn,
    _token: TokenGuard,
    specification: Json<NewProductSpecification>
) -> Result<Json<ProductSpecification>, Status> {
    // 检查产品名称是否已存在
    let exists = conn.run(move |c| {
        product_specifications::table
            .filter(product_specifications::product_name.eq(&specification.product_name))
            .count()
            .get_result::<i64>(c)
    }).await;

    if let Ok(count) = exists {
        if count > 0 {
            return Err(Status::Conflict);
        }
    }

    let spec_with_timestamp = (
        product_specifications::product_name.eq(&specification.product_name),
        product_specifications::model.eq(&specification.model),
        product_specifications::material_type.eq(&specification.material_type),
        product_specifications::color.eq(&specification.color),
        product_specifications::dimensions.eq(&specification.dimensions),
        product_specifications::created_by.eq(specification.created_by),
        product_specifications::created_at.eq(Utc::now().naive_utc()),
    );

    conn.run(move |c| {
        diesel::insert_into(product_specifications::table)
            .values(spec_with_timestamp)
            .get_result(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

#[put("/product_specifications/<product_id>", data = "<specification>")]
pub async fn update_product_specification(
    conn: DbConn,
    _token: TokenGuard,
    product_id: i32,
    specification: Json<NewProductSpecification>
) -> Result<Json<ProductSpecification>, Status> {
    // 检查新的产品名称是否与其他产品冲突
    let exists = conn.run(move |c| {
        product_specifications::table
            .filter(product_specifications::product_name.eq(&specification.product_name))
            .filter(product_specifications::product_id.ne(product_id))
            .count()
            .get_result::<i64>(c)
    }).await;

    if let Ok(count) = exists {
        if count > 0 {
            return Err(Status::Conflict);
        }
    }

    conn.run(move |c| {
        diesel::update(product_specifications::table.find(product_id))
            .set((
                product_specifications::product_name.eq(&specification.product_name),
                product_specifications::model.eq(&specification.model),
                product_specifications::material_type.eq(&specification.material_type),
                product_specifications::color.eq(&specification.color),
                product_specifications::dimensions.eq(&specification.dimensions),
                product_specifications::created_by.eq(specification.created_by),
            ))
            .get_result(c)
    }).await
    .map(Json)
    .map_err(|_| Status::NotFound)
}

#[delete("/product_specifications/<product_id>")]
pub async fn delete_product_specification(
    conn: DbConn,
    _token: TokenGuard,
    product_id: i32
) -> Result<Status, Status> {
    conn.run(move |c| {
        diesel::delete(product_specifications::table.find(product_id))
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

// 搜索产品规格
#[get("/product_specifications/search?<query>&<material_type>&<model>")]
pub async fn search_specifications(
    conn: DbConn,
    _token: TokenGuard,
    query: Option<String>,
    material_type: Option<String>,
    model: Option<String>
) -> Result<Json<Vec<ProductSpecification>>, Status> {
    conn.run(move |c| {
        let mut query_builder = product_specifications::table
            .into_boxed();

        if let Some(q) = query {
            query_builder = query_builder.filter(
                product_specifications::product_name.like(format!("%{}%", q))
            );
        }

        if let Some(mat_type) = material_type {
            query_builder = query_builder.filter(
                product_specifications::material_type.eq(mat_type)
            );
        }

        if let Some(m) = model {
            query_builder = query_builder.filter(
                product_specifications::model.eq(m)
            );
        }

        query_builder
            .order(product_specifications::created_at.desc())
            .select(ProductSpecification::as_select())
            .load(c)
    }).await
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}
